//! Chip implementations for the Sinsemilla gadgets.

use super::SinsemillaInstructionsOptimized;
use crate::{
    ecc::{chip::NonIdentityEccPoint, FixedPoints},
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        message::{Message, MessagePiece},
        primitives as sinsemilla, CommitDomains, HashDomains, SinsemillaInstructions,
    },
    utilities_opt::lookup_range_check::DefaultLookupRangeCheckConfigOptimized,
};
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{
        Advice, Column, ConstraintSystem, Constraints, Error, Fixed, TableColumn, VirtualCells,
    },
    poly::Rotation,
};
use pasta_curves::pallas;

pub(crate) mod generator_table;

mod hash_to_point;

/// A chip that implements 10-bit Sinsemilla using a lookup table and 5 advice columns.
///
/// [Chip description](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html#plonk--halo-2-constraints).
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaChipOptimized<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    inner: SinsemillaChip<Hash, Commit, Fixed, DefaultLookupRangeCheckConfigOptimized>,
}

impl<Hash, Commit, Fixed> Chip<pallas::Base> for SinsemillaChipOptimized<Hash, Commit, Fixed>
where
    Hash: HashDomains<pallas::Affine>,
    Fixed: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, Fixed, Hash>,
{
    type Config = SinsemillaConfig<Hash, Commit, Fixed, DefaultLookupRangeCheckConfigOptimized>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        self.inner.config()
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Hash, Commit, F> SinsemillaChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    /// Reconstructs this chip from the given config.
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self {
            inner:
                SinsemillaChip::<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized>::construct(
                    config,
                ),
        }
    }

    /// Loads the lookup table required by this chip into the circuit.
    pub fn load(
        config: SinsemillaConfig<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized>,
        layouter: &mut impl Layouter<pallas::Base>,
    ) -> Result<<Self as Chip<pallas::Base>>::Loaded, Error> {
        // Load the lookup table.
        generator_table::load_with_tag(
            &config.generator_table,
            config.lookup_config.table_range_check_tag(),
            layouter,
        )
    }

    #[allow(non_snake_case)]
    fn create_initial_y_q_gate(
        meta: &mut ConstraintSystem<pallas::Base>,
        config: &SinsemillaConfig<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized>,
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
            let y_q = meta.query_advice(config.double_and_add.x_p, Rotation::prev());

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A_cur = Y_A(meta, Rotation::cur());

            // 2 * y_q - Y_{A,0} = 0
            let init_y_q_check = y_q * two - Y_A_cur;

            Constraints::with_selector(q_s4, Some(("init_y_q_check", init_y_q_check)))
        });
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
        range_check: DefaultLookupRangeCheckConfigOptimized,
    ) -> <Self as Chip<pallas::Base>>::Config {
        let config = SinsemillaChip::<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized>::create_config(
            meta,
            advices,
            witness_pieces,
            fixed_y_q,
            lookup,
            range_check,
        );

        Self::create_initial_y_q_gate(meta, &config);

        SinsemillaChip::<Hash, Commit, F, DefaultLookupRangeCheckConfigOptimized>::create_sinsemilla_gate(
            meta, &config,
        );

        config
    }
}

// Implement `SinsemillaInstructions` for `SinsemillaChip`
impl<Hash, Commit, F> SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChipOptimized<Hash, Commit, F>
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
        layouter: impl Layouter<pallas::Base>,
        field_elem: Value<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        self.inner
            .witness_message_piece(layouter, field_elem, num_words)
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
            |mut region| self.inner.hash_message_zsa(&mut region, Q, &message),
        )
    }

    fn extract(point: &Self::NonIdentityPoint) -> Self::X {
        point.x()
    }
}

// Implement `SinsemillaInstructions` for `SinsemillaChip`
impl<Hash, Commit, F>
    SinsemillaInstructionsOptimized<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChipOptimized<Hash, Commit, F>
where
    Hash: HashDomains<pallas::Affine>,
    F: FixedPoints<pallas::Affine>,
    Commit: CommitDomains<pallas::Affine, F, Hash>,
{
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point_with_private_init(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        Q: &Self::NonIdentityPoint,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Self::RunningSum>), Error> {
        layouter.assign_region(
            || "hash_to_point",
            |mut region| {
                self.inner
                    .hash_message_with_private_init(&mut region, Q, &message)
            },
        )
    }
}
