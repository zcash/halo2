//! Chip implementations for the Sinsemilla gadgets.

use super::{
    message::{Message, MessagePiece},
    primitives as sinsemilla, CommitDomains, HashDomains, SinsemillaInstructions,
};
use crate::{
    ecc::{chip::NonIdentityEccPoint, FixedPoints},
    utilities::{double_and_add::DoubleAndAdd, lookup_range_check::LookupRangeCheckConfig},
};
use std::marker::PhantomData;

use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{
        Advice, Column, ConstraintSystem, Constraints, Error, Expression, Fixed, Selector,
        TableColumn, VirtualCells,
    },
    poly::Rotation,
};
use pasta_curves::pallas;

mod generator_table;
use generator_table::GeneratorTableConfig;

mod hash_to_point;

/// Configuration for the Sinsemilla hash chip
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaConfig<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    /// Binary selector used in lookup argument and in the body of the Sinsemilla hash.
    q_sinsemilla1: Selector,
    /// Non-binary selector used in lookup argument and in the body of the Sinsemilla hash.
    q_sinsemilla2: Column<Fixed>,
    /// q_sinsemilla2 is used to define a synthetic selector,
    ///         q_sinsemilla3 = (q_sinsemilla2) ⋅ (q_sinsemilla2 - 1)
    /// Simple selector used to constrain hash initialization to be consistent with
    /// the y-coordinate of the domain $Q$.
    q_sinsemilla4: Selector,
    /// Fixed column used to load the y-coordinate of the domain $Q$.
    fixed_y_q: Column<Fixed>,
    /// Logic specific to merged double-and-add.
    double_and_add: DoubleAndAdd<pallas::Affine>,
    /// Advice column used to load the message.
    bits: Column<Advice>,
    /// Advice column used to witness message pieces. This may or may not be the same
    /// column as `bits`.
    witness_pieces: Column<Advice>,
    /// The lookup table where $(\mathsf{idx}, x_p, y_p)$ are loaded for the $2^K$
    /// generators of the Sinsemilla hash.
    pub(super) generator_table: GeneratorTableConfig,
    /// An advice column configured to perform lookup range checks.
    lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    _marker: PhantomData<(Hash, Commit, F)>,
}

impl<Hash, Commit, F> SinsemillaConfig<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    /// Returns an array of all advice columns in this config, in arbitrary order.
    pub(super) fn advices(&self) -> [Column<Advice>; 5] {
        [
            self.double_and_add.x_a,
            self.double_and_add.x_p,
            self.bits,
            self.double_and_add.lambda_1,
            self.double_and_add.lambda_2,
        ]
    }

    /// Returns the lookup range check config used in this config.
    pub fn lookup_config(&self) -> LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }> {
        self.lookup_config
    }
}

/// Derives the expression `q_s3 = (q_s2) * (q_s2 - 1)`.
fn q_s3(
    meta: &mut VirtualCells<pallas::Base>,
    q_sinsemilla2: Column<Fixed>,
) -> Expression<pallas::Base> {
    let one = Expression::Constant(pallas::Base::one());
    let q_s2 = meta.query_fixed(q_sinsemilla2);
    q_s2.clone() * (q_s2 - one)
}

/// A chip that implements 10-bit Sinsemilla using a lookup table and 5 advice columns.
///
/// [Chip description](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html#plonk--halo-2-constraints).
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaChip<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    config: SinsemillaConfig<Hash, Commit, Fixed>,
}

impl<Hash, Commit, Fixed> Chip<pallas::Base> for SinsemillaChip<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    type Config = SinsemillaConfig<Hash, Commit, Fixed>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Hash, Commit, F> SinsemillaChip<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    /// Reconstructs this chip from the given config.
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self { config }
    }

    /// Loads the lookup table required by this chip into the circuit.
    pub fn load(
        config: SinsemillaConfig<Hash, Commit, F>,
        layouter: &mut impl Layouter<pallas::Base>,
    ) -> Result<<Self as Chip<pallas::Base>>::Loaded, Error> {
        // Load the lookup table.
        config.generator_table.load(layouter)
    }

    /// # Side-effects
    ///
    /// All columns in `advices` and will be equality-enabled.
    #[allow(clippy::too_many_arguments)]
    #[allow(non_snake_case)]
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 5],
        witness_pieces: Column<Advice>,
        fixed_y_q: Column<Fixed>,
        lookup: (TableColumn, TableColumn, TableColumn),
        range_check: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    ) -> <Self as Chip<pallas::Base>>::Config {
        // Enable equality on all advice columns
        for advice in advices.iter() {
            meta.enable_equality(*advice);
        }

        let q_sinsemilla1 = meta.complex_selector();
        let q_sinsemilla2 = meta.fixed_column();
        let q_sinsemilla4 = meta.selector();

        // Gradient check selector: q_s1 * (1 - q_s3 / 2)
        let gradient_selector = |meta: &mut VirtualCells<pallas::Base>| {
            let q_s1 = meta.query_selector(q_sinsemilla1);
            let q_s3_div_2 = q_s3(meta, q_sinsemilla2) * pallas::Base::TWO_INV;
            let one = Expression::Constant(pallas::Base::one());

            q_s1 * (one - q_s3_div_2)
        };

        let config = SinsemillaConfig::<Hash, Commit, F> {
            q_sinsemilla1,
            q_sinsemilla2,
            q_sinsemilla4,
            fixed_y_q,
            double_and_add: DoubleAndAdd::configure(
                meta,
                advices[0],
                advices[1],
                advices[3],
                advices[4],
                &|meta: &mut VirtualCells<pallas::Base>| meta.query_selector(q_sinsemilla1),
                &gradient_selector,
            ),
            bits: advices[2],
            witness_pieces,
            generator_table: GeneratorTableConfig {
                table_idx: lookup.0,
                table_x: lookup.1,
                table_y: lookup.2,
            },
            lookup_config: range_check,
            _marker: PhantomData,
        };

        // Set up lookup argument
        GeneratorTableConfig::configure(meta, config.clone());

        // Check that the initial x_A, x_P, lambda_1, lambda_2 are consistent with y_Q.
        // https://p.z.cash/halo2-0.1:sinsemilla-constraints?partial
        meta.create_gate("Initial y_Q", |meta| {
            let q_s4 = meta.query_selector(config.q_sinsemilla4);
            let y_q = meta.query_fixed(config.fixed_y_q);

            // y_a_derived = (lambda_1 + lambda_2) * (x_a - x_r)
            let y_a_derived = config.double_and_add.y_a(meta, Rotation::cur());

            Constraints::with_selector(q_s4, Some(("init_y_q_check", y_q - y_a_derived)))
        });

        // Final double-and-add gate
        meta.create_gate("final check", |meta| {
            // x_{A,i}
            let x_a_cur = meta.query_advice(config.double_and_add.x_a, Rotation::cur());
            // x_{A,i-1}
            let x_a_next = meta.query_advice(config.double_and_add.x_a, Rotation::next());
            // λ_{2,i}
            let lambda2_cur = meta.query_advice(config.double_and_add.lambda_2, Rotation::cur());
            let y_a_cur = config.double_and_add.y_a(meta, Rotation::cur());

            // Final round selector: q_s1 * (q_s3 / 2)
            let selector = {
                let q_s1 = meta.query_selector(q_sinsemilla1);
                let q_s3_div_2 = q_s3(meta, q_sinsemilla2) * pallas::Base::TWO_INV;

                q_s1 * q_s3_div_2
            };

            let y_a_check = {
                let lhs = lambda2_cur * (x_a_cur - x_a_next);
                let rhs = {
                    let y_a_final = meta.query_advice(advices[3], Rotation::next());
                    y_a_cur + y_a_final
                };
                lhs - rhs
            };

            Constraints::with_selector(selector, [("final y_a_check", y_a_check)])
        });

        config
    }
}

// Implement `SinsemillaInstructions` for `SinsemillaChip`
impl<Hash, Commit, F> SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChip<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    type CellValue = AssignedCell<pallas::Base, pallas::Base>;

    type Message = Message<pallas::Base, { sinsemilla::K }, { sinsemilla::C }>;
    type MessagePiece = MessagePiece<pallas::Base, { sinsemilla::K }>;

    type RunningSum = Vec<Self::CellValue>;

    type X = AssignedCell<pallas::Base, pallas::Base>;
    type NonIdentityPoint = NonIdentityEccPoint;
    type FixedPoints = F;

    type HashDomains = Hash;
    type CommitDomains = Commit;

    fn witness_message_piece(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        field_elem: Value<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().clone();

        let cell = layouter.assign_region(
            || "witness message piece",
            |mut region| {
                region.assign_advice(
                    || "witness message piece",
                    config.witness_pieces,
                    0,
                    || field_elem,
                )
            },
        )?;
        Ok(MessagePiece::new(cell, num_words))
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Self::RunningSum>), Error> {
        layouter.assign_region(
            || "hash_to_point",
            |mut region| self.hash_message(&mut region, Q, &message),
        )
    }

    fn extract(point: &Self::NonIdentityPoint) -> Self::X {
        point.x()
    }
}
