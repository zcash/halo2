#[macro_use]
extern crate criterion;

use group::ff::Field;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::circuit::{Cell, Layouter, SimpleFloorPlanner};
use halo2_proofs::pairing::bn256::{Bn256, Fr as Fp, G1Affine};
use halo2_proofs::plonk::*;
use halo2_proofs::poly::{
    commitment::{Params, ParamsVerifier},
    Rotation,
};
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use rand_core::OsRng;

use std::marker::PhantomData;

use criterion::{BenchmarkId, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    #[derive(Clone)]
    struct PlonkConfig {
        a: Column<Advice>,
        b: Column<Advice>,
        c: Column<Advice>,

        sa: Column<Fixed>,
        sb: Column<Fixed>,
        sc: Column<Fixed>,
        sm: Column<Fixed>,
    }

    trait StandardCs<FF: FieldExt> {
        fn raw_multiply<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Result<(FF, FF, FF), Error>;
        fn raw_add<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Result<(FF, FF, FF), Error>;
        fn copy(&self, layouter: &mut impl Layouter<FF>, a: Cell, b: Cell) -> Result<(), Error>;
    }

    #[derive(Clone)]
    struct MyCircuit<F: FieldExt> {
        a: Option<F>,
        k: u32,
    }

    struct StandardPlonk<F: FieldExt> {
        config: PlonkConfig,
        _marker: PhantomData<F>,
    }

    impl<FF: FieldExt> StandardPlonk<FF> {
        fn new(config: PlonkConfig) -> Self {
            StandardPlonk {
                config,
                _marker: PhantomData,
            }
        }
    }

    impl<FF: FieldExt> StandardCs<FF> for StandardPlonk<FF> {
        fn raw_multiply<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            mut f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Result<(FF, FF, FF), Error>,
        {
            layouter.assign_region(
                || "raw_multiply",
                |mut region| {
                    let mut value = None;
                    let lhs = region.assign_advice(
                        || "lhs",
                        self.config.a,
                        0,
                        || {
                            value = Some(f()?);
                            Ok(value.ok_or(Error::Synthesis)?.0)
                        },
                    )?;
                    let rhs = region.assign_advice(
                        || "rhs",
                        self.config.b,
                        0,
                        || Ok(value.ok_or(Error::Synthesis)?.1),
                    )?;
                    let out = region.assign_advice(
                        || "out",
                        self.config.c,
                        0,
                        || Ok(value.ok_or(Error::Synthesis)?.2),
                    )?;

                    region.assign_fixed(|| "a", self.config.sa, 0, || Ok(FF::zero()))?;
                    region.assign_fixed(|| "b", self.config.sb, 0, || Ok(FF::zero()))?;
                    region.assign_fixed(|| "c", self.config.sc, 0, || Ok(FF::one()))?;
                    region.assign_fixed(|| "a * b", self.config.sm, 0, || Ok(FF::one()))?;
                    Ok((lhs.cell(), rhs.cell(), out.cell()))
                },
            )
        }
        fn raw_add<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            mut f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Result<(FF, FF, FF), Error>,
        {
            layouter.assign_region(
                || "raw_add",
                |mut region| {
                    let mut value = None;
                    let lhs = region.assign_advice(
                        || "lhs",
                        self.config.a,
                        0,
                        || {
                            value = Some(f()?);
                            Ok(value.ok_or(Error::Synthesis)?.0)
                        },
                    )?;
                    let rhs = region.assign_advice(
                        || "rhs",
                        self.config.b,
                        0,
                        || Ok(value.ok_or(Error::Synthesis)?.1),
                    )?;
                    let out = region.assign_advice(
                        || "out",
                        self.config.c,
                        0,
                        || Ok(value.ok_or(Error::Synthesis)?.2),
                    )?;

                    region.assign_fixed(|| "a", self.config.sa, 0, || Ok(FF::one()))?;
                    region.assign_fixed(|| "b", self.config.sb, 0, || Ok(FF::one()))?;
                    region.assign_fixed(|| "c", self.config.sc, 0, || Ok(FF::one()))?;
                    region.assign_fixed(|| "a * b", self.config.sm, 0, || Ok(FF::zero()))?;
                    Ok((lhs.cell(), rhs.cell(), out.cell()))
                },
            )
        }
        fn copy(
            &self,
            layouter: &mut impl Layouter<FF>,
            left: Cell,
            right: Cell,
        ) -> Result<(), Error> {
            layouter.assign_region(|| "copy", |mut region| region.constrain_equal(left, right))
        }
    }

    impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
        type Config = PlonkConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self { a: None, k: self.k }
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> PlonkConfig {
            meta.set_minimum_degree(5);

            let a = meta.advice_column();
            let b = meta.advice_column();
            let c = meta.advice_column();

            meta.enable_equality(a);
            meta.enable_equality(b);
            meta.enable_equality(c);

            let sm = meta.fixed_column();
            let sa = meta.fixed_column();
            let sb = meta.fixed_column();
            let sc = meta.fixed_column();

            meta.create_gate("Combined add-mult", |meta| {
                let a = meta.query_advice(a, Rotation::cur());
                let b = meta.query_advice(b, Rotation::cur());
                let c = meta.query_advice(c, Rotation::cur());

                let sa = meta.query_fixed(sa, Rotation::cur());
                let sb = meta.query_fixed(sb, Rotation::cur());
                let sc = meta.query_fixed(sc, Rotation::cur());
                let sm = meta.query_fixed(sm, Rotation::cur());

                vec![a.clone() * sa + b.clone() * sb + a * b * sm - (c * sc)]
            });

            PlonkConfig {
                a,
                b,
                c,
                sa,
                sb,
                sc,
                sm,
            }
        }

        fn synthesize(
            &self,
            config: PlonkConfig,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let cs = StandardPlonk::new(config);

            for _ in 0..((1 << (self.k - 1)) - 3) {
                let mut a_squared = None;
                let (a0, _, c0) = cs.raw_multiply(&mut layouter, || {
                    a_squared = self.a.map(|a| a.square());
                    Ok((
                        self.a.ok_or(Error::Synthesis)?,
                        self.a.ok_or(Error::Synthesis)?,
                        a_squared.ok_or(Error::Synthesis)?,
                    ))
                })?;
                let (a1, b1, _) = cs.raw_add(&mut layouter, || {
                    let fin = a_squared.and_then(|a2| self.a.map(|a| a + a2));
                    Ok((
                        self.a.ok_or(Error::Synthesis)?,
                        a_squared.ok_or(Error::Synthesis)?,
                        fin.ok_or(Error::Synthesis)?,
                    ))
                })?;
                cs.copy(&mut layouter, a0, a1)?;
                cs.copy(&mut layouter, b1, c0)?;
            }

            Ok(())
        }
    }

    fn keygen(
        k: u32,
    ) -> (
        Params<G1Affine>,
        ParamsVerifier<Bn256>,
        ProvingKey<G1Affine>,
    ) {
        let params: Params<G1Affine> = Params::<G1Affine>::unsafe_setup::<Bn256>(k);
        let params_verifier: ParamsVerifier<Bn256> = params.verifier(0).unwrap();

        let empty_circuit: MyCircuit<Fp> = MyCircuit { a: None, k };
        let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
        let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");
        (params, params_verifier, pk)
    }

    fn prover(k: u32, params: &Params<G1Affine>, pk: &ProvingKey<G1Affine>) -> Vec<u8> {
        let rng = OsRng;

        let circuit: MyCircuit<Fp> = MyCircuit {
            a: Some(Fp::random(rng)),
            k,
        };

        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        create_proof(params, pk, &[circuit], &[&[]], rng, &mut transcript)
            .expect("proof generation should not fail");
        transcript.finalize()
    }

    fn verifier(params: &ParamsVerifier<Bn256>, vk: &VerifyingKey<G1Affine>, proof: &[u8]) {
        let strategy = SingleVerifier::new(params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(proof);
        verify_proof(params, vk, strategy, &[&[]], &mut transcript).unwrap();
    }

    let k_range = 8..=16;

    let mut keygen_group = c.benchmark_group("plonk-keygen");
    keygen_group.sample_size(10);
    for k in k_range.clone() {
        keygen_group.bench_with_input(BenchmarkId::from_parameter(k), &k, |b, &k| {
            b.iter(|| keygen(k));
        });
    }
    keygen_group.finish();

    let mut prover_group = c.benchmark_group("plonk-prover");
    prover_group.sample_size(10);
    for k in k_range.clone() {
        let (params, _, pk) = keygen(k);

        prover_group.bench_with_input(
            BenchmarkId::from_parameter(k),
            &(k, &params, &pk),
            |b, &(k, params, pk)| {
                b.iter(|| prover(k, params, pk));
            },
        );
    }
    prover_group.finish();

    let mut verifier_group = c.benchmark_group("plonk-verifier");
    for k in k_range {
        let (params, params_verifier, pk) = keygen(k);
        let proof = prover(k, &params, &pk);

        verifier_group.bench_with_input(
            BenchmarkId::from_parameter(k),
            &(&params, pk.get_vk(), &proof[..]),
            |b, &(_, vk, proof)| {
                b.iter(|| verifier(&params_verifier, vk, proof));
            },
        );
    }
    verifier_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
