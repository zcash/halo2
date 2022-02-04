//! Chip implementing a Merkle hash using Sinsemilla as the hash function.

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use super::MerkleInstructions;

use crate::{
    primitives::sinsemilla,
    {
        ecc::FixedPoints,
        sinsemilla::{
            chip::{SinsemillaChip, SinsemillaConfig},
            CommitDomains, HashDomains, SinsemillaInstructions,
        },
        utilities::{
            bitrange_subset,
            cond_swap::{CondSwapChip, CondSwapConfig, CondSwapInstructions},
            UtilitiesInstructions,
        },
    },
};
use group::ff::PrimeField;
use std::array;

/// Configuration for the `MerkleChip` implementation.
#[derive(Clone, Debug)]
pub struct MerkleConfig<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    advices: [Column<Advice>; 5],
    q_decompose: Selector,
    pub(super) cond_swap_config: CondSwapConfig,
    pub(super) sinsemilla_config: SinsemillaConfig<Hash, Commit, Fixed>,
}

/// Chip implementing `MerkleInstructions`.
#[derive(Clone, Debug)]
pub struct MerkleChip<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    config: MerkleConfig<Hash, Commit, Fixed>,
}

impl<Hash, Commit, Fixed> Chip<pallas::Base> for MerkleChip<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    type Config = MerkleConfig<Hash, Commit, Fixed>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Hash, Commit, F> MerkleChip<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    /// Configures the [`MerkleChip`].
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        sinsemilla_config: SinsemillaConfig<Hash, Commit, F>,
    ) -> MerkleConfig<Hash, Commit, F> {
        // All five advice columns are equality-enabled by SinsemillaConfig.
        let advices = sinsemilla_config.advices();
        let cond_swap_config = CondSwapChip::configure(meta, advices);

        // This selector enables the decomposition gate.
        let q_decompose = meta.selector();

        // Check that pieces have been decomposed correctly for Sinsemilla hash.
        // <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
        //
        // a = a_0||a_1 = l || (bits 0..=239 of left)
        // b = b_0||b_1||b_2
        //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
        // c = bits 5..=254 of right
        //
        // The message pieces `a`, `b`, `c` are constrained by Sinsemilla to be
        // 250 bits, 20 bits, and 250 bits respectively.
        //
        /*
            The pieces and subpieces are arranged in the following configuration:
            |  A_0  |  A_1  |  A_2  |  A_3  |  A_4  | q_decompose |
            -------------------------------------------------------
            |   a   |   b   |   c   |  left | right |      1      |
            |  z1_a |  z1_b |  b_1  |  b_2  |   l   |             |
        */
        meta.create_gate("Decomposition check", |meta| {
            let q_decompose = meta.query_selector(q_decompose);
            let l_whole = meta.query_advice(advices[4], Rotation::next());

            let two_pow_5 = pallas::Base::from(1 << 5);
            let two_pow_10 = two_pow_5.square();

            // a_whole is constrained by Sinsemilla to be 250 bits.
            let a_whole = meta.query_advice(advices[0], Rotation::cur());
            // b_whole is constrained by Sinsemilla to be 20 bits.
            let b_whole = meta.query_advice(advices[1], Rotation::cur());
            // c_whole is constrained by Sinsemilla to be 250 bits.
            let c_whole = meta.query_advice(advices[2], Rotation::cur());
            let left_node = meta.query_advice(advices[3], Rotation::cur());
            let right_node = meta.query_advice(advices[4], Rotation::cur());

            // a = a_0||a_1 = l || (bits 0..=239 of left)
            // Check that a_0 = l
            //
            // z_1 of SinsemillaHash(a) = a_1
            let z1_a = meta.query_advice(advices[0], Rotation::next());
            let a_1 = z1_a;
            // a_0 = a - (a_1 * 2^10)
            let a_0 = a_whole - a_1.clone() * pallas::Base::from(1 << 10);
            let l_check = a_0 - l_whole;

            // b = b_0||b_1||b_2
            //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
            // The Orchard specification allows this representation to be non-canonical.
            // <https://zips.z.cash/protocol/protocol.pdf#merklepath>
            //
            //    z_1 of SinsemillaHash(b) = b_1 + 2^5 b_2
            // => b_0 = b - (z1_b * 2^10)
            let z1_b = meta.query_advice(advices[1], Rotation::next());
            // b_1 has been constrained to be 5 bits outside this gate.
            let b_1 = meta.query_advice(advices[2], Rotation::next());
            // b_2 has been constrained to be 5 bits outside this gate.
            let b_2 = meta.query_advice(advices[3], Rotation::next());
            // Constrain b_1 + 2^5 b_2 = z1_b
            let b1_b2_check = z1_b.clone() - (b_1.clone() + b_2.clone() * two_pow_5);
            // Derive b_0 (constrained by SinsemillaHash to be 10 bits)
            let b_0 = b_whole - (z1_b * two_pow_10);

            // Check that left = a_1 (240 bits) || b_0 (10 bits) || b_1 (5 bits)
            let left_check = {
                let reconstructed = {
                    let two_pow_240 = pallas::Base::from_u128(1 << 120).square();
                    a_1 + (b_0 + b_1 * two_pow_10) * two_pow_240
                };
                reconstructed - left_node
            };

            // Check that right = b_2 (5 bits) || c (250 bits)
            // The Orchard specification allows this representation to be non-canonical.
            // <https://zips.z.cash/protocol/protocol.pdf#merklepath>
            let right_check = b_2 + c_whole * two_pow_5 - right_node;

            array::IntoIter::new([
                ("l_check", l_check),
                ("left_check", left_check),
                ("right_check", right_check),
                ("b1_b2_check", b1_b2_check),
            ])
            .map(move |(name, poly)| (name, q_decompose.clone() * poly))
        });

        MerkleConfig {
            advices,
            q_decompose,
            cond_swap_config,
            sinsemilla_config,
        }
    }

    /// Constructs a [`MerkleChip`] given a [`MerkleConfig`].
    pub fn construct(config: MerkleConfig<Hash, Commit, F>) -> Self {
        MerkleChip { config }
    }
}

impl<Hash, Commit, F, const MERKLE_DEPTH: usize>
    MerkleInstructions<pallas::Affine, MERKLE_DEPTH, { sinsemilla::K }, { sinsemilla::C }>
    for MerkleChip<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    #[allow(non_snake_case)]
    fn hash_layer(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        // l = MERKLE_DEPTH - layer - 1
        l: usize,
        left: Self::Var,
        right: Self::Var,
    ) -> Result<Self::Var, Error> {
        let config = self.config().clone();

        // <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
        // We need to hash `l || left || right`, where `l` is a 10-bit value.
        // We allow `left` and `right` to be non-canonical 255-bit encodings.
        //
        // a = a_0||a_1 = l || (bits 0..=239 of left)
        // b = b_0||b_1||b_2
        //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
        // c = bits 5..=254 of right

        // `a = a_0||a_1` = `l` || (bits 0..=239 of `left`)
        let a = {
            let a = {
                // a_0 = l
                let a_0 = bitrange_subset(&pallas::Base::from(l as u64), 0..10);

                // a_1 = (bits 0..=239 of `left`)
                let a_1 = left.value().map(|value| bitrange_subset(value, 0..240));

                a_1.map(|a_1| a_0 + a_1 * pallas::Base::from(1 << 10))
            };

            self.witness_message_piece(layouter.namespace(|| "Witness a = a_0 || a_1"), a, 25)?
        };

        // b = b_0 || b_1 || b_2
        //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
        let (b_1, b_2, b) = {
            // b_0 = (bits 240..=249 of `left`)
            let b_0 = left.value().map(|value| bitrange_subset(value, 240..250));

            // b_1 = (bits 250..=254 of `left`)
            // Constrain b_1 to 5 bits.
            let b_1 = {
                let b_1 = left
                    .value()
                    .map(|value| bitrange_subset(value, 250..(pallas::Base::NUM_BITS as usize)));

                config
                    .sinsemilla_config
                    .lookup_config()
                    .witness_short_check(layouter.namespace(|| "Constrain b_1 to 5 bits"), b_1, 5)?
            };

            // b_2 = (bits 0..=4 of `right`)
            // Constrain b_2 to 5 bits.
            let b_2 = {
                let b_2 = right.value().map(|value| bitrange_subset(value, 0..5));

                config
                    .sinsemilla_config
                    .lookup_config()
                    .witness_short_check(layouter.namespace(|| "Constrain b_2 to 5 bits"), b_2, 5)?
            };

            let b = {
                let b = b_0
                    .zip(b_1.value())
                    .zip(b_2.value())
                    .map(|((b_0, b_1), b_2)| {
                        b_0 + b_1 * pallas::Base::from(1 << 10) + b_2 * pallas::Base::from(1 << 15)
                    });
                self.witness_message_piece(
                    layouter.namespace(|| "Witness b = b_0 || b_1 || b_2"),
                    b,
                    2,
                )?
            };

            (b_1, b_2, b)
        };

        let c = {
            // `c = bits 5..=254 of `right`
            let c = right
                .value()
                .map(|value| bitrange_subset(value, 5..(pallas::Base::NUM_BITS as usize)));
            self.witness_message_piece(layouter.namespace(|| "Witness c"), c, 25)?
        };

        let (point, zs) = self.hash_to_point(
            layouter.namespace(|| format!("hash at l = {}", l)),
            Q,
            vec![a.clone(), b.clone(), c.clone()].into(),
        )?;
        let z1_a = zs[0][1].clone();
        let z1_b = zs[1][1].clone();

        // Check that the pieces have been decomposed properly.
        /*
            The pieces and subpieces are arranged in the following configuration:
            |  A_0  |  A_1  |  A_2  |  A_3  |  A_4  | q_decompose |
            -------------------------------------------------------
            |   a   |   b   |   c   |  left | right |      1      |
            |  z1_a |  z1_b |  b_1  |  b_2  |   l   |             |
        */
        {
            layouter.assign_region(
                || "Check piece decomposition",
                |mut region| {
                    // Set the fixed column `l` to the current l.
                    // Recall that l = MERKLE_DEPTH - layer - 1.
                    // The layer with 2^n nodes is called "layer n".
                    config.q_decompose.enable(&mut region, 0)?;
                    region.assign_advice_from_constant(
                        || format!("l {}", l),
                        config.advices[4],
                        1,
                        pallas::Base::from(l as u64),
                    )?;

                    // Offset 0
                    // Copy and assign `a` at the correct position.
                    a.cell_value()
                        .copy_advice(|| "copy a", &mut region, config.advices[0], 0)?;
                    // Copy and assign `b` at the correct position.
                    b.cell_value()
                        .copy_advice(|| "copy b", &mut region, config.advices[1], 0)?;
                    // Copy and assign `c` at the correct position.
                    c.cell_value()
                        .copy_advice(|| "copy c", &mut region, config.advices[2], 0)?;
                    // Copy and assign the left node at the correct position.
                    left.copy_advice(|| "left", &mut region, config.advices[3], 0)?;
                    // Copy and assign the right node at the correct position.
                    right.copy_advice(|| "right", &mut region, config.advices[4], 0)?;

                    // Offset 1
                    // Copy and assign z_1 of SinsemillaHash(a) = a_1
                    z1_a.copy_advice(|| "z1_a", &mut region, config.advices[0], 1)?;
                    // Copy and assign z_1 of SinsemillaHash(b) = b_1
                    z1_b.copy_advice(|| "z1_b", &mut region, config.advices[1], 1)?;
                    // Copy `b_1`, which has been constrained to be a 5-bit value
                    b_1.copy_advice(|| "b_1", &mut region, config.advices[2], 1)?;
                    // Copy `b_2`, which has been constrained to be a 5-bit value
                    b_2.copy_advice(|| "b_2", &mut region, config.advices[3], 1)?;

                    Ok(())
                },
            )?;
        }

        let result = Self::extract(&point);

        // Check layer hash output against Sinsemilla primitives hash
        #[cfg(test)]
        {
            use crate::{primitives::sinsemilla::HashDomain, utilities::i2lebsp};
            use group::ff::PrimeFieldBits;

            if let (Some(left), Some(right)) = (left.value(), right.value()) {
                let l = i2lebsp::<10>(l as u64);
                let left: Vec<_> = left
                    .to_le_bits()
                    .iter()
                    .by_val()
                    .take(pallas::Base::NUM_BITS as usize)
                    .collect();
                let right: Vec<_> = right
                    .to_le_bits()
                    .iter()
                    .by_val()
                    .take(pallas::Base::NUM_BITS as usize)
                    .collect();
                let merkle_crh = HashDomain::from_Q(Q.into());

                let mut message = l.to_vec();
                message.extend_from_slice(&left);
                message.extend_from_slice(&right);

                let expected = merkle_crh.hash(message.into_iter()).unwrap();

                assert_eq!(expected.to_repr(), result.value().unwrap().to_repr());
            }
        }

        Ok(result)
    }
}

impl<Hash, Commit, F> UtilitiesInstructions<pallas::Base> for MerkleChip<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    type Var = AssignedCell<pallas::Base, pallas::Base>;
}

impl<Hash, Commit, F> CondSwapInstructions<pallas::Base> for MerkleChip<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    #[allow(clippy::type_complexity)]
    fn swap(
        &self,
        layouter: impl Layouter<pallas::Base>,
        pair: (Self::Var, Option<pallas::Base>),
        swap: Option<bool>,
    ) -> Result<(Self::Var, Self::Var), Error> {
        let config = self.config().cond_swap_config.clone();
        let chip = CondSwapChip::<pallas::Base>::construct(config);
        chip.swap(layouter, pair, swap)
    }
}

impl<Hash, Commit, F> SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for MerkleChip<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    type CellValue = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::CellValue;

    type Message = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::Message;
    type MessagePiece = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::MessagePiece;
    type RunningSum = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::RunningSum;

    type X = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::X;
    type NonIdentityPoint = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::NonIdentityPoint;
    type FixedPoints = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::FixedPoints;

    type HashDomains = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::HashDomains;
    type CommitDomains = <SinsemillaChip<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::CommitDomains;

    fn witness_message_piece(
        &self,
        layouter: impl Layouter<pallas::Base>,
        value: Option<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::<Hash, Commit, F>::construct(config);
        chip.witness_message_piece(layouter, value, num_words)
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Vec<Self::CellValue>>), Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::<Hash, Commit, F>::construct(config);
        chip.hash_to_point(layouter, Q, message)
    }

    fn extract(point: &Self::NonIdentityPoint) -> Self::X {
        SinsemillaChip::<Hash, Commit, F>::extract(point)
    }
}
