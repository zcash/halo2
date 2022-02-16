use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Layouter, SimpleFloorPlanner},
    plonk::*,
    poly::{commitment::Params, commitment::ParamsVerifier, Rotation},
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use pairing::bn256::{Bn256, Fr as Fp, G1Affine};
use rand_core::OsRng;

use std::marker::PhantomData;

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
            || "mul",
            |mut region| {
                let mut values = None;
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.a,
                    0,
                    || {
                        values = Some(f()?);
                        Ok(values.ok_or(Error::Synthesis)?.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.b,
                    0,
                    || Ok(values.ok_or(Error::Synthesis)?.1),
                )?;

                let out = region.assign_advice(
                    || "out",
                    self.config.c,
                    0,
                    || Ok(values.ok_or(Error::Synthesis)?.2),
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
            || "mul",
            |mut region| {
                let mut values = None;
                let lhs = region.assign_advice(
                    || "lhs",
                    self.config.a,
                    0,
                    || {
                        values = Some(f()?);
                        Ok(values.ok_or(Error::Synthesis)?.0)
                    },
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    self.config.b,
                    0,
                    || Ok(values.ok_or(Error::Synthesis)?.1),
                )?;

                let out = region.assign_advice(
                    || "out",
                    self.config.c,
                    0,
                    || Ok(values.ok_or(Error::Synthesis)?.2),
                )?;

                region.assign_fixed(|| "a", self.config.sa, 0, || Ok(FF::one()))?;
                region.assign_fixed(|| "b", self.config.sb, 0, || Ok(FF::one()))?;
                region.assign_fixed(|| "c", self.config.sc, 0, || Ok(FF::one()))?;
                region.assign_fixed(|| "a * b", self.config.sm, 0, || Ok(FF::zero()))?;

                Ok((lhs.cell(), rhs.cell(), out.cell()))
            },
        )
    }

    fn copy(&self, layouter: &mut impl Layouter<FF>, left: Cell, right: Cell) -> Result<(), Error> {
        layouter.assign_region(
            || "copy",
            |mut region| {
                region.constrain_equal(left, right)?;
                region.constrain_equal(left, right)
            },
        )
    }
}

impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
    type Config = PlonkConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { a: None, k: self.k }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> PlonkConfig {
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

        meta.create_gate("mini plonk", |meta| {
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());

            let sa = meta.query_fixed(sa, Rotation::cur());
            let sb = meta.query_fixed(sb, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());
            let sm = meta.query_fixed(sm, Rotation::cur());

            vec![a.clone() * sa + b.clone() * sb + a * b * sm + (c * sc * (-F::one()))]
        });

        PlonkConfig {
            a,
            b,
            c,
            sa,
            sb,
            sc,
            sm,
            // perm,
        }
    }

    fn synthesize(&self, config: PlonkConfig, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let cs = StandardPlonk::new(config);

        for _ in 0..1 << ((self.k - 1) - 3) {
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

fn main() {
    let k = 8;
    let public_inputs_size = 0;

    let empty_circuit: MyCircuit<Fp> = MyCircuit { a: None, k };

    // Initialize the polynomial commitment parameters
    let params: Params<G1Affine> = Params::<G1Affine>::unsafe_setup::<Bn256>(k);
    let params_verifier: ParamsVerifier<Bn256> = params.verifier(public_inputs_size).unwrap();

    // Initialize the proving key
    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let circuit: MyCircuit<Fp> = MyCircuit {
        a: Some(Fp::from(5)),
        k,
    };

    // Create a proof
    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

    use std::time::Instant;
    let _dur = Instant::now();

    create_proof(&params, &pk, &[circuit], &[&[]], OsRng, &mut transcript)
        .expect("proof generation should not fail");

    println!("proving period: {:?}", _dur.elapsed());

    let proof = transcript.finalize();

    let strategy = SingleVerifier::new(&params_verifier);
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

    verify_proof(
        &params_verifier,
        pk.get_vk(),
        strategy,
        &[&[]],
        &mut transcript,
    )
    .unwrap();
}
