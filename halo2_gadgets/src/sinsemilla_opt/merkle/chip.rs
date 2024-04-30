//! Chip implementing a Merkle hash using Sinsemilla as the hash function.

use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{ConstraintSystem, Error},
};
use pasta_curves::pallas;

use crate::sinsemilla::MessagePiece;
use crate::utilities::RangeConstrained;
use crate::{
    sinsemilla::{
        merkle::{
            chip::{MerkleChip, MerkleConfig},
            MerkleInstructions,
        },
        primitives as sinsemilla,
    },
    sinsemilla_opt::chip::SinsemillaChipOptimized,
    utilities_opt::lookup_range_check::DefaultLookupRangeCheckConfigOptimized,
    {
        ecc::FixedPoints,
        sinsemilla::{chip::SinsemillaConfig, CommitDomains, HashDomains, SinsemillaInstructions},
        utilities::{cond_swap::CondSwapInstructions, UtilitiesInstructions},
    },
};

/// Chip implementing `MerkleInstructions`.
///
/// This chip specifically implements `MerkleInstructions::hash_layer` as the `MerkleCRH`
/// function `hash = SinsemillaHash(Q, 𝑙⋆ || left⋆ || right⋆)`, where:
/// - `𝑙⋆ = I2LEBSP_10(l)`
/// - `left⋆ = I2LEBSP_255(left)`
/// - `right⋆ = I2LEBSP_255(right)`
///
/// This chip does **NOT** constrain `left⋆` and `right⋆` to be canonical encodings of
/// `left` and `right`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleChipOptimized<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    base: MerkleChip<Hash, Commit, Fixed, DefaultLookupRangeCheckConfigOptimized>,
}

impl<Hash, Commit, Fixed> Chip<pallas::Base> for MerkleChipOptimized<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    type Config = MerkleConfig<Hash, Commit, Fixed, DefaultLookupRangeCheckConfigOptimized>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.base.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Hash, Commit, F> MerkleChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    /// Configures the [`MerkleChip`].
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        sinsemilla_config: SinsemillaConfig<
            Hash,
            Commit,
            F,
            DefaultLookupRangeCheckConfigOptimized,
        >,
    ) -> MerkleConfig<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized> {
        MerkleChip::configure(meta, sinsemilla_config)
    }

    /// Constructs a [`MerkleChip`] given a [`MerkleConfig`].
    pub fn construct(
        config: MerkleConfig<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized>,
    ) -> Self {
        MerkleChipOptimized {
            base: MerkleChip { config },
        }
    }
}

impl<Hash, Commit, F, const MERKLE_DEPTH: usize>
    MerkleInstructions<pallas::Affine, MERKLE_DEPTH, { sinsemilla::K }, { sinsemilla::C }>
    for MerkleChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine> + Eq,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash> + Eq,
{
    // Todo: simplify the body?
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
        // Todo: simplify the body? copied and pasted from the body of the hash_layer function in halo2_gadgets/src/sinsemilla/merkle/chip.rs

        let config = self.base.config().clone();

        // We need to hash `l || left || right`, where `l` is a 10-bit value.
        // We allow `left` and `right` to be non-canonical 255-bit encodings.
        //
        // a = a_0||a_1 = l || (bits 0..=239 of left)
        // b = b_0||b_1||b_2
        //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
        // c = bits 5..=254 of right
        //
        // We start by witnessing all of the individual pieces, and range-constraining the
        // short pieces b_1 and b_2.
        //
        // https://p.z.cash/halo2-0.1:sinsemilla-merkle-crh-bit-lengths?partial

        // `a = a_0||a_1` = `l` || (bits 0..=239 of `left`)
        let a = MessagePiece::from_subpieces(
            self.base.clone(),
            layouter.namespace(|| "Witness a = a_0 || a_1"),
            [
                RangeConstrained::bitrange_of(Value::known(&pallas::Base::from(l as u64)), 0..10),
                RangeConstrained::bitrange_of(left.value(), 0..240),
            ],
        )?;

        // b = b_0 || b_1 || b_2
        //   = (bits 240..=249 of left) || (bits 250..=254 of left) || (bits 0..=4 of right)
        let (b_1, b_2, b) = {
            // b_0 = (bits 240..=249 of `left`)
            let b_0 = RangeConstrained::bitrange_of(left.value(), 240..250);

            // b_1 = (bits 250..=254 of `left`)
            // Constrain b_1 to 5 bits.
            let b_1 = RangeConstrained::witness_short(
                &config.sinsemilla_config.lookup_config(),
                layouter.namespace(|| "b_1"),
                left.value(),
                250..(pallas::Base::NUM_BITS as usize),
            )?;

            // b_2 = (bits 0..=4 of `right`)
            // Constrain b_2 to 5 bits.
            let b_2 = RangeConstrained::witness_short(
                &config.sinsemilla_config.lookup_config(),
                layouter.namespace(|| "b_2"),
                right.value(),
                0..5,
            )?;

            let b = MessagePiece::from_subpieces(
                self.base.clone(),
                layouter.namespace(|| "Witness b = b_0 || b_1 || b_2"),
                [b_0, b_1.value(), b_2.value()],
            )?;

            (b_1, b_2, b)
        };

        // c = bits 5..=254 of `right`
        let c = MessagePiece::from_subpieces(
            self.base.clone(),
            layouter.namespace(|| "Witness c"),
            [RangeConstrained::bitrange_of(
                right.value(),
                5..(pallas::Base::NUM_BITS as usize),
            )],
        )?;

        // hash = SinsemillaHash(Q, 𝑙⋆ || left⋆ || right⋆)
        //
        // `hash = ⊥` is handled internally to `SinsemillaChip::hash_to_point`: incomplete
        // addition constraints allows ⊥ to occur, and then during synthesis it detects
        // these edge cases and raises an error (aborting proof creation).
        //
        // Note that MerkleCRH as-defined maps ⊥ to 0. This is for completeness outside
        // the circuit (so that the ⊥ does not propagate into the type system). The chip
        // explicitly doesn't map ⊥ to 0; in fact it cannot, as doing so would require
        // constraints that amount to using complete addition. The rationale for excluding
        // this map is the same as why Sinsemilla uses incomplete addition: this situation
        // yields a nontrivial discrete log relation, and by assumption it is hard to find
        // these.
        //
        // https://p.z.cash/proto:merkle-crh-orchard
        let (point, zs) = self.hash_to_point(
            layouter.namespace(|| format!("hash at l = {}", l)),
            Q,
            vec![a.inner(), b.inner(), c.inner()].into(),
        )?;
        let hash = Self::extract(&point);

        // `SinsemillaChip::hash_to_point` returns the running sum for each `MessagePiece`.
        // Grab the outputs we need for the decomposition constraints.
        let z1_a = zs[0][1].clone();
        let z1_b = zs[1][1].clone();

        // Check that the pieces have been decomposed properly.
        //
        // The pieces and subpieces are arranged in the following configuration:
        // |  A_0  |  A_1  |  A_2  |  A_3  |  A_4  | q_decompose |
        // -------------------------------------------------------
        // |   a   |   b   |   c   |  left | right |      1      |
        // |  z1_a |  z1_b |  b_1  |  b_2  |   l   |      0      |
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
                    a.inner().cell_value().copy_advice(
                        || "copy a",
                        &mut region,
                        config.advices[0],
                        0,
                    )?;
                    // Copy and assign `b` at the correct position.
                    b.inner().cell_value().copy_advice(
                        || "copy b",
                        &mut region,
                        config.advices[1],
                        0,
                    )?;
                    // Copy and assign `c` at the correct position.
                    c.inner().cell_value().copy_advice(
                        || "copy c",
                        &mut region,
                        config.advices[2],
                        0,
                    )?;
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
                    b_1.inner()
                        .copy_advice(|| "b_1", &mut region, config.advices[2], 1)?;
                    // Copy `b_2`, which has been constrained to be a 5-bit value
                    b_2.inner()
                        .copy_advice(|| "b_2", &mut region, config.advices[3], 1)?;

                    Ok(())
                },
            )?;
        }
        // Check layer hash output against Sinsemilla primitives hash
        #[cfg(test)]
        {
            use crate::{sinsemilla::primitives::HashDomain, utilities::i2lebsp};

            use group::ff::PrimeFieldBits;

            left.value()
                .zip(right.value())
                .zip(hash.value())
                .assert_if_known(|((left, right), hash)| {
                    let l = i2lebsp::<10>(l as u64);
                    let left: Vec<_> = left
                        .to_le_bits()
                        .iter()
                        .by_vals()
                        .take(pallas::Base::NUM_BITS as usize)
                        .collect();
                    let right: Vec<_> = right
                        .to_le_bits()
                        .iter()
                        .by_vals()
                        .take(pallas::Base::NUM_BITS as usize)
                        .collect();
                    let merkle_crh = HashDomain::from_Q(Q.into());

                    let mut message = l.to_vec();
                    message.extend_from_slice(&left);
                    message.extend_from_slice(&right);

                    let expected = merkle_crh.hash(message.into_iter()).unwrap();

                    expected.to_repr() == hash.to_repr()
                });
        }

        Ok(hash)
    }
}

impl<Hash, Commit, F> UtilitiesInstructions<pallas::Base> for MerkleChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    type Var = AssignedCell<pallas::Base, pallas::Base>;
}

impl<Hash, Commit, F> CondSwapInstructions<pallas::Base> for MerkleChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    #[allow(clippy::type_complexity)]
    fn swap(
        &self,
        layouter: impl Layouter<pallas::Base>,
        pair: (Self::Var, Value<pallas::Base>),
        swap: Value<bool>,
    ) -> Result<(Self::Var, Self::Var), Error> {
        self.base.swap(layouter, pair, swap)
    }
}

impl<Hash, Commit, F> SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for MerkleChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    type CellValue = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::CellValue;

    type Message = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::Message;
    type MessagePiece = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::MessagePiece;
    type RunningSum = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::RunningSum;

    type X = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::X;
    type NonIdentityPoint = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::NonIdentityPoint;
    type FixedPoints = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::FixedPoints;

    type HashDomains = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::HashDomains;
    type CommitDomains = <SinsemillaChipOptimized<Hash, Commit, F> as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::CommitDomains;

    fn witness_message_piece(
        &self,
        layouter: impl Layouter<pallas::Base>,
        value: Value<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChipOptimized::<Hash, Commit, F>::construct(config);
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
        let chip = SinsemillaChipOptimized::<Hash, Commit, F>::construct(config);
        chip.hash_to_point(layouter, Q, message)
    }

    fn extract(point: &Self::NonIdentityPoint) -> Self::X {
        SinsemillaChipOptimized::<Hash, Commit, F>::extract(point)
    }
}
