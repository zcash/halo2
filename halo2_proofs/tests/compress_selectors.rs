#![allow(non_snake_case)]

use std::marker::PhantomData;

use ff::PrimeField;
use halo2_frontend::plonk::Error;
use halo2_proofs::circuit::{Cell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::poly::Rotation;

use halo2_backend::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
};
use halo2_middleware::zal::impls::{H2cEngine, PlonkEngineConfig};
use halo2_proofs::arithmetic::Field;
use halo2_proofs::plonk::{
    create_proof_with_engine, keygen_pk_custom, keygen_vk_custom, verify_proof, Advice, Assigned,
    Circuit, Column, ConstraintSystem, Instance, Selector,
};
use halo2_proofs::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
use halo2_proofs::poly::kzg::multiopen::{ProverSHPLONK, VerifierSHPLONK};
use halo2_proofs::poly::kzg::strategy::SingleStrategy;
use halo2curves::bn256::{Bn256, Fr, G1Affine};
use rand_core::block::BlockRng;
use rand_core::block::BlockRngCore;

// One number generator, that can be used as a deterministic Rng, outputing fixed values.
pub struct OneNg {}

impl BlockRngCore for OneNg {
    type Item = u32;
    type Results = [u32; 16];

    fn generate(&mut self, results: &mut Self::Results) {
        for elem in results.iter_mut() {
            *elem = 1;
        }
    }
}

#[derive(Debug, Clone)]
struct MyCircuitConfig {
    l: Column<Advice>,
    r: Column<Advice>,
    o: Column<Advice>,

    s_add: Selector,
    s_mul: Selector,
    #[allow(dead_code)]
    s_cubed: Selector,

    PI: Column<Instance>,
}

#[derive(Debug)]
struct MyCircuitChip<F: Field> {
    config: MyCircuitConfig,
    marker: PhantomData<F>,
}

trait MyCircuitComposer<F: Field> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error>;

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error>;

    #[allow(dead_code)]
    fn cube<FM>(&self, layouter: &mut impl Layouter<F>, f: FM) -> Result<(Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>)>;
}

impl<F: Field> MyCircuitChip<F> {
    fn construct(config: MyCircuitConfig) -> Self {
        Self {
            config,
            marker: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> MyCircuitConfig {
        let l = meta.advice_column();
        let r = meta.advice_column();
        let o = meta.advice_column();

        let s_add = meta.selector();
        let s_mul = meta.selector();
        let s_cubed = meta.selector();

        let PI = meta.instance_column();

        meta.enable_equality(l);
        meta.enable_equality(r);
        meta.enable_equality(o);

        meta.enable_equality(PI);

        meta.create_gate("add", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let r = meta.query_advice(r, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let s_add = meta.query_selector(s_add);

            vec![s_add * (l + r - o)]
        });

        meta.create_gate("mul", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let r = meta.query_advice(r, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let s_mul = meta.query_selector(s_mul);

            vec![s_mul * (l * r - o)]
        });

        // NOTE: This gate is placement for "compress_selectors" logic testing. Not really used.
        meta.create_gate("cubed", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let s_cubed = meta.query_selector(s_cubed);

            vec![s_cubed * (l.clone() * l.clone() * l - o)]
        });

        MyCircuitConfig {
            l,
            r,
            o,
            s_add,
            s_mul,
            s_cubed,
            PI,
        }
    }
}

impl<F: Field> MyCircuitComposer<F> for MyCircuitChip<F> {
    fn raw_multiply<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        let mut values = None;
        layouter.assign_region(
            || "multiply",
            |mut region| {
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|x| x.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|x| x.1),
                )?;
                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|x| x.2),
                )?;

                region.enable_selector(|| "mul", &self.config.s_mul, 0)?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn raw_add<FM>(
        &self,
        layouter: &mut impl Layouter<F>,
        mut f: FM,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>,
    {
        let mut values = None;
        layouter.assign_region(
            || "add",
            |mut region| {
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|x| x.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.r,
                    0,
                    || values.unwrap().map(|x| x.1),
                )?;
                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|x| x.2),
                )?;

                region.enable_selector(|| "add", &self.config.s_add, 0)?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn copy(&self, layouter: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error> {
        layouter.assign_region(|| "copy values", |mut region| region.constrain_equal(a, b))
    }

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: Cell,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell, self.config.PI, row)
    }

    fn cube<FM>(&self, layouter: &mut impl Layouter<F>, mut f: FM) -> Result<(Cell, Cell), Error>
    where
        FM: FnMut() -> Value<(Assigned<F>, Assigned<F>)>,
    {
        let mut values = None;
        layouter.assign_region(
            || "cube",
            |mut region| {
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.l,
                    0,
                    || {
                        values = Some(f());
                        values.unwrap().map(|x| x.0)
                    },
                )?;
                let out = region.assign_advice(
                    || "out",
                    self.config.o,
                    0,
                    || values.unwrap().map(|x| x.1),
                )?;

                region.enable_selector(|| "cube", &self.config.s_cubed, 0)?;

                Ok((lhs.cell(), out.cell()))
            },
        )
    }
}

#[derive(Debug, Clone, Default)]
struct MyCircuit<F: Field> {
    x: Value<F>,
    y: Value<F>,
    constant: F,
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    type Config = MyCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        MyCircuitChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let cs = MyCircuitChip::construct(config);

        let x: Value<Assigned<F>> = self.x.into();
        let y: Value<Assigned<F>> = self.y.into();
        let consty = Assigned::from(self.constant);

        let (a0, b0, c0) = cs.raw_multiply(&mut layouter, || x.map(|x| (x, x, x * x)))?;
        cs.copy(&mut layouter, a0, b0)?;

        let (a1, b1, c1) = cs.raw_multiply(&mut layouter, || y.map(|y| (y, y, y * y)))?;
        cs.copy(&mut layouter, a1, b1)?;

        let (a2, b2, c2) = cs.raw_add(&mut layouter, || {
            x.zip(y).map(|(x, y)| (x * x, y * y, x * x + y * y))
        })?;
        cs.copy(&mut layouter, a2, c0)?;
        cs.copy(&mut layouter, b2, c1)?;

        let (a3, b3, c3) = cs.raw_add(&mut layouter, || {
            x.zip(y)
                .map(|(x, y)| (x * x + y * y, consty, x * x + y * y + consty))
        })?;
        cs.copy(&mut layouter, a3, c2)?;
        cs.expose_public(&mut layouter, b3, 0)?;

        cs.expose_public(&mut layouter, c3, 1)?;

        Ok(())
    }
}

fn test_mycircuit(
    vk_keygen_compress_selectors: bool,
    pk_keygen_compress_selectors: bool,
) -> Result<(), halo2_proofs::plonk::Error> {
    let engine = PlonkEngineConfig::new()
        .set_curve::<G1Affine>()
        .set_msm(H2cEngine::new())
        .build();
    let k = 4;
    let circuit: MyCircuit<Fr> = MyCircuit {
        x: Value::known(Fr::one()),
        y: Value::known(Fr::one()),
        constant: Fr::one(),
    };

    // Setup
    let mut rng = BlockRng::new(OneNg {});
    let params = ParamsKZG::<Bn256>::setup(k, &mut rng);
    let verifier_params = params.verifier_params();
    let vk = keygen_vk_custom(&params, &circuit, vk_keygen_compress_selectors)?;
    let pk = keygen_pk_custom(&params, vk.clone(), &circuit, pk_keygen_compress_selectors)?;

    // Proving
    #[allow(clippy::useless_vec)]
    let instances = vec![vec![Fr::one(), Fr::from_u128(3)]];
    let instances_slice: &[&[Fr]] = &(instances
        .iter()
        .map(|instance| instance.as_slice())
        .collect::<Vec<_>>());

    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
    create_proof_with_engine::<KZGCommitmentScheme<Bn256>, ProverSHPLONK<'_, Bn256>, _, _, _, _, _>(
        engine,
        &params,
        &pk,
        &[circuit],
        &[instances_slice],
        &mut rng,
        &mut transcript,
    )?;
    let proof = transcript.finalize();

    // Verify
    let mut verifier_transcript =
        Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof.as_slice());
    let strategy = SingleStrategy::new(&verifier_params);

    verify_proof::<KZGCommitmentScheme<Bn256>, VerifierSHPLONK<Bn256>, _, _, _>(
        &verifier_params,
        &vk,
        strategy,
        &[instances_slice],
        &mut verifier_transcript,
    )
    .map_err(halo2_proofs::plonk::Error::Backend)
}

/*

How the `compress_selectors` works in `MyCircuit` under the hood:

# compress = false

    selector `s_add` -> fixed `s_add`
    - 1 when `s_add` enabled, 0 otherwise

    selector `s_mul` -> fixed `s_mul`
    - 1 when `s_mul` enabled, 0 otherwise

    selector `s_cubed` -> fixed `s_cubed`
    - 1 when `s_cubed` enabled, 0 otherwise

    Selector queries in expressions become the corresponding fixed column queries
    at rotation 0.


# compress = true

    selector `s_add`, `s_mul` -> fixed `s_add_mul`
    - 0 when `s_add` disabled and `s_mul` disabled
    - 1 when only `s_add` enabled
    - 2 when only `s_mul` enabled

    selector `s_cubed` -> fixed `s_cubed`
    - 1 when `s_cubed` enabled, 0 otherwise
    - NOTE: `s_cubed` is not compressed to avoid growing the max degree which is 3

    Selector query for `s_add` becomes (`s_add_mul`)*(2 - `s_add_mul`)
    Selector query for `s_mul` becomes (`s_add_mul`)*(1 - `s_add_mul`)
    Selector query for `s_cubed` becomes the corresponding fixed column query
    at rotation 0.

*/

#[test]
fn test_success() {
    // vk & pk keygen both WITH compress
    assert!(test_mycircuit(true, true).is_ok());

    // vk & pk keygen both WITHOUT compress
    assert!(test_mycircuit(false, false).is_ok());
}

#[should_panic]
#[test]
fn test_failure_1() {
    // vk keygen WITH compress
    // pk keygen WITHOUT compress
    assert!(test_mycircuit(false, true).is_err());
}

#[test]
fn test_failure_2() {
    // vk keygen WITHOUT compress
    // pk keygen WITH compress
    assert!(test_mycircuit(true, false).is_err());
}
