//! Chip implementations for the Sinsemilla gadgets.

use super::{
    message::{Message, MessagePiece},
    primitives as sinsemilla, CommitDomains, HashDomains, SinsemillaInstructions,
};
use crate::{
    ecc::{
        chip::{DoubleAndAdd, NonIdentityEccPoint},
        FixedPoints,
    },
    utilities::lookup_range_check::DefaultLookupRangeCheck,
};
use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{
        Advice, Column, ConstraintSystem, Constraints, Error, Expression, Fixed, Selector,
        VirtualCells,
    },
    poly::Rotation,
};
use pasta_curves::pallas;
use pasta_curves::pallas::Base;
pub(crate) mod generator_table;
use crate::sinsemilla::chip::generator_table::{DefaultGeneratorTable};

pub(crate) mod hash_to_point;

/// Configuration for the Sinsemilla hash chip
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaConfig<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
{
    /// Binary selector used in lookup argument and in the body of the Sinsemilla hash.
    pub(crate) q_sinsemilla1: Selector,
    /// Non-binary selector used in lookup argument and in the body of the Sinsemilla hash.
    pub(crate) q_sinsemilla2: Column<Fixed>,
    /// q_sinsemilla2 is used to define a synthetic selector,
    ///         q_sinsemilla3 = (q_sinsemilla2) ⋅ (q_sinsemilla2 - 1)
    /// Simple selector used to constrain hash initialization to be consistent with
    /// the y-coordinate of the domain $Q$.
    pub(crate) q_sinsemilla4: Selector,
    /// Fixed column used to load the y-coordinate of the domain $Q$.
    pub(crate) fixed_y_q: Column<Fixed>,
    /// Logic specific to merged double-and-add.
    pub(crate) double_and_add: DoubleAndAdd,
    /// Advice column used to load the message.
    pub(crate) bits: Column<Advice>,
    /// Advice column used to witness message pieces. This may or may not be the same
    /// column as `bits`.
    pub(crate) witness_pieces: Column<Advice>,
    /// The lookup table where $(\mathsf{idx}, x_p, y_p)$ are loaded for the $2^K$
    /// generators of the Sinsemilla hash.
    pub(crate) generator_table: GeneratorTableConfigType,
    /// An advice column configured to perform lookup range checks.
    pub(crate) lookup_config: LookupRangeCheckConfig,
    _marker: PhantomData<(Hash, Commit, F)>,
}

impl<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
    SinsemillaConfig<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
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
    pub fn lookup_config(&self) -> LookupRangeCheckConfig {
        self.lookup_config
    }

    /// Derives the expression `q_s3 = (q_s2) * (q_s2 - 1)`.
    fn q_s3(&self, meta: &mut VirtualCells<pallas::Base>) -> Expression<pallas::Base> {
        let one = Expression::Constant(pallas::Base::one());
        let q_s2 = meta.query_fixed(self.q_sinsemilla2);
        q_s2.clone() * (q_s2 - one)
    }
}

/// A chip that implements 10-bit Sinsemilla using a lookup table and 5 advice columns.
///
/// [Chip description](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html#plonk--halo-2-constraints).
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaChip<Hash, Commit, Fixed, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
{
    config: SinsemillaConfig<Hash, Commit, Fixed, LookupRangeCheckConfig, GeneratorTableConfigType>,
}

impl<Hash, Commit, Fixed, LookupRangeCheckConfig, GeneratorTableConfigType> Chip<pallas::Base>
    for SinsemillaChip<Hash, Commit, Fixed, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
{
    type Config =
        SinsemillaConfig<Hash, Commit, Fixed, LookupRangeCheckConfig, GeneratorTableConfigType>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
    SinsemillaChip<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
{
    /// Reconstructs this chip from the given config.
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self { config }
    }

    /// Loads the lookup table required by this chip into the circuit.
    pub fn load(
        config: SinsemillaConfig<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>,
        layouter: &mut impl Layouter<pallas::Base>,
    ) -> Result<<Self as Chip<pallas::Base>>::Loaded, Error> {
        // Load the lookup table.
        config.generator_table.load(layouter)
    }

    /// Query a fixed value from the circuit's fixed column using the configuration `fixed_y_q`.
    fn get_y_q_fixed(
        meta: &mut VirtualCells<Base>,
        config: &SinsemillaConfig<
            Hash,
            Commit,
            F,
            LookupRangeCheckConfig,
            GeneratorTableConfigType,
        >,
    ) -> Expression<Base> {
        meta.query_fixed(config.fixed_y_q)
    }

    /// Query an advice value 'y_q' from a specific advice column `x_p` at the previous rotation.
    fn get_y_q_advice(
        meta: &mut VirtualCells<Base>,
        config: &SinsemillaConfig<
            Hash,
            Commit,
            F,
            LookupRangeCheckConfig,
            GeneratorTableConfigType,
        >,
    ) -> Expression<Base> {
        meta.query_advice(config.double_and_add.x_p, Rotation::prev())
    }

    #[allow(non_snake_case)]
    pub(crate) fn create_initial_y_q_gate(
        is_Q_public: bool,
        meta: &mut ConstraintSystem<pallas::Base>,
        config: &SinsemillaConfig<
            Hash,
            Commit,
            F,
            LookupRangeCheckConfig,
            GeneratorTableConfigType,
        >,
    ) {
        let two = pallas::Base::from(2);

        // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
        let Y_A = |meta: &mut VirtualCells<pallas::Base>, rotation| {
            config.double_and_add.Y_A(meta, rotation)
        };

        // Check that the initial x_A, x_P, lambda_1, lambda_2 are consistent with y_Q.
        // https://p.z.cash/halo2-0.1:sinsemilla-constraints?partial
        meta.create_gate("Initial y_Q", |meta| {
            let q_s4 = meta.query_selector(config.q_sinsemilla4);
            // fixme: how to change to optimized get_y_q in a simple way?
            let y_q = if is_Q_public {
                Self::get_y_q_fixed(meta, &config)
            } else {
                Self::get_y_q_advice(meta, &config)
            };

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A_cur = Y_A(meta, Rotation::cur());

            // 2 * y_q - Y_{A,0} = 0
            let init_y_q_check = y_q * two - Y_A_cur;

            Constraints::with_selector(q_s4, Some(("init_y_q_check", init_y_q_check)))
        });
    }

    #[allow(non_snake_case)]
    pub(crate) fn create_sinsemilla_gate(
        meta: &mut ConstraintSystem<pallas::Base>,
        config: &SinsemillaConfig<
            Hash,
            Commit,
            F,
            LookupRangeCheckConfig,
            GeneratorTableConfigType,
        >,
    ) {
        let two = pallas::Base::from(2);

        // Closures for expressions that are derived multiple times
        // x_r = lambda_1^2 - x_a - x_p
        let x_r = |meta: &mut VirtualCells<pallas::Base>, rotation| {
            config.double_and_add.x_r(meta, rotation)
        };

        // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
        let Y_A = |meta: &mut VirtualCells<pallas::Base>, rotation| {
            config.double_and_add.Y_A(meta, rotation)
        };

        // https://p.z.cash/halo2-0.1:sinsemilla-constraints?partial
        meta.create_gate("Sinsemilla gate", |meta| {
            let q_s1 = meta.query_selector(config.q_sinsemilla1);
            let q_s3 = config.q_s3(meta);

            let lambda_1_next = meta.query_advice(config.double_and_add.lambda_1, Rotation::next());
            let lambda_2_cur = meta.query_advice(config.double_and_add.lambda_2, Rotation::cur());
            let x_a_cur = meta.query_advice(config.double_and_add.x_a, Rotation::cur());
            let x_a_next = meta.query_advice(config.double_and_add.x_a, Rotation::next());

            // x_r = lambda_1^2 - x_a_cur - x_p
            let x_r = x_r(meta, Rotation::cur());

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A_cur = Y_A(meta, Rotation::cur());

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A_next = Y_A(meta, Rotation::next());

            // lambda2^2 - (x_a_next + x_r + x_a_cur) = 0
            let secant_line =
                lambda_2_cur.clone().square() - (x_a_next.clone() + x_r + x_a_cur.clone());

            // lhs - rhs = 0, where
            //    - lhs = 4 * lambda_2_cur * (x_a_cur - x_a_next)
            //    - rhs = (2 * Y_A_cur + (2 - q_s3) * Y_A_next + 2 * q_s3 * y_a_final)
            let y_check = {
                // lhs = 4 * lambda_2_cur * (x_a_cur - x_a_next)
                let lhs = lambda_2_cur * pallas::Base::from(4) * (x_a_cur - x_a_next);

                // rhs = 2 * Y_A_cur + (2 - q_s3) * Y_A_next + 2 * q_s3 * y_a_final
                let rhs = {
                    // y_a_final is assigned to the lambda1 column on the next offset.
                    let y_a_final = lambda_1_next;

                    Y_A_cur * two
                        + (Expression::Constant(two) - q_s3.clone()) * Y_A_next
                        + q_s3 * two * y_a_final
                };
                lhs - rhs
            };

            Constraints::with_selector(q_s1, [("Secant line", secant_line), ("y check", y_check)])
        });
    }

    pub(crate) fn create_config(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 5],
        witness_pieces: Column<Advice>,
        fixed_y_q: Column<Fixed>,
        table: GeneratorTableConfigType,
        range_check: LookupRangeCheckConfig,
    ) -> <Self as Chip<pallas::Base>>::Config {
        // Enable equality on all advice columns
        for advice in advices.iter() {
            meta.enable_equality(*advice);
        }

        let config =
            SinsemillaConfig::<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType> {
                q_sinsemilla1: meta.complex_selector(),
                q_sinsemilla2: meta.fixed_column(),
                q_sinsemilla4: meta.selector(),
                fixed_y_q,
                double_and_add: DoubleAndAdd {
                    x_a: advices[0],
                    x_p: advices[1],
                    lambda_1: advices[3],
                    lambda_2: advices[4],
                },
                bits: advices[2],
                witness_pieces,
                // todo: check
                generator_table: table,
                lookup_config: range_check,
                _marker: PhantomData,
            };

        // Set up lookup argument
        config.generator_table.configure(meta, &config);

        config
    }

    /// # Side-effects
    ///
    /// All columns in `advices` and will be equality-enabled.
    #[allow(clippy::too_many_arguments)]
    #[allow(non_snake_case)]
    pub fn configure(
        is_Q_public: bool,
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 5],
        witness_pieces: Column<Advice>,
        fixed_y_q: Column<Fixed>,
        table: GeneratorTableConfigType,
        range_check: LookupRangeCheckConfig,
    ) -> <Self as Chip<pallas::Base>>::Config {
        let config =
            Self::create_config(meta, advices, witness_pieces, fixed_y_q, table, range_check);

        Self::create_initial_y_q_gate(is_Q_public, meta, &config);

        Self::create_sinsemilla_gate(meta, &config);

        config
    }
}

// Implement `SinsemillaInstructions` for `SinsemillaChip`
impl<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
    SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChip<Hash, Commit, F, LookupRangeCheckConfig, GeneratorTableConfigType>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
    LookupRangeCheckConfig: DefaultLookupRangeCheck,
    GeneratorTableConfigType: DefaultGeneratorTable,
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
        is_Q_public: bool,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Self::RunningSum>), Error> {
        layouter.assign_region(
            || "hash_to_point",
            |mut region| self.hash_message(is_Q_public, &mut region, Q, &message),
        )
    }

    fn extract(point: &Self::NonIdentityPoint) -> Self::X {
        point.x()
    }
}
