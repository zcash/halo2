#![allow(clippy::many_single_char_names)]
#![allow(clippy::op_ref)]

#[cfg(feature = "heap-profiling")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use halo2_backend::{
    plonk::{
        keygen::{keygen_pk_v2, keygen_vk_v2},
        prover::ProverV2Single,
        verifier::{verify_proof, verify_proof_single},
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use halo2_frontend::{
    circuit::{
        compile_circuit, AssignedCell, Layouter, Region, SimpleFloorPlanner, Value,
        WitnessCalculator,
    },
    dev::MockProver,
    plonk::{
        circuit::{Challenge, Column},
        Advice, Circuit, ConstraintSystem, Error as ErrorFront, Expression, FirstPhase, Fixed,
        Instance, SecondPhase, Selector,
    },
};
use halo2_middleware::{ff::Field, poly::Rotation};
use halo2_proofs::poly::commitment::ParamsProver;
use std::collections::HashMap;

#[derive(Clone)]
struct MyCircuitConfig {
    // A gate that uses selector, fixed, advice, has addition, multiplication and rotation
    // s_gate[0] * (a[0] + b[0] * c[0] * d[0] - a[1])
    s_gate: Selector,
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    d: Column<Fixed>,

    // Copy constraints between columns (a, b) and (a, d)

    // A dynamic lookup: s_lookup * [1, a[0], b[0]] in s_ltable * [1, d[0], c[0]]
    s_lookup: Column<Fixed>,
    s_ltable: Column<Fixed>,

    // A shuffle: s_shufle * [1, a[0]] shuffle_of s_stable * [1, b[0]]
    s_shuffle: Column<Fixed>,
    s_stable: Column<Fixed>,

    // A FirstPhase challenge and SecondPhase column.  We define the following gates:
    // s_rlc * (a[0] + challenge * b[0] - e[0])
    // s_rlc * (c[0] + challenge * d[0] - e[0])
    s_rlc: Selector,
    e: Column<Advice>,
    challenge: Challenge,

    // Instance with a gate: s_instance * (a[0] - instance[0])
    s_instance: Selector,
    instance: Column<Instance>,
}

impl MyCircuitConfig {
    #[allow(clippy::type_complexity)]
    fn assign_gate<F: Field + From<u64>>(
        &self,
        region: &mut Region<'_, F>,
        offset: &mut usize,
        a_assigned: Option<AssignedCell<F, F>>,
        abcd: [u64; 4],
    ) -> Result<(AssignedCell<F, F>, [AssignedCell<F, F>; 4]), ErrorFront> {
        let [a, b, c, d] = abcd;
        self.s_gate.enable(region, *offset)?;
        let a_assigned = if let Some(a_assigned) = a_assigned {
            a_assigned
        } else {
            region.assign_advice(|| "", self.a, *offset, || Value::known(F::from(a)))?
        };
        let a = a_assigned.value();
        let [b, c, d] = [b, c, d].map(|v| Value::known(F::from(v)));
        let b_assigned = region.assign_advice(|| "", self.b, *offset, || b)?;
        let c_assigned = region.assign_advice(|| "", self.c, *offset, || c)?;
        let d_assigned = region.assign_fixed(|| "", self.d, *offset, || d)?;
        *offset += 1;
        // let res = a + b * c * d;
        let res = a
            .zip(b.zip(c.zip(d)))
            .map(|(a, (b, (c, d)))| *a + b * c * d);
        let res_assigned = region.assign_advice(|| "", self.a, *offset, || res)?;
        Ok((
            res_assigned,
            [a_assigned, b_assigned, c_assigned, d_assigned],
        ))
    }
}

#[derive(Clone)]
struct MyCircuit<F: Field, const WIDTH_FACTOR: usize> {
    k: u32,
    input: u64,
    _marker: std::marker::PhantomData<F>,
}

impl<F: Field + From<u64>, const WIDTH_FACTOR: usize> MyCircuit<F, WIDTH_FACTOR> {
    fn new(k: u32, input: u64) -> Self {
        Self {
            k,
            input,
            _marker: std::marker::PhantomData {},
        }
    }

    fn instance(&self) -> Vec<F> {
        let mut instance = Vec::new();
        let res = F::from(self.input);
        instance.push(res);
        let (b, c, d) = (3, 4, 1);
        let res = res + F::from(b) * F::from(c) * F::from(d);
        instance.push(res);
        let (b, c, d) = (6, 7, 1);
        let res = res + F::from(b) * F::from(c) * F::from(d);
        instance.push(res);
        let (b, c, d) = (8, 9, 1);
        let res = res + F::from(b) * F::from(c) * F::from(d);
        instance.push(res);
        instance.push(F::from(2));
        instance.push(F::from(2));
        instance
    }

    fn instances(&self) -> Vec<Vec<F>> {
        let instance = self.instance();
        (0..WIDTH_FACTOR).map(|_| instance.clone()).collect()
    }

    fn configure_single(meta: &mut ConstraintSystem<F>, id: usize) -> MyCircuitConfig {
        let s_gate = meta.selector();
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let d = meta.fixed_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(d);

        let s_lookup = meta.fixed_column();
        let s_ltable = meta.fixed_column();

        let s_shuffle = meta.fixed_column();
        let s_stable = meta.fixed_column();

        let s_rlc = meta.selector();
        let e = meta.advice_column_in(SecondPhase);
        let challenge = meta.challenge_usable_after(FirstPhase);

        let s_instance = meta.selector();
        let instance = meta.instance_column();
        meta.enable_equality(instance);

        let one = Expression::Constant(F::ONE);

        meta.create_gate(format!("gate_a.{id}"), |meta| {
            let s_gate = meta.query_selector(s_gate);
            let b = meta.query_advice(b, Rotation::cur());
            let a1 = meta.query_advice(a, Rotation::next());
            let a0 = meta.query_advice(a, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            let d = meta.query_fixed(d, Rotation::cur());

            vec![s_gate * (a0 + b * c * d - a1)]
        });

        meta.lookup_any(format!("lookup.{id}"), |meta| {
            let s_lookup = meta.query_fixed(s_lookup, Rotation::cur());
            let s_ltable = meta.query_fixed(s_ltable, Rotation::cur());
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            let d = meta.query_fixed(d, Rotation::cur());
            let lhs = [one.clone(), a, b].map(|c| c * s_lookup.clone());
            let rhs = [one.clone(), d, c].map(|c| c * s_ltable.clone());
            lhs.into_iter().zip(rhs).collect()
        });

        meta.shuffle(format!("shuffle.{id}"), |meta| {
            let s_shuffle = meta.query_fixed(s_shuffle, Rotation::cur());
            let s_stable = meta.query_fixed(s_stable, Rotation::cur());
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let lhs = [one.clone(), a].map(|c| c * s_shuffle.clone());
            let rhs = [one.clone(), b].map(|c| c * s_stable.clone());
            lhs.into_iter().zip(rhs).collect()
        });

        meta.create_gate(format!("gate_rlc.{id}"), |meta| {
            let s_rlc = meta.query_selector(s_rlc);
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            let d = meta.query_fixed(d, Rotation::cur());
            let e = meta.query_advice(e, Rotation::cur());
            let challenge = meta.query_challenge(challenge);

            vec![
                s_rlc.clone() * (a + challenge.clone() * b - e.clone()),
                s_rlc * (c + challenge * d - e),
            ]
        });

        MyCircuitConfig {
            s_gate,
            a,
            b,
            c,
            d,
            s_lookup,
            s_ltable,
            s_rlc,
            e,
            challenge,
            s_shuffle,
            s_stable,
            s_instance,
            instance,
        }
    }

    fn synthesize_unit(
        &self,
        config: &MyCircuitConfig,
        layouter: &mut impl Layouter<F>,
        id: usize,
        unit_id: usize,
    ) -> Result<(usize, Vec<AssignedCell<F, F>>), ErrorFront> {
        let challenge = layouter.get_challenge(config.challenge);
        let (rows, instance_copy) = layouter.assign_region(
            || format!("unit.{id}-{unit_id}"),
            |mut region| {
                // Column annotations
                region.name_column(|| format!("a.{id}"), config.a);
                region.name_column(|| format!("b.{id}"), config.b);
                region.name_column(|| format!("c.{id}"), config.c);
                region.name_column(|| format!("d.{id}"), config.d);
                region.name_column(|| format!("e.{id}"), config.e);
                region.name_column(|| format!("instance.{id}"), config.instance);
                region.name_column(|| format!("s_lookup.{id}"), config.s_lookup);
                region.name_column(|| format!("s_ltable.{id}"), config.s_ltable);
                region.name_column(|| format!("s_shuffle.{id}"), config.s_shuffle);
                region.name_column(|| format!("s_stable.{id}"), config.s_stable);

                let mut offset = 0;
                let mut instance_copy = Vec::new();
                // First "a" value comes from instance
                config.s_instance.enable(&mut region, offset).expect("todo");
                let res = region
                    .assign_advice_from_instance(|| "", config.instance, 0, config.a, offset)
                    .expect("todo");
                // Enable the gate on a few consecutive rows with rotations
                let (res, _) = config
                    .assign_gate(&mut region, &mut offset, Some(res), [0, 3, 4, 1])
                    .expect("todo");
                instance_copy.push(res.clone());
                let (res, _) = config
                    .assign_gate(&mut region, &mut offset, Some(res), [0, 6, 7, 1])
                    .expect("todo");
                instance_copy.push(res.clone());
                let (res, _) = config
                    .assign_gate(&mut region, &mut offset, Some(res), [0, 8, 9, 1])
                    .expect("todo");
                instance_copy.push(res.clone());
                let (res, _) = config
                    .assign_gate(
                        &mut region,
                        &mut offset,
                        Some(res),
                        [0, 0xffffffff, 0xdeadbeef, 1],
                    )
                    .expect("todo");
                let _ = config
                    .assign_gate(
                        &mut region,
                        &mut offset,
                        Some(res),
                        [0, 0xabad1d3a, 0x12345678, 0x42424242],
                    )
                    .expect("todo");
                offset += 1;

                // Enable the gate on non-consecutive rows with advice-advice copy constraints enabled
                let (_, abcd1) = config
                    .assign_gate(&mut region, &mut offset, None, [5, 2, 1, 1])
                    .expect("todo");
                offset += 1;
                let (_, abcd2) = config
                    .assign_gate(&mut region, &mut offset, None, [2, 3, 1, 1])
                    .expect("todo");
                offset += 1;
                let (_, abcd3) = config
                    .assign_gate(&mut region, &mut offset, None, [4, 2, 1, 1])
                    .expect("todo");
                offset += 1;
                region
                    .constrain_equal(abcd1[1].cell(), abcd2[0].cell())
                    .expect("todo");
                region
                    .constrain_equal(abcd2[0].cell(), abcd3[1].cell())
                    .expect("todo");
                instance_copy.push(abcd1[1].clone());
                instance_copy.push(abcd2[0].clone());

                // Enable the gate on non-consecutive rows with advice-fixed copy constraints enabled
                let (_, abcd1) = config
                    .assign_gate(&mut region, &mut offset, None, [5, 9, 1, 9])
                    .expect("todo");
                offset += 1;
                let (_, abcd2) = config
                    .assign_gate(&mut region, &mut offset, None, [2, 9, 1, 1])
                    .expect("todo");
                offset += 1;
                let (_, abcd3) = config
                    .assign_gate(&mut region, &mut offset, None, [9, 2, 1, 1])
                    .expect("todo");
                offset += 1;
                region
                    .constrain_equal(abcd1[1].cell(), abcd1[3].cell())
                    .expect("todo");
                region
                    .constrain_equal(abcd2[1].cell(), abcd1[3].cell())
                    .expect("todo");
                region
                    .constrain_equal(abcd3[0].cell(), abcd1[3].cell())
                    .expect("todo");

                // Enable a dynamic lookup (powers of two)
                let table: Vec<_> = (0u64..=10).map(|exp| (exp, 2u64.pow(exp as u32))).collect();
                let lookups = [(2, 4), (2, 4), (10, 1024), (0, 1), (2, 4)];
                for (table_row, lookup_row) in table
                    .iter()
                    .zip(lookups.iter().chain(std::iter::repeat(&(0, 1))))
                {
                    region
                        .assign_fixed(|| "", config.s_lookup, offset, || Value::known(F::ONE))
                        .expect("todo");
                    region
                        .assign_fixed(|| "", config.s_ltable, offset, || Value::known(F::ONE))
                        .expect("todo");
                    let lookup_row0 = Value::known(F::from(lookup_row.0));
                    let lookup_row1 = Value::known(F::from(lookup_row.1));
                    region
                        .assign_advice(|| "", config.a, offset, || lookup_row0)
                        .expect("todo");
                    region
                        .assign_advice(|| "", config.b, offset, || lookup_row1)
                        .expect("todo");
                    let table_row0 = Value::known(F::from(table_row.0));
                    let table_row1 = Value::known(F::from(table_row.1));
                    region
                        .assign_fixed(|| "", config.d, offset, || table_row0)
                        .expect("todo");
                    region
                        .assign_advice(|| "", config.c, offset, || table_row1)
                        .expect("todo");
                    offset += 1;
                }

                // Enable RLC gate 3 times
                for abcd in [[3, 5, 3, 5], [8, 9, 8, 9], [111, 222, 111, 222]] {
                    config.s_rlc.enable(&mut region, offset)?;
                    let (_, _) = config
                        .assign_gate(&mut region, &mut offset, None, abcd)
                        .expect("todo");
                    let rlc = challenge.map(|ch| {
                        let [a, b, ..] = abcd;
                        F::from(a) + ch * F::from(b)
                    });
                    region
                        .assign_advice(|| "", config.e, offset - 1, || rlc)
                        .expect("todo");
                    offset += 1;
                }

                // Enable a dynamic shuffle (sequence from 0 to 15)
                let table: Vec<_> = (0u64..16).collect();
                let shuffle = [0u64, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15];
                assert_eq!(table.len(), shuffle.len());

                for (table_row, shuffle_row) in table.iter().zip(shuffle.iter()) {
                    region
                        .assign_fixed(|| "", config.s_shuffle, offset, || Value::known(F::ONE))
                        .expect("todo");
                    region
                        .assign_fixed(|| "", config.s_stable, offset, || Value::known(F::ONE))
                        .expect("todo");
                    let shuffle_row0 = Value::known(F::from(*shuffle_row));
                    region
                        .assign_advice(|| "", config.a, offset, || shuffle_row0)
                        .expect("todo");
                    let table_row0 = Value::known(F::from(*table_row));
                    region
                        .assign_advice(|| "", config.b, offset, || table_row0)
                        .expect("todo");
                    offset += 1;
                }

                Ok((offset, instance_copy))
            },
        )?;

        Ok((rows, instance_copy))
    }
}

impl<F: Field + From<u64>, const WIDTH_FACTOR: usize> Circuit<F> for MyCircuit<F, WIDTH_FACTOR> {
    type Config = Vec<MyCircuitConfig>;
    type FloorPlanner = SimpleFloorPlanner;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Vec<MyCircuitConfig> {
        assert!(WIDTH_FACTOR > 0);
        (0..WIDTH_FACTOR)
            .map(|id| Self::configure_single(meta, id))
            .collect()
    }

    fn synthesize(
        &self,
        config: Vec<MyCircuitConfig>,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), ErrorFront> {
        // - 2 queries from first gate
        // - 3 for permutation argument
        // - 1 for multipoen
        // - 1 for the last row of grand product poly to check that the product result is 1
        // - 1 for off-by-one errors
        let unusable_rows = 2 + 3 + 1 + 1 + 1;
        let max_rows = 2usize.pow(self.k) - unusable_rows;
        for (id, config) in config.iter().enumerate() {
            let mut total_rows = 0;
            let mut unit_id = 0;
            loop {
                let (rows, instance_copy) = self
                    .synthesize_unit(config, &mut layouter, id, unit_id)
                    .expect("todo");
                if total_rows == 0 {
                    for (i, instance) in instance_copy.iter().enumerate() {
                        layouter.constrain_instance(instance.cell(), config.instance, 1 + i)?;
                    }
                }
                total_rows += rows;
                if total_rows + rows > max_rows {
                    break;
                }
                unit_id += 1;
            }
            assert!(total_rows <= max_rows);
        }
        Ok(())
    }
}

use halo2_proofs::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
use halo2_proofs::poly::kzg::multiopen::{ProverSHPLONK, VerifierSHPLONK};
use halo2_proofs::poly::kzg::strategy::SingleStrategy;
use halo2curves::bn256::{Bn256, Fr, G1Affine};
use rand_core::block::BlockRng;
use rand_core::block::BlockRngCore;

// One number generator, that can be used as a deterministic Rng, outputing fixed values.
struct OneNg {}

impl BlockRngCore for OneNg {
    type Item = u32;
    type Results = [u32; 16];

    fn generate(&mut self, results: &mut Self::Results) {
        for elem in results.iter_mut() {
            *elem = 1;
        }
    }
}

#[test]
fn test_mycircuit_mock() {
    let k = 6;
    const WIDTH_FACTOR: usize = 2;
    let circuit: MyCircuit<Fr, WIDTH_FACTOR> = MyCircuit::new(k, 42);
    let instances = circuit.instances();
    let prover = MockProver::run(k, &circuit, instances).unwrap();
    prover.assert_satisfied();
}

use std::time::Instant;

const K: u32 = 6;
const WIDTH_FACTOR: usize = 1;

#[test]
fn test_mycircuit_full_legacy() {
    #[cfg(feature = "heap-profiling")]
    let _profiler = dhat::Profiler::new_heap();

    use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk};

    let k = K;
    let circuit: MyCircuit<Fr, WIDTH_FACTOR> = MyCircuit::new(k, 42);

    // Setup
    let mut rng = BlockRng::new(OneNg {});
    let params = ParamsKZG::<Bn256>::setup(k, &mut rng);
    let verifier_params = params.verifier_params();
    let start = Instant::now();
    let vk = keygen_vk(&params, &circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("keygen_pk should not fail");
    println!("Keygen: {:?}", start.elapsed());

    // Proving
    let instances = circuit.instances();
    let instances_slice: &[&[Fr]] = &(instances
        .iter()
        .map(|instance| instance.as_slice())
        .collect::<Vec<_>>());

    let start = Instant::now();
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
    create_proof::<KZGCommitmentScheme<Bn256>, ProverSHPLONK<'_, Bn256>, _, _, _, _>(
        &params,
        &pk,
        &[circuit],
        &[instances_slice],
        &mut rng,
        &mut transcript,
    )
    .expect("proof generation should not fail");
    let proof = transcript.finalize();
    println!("Prove: {:?}", start.elapsed());

    // Verify
    let start = Instant::now();
    let mut verifier_transcript =
        Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof.as_slice());
    let strategy = SingleStrategy::new(verifier_params);

    verify_proof::<KZGCommitmentScheme<Bn256>, VerifierSHPLONK<'_, Bn256>, _, _, _>(
        &params,
        &vk,
        strategy,
        &[instances_slice],
        &mut verifier_transcript,
    )
    .expect("verify succeeds");
    println!("Verify: {:?}", start.elapsed());
}

#[test]
fn test_mycircuit_full_split() {
    #[cfg(feature = "heap-profiling")]
    let _profiler = dhat::Profiler::new_heap();

    let k = K;
    let circuit: MyCircuit<Fr, WIDTH_FACTOR> = MyCircuit::new(k, 42);
    let (compiled_circuit, config, cs) = compile_circuit(k, &circuit, false).unwrap();

    // Setup
    let mut rng = BlockRng::new(OneNg {});
    let params = ParamsKZG::<Bn256>::setup(k, &mut rng);
    let verifier_params = params.verifier_params();
    let start = Instant::now();
    let vk = keygen_vk_v2(&params, &compiled_circuit).expect("keygen_vk should not fail");
    let pk =
        keygen_pk_v2(&params, vk.clone(), &compiled_circuit).expect("keygen_pk should not fail");
    println!("Keygen: {:?}", start.elapsed());
    drop(compiled_circuit);

    // Proving
    println!("Proving...");
    let instances = circuit.instances();
    let instances_slice: &[&[Fr]] = &(instances
        .iter()
        .map(|instance| instance.as_slice())
        .collect::<Vec<_>>());

    let start = Instant::now();
    let mut witness_calc = WitnessCalculator::new(k, &circuit, &config, &cs, instances_slice);
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
    let mut prover =
        ProverV2Single::<KZGCommitmentScheme<Bn256>, ProverSHPLONK<'_, Bn256>, _, _, _>::new(
            &params,
            &pk,
            instances_slice,
            &mut rng,
            &mut transcript,
        )
        .unwrap();
    let mut challenges = HashMap::new();
    for phase in 0..cs.phases().count() {
        println!("phase {phase}");
        let witness = witness_calc.calc(phase as u8, &challenges).unwrap();
        challenges = prover.commit_phase(phase as u8, witness).unwrap();
    }
    prover.create_proof().unwrap();
    let proof = transcript.finalize();
    println!("Prove: {:?}", start.elapsed());

    // Verify
    let start = Instant::now();
    println!("Verifying...");
    let mut verifier_transcript =
        Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof.as_slice());
    let strategy = SingleStrategy::new(verifier_params);

    verify_proof_single::<KZGCommitmentScheme<Bn256>, VerifierSHPLONK<'_, Bn256>, _, _, _>(
        &params,
        &vk,
        strategy,
        instances_slice,
        &mut verifier_transcript,
    )
    .expect("verify succeeds");
    println!("Verify: {:?}", start.elapsed());
}
