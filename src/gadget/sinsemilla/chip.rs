use ff::Field;
use std::marker::PhantomData;

use super::{SinsemillaInstructions, Q_DOMAIN_PREFIX, Q_PERSONALIZATION};
use crate::{
    arithmetic::{
        Curve, CurveAffine, FieldExt, HashToCurve, Shake128, SimplifiedSWUWithDegree3Isogeny,
    },
    circuit::{Chip, Layouter},
    pasta::{EpAffine, Fp, IsoEpAffine},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed},
    poly::Rotation,
};

mod generator_table;

use generator_table::*;

/// Configuration for the Sinsemilla chip
#[derive(Clone, Debug)]
pub struct SinsemillaConfig {
    /// Number of bits in a message word
    pub k: usize,
    /// Number of rounds in the Sinsemilla hash
    pub rounds: usize,
    ecc: Column<Fixed>,
    x_a: Column<Advice>,
    y_a: Column<Advice>,
    z: Column<Advice>,
    lambda1: Column<Advice>,
    lambda2: Column<Advice>,
    m: Column<Advice>,
    x_p: Column<Advice>,
    y_p: Column<Advice>,
    lookup_inputs: GeneratorInputs,
    lookup_table: GeneratorTable,
}

/// A chip implementing SinsemillaInstructions
#[derive(Debug)]
pub struct SinsemillaChip<F: FieldExt> {
    _marker: PhantomData<F>,
}

impl<F: FieldExt> SinsemillaChip<F> {
    /// Configures this chip for use in a circuit.
    pub fn configure(meta: &mut ConstraintSystem<F>, k: usize, rounds: usize) -> SinsemillaConfig {
        // Columns required by this chip:
        // Fixed column for ECC selector
        let ecc = meta.fixed_column();

        let x_a = meta.advice_column();
        let y_a = meta.advice_column();
        let z = meta.advice_column();
        let lambda1 = meta.advice_column();
        let lambda2 = meta.advice_column();

        // - Three advice columns to interact with the lookup table.
        let m = meta.advice_column();
        let x_p = meta.advice_column();
        let y_p = meta.advice_column();

        let (lookup_inputs, lookup_table) = GeneratorTable::configure(meta, k, m, x_p, y_p);

        // ECCLookup gate
        meta.create_gate("ECCLookup", |meta| {
            let ecc = meta.query_fixed(ecc, Rotation::cur());

            let x_a_cur = meta.query_advice(x_a, Rotation::cur());
            let y_a_cur = meta.query_advice(y_a, Rotation::cur());
            let x_a_next = meta.query_advice(x_a, Rotation::next());
            let y_a_next = meta.query_advice(y_a, Rotation::next());
            let lambda1 = meta.query_advice(lambda1, Rotation::cur());
            let lambda2 = meta.query_advice(lambda2, Rotation::cur());
            let x_p = meta.query_advice(x_p, Rotation::cur());
            let y_p = meta.query_advice(y_p, Rotation::cur());

            // λ_{1,i}⋅(x_{A,i} − x_{P,i}) − y_{A,i} + y_{P,i} = 0
            let expr1 = lambda1.clone() * (x_a_cur.clone() + x_p.clone() * (-F::one()))
                + y_a_cur.clone() * (-F::one())
                + y_p;

            // λ_{1,i} + λ_{2,i})⋅(x_{A,i} − (λ_{1,i}^2 − x_{A,i} − x_{P,i}))−2y_{A,i} = 0
            let expr2 = (lambda1.clone() + lambda2.clone())
                * (x_a_cur.clone()
                    + lambda1.clone() * lambda1.clone() * (-F::one())
                    + x_a_cur.clone()
                    + x_p.clone())
                + y_a_cur.clone() * (-F::one() - F::one());

            // λ_{2,i}^2 − x_{A,i+1} −(λ_{1,i}^2 − x_{A,i} − x_{P,i}) − x_{A,i} = 0
            let expr3 = lambda2.clone() * lambda2.clone()
                + x_a_next.clone() * (-F::one())
                + (lambda1.clone() * lambda1) * (-F::one())
                + x_p;

            // λ_{2,i}⋅(x_{A,i} − x_{A,i+1}) − y_{A,i} − y_{A,i+1} = 0
            let expr4 =
                lambda2 * (x_a_cur + x_a_next * (-F::one())) + (y_a_cur + y_a_next) * (-F::one());

            let z_cur = meta.query_advice(z, Rotation::cur());
            let z_next = meta.query_advice(z, Rotation::next());
            let m = meta.query_advice(m, Rotation::cur());

            let decompose_z = z_cur * F::from_u64((1 << k) as u64) + m + z_next * (-F::one());

            ecc * (expr1 + expr2 + expr3 + expr4 + decompose_z)
        });

        SinsemillaConfig {
            k,
            rounds,
            ecc,
            x_a,
            y_a,
            z,
            lambda1,
            lambda2,
            m,
            x_p,
            y_p,
            lookup_inputs,
            lookup_table,
        }
    }
}

impl<F: FieldExt> Chip for SinsemillaChip<F> {
    type Config = SinsemillaConfig;
    type Field = F;

    fn load(layouter: &mut impl Layouter<Self>) -> Result<(), Error> {
        // let table = layouter.config().lookup_table.clone();
        // table.load(layouter, &*pallas::MAP)
        Ok(())
    }
}

impl SinsemillaInstructions<Fp, IsoEpAffine, EpAffine> for SinsemillaChip<Fp> {
    type Message = Vec<u8>; // TODO: Vec<bool>
    type PaddedMessage = Vec<u8>;
    type HashOutput = EpAffine;
    type Map = SimplifiedSWUWithDegree3Isogeny<Fp, IsoEpAffine, EpAffine>;

    fn q(map: &Self::Map) -> (Fp, Fp) {
        let hash = &map.hash_to_curve(Q_DOMAIN_PREFIX, Shake128::default());
        let q = hash(&Q_PERSONALIZATION).to_affine();
        q.get_xy().unwrap()
    }

    fn load_p(layouter: &mut impl Layouter<Self>, map: &'static Self::Map) -> Result<(), Error> {
        let table = layouter.config().lookup_table.clone();
        table.load(layouter, map)
    }

    // Loads Q into the circuit.
    fn load_q(layouter: &mut impl Layouter<Self>, map: &Self::Map) -> Result<(), Error> {
        let (x_q, y_q) = Self::q(map);
        let config = layouter.config().clone();
        layouter.assign_region(
            || "Q initialization",
            |mut region| {
                region.assign_advice(|| "x_q", config.x_a, 0, || Ok(x_q))?;
                region.assign_advice(|| "y_q", config.y_a, 0, || Ok(y_q))?;

                Ok(())
            },
        )
    }

    /// Pads the given message to `kn` bits.
    fn pad(
        layouter: &mut impl Layouter<Self>,
        message: Self::Message,
    ) -> Result<Self::PaddedMessage, Error> {
        let k = layouter.config().k;
        let rounds = layouter.config().rounds;
        let padding = vec![0u8; k * rounds - message.len()];
        let mut message = message.clone();
        message.extend_from_slice(&padding);

        Ok(message.to_vec())
    }

    /// Hashes the given message.
    ///
    /// TODO: Since the output is always curve point, maybe this should return
    /// `<Self as EccInstructions>::Point` instead of an associated type.
    fn hash(
        layouter: &mut impl Layouter<Self>,
        message: Self::PaddedMessage,
        map: &Self::Map,
    ) -> Result<Self::HashOutput, Error> {
        let config = layouter.config().clone();

        let bytes_per_round = config.k / 8;

        // Parse message into n k-bit words
        let words: Vec<u32> = (0..config.rounds)
            .map(|i| {
                let mut word = 0u32;
                for j in 0..bytes_per_round {
                    word += (message[i + j] as u32) << (8 * j);
                }
                word
            })
            .collect();

        // z_0 to z_n
        let mut z_vec: Vec<_> = Vec::with_capacity(config.rounds + 1);
        z_vec.push(0);
        for i in 1..(config.rounds + 1) {
            let z_prev = z_vec[i - 1];
            let z = z_prev * (1 << config.k) + words[i - 1];
            z_vec.push(z);
        }

        // Get (x_p, y_p) for each word
        let points: Vec<(Fp, Fp)> = words
            .iter()
            .map(|word| get_p_by_idx(&map, *word).unwrap())
            .collect();

        // Get lambda1, lambda2, x_a, y_a for each word
        let mut lambda1_vec = Vec::with_capacity(config.rounds);
        let mut lambda2_vec = Vec::with_capacity(config.rounds);
        let mut x_a_vec = Vec::with_capacity(config.rounds);
        let mut y_a_vec = Vec::with_capacity(config.rounds);

        let (x_a_0, y_a_0) = Self::q(map);
        x_a_vec.push(x_a_0);
        y_a_vec.push(y_a_0);

        for i in 0..config.rounds {
            let x_a = x_a_vec[i];
            let y_a = y_a_vec[i];
            let point_a = EpAffine::from_xy(x_a, y_a).unwrap();

            let (x_p, y_p) = points[i];
            let point_p = EpAffine::from_xy(x_p, y_p).unwrap();

            let point_r = point_a + point_p;
            let (x_r, _y_r) = point_r.to_affine().get_xy().unwrap();

            let lambda1 = (y_a - y_p) * (x_a - x_p).invert().unwrap();
            let lambda2 = Fp::from_u64(2) * y_a * (x_a - x_r).invert().unwrap() - lambda1;

            let x_a_new = lambda2 * lambda2 - x_a - x_r;
            let y_a_new = lambda2 * (x_a - x_a_new) - y_a;

            lambda1_vec.push(lambda1);
            lambda2_vec.push(lambda2);
            x_a_vec.push(x_a_new);
            y_a_vec.push(y_a_new);
        }

        layouter.assign_region(
            || "Assign message",
            |mut region| {
                let (x_q, y_q) = Self::q(map);
                region.assign_advice(|| "x_q", config.x_a, 0, || Ok(x_q))?;
                region.assign_advice(|| "y_q", config.y_a, 0, || Ok(y_q))?;

                for (row, word) in words.iter().enumerate() {
                    region.assign_fixed(|| "ECC lookup", config.ecc, row, || Ok(Fp::one()))?;
                    region.assign_advice(
                        || "message word",
                        config.m,
                        row,
                        || Ok(Fp::from_u64(*word as u64)),
                    )?;
                }

                for (row, z) in z_vec.iter().enumerate() {
                    region.assign_advice(|| "z", config.z, row, || Ok(Fp::from_u64(*z as u64)))?;
                }

                for (row, point) in points.iter().enumerate() {
                    region.assign_advice(|| "x_p", config.x_p, row, || Ok((*point).0))?;
                    region.assign_advice(|| "y_p", config.y_p, row, || Ok((*point).1))?;
                }

                for row in points.len()..(1 << config.k) {
                    let point = points[0];
                    region.assign_advice(|| "m", config.m, row, || Ok(Fp::zero()))?;
                    region.assign_advice(|| "x_p", config.x_p, row, || Ok(point.0))?;
                    region.assign_advice(|| "y_p", config.y_p, row, || Ok(point.1))?;
                }

                for (row, lambda1) in lambda1_vec.iter().enumerate() {
                    region.assign_advice(|| "lambda1", config.lambda1, row, || Ok(*lambda1))?;
                }

                for (row, lambda2) in lambda2_vec.iter().enumerate() {
                    region.assign_advice(|| "lambda2", config.lambda2, row, || Ok(*lambda2))?;
                }

                for (row, x_a) in x_a_vec.iter().enumerate() {
                    region.assign_advice(|| "x_a", config.x_a, row, || Ok(*x_a))?;
                }

                for (row, y_a) in y_a_vec.iter().enumerate() {
                    region.assign_advice(|| "y_a", config.y_a, row, || Ok(*y_a))?;
                }

                Ok(())
            },
        )?;

        Ok(EpAffine::from_xy(x_a_vec[config.rounds], y_a_vec[config.rounds]).unwrap())
    }
}
