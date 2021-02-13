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

        let lambda1_next = meta.query_advice(columns.lambda1, Rotation::next());
        let lambda2_next = meta.query_advice(columns.lambda2, Rotation::next());
        let x_a_next = meta.query_advice(columns.x_a, Rotation::next());
        let x_p_next = meta.query_advice(columns.x_p, Rotation::next());
        let y_a_next = (lambda1_next.clone() + lambda2_next)
            * (x_a_next.clone()
                - (lambda1_next.clone() * lambda1_next - x_a_next.clone() - x_p_next))
            * F::TWO_INV;

        // Sinsemilla expr1 gate
        meta.create_gate("Sinsemilla expr1", |_| {
            // λ_{2,i}^2 − x_{A,i+1} −(λ_{1,i}^2 − x_{A,i} − x_{P,i}) − x_{A,i} = 0
            let expr1 = lambda2_cur.clone() * lambda2_cur.clone()
                - x_a_next.clone()
                - (lambda1_cur.clone() * lambda1_cur)
                + x_p_cur;

            sinsemilla_cur.clone() * expr1
        });

        // Sinsemilla expr2 gate
        meta.create_gate("Sinsemilla expr2", |_| {
            // λ_{2,i}⋅(x_{A,i} − x_{A,i+1}) − y_{A,i} − y_{A,i+1} = 0
            let expr2 = lambda2_cur * (x_a_cur - x_a_next) - y_a_cur - y_a_next;

            sinsemilla_cur.clone() * expr2
        });

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

        // Initialize `(x_a, y_a)` to be `(x_q, y_q)`
        let (mut x_a, mut y_a) = Self::q();

        layouter.assign_region(
            || "Assign message",
            |mut region| {
                // Initialize `(x_a, y_a)` to be `(x_q, y_q)`
                x_a = Self::q().0;
                y_a = Self::q().1;

                // Initialize `z_0` = 0;
                let mut z = 0u64;

                for row in 0..(config.rounds - 1) {
                    // Activate `Sinsemilla` custom gate
                    region.assign_fixed(
                        || "Sinsemilla expr1",
                        config.columns.sinsemilla,
                        row,
                        || Ok(F::one()),
                    )?;
                }

                // Assign initialized values
                region.assign_advice(|| "z_0", config.columns.z, 0, || Ok(F::from_u64(z)))?;
                region.assign_advice(|| "x_q", config.columns.x_a, 0, || Ok(x_a))?;

                for row in 0..config.rounds {
                    let word = words[row];
                    let gen = generators[row];
                    let x_p = gen.0;
                    let y_p = gen.1;

                    // Assign `x_p`
                    region.assign_advice(|| "x_p", config.columns.x_p, row, || Ok(x_p))?;

                    // Compute and assign `z` for the next row
                    z = z * (1 << config.k) + word;
                    region.assign_advice(
                        || "z",
                        config.columns.z,
                        row + 1,
                        || Ok(F::from_u64(z)),
                    )?;

                    // Compute and assign `lambda1, lambda2`
                    let lambda1 = (y_a - y_p) * (x_a - x_p).invert().unwrap();
                    let x_r = lambda1 * lambda1 - x_a - x_p;
                    let lambda2 = F::from_u64(2) * y_a * (x_a - x_r).invert().unwrap() - lambda1;
                    region.assign_advice(
                        || "lambda1",
                        config.columns.lambda1,
                        row,
                        || Ok(lambda1),
                    )?;
                    region.assign_advice(
                        || "lambda2",
                        config.columns.lambda2,
                        row,
                        || Ok(lambda2),
                    )?;

                    // Compute and assign `x_a` for the next row
                    let x_a_new = lambda2 * lambda2 - x_a - x_r;
                    y_a = lambda2 * (x_a - x_a_new) - y_a;
                    x_a = x_a_new;
                    region.assign_advice(|| "x_a", config.columns.x_a, row + 1, || Ok(x_a))?;
                }

                Ok(())
            },
        )?;

        Ok(C::from_xy(x_a, y_a).unwrap())
    }
}
