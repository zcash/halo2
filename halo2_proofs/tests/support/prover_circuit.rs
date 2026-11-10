//! A test circuit exercising every prover component, shared by the
//! `prover_determinism` test and the `prover` benchmark: several circuits in
//! one proof, an instance column, a custom gate, a lookup, and enough
//! equality-enabled columns to split the permutation argument into several
//! chained sets.

#![allow(dead_code)]

use std::convert::Infallible;

use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{
    create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column, ConstraintSystem,
    Error, Instance, ProvingKey, Selector, SingleVerifier, TableColumn,
};
use halo2_proofs::poly::{commitment::Params, Rotation};
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::{Rng, TryRng};

/// A xorshift64* generator: deterministic, dependency free, and NOT
/// cryptographically secure. Only for reproducible tests.
pub struct TestRng(pub u64);

impl TryRng for TestRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        self.try_next_u64().map(|v| (v >> 32) as u32)
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        Ok(self.0.wrapping_mul(0x2545_f491_4f6c_dd1d))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        for chunk in dst.chunks_mut(8) {
            let bytes = self.try_next_u64()?.to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
        Ok(())
    }
}

const TABLE_SIZE: u64 = 16;

#[derive(Clone)]
pub struct Config {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    d: Column<Advice>,
    instance: Column<Instance>,
    mul: Selector,
    table: TableColumn,
}

/// Fills `rows` rows with `a = (offset + i) mod 16`, `b = i + 1`, `c = a * b`,
/// and `d` a copy of `c`, exposing `a` at row 1 as the public input.
#[derive(Clone)]
pub struct TestCircuit {
    pub rows: usize,
    pub offset: u64,
}

impl TestCircuit {
    fn a(&self, row: usize) -> Fp {
        Fp::from((self.offset + row as u64) % TABLE_SIZE)
    }

    pub fn public_input(&self) -> Fp {
        self.a(1)
    }
}

impl Circuit<Fp> for TestCircuit {
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Config {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let d = meta.advice_column();
        let instance = meta.instance_column();
        let mul = meta.selector();
        let table = meta.lookup_table_column();

        for column in [a, b, c, d] {
            meta.enable_equality(column);
        }
        meta.enable_equality(instance);

        meta.create_gate("mul", |meta| {
            let s = meta.query_selector(mul);
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            vec![s * (a * b - c)]
        });

        meta.lookup(|meta| vec![(meta.query_advice(a, Rotation::cur()), table)]);

        Config {
            a,
            b,
            c,
            d,
            instance,
            mul,
            table,
        }
    }

    fn synthesize(&self, config: Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_table(
            || "table",
            |mut table| {
                for i in 0..TABLE_SIZE {
                    table.assign_cell(
                        || "entry",
                        config.table,
                        i as usize,
                        || Value::known(Fp::from(i)),
                    )?;
                }
                Ok(())
            },
        )?;

        let public = layouter.assign_region(
            || "rows",
            |mut region| {
                let mut public = None;
                for row in 0..self.rows {
                    config.mul.enable(&mut region, row)?;
                    let a = self.a(row);
                    let b = Fp::from(row as u64 + 1);
                    let a_cell = region.assign_advice(|| "a", config.a, row, || Value::known(a))?;
                    region.assign_advice(|| "b", config.b, row, || Value::known(b))?;
                    let c = region.assign_advice(|| "c", config.c, row, || Value::known(a * b))?;
                    let d = region.assign_advice(|| "d", config.d, row, || Value::known(a * b))?;
                    region.constrain_equal(c.cell(), d.cell())?;
                    if row == 1 {
                        public = Some(a_cell.cell());
                    }
                }
                Ok(public.expect("at least two rows"))
            },
        )?;

        layouter.constrain_instance(public, config.instance, 0)
    }
}

/// Builds the parameters and proving key for circuits of `2^k` rows.
pub fn setup(k: u32) -> (Params<EqAffine>, ProvingKey<EqAffine>) {
    let params: Params<EqAffine> = Params::new(k);
    let circuit = TestCircuit {
        rows: (1 << k) / 2,
        offset: 0,
    };
    let vk = keygen_vk(&params, &circuit).expect("keygen_vk");
    let pk = keygen_pk(&params, vk, &circuit).expect("keygen_pk");
    (params, pk)
}

/// `count` instances of the test circuit for `2^k` rows, each with its own
/// witness.
pub fn circuits(k: u32, count: usize) -> Vec<TestCircuit> {
    (0..count)
        .map(|i| TestCircuit {
            rows: (1 << k) / 2,
            offset: 5 * i as u64,
        })
        .collect()
}

/// Proves `circuits` in one proof and returns the proof bytes.
pub fn prove(
    params: &Params<EqAffine>,
    pk: &ProvingKey<EqAffine>,
    circuits: &[TestCircuit],
    rng: impl Rng,
) -> Vec<u8> {
    let public: Vec<[Fp; 1]> = circuits.iter().map(|c| [c.public_input()]).collect();
    let columns: Vec<[&[Fp]; 1]> = public.iter().map(|p| [&p[..]]).collect();
    let instances: Vec<&[&[Fp]]> = columns.iter().map(|c| &c[..]).collect();

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(params, pk, circuits, &instances, rng, &mut transcript).expect("proof generation");
    transcript.finalize()
}

/// Checks that `proof` verifies for `circuits`.
pub fn verify(
    params: &Params<EqAffine>,
    pk: &ProvingKey<EqAffine>,
    circuits: &[TestCircuit],
    proof: &[u8],
) -> bool {
    let public: Vec<[Fp; 1]> = circuits.iter().map(|c| [c.public_input()]).collect();
    let columns: Vec<[&[Fp]; 1]> = public.iter().map(|p| [&p[..]]).collect();
    let instances: Vec<&[&[Fp]]> = columns.iter().map(|c| &c[..]).collect();

    let strategy = SingleVerifier::new(params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(proof);
    verify_proof(params, pk.get_vk(), strategy, &instances, &mut transcript).is_ok()
}
