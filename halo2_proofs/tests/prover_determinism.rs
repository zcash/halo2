//! Pins the exact bytes the prover writes for a fixed circuit and a fixed RNG.
//!
//! The prover is a deterministic function of its inputs and the RNG stream, so
//! any change to how its work is scheduled (sequential or parallel, in which
//! order, on how many threads) must leave the proof unchanged. A change that
//! alters these bytes either changed the protocol, changed the order in which
//! randomness is drawn, or introduced nondeterminism; all three need a
//! deliberate decision rather than an accidental one.
//!
//! The circuit exercises every prover component: several circuits in one
//! proof, an instance column, a custom gate, a lookup, and enough
//! equality-enabled columns to split the permutation argument into several
//! chained sets. It is pinned at three sizes because the prover switches
//! between sequential and parallel code paths by vector length: K = 12 is the
//! smallest size here at which every parallel path is taken.

use std::convert::Infallible;

use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{
    create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column, ConstraintSystem,
    Error, Instance, Selector, SingleVerifier, TableColumn,
};
use halo2_proofs::poly::{commitment::Params, Rotation};
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::TryRng;

/// A xorshift64* generator: deterministic, dependency free, and NOT
/// cryptographically secure. Only for reproducible tests.
struct TestRng(u64);

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
struct Config {
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
struct TestCircuit {
    rows: usize,
    offset: u64,
}

impl TestCircuit {
    fn a(&self, row: usize) -> Fp {
        Fp::from((self.offset + row as u64) % TABLE_SIZE)
    }

    fn public_input(&self) -> Fp {
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

/// Proves two instances of the test circuit with a seeded RNG and returns
/// the proof bytes, after checking that the proof verifies.
fn prove(k: u32) -> Vec<u8> {
    let params: Params<EqAffine> = Params::new(k);
    let rows = (1 << k) / 2;
    let circuits = [
        TestCircuit { rows, offset: 0 },
        TestCircuit { rows, offset: 5 },
    ];
    let vk = keygen_vk(&params, &circuits[0]).expect("keygen_vk");
    let pk = keygen_pk(&params, vk, &circuits[0]).expect("keygen_pk");

    let public: Vec<[Fp; 1]> = circuits.iter().map(|c| [c.public_input()]).collect();
    let columns: Vec<[&[Fp]; 1]> = public.iter().map(|p| [&p[..]]).collect();
    let instances: Vec<&[&[Fp]]> = columns.iter().map(|c| &c[..]).collect();

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        &pk,
        &circuits,
        &instances,
        TestRng(0x9e37_79b9_7f4a_7c15),
        &mut transcript,
    )
    .expect("proof generation");
    let proof = transcript.finalize();

    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    assert!(verify_proof(&params, pk.get_vk(), strategy, &instances, &mut transcript).is_ok());

    proof
}

fn digest(proof: &[u8]) -> String {
    blake2b_simd::Params::new()
        .hash_length(32)
        .hash(proof)
        .to_hex()
        .to_string()
}

#[test]
fn proof_bytes_are_pinned_k5() {
    assert_eq!(digest(&prove(5)), EXPECTED_K5);
}

#[test]
fn proof_bytes_are_pinned_k10() {
    assert_eq!(digest(&prove(10)), EXPECTED_K10);
}

#[test]
fn proof_bytes_are_pinned_k12() {
    assert_eq!(digest(&prove(12)), EXPECTED_K12);
}

#[test]
fn proving_is_deterministic() {
    assert_eq!(prove(6), prove(6));
}

const EXPECTED_K5: &str = "bfe78e5ff358190e7d917100393c899406b86a8ccbd03c61f3c3dc53b4c86c55";
const EXPECTED_K10: &str = "ad9f1c6cf228c62c5ac0e48093efbeb277fbb89919dedf9ba26bb13958b23c3d";
const EXPECTED_K12: &str = "ba8ff157c532f7df3b854426e0db1565eca6c3b041a3988422a02ec3fcfdffcb";
