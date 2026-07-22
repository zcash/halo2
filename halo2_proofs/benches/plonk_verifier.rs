//! Benchmarks single-proof verification with the instance layout used by
//! Orchard: one proof containing multiple action circuits over a size-$2^{11}$
//! domain, with nine instance values per action. The benchmark circuit is
//! intentionally minimal, so this is not a full Orchard benchmark.

#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::{EqAffine, Fp},
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column,
        ConstraintSystem, Error, Instance, ProvingKey, SingleVerifier,
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use rand_core::OsRng;

const K: u32 = 11;
const NUM_INSTANCE_VALUES: usize = 9;
const ACTION_COUNTS: [usize; 4] = [1, 2, 16, 64];

#[derive(Clone, Debug)]
struct ActionConfig {
    advice: Column<Advice>,
    instance: Column<Instance>,
}

#[derive(Clone)]
struct ActionCircuit {
    values: Vec<Value<Fp>>,
}

impl Circuit<Fp> for ActionCircuit {
    type Config = ActionConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            values: vec![Value::unknown(); self.values.len()],
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let advice = meta.advice_column();
        let instance = meta.instance_column();
        meta.enable_equality(advice);
        meta.enable_equality(instance);

        ActionConfig { advice, instance }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let cells = layouter.assign_region(
            || "load action instance values",
            |mut region| {
                self.values
                    .iter()
                    .enumerate()
                    .map(|(offset, value)| {
                        region
                            .assign_advice(
                                || "action instance value",
                                config.advice,
                                offset,
                                || *value,
                            )
                            .map(|cell| cell.cell())
                    })
                    .collect::<Result<Vec<_>, _>>()
            },
        )?;

        for (row, cell) in cells.into_iter().enumerate() {
            layouter.constrain_instance(cell, config.instance, row)?;
        }

        Ok(())
    }
}

struct VerificationCase {
    proof: Vec<u8>,
    public_inputs: Vec<Vec<Fp>>,
}

fn create_verification_case(
    params: &Params<EqAffine>,
    pk: &ProvingKey<EqAffine>,
    num_actions: usize,
) -> VerificationCase {
    let public_inputs = (0..num_actions)
        .map(|action| {
            (0..NUM_INSTANCE_VALUES)
                .map(|value| Fp::from((action * NUM_INSTANCE_VALUES + value) as u64))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let circuits = public_inputs
        .iter()
        .map(|values| ActionCircuit {
            values: values.iter().copied().map(Value::known).collect(),
        })
        .collect::<Vec<_>>();
    let instance_columns = public_inputs
        .iter()
        .map(|values| [values.as_slice()])
        .collect::<Vec<_>>();
    let instances = instance_columns
        .iter()
        .map(|columns| columns.as_slice())
        .collect::<Vec<_>>();

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(Vec::new());
    create_proof(params, pk, &circuits, &instances, OsRng, &mut transcript)
        .expect("proof generation should not fail");

    VerificationCase {
        proof: transcript.finalize(),
        public_inputs,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let params = Params::<EqAffine>::new(K);
    let empty_circuit = ActionCircuit {
        values: vec![Value::unknown(); NUM_INSTANCE_VALUES],
    };
    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let mut group = c.benchmark_group("plonk-verifier-orchard-shaped");
    for num_actions in ACTION_COUNTS {
        let verification_case = create_verification_case(&params, &pk, num_actions);
        let instance_columns = verification_case
            .public_inputs
            .iter()
            .map(|values| [values.as_slice()])
            .collect::<Vec<_>>();
        let instances = instance_columns
            .iter()
            .map(|columns| columns.as_slice())
            .collect::<Vec<_>>();

        group.bench_function(BenchmarkId::new("actions", num_actions), |b| {
            b.iter(|| {
                let strategy = SingleVerifier::new(&params);
                let mut transcript =
                    Blake2bRead::<_, _, Challenge255<_>>::init(&verification_case.proof[..]);
                verify_proof(&params, pk.get_vk(), strategy, &instances, &mut transcript)
                    .expect("proof verification should not fail");
            });
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
