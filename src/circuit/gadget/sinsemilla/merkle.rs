use halo2::{
    circuit::{Chip, Layouter},
    plonk::Error,
};
use pasta_curves::arithmetic::CurveAffine;

use super::{HashDomains, SinsemillaInstructions};

use crate::{
    circuit::gadget::utilities::{
        cond_swap::CondSwapInstructions, transpose_option_array, UtilitiesInstructions,
    },
    spec::i2lebsp,
};
use std::iter;

pub(in crate::circuit) mod chip;

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
    /// Compute MerkleCRH for a given `layer`. The hash that computes the root
    /// is at layer 0, and the hashes that are applied to two leaves are at
    /// layer `MERKLE_DEPTH_ORCHARD - 1` = layer 31.
    #[allow(non_snake_case)]
    fn hash_layer(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: C,
        l: usize,
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
    pub(in crate::circuit) chip_1: MerkleChip,
    pub(in crate::circuit) chip_2: MerkleChip,
    pub(in crate::circuit) domain: MerkleChip::HashDomains,
    pub(in crate::circuit) leaf_pos: Option<u32>,
    // The Merkle path is ordered from leaves to root.
    pub(in crate::circuit) path: Option<[C::Base; PATH_LENGTH]>,
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
    pub(in crate::circuit) fn calculate_root(
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

        // The Merkle path is ordered from leaves to root, which is consistent with the
        // little-endian representation of `pos` below.
        let path = transpose_option_array(self.path);

        // Get position as a PATH_LENGTH-bit bitstring (little-endian bit order).
        let pos: [Option<bool>; PATH_LENGTH] = {
            let pos: Option<[bool; PATH_LENGTH]> = self.leaf_pos.map(|pos| i2lebsp(pos as u64));
            transpose_option_array(pos)
        };

        let Q = self.domain.Q();

        let mut node = leaf;
        for (l, ((sibling, pos), chip)) in path.iter().zip(pos.iter()).zip(chips).enumerate() {
            // `l` = MERKLE_DEPTH_ORCHARD - layer - 1, which is the index obtained from
            // enumerating this Merkle path (going from leaf to root).
            // For example, when `layer = 31` (the first sibling on the Merkle path),
            // we have `l` = 32 - 31 - 1 = 0.
            // On the other hand, when `layer = 0` (the final sibling on the Merkle path),
            // we have `l` = 32 - 0 - 1 = 31.
            let pair = {
                let pair = (node, *sibling);

                // Swap node and sibling if needed
                chip.swap(layouter.namespace(|| "node position"), pair, *pos)?
            };

            // Each `hash_layer` consists of 52 Sinsemilla words:
            //  - l (10 bits) = 1 word
            //  - left (255 bits) || right (255 bits) = 51 words (510 bits)
            node = chip.hash_layer(
                layouter.namespace(|| format!("hash l {}", l)),
                Q,
                l,
                pair.0,
                pair.1,
            )?;
        }

        Ok(node)
    }
}

#[cfg(test)]
pub mod tests {
    use super::{
        chip::{MerkleChip, MerkleConfig},
        MerklePath,
    };

    use crate::{
        circuit::gadget::{
            sinsemilla::chip::{SinsemillaChip, SinsemillaHashDomains},
            utilities::{lookup_range_check::LookupRangeCheckConfig, UtilitiesInstructions, Var},
        },
        constants::MERKLE_DEPTH_ORCHARD,
        note::commitment::ExtractedNoteCommitment,
        tree,
    };

    use halo2::{
        arithmetic::FieldExt,
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        pasta::pallas,
        plonk::{Circuit, ConstraintSystem, Error},
    };

    use rand::random;
    use std::convert::TryInto;

    #[derive(Default)]
    struct MyCircuit {
        leaf: Option<pallas::Base>,
        leaf_pos: Option<u32>,
        merkle_path: Option<[pallas::Base; MERKLE_DEPTH_ORCHARD]>,
    }

    impl Circuit<pallas::Base> for MyCircuit {
        type Config = (MerkleConfig, MerkleConfig);
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
            let advices = [
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            ];

            // Shared fixed column for loading constants
            let constants = meta.fixed_column();
            meta.enable_constant(constants);

            // NB: In the actual Action circuit, these fixed columns will be reused
            // by other chips. For this test, we are creating new fixed columns.
            let fixed_y_q_1 = meta.fixed_column();
            let fixed_y_q_2 = meta.fixed_column();

            // Fixed columns for the Sinsemilla generator lookup table
            let lookup = (
                meta.lookup_table_column(),
                meta.lookup_table_column(),
                meta.lookup_table_column(),
            );

            let range_check = LookupRangeCheckConfig::configure(meta, advices[9], lookup.0);

            let sinsemilla_config_1 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                advices[7],
                fixed_y_q_1,
                lookup,
                range_check.clone(),
            );
            let config1 = MerkleChip::configure(meta, sinsemilla_config_1);

            let sinsemilla_config_2 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                advices[2],
                fixed_y_q_2,
                lookup,
                range_check,
            );
            let config2 = MerkleChip::configure(meta, sinsemilla_config_2);

            (config1, config2)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            // Load generator table (shared across both configs)
            SinsemillaChip::load(config.0.sinsemilla_config.clone(), &mut layouter)?;

            // Construct Merkle chips which will be placed side-by-side in the circuit.
            let chip_1 = MerkleChip::construct(config.0.clone());
            let chip_2 = MerkleChip::construct(config.1.clone());

            let leaf = chip_1.load_private(
                layouter.namespace(|| ""),
                config.0.cond_swap_config.a,
                self.leaf,
            )?;

            let path = MerklePath {
                chip_1,
                chip_2,
                domain: SinsemillaHashDomains::MerkleCrh,
                leaf_pos: self.leaf_pos,
                path: self.merkle_path,
            };

            let computed_final_root =
                path.calculate_root(layouter.namespace(|| "calculate root"), leaf)?;

            if let Some(leaf_pos) = self.leaf_pos {
                // The expected final root
                let final_root = {
                    let path = tree::MerklePath::new(leaf_pos, self.merkle_path.unwrap());
                    let leaf = ExtractedNoteCommitment::from_bytes(&self.leaf.unwrap().to_bytes())
                        .unwrap();
                    path.root(leaf)
                };

                // Check the computed final root against the expected final root.
                assert_eq!(computed_final_root.value().unwrap(), final_root.inner());
            }

            Ok(())
        }
    }

    #[test]
    fn merkle_chip() {
        // Choose a random leaf and position
        let leaf = pallas::Base::rand();
        let pos = random::<u32>();

        // Choose a path of random inner nodes
        let path: Vec<_> = (0..(MERKLE_DEPTH_ORCHARD))
            .map(|_| pallas::Base::rand())
            .collect();

        // The root is provided as a public input in the Orchard circuit.

        let circuit = MyCircuit {
            leaf: Some(leaf),
            leaf_pos: Some(pos),
            merkle_path: Some(path.try_into().unwrap()),
        };

        let prover = MockProver::run(11, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_merkle_chip() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("merkle-path-layout.png", (1024, 7680)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("MerkleCRH Path", ("sans-serif", 60)).unwrap();

        let circuit = MyCircuit::default();
        halo2::dev::CircuitLayout::default()
            .show_labels(false)
            .render(11, &circuit, &root)
            .unwrap();
    }
}
