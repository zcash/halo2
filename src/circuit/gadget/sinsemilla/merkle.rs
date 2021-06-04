use halo2::{
    circuit::{Chip, Layouter},
    plonk::Error,
};
use pasta_curves::arithmetic::CurveAffine;

use super::{HashDomains, SinsemillaInstructions};

use crate::{
    circuit::gadget::utilities::{cond_swap::CondSwapInstructions, UtilitiesInstructions},
    spec::i2lebsp,
};
use std::{convert::TryInto, iter};

// mod chip;

/// Instructions to check the validity of a Merkle path of a given `PATH_LENGTH`.
/// The hash function used is a Sinsemilla instance with `K`-bit words.
/// The hash function can process `MAX_WORDS` words.
pub trait MerkleInstructions<
    C: CurveAffine,
    const PATH_LENGTH: usize,
    const K: usize,
    const MAX_WORDS: usize,
>:
    SinsemillaInstructions<C, K, MAX_WORDS>
    + CondSwapInstructions<C::Base>
    + UtilitiesInstructions<C::Base>
    + Chip<C::Base>
{
    /// Compute MerkleCRH for a given `layer`. The root is at `layer 0`, and the
    /// leaves are at `layer MERKLE_DEPTH_ORCHARD` = `layer 32`.
    #[allow(non_snake_case)]
    fn hash_layer(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: C,
        l_star: usize,
        left: Self::Var,
        right: Self::Var,
    ) -> Result<Self::Var, Error>;
}

#[derive(Clone, Debug)]
pub struct MerklePath<
    C: CurveAffine,
    MerkleChip,
    const PATH_LENGTH: usize,
    const K: usize,
    const MAX_WORDS: usize,
> where
    MerkleChip: MerkleInstructions<C, PATH_LENGTH, K, MAX_WORDS> + Clone,
{
    chip_1: MerkleChip,
    chip_2: MerkleChip,
    domain: MerkleChip::HashDomains,
    leaf_pos: Option<u32>,
    path: Option<[C::Base; PATH_LENGTH]>,
}

#[allow(non_snake_case)]
impl<
        C: CurveAffine,
        MerkleChip,
        const PATH_LENGTH: usize,
        const K: usize,
        const MAX_WORDS: usize,
    > MerklePath<C, MerkleChip, PATH_LENGTH, K, MAX_WORDS>
where
    MerkleChip: MerkleInstructions<C, PATH_LENGTH, K, MAX_WORDS> + Clone,
{
    /// Calculates the root of the tree containing the given leaf at this Merkle path.
    fn calculate_root(
        &self,
        mut layouter: impl Layouter<C::Base>,
        leaf: MerkleChip::Var,
    ) -> Result<MerkleChip::Var, Error> {
        // A Sinsemilla chip uses 5 advice columns, but the full Orchard action circuit
        // uses 10 advice columns. We distribute the path hashing across two Sinsemilla
        // chips to make better use of the available circuit area.
        let chips = iter::empty()
            .chain(iter::repeat(self.chip_1.clone()).take(PATH_LENGTH / 2))
            .chain(iter::repeat(self.chip_2.clone()));

        let path = if let Some(path) = self.path {
            path.iter()
                .map(|node| Some(*node))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        } else {
            [None; PATH_LENGTH]
        };

        // Get position as a PATH_LENGTH-bit bitstring (little-endian bit order).
        let pos: [Option<bool>; PATH_LENGTH] = {
            let pos: Option<[bool; PATH_LENGTH]> = self.leaf_pos.map(|pos| i2lebsp(pos as u64));
            let pos: [Option<bool>; PATH_LENGTH] = if let Some(pos) = pos {
                pos.iter()
                    .map(|pos| Some(*pos))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            } else {
                [None; PATH_LENGTH]
            };
            pos
        };

        let Q = self.domain.Q();

        let mut node = leaf;
        for (l_star, ((sibling, pos), chip)) in path.iter().zip(pos.iter()).zip(chips).enumerate() {
            // `l_star` = MERKLE_DEPTH_ORCHARD - layer - 1, which is the index obtained from
            // enumerating this Merkle path (going from leaf to root).
            // For example, when `layer = 31` (the first sibling on the Merkle path),
            // we have `l_star` = 32 - 31 - 1 = 0.
            // On the other hand, when `layer = 0` (the final sibling on the Merkle path),
            // we have `l_star` = 32 - 0 - 1 = 31.
            let pair = {
                let pair = (node, *sibling);

                // Swap node and sibling if needed
                chip.swap(layouter.namespace(|| "node position"), pair, *pos)?
            };

            // Each `hash_layer` consists of 52 Sinsemilla words:
            //  - l_star (10 bits) = 1 word
            //  - left (255 bits) || right (255 bits) = 51 words (510 bits)
            node = chip.hash_layer(
                layouter.namespace(|| format!("hash l_star {}", l_star)),
                Q,
                l_star,
                pair.0,
                pair.1,
            )?;
        }

        Ok(node)
    }
}
