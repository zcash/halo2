use super::{
    message::{Message, MessagePiece},
    CommitDomains, HashDomains, SinsemillaInstructions,
};
use crate::{
    circuit::gadget::{
        ecc::chip::NonIdentityEccPoint,
        utilities::{lookup_range_check::LookupRangeCheckConfig, CellValue, Var},
    },
    constants::OrchardFixedBasesFull,
    primitives::sinsemilla::{
        self, Q_COMMIT_IVK_M_GENERATOR, Q_MERKLE_CRH, Q_NOTE_COMMITMENT_M_GENERATOR,
    },
};

use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Chip, Layouter},
    plonk::{
        Advice, Column, ConstraintSystem, Error, Expression, Fixed, Selector, TableColumn,
        VirtualCells,
    },
    poly::Rotation,
};
use pasta_curves::pallas;

mod generator_table;
use generator_table::GeneratorTableConfig;

mod hash_to_point;

/// Configuration for the Sinsemilla hash chip
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaConfig {
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
    /// Advice column used to store the x-coordinate of the accumulator at each
    /// iteration of the hash.
    x_a: Column<Advice>,
    /// Advice column used to store the x-coordinate of the generator corresponding
    /// to the message word at each iteration of the hash. This is looked up in the
    /// generator table.
    x_p: Column<Advice>,
    /// Advice column used to load the message.
    bits: Column<Advice>,
    /// Advice column used to store the $\lambda_1$ intermediate value at each
    /// iteration.
    lambda_1: Column<Advice>,
    /// Advice column used to store the $\lambda_2$ intermediate value at each
    /// iteration.
    lambda_2: Column<Advice>,
    /// Advice column used to witness message pieces. This may or may not be the same
    /// column as `bits`.
    witness_pieces: Column<Advice>,
    /// The lookup table where $(\mathsf{idx}, x_p, y_p)$ are loaded for the $2^K$
    /// generators of the Sinsemilla hash.
    pub(super) generator_table: GeneratorTableConfig,
    /// An advice column configured to perform lookup range checks.
    pub(super) lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
}

impl SinsemillaConfig {
    /// Returns an array of all advice columns in this config, in arbitrary order.
    pub(super) fn advices(&self) -> [Column<Advice>; 5] {
        [self.x_a, self.x_p, self.bits, self.lambda_1, self.lambda_2]
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaChip {
    config: SinsemillaConfig,
}

impl Chip<pallas::Base> for SinsemillaChip {
    type Config = SinsemillaConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl SinsemillaChip {
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self { config }
    }

    pub fn load(
        config: SinsemillaConfig,
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
            meta.enable_equality((*advice).into())
        }

        let config = SinsemillaConfig {
            q_sinsemilla1: meta.complex_selector(),
            q_sinsemilla2: meta.fixed_column(),
            q_sinsemilla4: meta.selector(),
            fixed_y_q,
            x_a: advices[0],
            x_p: advices[1],
            bits: advices[2],
            lambda_1: advices[3],
            lambda_2: advices[4],
            witness_pieces,
            generator_table: GeneratorTableConfig {
                table_idx: lookup.0,
                table_x: lookup.1,
                table_y: lookup.2,
            },
            lookup_config: range_check,
        };

        // Set up lookup argument
        GeneratorTableConfig::configure(meta, config.clone());

        let two = pallas::Base::from_u64(2);

        // Closures for expressions that are derived multiple times
        // x_r = lambda_1^2 - x_a - x_p
        let x_r = |meta: &mut VirtualCells<pallas::Base>, rotation| {
            let x_a = meta.query_advice(config.x_a, rotation);
            let x_p = meta.query_advice(config.x_p, rotation);
            let lambda_1 = meta.query_advice(config.lambda_1, rotation);
            lambda_1.square() - x_a - x_p
        };

        // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
        let Y_A = |meta: &mut VirtualCells<pallas::Base>, rotation| {
            let x_a = meta.query_advice(config.x_a, rotation);
            let lambda_1 = meta.query_advice(config.lambda_1, rotation);
            let lambda_2 = meta.query_advice(config.lambda_2, rotation);
            (lambda_1 + lambda_2) * (x_a - x_r(meta, rotation))
        };

        // Check that the initial x_A, x_P, lambda_1, lambda_2 are consistent with y_Q.
        meta.create_gate("Initial y_Q", |meta| {
            let q_s4 = meta.query_selector(config.q_sinsemilla4);
            let y_q = meta.query_fixed(config.fixed_y_q, Rotation::cur());

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A_cur = Y_A(meta, Rotation::cur());

            // 2 * y_q - Y_{A,0} = 0
            let init_y_q_check = y_q * two - Y_A_cur;

            vec![q_s4 * init_y_q_check]
        });

        meta.create_gate("Sinsemilla gate", |meta| {
            let q_s1 = meta.query_selector(config.q_sinsemilla1);
            // q_s3 = (q_s2) * (q_s2 - 1)
            let q_s3 = {
                let one = Expression::Constant(pallas::Base::one());
                let q_s2 = meta.query_fixed(config.q_sinsemilla2, Rotation::cur());
                q_s2.clone() * (q_s2 - one)
            };

            let lambda_1_next = meta.query_advice(config.lambda_1, Rotation::next());
            let lambda_2_cur = meta.query_advice(config.lambda_2, Rotation::cur());
            let x_a_cur = meta.query_advice(config.x_a, Rotation::cur());
            let x_a_next = meta.query_advice(config.x_a, Rotation::next());

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
                let lhs = lambda_2_cur * pallas::Base::from_u64(4) * (x_a_cur - x_a_next);

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

            vec![
                ("Secant line", q_s1.clone() * secant_line),
                ("y check", q_s1 * y_check),
            ]
        });

        config
    }
}

// Implement `SinsemillaInstructions` for `SinsemillaChip`
impl SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChip
{
    type CellValue = CellValue<pallas::Base>;

    type Message = Message<pallas::Base, { sinsemilla::K }, { sinsemilla::C }>;
    type MessagePiece = MessagePiece<pallas::Base, { sinsemilla::K }>;

    type RunningSum = Vec<Self::CellValue>;

    type X = CellValue<pallas::Base>;
    type NonIdentityPoint = NonIdentityEccPoint;
    type FixedPoints = OrchardFixedBasesFull;

    type HashDomains = SinsemillaHashDomains;
    type CommitDomains = SinsemillaCommitDomains;

    fn witness_message_piece(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        field_elem: Option<pallas::Base>,
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
                    || field_elem.ok_or(Error::SynthesisError),
                )
            },
        )?;
        Ok(MessagePiece::new(cell, field_elem, num_words))
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

#[derive(Clone, Debug)]
pub enum SinsemillaHashDomains {
    NoteCommit,
    CommitIvk,
    MerkleCrh,
}

#[allow(non_snake_case)]
impl HashDomains<pallas::Affine> for SinsemillaHashDomains {
    fn Q(&self) -> pallas::Affine {
        match self {
            SinsemillaHashDomains::CommitIvk => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.0).unwrap(),
                pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.1).unwrap(),
            )
            .unwrap(),
            SinsemillaHashDomains::NoteCommit => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.0).unwrap(),
                pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.1).unwrap(),
            )
            .unwrap(),
            SinsemillaHashDomains::MerkleCrh => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_MERKLE_CRH.0).unwrap(),
                pallas::Base::from_bytes(&Q_MERKLE_CRH.1).unwrap(),
            )
            .unwrap(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SinsemillaCommitDomains {
    NoteCommit,
    CommitIvk,
}

impl CommitDomains<pallas::Affine, OrchardFixedBasesFull, SinsemillaHashDomains>
    for SinsemillaCommitDomains
{
    fn r(&self) -> OrchardFixedBasesFull {
        match self {
            Self::NoteCommit => OrchardFixedBasesFull::NoteCommitR,
            Self::CommitIvk => OrchardFixedBasesFull::CommitIvkR,
        }
    }

    fn hash_domain(&self) -> SinsemillaHashDomains {
        match self {
            Self::NoteCommit => SinsemillaHashDomains::NoteCommit,
            Self::CommitIvk => SinsemillaHashDomains::CommitIvk,
        }
    }
}
