//! End-to-end law for batch verification: on any batch, `BatchVerifier`
//! accepts exactly when every proof is accepted by the single verifier.
//!
//! The proofs are fixed; the property mutates the public inputs the batch is
//! given (a changed value, an appended zero or non-zero row, a dropped row),
//! so both the instance-commitment fast path and its fallback are compared
//! against `verify_proof` with `SingleVerifier`. An appended zero must be
//! accepted by both, because a commitment to Lagrange evaluations does not see
//! trailing zeros.
#![cfg(feature = "batch")]

use std::sync::OnceLock;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::{EqAffine, Fp},
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Advice, BatchVerifier, Circuit, Column,
        ConstraintSystem, Error, Instance, ProvingKey, SingleVerifier,
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use proptest::{collection::vec, prelude::*};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

const K: u32 = 5;
const ROWS: usize = 10;
const PROOFS: usize = 4;

#[derive(Clone, Debug)]
struct Config {
    advice: [Column<Advice>; 2],
    instance: [Column<Instance>; 2],
}

/// Copies two advice columns into two instance columns, row by row.
#[derive(Clone, Default)]
struct CopyToInstances {
    values: [Vec<Value<Fp>>; 2],
}

impl Circuit<Fp> for CopyToInstances {
    type Config = Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            values: [vec![Value::unknown(); ROWS], vec![Value::unknown(); ROWS]],
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Config {
        let advice = [meta.advice_column(), meta.advice_column()];
        let instance = [meta.instance_column(), meta.instance_column()];
        for column in advice {
            meta.enable_equality(column);
        }
        for column in instance {
            meta.enable_equality(column);
        }
        Config { advice, instance }
    }

    fn synthesize(&self, config: Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        for column in 0..2 {
            let cells = layouter.assign_region(
                || "values",
                |mut region| {
                    self.values[column]
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

struct Fixture {
    params: Params<EqAffine>,
    pk: ProvingKey<EqAffine>,
    proofs: Vec<(Vec<u8>, [Vec<Fp>; 2])>,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let params = Params::new(K);
        let empty = CopyToInstances::default().without_witnesses();
        let vk = keygen_vk(&params, &empty).unwrap();
        let pk = keygen_pk(&params, vk, &empty).unwrap();
        let proofs = (0..PROOFS)
            .map(|proof| {
                let instances = [0, 1].map(|column| {
                    (0..ROWS)
                        .map(|row| Fp::from((1000 * proof + 100 * column + row + 1) as u64))
                        .collect::<Vec<_>>()
                });
                let circuit = CopyToInstances {
                    values: instances
                        .clone()
                        .map(|c| c.into_iter().map(Value::known).collect()),
                };
                let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
                create_proof(
                    &params,
                    &pk,
                    &[circuit],
                    &[&[&instances[0], &instances[1]]],
                    UnwrapErr(SysRng),
                    &mut transcript,
                )
                .unwrap();
                (transcript.finalize(), instances)
            })
            .collect();
        Fixture { params, pk, proofs }
    })
}

#[derive(Clone, Debug)]
enum Mutation {
    None,
    Change {
        column: usize,
        row: usize,
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

fn arb_mutation() -> impl Strategy<Value = Mutation> {
    prop_oneof![
        3 => Just(Mutation::None),
        3 => (0..2usize, 0..ROWS, 1..u64::MAX)
            .prop_map(|(column, row, delta)| Mutation::Change { column, row, delta }),
        1 => (0..2usize).prop_map(|column| Mutation::AppendZero { column }),
        1 => (0..2usize, 1..u64::MAX).prop_map(|(column, value)| Mutation::Append { column, value }),
        1 => (0..2usize).prop_map(|column| Mutation::DropLast { column }),
    ]
}

fn apply(instances: &[Vec<Fp>; 2], mutation: &Mutation) -> [Vec<Fp>; 2] {
    let mut instances = instances.clone();
    match *mutation {
        Mutation::None => {}
        Mutation::Change { column, row, delta } => instances[column][row] += Fp::from(delta),
        Mutation::AppendZero { column } => instances[column].push(Fp::zero()),
        Mutation::Append { column, value } => instances[column].push(Fp::from(value)),
        Mutation::DropLast { column } => {
            instances[column].pop();
        }
    }
    instances
}

fn single_accepts(fixture: &Fixture, proof: &[u8], instances: &[Vec<Fp>; 2]) -> bool {
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(proof);
    verify_proof(
        &fixture.params,
        fixture.pk.get_vk(),
        SingleVerifier::new(&fixture.params),
        &[&[&instances[0], &instances[1]]],
        &mut transcript,
    )
    .is_ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn batch_agrees_with_single_verification(
        entries in vec((0..PROOFS, arb_mutation()), 1..=PROOFS),
    ) {
        let fixture = fixture();
        let mut batch = BatchVerifier::new();
        let mut all_accepted = true;
        for (proof, mutation) in &entries {
            let (bytes, instances) = &fixture.proofs[*proof];
            let instances = apply(instances, mutation);
            all_accepted &= single_accepts(fixture, bytes, &instances);
            batch.add_proof(vec![instances.to_vec()], bytes.clone());
        }
        prop_assert_eq!(batch.finalize(&fixture.params, fixture.pk.get_vk()), all_accepted);
    }
}
