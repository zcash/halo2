use std::marker::PhantomData;

use super::{SinsemillaInstructions, Q_DOMAIN_PREFIX, Q_PERSONALIZATION};
use crate::{
    arithmetic::{CurveAffine, CurveExt, FieldExt},
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

use group::Curve;

mod generator_table;
use generator_table::*;

/// Configuration for the Sinsemilla chip
#[derive(Clone, Debug)]
pub struct SinsemillaConfig {
    /// Number of bits in a message word
    pub k: usize,
    /// Number of rounds in the Sinsemilla hash
    pub rounds: usize,
    columns: SinsemillaColumns,
    lookup_table: GeneratorTable,
}

/// Columns needed for one Sinsemilla hash
#[derive(Clone, Debug)]
pub struct SinsemillaColumns {
    sinsemilla: Column<Fixed>,
    x_a: Column<Advice>,
    z: Column<Advice>,
    lambda1: Column<Advice>,
    lambda2: Column<Advice>,
    x_p: Column<Advice>,
}

impl SinsemillaColumns {
    /// Construct a new instance of `SinsemillaColumns`
    pub fn new(
        sinsemilla: Column<Fixed>,
        x_a: Column<Advice>,
        z: Column<Advice>,
        lambda1: Column<Advice>,
        lambda2: Column<Advice>,
        x_p: Column<Advice>,
    ) -> Self {
        SinsemillaColumns {
            sinsemilla,
            x_a,
            z,
            lambda1,
            lambda2,
            x_p,
        }
    }
}

/// A chip implementing SinsemillaInstructions
#[derive(Debug)]
pub struct SinsemillaChip<C: CurveAffine> {
    _marker_c: PhantomData<C>,
}

impl<F: FieldExt, C: CurveAffine<Base = F>> SinsemillaChip<C> {
    /// Configures this chip for use in a circuit.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        k: usize,
        rounds: usize,
        columns: SinsemillaColumns,
    ) -> SinsemillaConfig {
        // Columns required by this chip:
        // Fixed columns for Sinsemilla selectors
        let sinsemilla_cur = meta.query_fixed(columns.sinsemilla, Rotation::cur());

        // m_i = z_{i + 1} - (z_i * 2^k)
        let z_cur = meta.query_advice(columns.z, Rotation::cur());
        let z_next = meta.query_advice(columns.z, Rotation::next());
        let m = z_next - z_cur * F::from_u64((1 << k) as u64);

        // y_a = (1/2) ⋅ (lambda1 + lambda2) ⋅ (x_a - (lambda1^2 - x_a - x_p))
        let lambda1_cur = meta.query_advice(columns.lambda1, Rotation::cur());
        let lambda2_cur = meta.query_advice(columns.lambda2, Rotation::cur());
        let x_a_cur = meta.query_advice(columns.x_a, Rotation::cur());
        let x_p_cur = meta.query_advice(columns.x_p, Rotation::cur());
        let y_a_cur = (lambda1_cur.clone() + lambda2_cur.clone())
            * (x_a_cur.clone()
                - (lambda1_cur.clone() * lambda1_cur.clone() - x_a_cur.clone() - x_p_cur.clone()))
            * F::TWO_INV;

        // y_p = y_a - lambda1 ⋅ (x_a - x_p)
        let y_p = y_a_cur.clone() - lambda1_cur.clone() * (x_a_cur.clone() - x_p_cur.clone());

        let (x_p_init, y_p_init) = get_s_by_idx::<F, C>(0).to_affine().get_xy().unwrap();

        let lookup_table = GeneratorTable::configure::<F, C>(
            meta,
            k,
            sinsemilla_cur.clone() * m
                + (Expression::Constant(F::one()) - sinsemilla_cur.clone()) * F::zero(),
            sinsemilla_cur.clone() * x_p_cur.clone()
                + (Expression::Constant(F::one()) - sinsemilla_cur.clone()) * x_p_init,
            sinsemilla_cur.clone() * y_p
                + (Expression::Constant(F::one()) - sinsemilla_cur.clone()) * y_p_init,
        );

        // TODO: Create gate

        SinsemillaConfig {
            k,
            rounds,
            columns,
            lookup_table,
        }
    }
}

impl<C: CurveAffine> Chip for SinsemillaChip<C> {
    type Config = SinsemillaConfig;
    type Field = C::Base;

    fn load(layouter: &mut impl Layouter<Self>) -> Result<(), Error> {
        let table = layouter.config().lookup_table.clone();
        table.load(layouter)
    }
}

impl<F: FieldExt, C: CurveAffine<Base = F>> SinsemillaInstructions<F> for SinsemillaChip<C> {
    type Message = Vec<bool>;
    type HashOutput = C;

    fn q() -> (F, F) {
        let hash = C::CurveExt::hash_to_curve(Q_DOMAIN_PREFIX);
        let q = hash(&Q_PERSONALIZATION).to_affine();
        q.get_xy().unwrap()
    }

    /// Hashes the given message.
    ///
    /// TODO: Since the output is always curve point, maybe this should return
    /// `<Self as EccInstructions>::Point` instead of an associated type.
    fn hash(
        layouter: &mut impl Layouter<Self>,
        message: Self::Message,
    ) -> Result<Self::HashOutput, Error> {
        let config = layouter.config().clone();

        // Message must be at most `kn` bits
        let max_len = config.k * config.rounds;
        assert!(message.len() <= max_len);

        let message: Vec<_> = message.chunks(config.k).collect();

        // Parse message into `k`-bit words from the big end
        let mut words: Vec<u64> = message
            .iter()
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0, |word, &bit| (word << 1) ^ (bit as u64))
            })
            .collect();

        // Pad with `zero` words until there are `n` words
        if words.len() < config.rounds {
            words.extend_from_slice(&vec![0u64; config.rounds - words.len()]);
        }
        let words = words;

        // Get (x_p, y_p) for each word. We precompute this here so that we can use `batch_normalize()`.
        let generators_projective: Vec<_> = words
            .iter()
            .map(|word| get_s_by_idx::<F, C>(*word))
            .collect();
        let mut generators = vec![C::default(); generators_projective.len()];
        C::Curve::batch_normalize(&generators_projective, &mut generators);
        let generators: Vec<(F, F)> = generators.iter().map(|gen| gen.get_xy().unwrap()).collect();

        // Initialize `z_0` = 0;
        let mut z = 0u64;

        // Initialize `(x_a, y_a)` to be `(x_q, y_q)`
        let (mut x_a, mut y_a) = Self::q();

        // TODO: Assign cells

        Ok(C::from_xy(x_a, y_a).unwrap())
    }
}
