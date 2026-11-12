//! Shared strategies, circuit and fixtures for the integration-test laws.
//!
//! Everything here goes through the crate's public API only.
#![allow(dead_code)]

use std::sync::OnceLock;

use ff::{Field, FromUniformBytes, PrimeField};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::{EqAffine, Fp},
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Advice, BatchVerifier, Circuit, Column,
        ConstraintSystem, Error, Instance, ProvingKey, SingleVerifier, VerifyingKey,
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use proptest::{
    collection::vec,
    prelude::*,
    sample::{select, Index},
    strategy::ValueTree,
    test_runner::TestRunner,
};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

/// Domain size `2^K`. At this size an instance column can exceed the
/// batch verifier's 64 cached rows.
pub const K: u32 = 7;
/// Rows the circuit copies into each instance column.
pub const ROWS: usize = 80;
/// Instance columns in the circuit.
pub const COLUMNS: usize = 2;
/// Proofs generated once and shared by the cheap laws.
pub const POOL: usize = 6;

// Strategies.

/// Field elements weighted towards the values that bit-level code gets wrong:
/// zero, one, `p - 1`, powers of two and their neighbours, small negatives,
/// and uniform values.
pub fn arb_scalar<F>() -> impl Strategy<Value = F>
where
    F: PrimeField + FromUniformBytes<64>,
{
    prop_oneof![
        select(vec![
            F::ZERO,
            F::ONE,
            -F::ONE,
            F::ONE.double(),
            -F::ONE.double()
        ]),
        (0u64..u64::from(F::NUM_BITS), select(vec![-1i8, 0, 1])).prop_map(|(k, delta)| {
            let power = F::ONE.double().pow_vartime([k]);
            match delta {
                -1 => power - F::ONE,
                1 => power + F::ONE,
                _ => power,
            }
        }),
        any::<u64>().prop_map(|v| -F::from(v)),
        any::<u64>().prop_map(F::from),
        (
            prop::array::uniform32(any::<u8>()),
            prop::array::uniform32(any::<u8>())
        )
            .prop_map(|(low, high)| {
                let mut bytes = [0; 64];
                bytes[..32].copy_from_slice(&low);
                bytes[32..].copy_from_slice(&high);
                F::from_uniform_bytes(&bytes)
            }),
    ]
}

/// A length in `0..=max`, weighted towards the batch verifier's 64-row cache
/// boundary and the ends of the range.
pub fn arb_length(max: usize) -> impl Strategy<Value = usize> {
    let edges = [0, 1, 63, 64, 65, max.saturating_sub(1), max]
        .into_iter()
        .filter(|len| *len <= max)
        .collect::<Vec<_>>();
    prop_oneof![select(edges), 0..=max]
}

/// One proof's instance columns, each of an [`arb_length`] up to [`ROWS`].
pub fn arb_columns<F>() -> impl Strategy<Value = Vec<Vec<F>>>
where
    F: PrimeField + FromUniformBytes<64>,
{
    vec(
        arb_length(ROWS).prop_flat_map(|len| vec(arb_scalar::<F>(), len)),
        COLUMNS,
    )
}

/// A change to the public inputs handed to a verifier.
#[derive(Clone, Debug)]
pub enum Mutation {
    None,
    Change {
        column: usize,
        row: Index,
        delta: u64,
    },
    AppendZero {
        column: usize,
    },
    Append {
        column: usize,
        value: u64,
    },
    DropLast {
        column: usize,
    },
}

pub fn arb_mutation() -> impl Strategy<Value = Mutation> {
    prop_oneof![
        3 => Just(Mutation::None),
        3 => (0..COLUMNS, any::<Index>(), 1..u64::MAX)
            .prop_map(|(column, row, delta)| Mutation::Change { column, row, delta }),
        1 => (0..COLUMNS).prop_map(|column| Mutation::AppendZero { column }),
        1 => (0..COLUMNS, 1..u64::MAX)
            .prop_map(|(column, value)| Mutation::Append { column, value }),
        1 => (0..COLUMNS).prop_map(|column| Mutation::DropLast { column }),
    ]
}

/// Indices into the proof pool, each with a mutation of its public inputs.
pub fn arb_entries(max: usize) -> impl Strategy<Value = Vec<(usize, Mutation)>> {
    vec((0..POOL, arb_mutation()), 0..=max)
}

/// Applies `mutation`; a change to an empty column becomes an append.
pub fn mutate(columns: &[Vec<Fp>], mutation: &Mutation) -> Vec<Vec<Fp>> {
    let mut columns = columns.to_vec();
    match mutation {
        Mutation::None => {}
        Mutation::Change { column, row, delta } => {
            let column = &mut columns[*column];
            if column.is_empty() {
                column.push(Fp::from(*delta));
            } else {
                let row = row.index(column.len());
                column[row] += Fp::from(*delta);
            }
        }
        Mutation::AppendZero { column } => columns[*column].push(Fp::ZERO),
        Mutation::Append { column, value } => columns[*column].push(Fp::from(*value)),
        Mutation::DropLast { column } => {
            columns[*column].pop();
        }
    }
    columns
}

// Circuit.

#[derive(Clone, Debug)]
pub struct CopyConfig {
    advice: [Column<Advice>; COLUMNS],
    instance: [Column<Instance>; COLUMNS],
}

/// Copies [`ROWS`] advice cells per column into the matching instance column.
/// Every proof shares one verifying key; a shorter public column is the same
/// statement with zeros in the remaining rows.
#[derive(Clone)]
pub struct CopyToInstances {
    values: Vec<Vec<Value<Fp>>>,
}

impl CopyToInstances {
    pub fn new(columns: &[Vec<Fp>]) -> Self {
        let values = columns
            .iter()
            .map(|column| {
                let mut column = column.iter().copied().map(Value::known).collect::<Vec<_>>();
                column.resize(ROWS, Value::known(Fp::ZERO));
                column
            })
            .collect();
        Self { values }
    }
}

impl Circuit<Fp> for CopyToInstances {
    type Config = CopyConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            values: vec![vec![Value::unknown(); ROWS]; COLUMNS],
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> CopyConfig {
        let advice = [meta.advice_column(), meta.advice_column()];
        let instance = [meta.instance_column(), meta.instance_column()];
        for column in advice {
            meta.enable_equality(column);
        }
        for column in instance {
            meta.enable_equality(column);
        }
        CopyConfig { advice, instance }
    }

    fn synthesize(&self, config: CopyConfig, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        for (column, values) in self.values.iter().enumerate() {
            let cells = layouter.assign_region(
                || "values",
                |mut region| {
                    values
                        .iter()
                        .enumerate()
                        .map(|(row, value)| {
                            region
                                .assign_advice(|| "value", config.advice[column], row, || *value)
                                .map(|cell| cell.cell())
                        })
                        .collect::<Result<Vec<_>, _>>()
                },
            )?;
            for (row, cell) in cells.into_iter().enumerate() {
                layouter.constrain_instance(cell, config.instance[column], row)?;
            }
        }
        Ok(())
    }
}

// Fixtures and verification.

/// A proof together with the public inputs it was made for.
#[derive(Clone)]
pub struct Statement {
    pub proof: Vec<u8>,
    pub columns: Vec<Vec<Fp>>,
}

pub struct Fixture {
    pub params: Params<EqAffine>,
    pub pk: ProvingKey<EqAffine>,
    pub pool: Vec<Statement>,
}

impl Fixture {
    pub fn vk(&self) -> &VerifyingKey<EqAffine> {
        self.pk.get_vk()
    }

    pub fn prove(&self, columns: Vec<Vec<Fp>>) -> Statement {
        let instances = columns.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        create_proof(
            &self.params,
            &self.pk,
            &[CopyToInstances::new(&columns)],
            &[&instances],
            UnwrapErr(SysRng),
            &mut transcript,
        )
        .expect("an honest proof is created");
        Statement {
            proof: transcript.finalize(),
            columns,
        }
    }

    /// The pool statements named by `entries`, with each mutation applied.
    pub fn select(&self, entries: &[(usize, Mutation)]) -> Vec<Statement> {
        entries
            .iter()
            .map(|(index, mutation)| {
                let statement = &self.pool[*index];
                Statement {
                    proof: statement.proof.clone(),
                    columns: mutate(&statement.columns, mutation),
                }
            })
            .collect()
    }
}

/// Keys and a pool of proofs whose public inputs come from [`arb_columns`]
/// under a fixed seed, built once per test binary.
pub fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let params = Params::new(K);
        let empty = CopyToInstances::new(&[]).without_witnesses();
        let vk = keygen_vk(&params, &empty).expect("keygen_vk");
        let pk = keygen_pk(&params, vk, &empty).expect("keygen_pk");
        let mut fixture = Fixture {
            params,
            pk,
            pool: vec![],
        };
        let mut runner = TestRunner::deterministic();
        fixture.pool = (0..POOL)
            .map(|_| {
                let columns = arb_columns::<Fp>()
                    .new_tree(&mut runner)
                    .expect("strategy")
                    .current();
                fixture.prove(columns)
            })
            .collect();
        fixture
    })
}

pub fn verify_single(
    params: &Params<EqAffine>,
    vk: &VerifyingKey<EqAffine>,
    statement: &Statement,
) -> Result<(), Error> {
    let columns = statement
        .columns
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&statement.proof[..]);
    verify_proof(
        params,
        vk,
        SingleVerifier::new(params),
        &[&columns],
        &mut transcript,
    )
    .map(|_| ())
}

pub fn verify_batch(
    params: &Params<EqAffine>,
    vk: &VerifyingKey<EqAffine>,
    statements: &[Statement],
) -> bool {
    let mut batch = BatchVerifier::new();
    for statement in statements {
        batch.add_proof(vec![statement.columns.clone()], statement.proof.clone());
    }
    batch.finalize(params, vk)
}
