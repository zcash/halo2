use halo2::{
    arithmetic::FieldExt,
    circuit::{Cell, Layouter, Region, SimpleFloorPlanner},
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, TableColumn},
    poly::Rotation,
};
use std::marker::PhantomData;

/// This represents an advice column at a certain row in the ConstraintSystem
#[derive(Copy, Clone, Debug)]
pub struct Variable(Column<Advice>, usize);

#[derive(Clone)]
struct PlonkConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    d: Column<Advice>,
    e: Column<Advice>,

    sa: Column<Fixed>,
    sb: Column<Fixed>,
    sc: Column<Fixed>,
    sm: Column<Fixed>,
    sl: TableColumn,
}

trait StandardCs<FF: FieldExt> {
    fn raw_multiply<F>(&self, region: &mut Region<FF>, f: F) -> Result<(Cell, Cell, Cell), Error>
    where
        F: FnMut() -> Result<(FF, FF, FF), Error>;
    fn raw_add<F>(&self, region: &mut Region<FF>, f: F) -> Result<(Cell, Cell, Cell), Error>
    where
        F: FnMut() -> Result<(FF, FF, FF), Error>;
    fn copy(&self, region: &mut Region<FF>, a: Cell, b: Cell) -> Result<(), Error>;
    fn lookup_table(&self, layouter: &mut impl Layouter<FF>, values: &[FF]) -> Result<(), Error>;
}

struct MyCircuit<F: FieldExt> {
    a: Option<F>,
    lookup_table: Vec<F>,
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
        region: &mut Region<FF>,
        mut f: F,
    ) -> Result<(Cell, Cell, Cell), Error>
    where
        F: FnMut() -> Result<(FF, FF, FF), Error>,
    {
        let mut value = None;
        let lhs = region.assign_advice(
            || "lhs",
            self.config.a,
            0,
            || {
                value = Some(f()?);
                Ok(value.ok_or(Error::SynthesisError)?.0)
            },
        )?;
        region.assign_advice(
            || "lhs^4",
            self.config.d,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.0.square().square()),
        )?;
        let rhs = region.assign_advice(
            || "rhs",
            self.config.b,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.1),
        )?;
        region.assign_advice(
            || "rhs^4",
            self.config.e,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.1.square().square()),
        )?;
        let out = region.assign_advice(
            || "out",
            self.config.c,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.2),
        )?;

        region.assign_fixed(|| "a", self.config.sa, 0, || Ok(FF::zero()))?;
        region.assign_fixed(|| "b", self.config.sb, 0, || Ok(FF::zero()))?;
        region.assign_fixed(|| "c", self.config.sc, 0, || Ok(FF::one()))?;
        region.assign_fixed(|| "a * b", self.config.sm, 0, || Ok(FF::one()))?;
        Ok((lhs, rhs, out))
    }
    fn raw_add<F>(&self, region: &mut Region<FF>, mut f: F) -> Result<(Cell, Cell, Cell), Error>
    where
        F: FnMut() -> Result<(FF, FF, FF), Error>,
    {
        let mut value = None;
        let lhs = region.assign_advice(
            || "lhs",
            self.config.a,
            0,
            || {
                value = Some(f()?);
                Ok(value.ok_or(Error::SynthesisError)?.0)
            },
        )?;
        region.assign_advice(
            || "lhs^4",
            self.config.d,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.0.square().square()),
        )?;
        let rhs = region.assign_advice(
            || "rhs",
            self.config.b,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.1),
        )?;
        region.assign_advice(
            || "rhs^4",
            self.config.e,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.1.square().square()),
        )?;
        let out = region.assign_advice(
            || "out",
            self.config.c,
            0,
            || Ok(value.ok_or(Error::SynthesisError)?.2),
        )?;

        region.assign_fixed(|| "a", self.config.sa, 0, || Ok(FF::one()))?;
        region.assign_fixed(|| "b", self.config.sb, 0, || Ok(FF::one()))?;
        region.assign_fixed(|| "c", self.config.sc, 0, || Ok(FF::one()))?;
        region.assign_fixed(|| "a * b", self.config.sm, 0, || Ok(FF::zero()))?;
        Ok((lhs, rhs, out))
    }
    fn copy(&self, region: &mut Region<FF>, left: Cell, right: Cell) -> Result<(), Error> {
        region.constrain_equal(left, right)
    }
    fn lookup_table(&self, layouter: &mut impl Layouter<FF>, values: &[FF]) -> Result<(), Error> {
        layouter.assign_table(
            || "",
            |mut table| {
                for (index, &value) in values.iter().enumerate() {
                    table.assign_cell(|| "table col", self.config.sl, index, || Ok(value))?;
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}

impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
    type Config = PlonkConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            a: None,
            lookup_table: self.lookup_table.clone(),
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn configure(meta: &mut ConstraintSystem<F>) -> PlonkConfig {
        let e = meta.advice_column();
        let a = meta.advice_column();
        let b = meta.advice_column();
        let sf = meta.fixed_column();
        let c = meta.advice_column();
        let d = meta.advice_column();

        meta.enable_equality(a.into());
        meta.enable_equality(b.into());
        meta.enable_equality(c.into());

        let sm = meta.fixed_column();
        let sa = meta.fixed_column();
        let sb = meta.fixed_column();
        let sc = meta.fixed_column();
        let sl = meta.lookup_table_column();

        /*
         *   A         B      ...  sl
         * [
         *   instance  0      ...  0
         *   a         a      ...  0
         *   a         a^2    ...  0
         *   a         a      ...  0
         *   a         a^2    ...  0
         *   ...       ...    ...  ...
         *   ...       ...    ...  instance
         *   ...       ...    ...  a
         *   ...       ...    ...  a
         *   ...       ...    ...  0
         * ]
         */
        meta.lookup(|meta| {
            let a_ = meta.query_any(a.into(), Rotation::cur());
            vec![(a_, sl)]
        });

        meta.create_gate("Combined add-mult", |meta| {
            let d = meta.query_advice(d, Rotation::next());
            let a = meta.query_advice(a, Rotation::cur());
            let sf = meta.query_fixed(sf, Rotation::cur());
            let e = meta.query_advice(e, Rotation::prev());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());

            let sa = meta.query_fixed(sa, Rotation::cur());
            let sb = meta.query_fixed(sb, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());
            let sm = meta.query_fixed(sm, Rotation::cur());

            vec![a.clone() * sa + b.clone() * sb + a * b * sm - (c * sc) + sf * (d * e)]
        });

        PlonkConfig {
            a,
            b,
            c,
            d,
            e,
            sa,
            sb,
            sc,
            sm,
            sl,
        }
    }

    fn synthesize(&self, config: PlonkConfig, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let cs = StandardPlonk::new(config);

        for i in 0..10 {
            layouter.assign_region(
                || format!("region_{}", i),
                |mut region| {
                    let mut a_squared = None;
                    let (a0, _, c0) = cs.raw_multiply(&mut region, || {
                        a_squared = self.a.map(|a| a.square());
                        Ok((
                            self.a.ok_or(Error::SynthesisError)?,
                            self.a.ok_or(Error::SynthesisError)?,
                            a_squared.ok_or(Error::SynthesisError)?,
                        ))
                    })?;
                    let (a1, b1, _) = cs.raw_add(&mut region, || {
                        let fin = a_squared.and_then(|a2| self.a.map(|a| a + a2));
                        Ok((
                            self.a.ok_or(Error::SynthesisError)?,
                            a_squared.ok_or(Error::SynthesisError)?,
                            fin.ok_or(Error::SynthesisError)?,
                        ))
                    })?;
                    cs.copy(&mut region, a0, a1)?;
                    cs.copy(&mut region, b1, c0)
                },
            )?;
        }

        cs.lookup_table(&mut layouter, &self.lookup_table)?;

        Ok(())
    }
}

// ANCHOR: dev-graph
fn main() {
    // Prepare the circuit you want to render.
    // You don't need to include any witness variables.
    let a = Fp::rand();
    let instance = Fp::one() + Fp::one();
    let lookup_table = vec![instance, a, a, Fp::zero()];
    let circuit: MyCircuit<Fp> = MyCircuit {
        a: None,
        lookup_table,
    };

    // Create the area you want to draw on.
    // Use SVGBackend if you want to render to .svg instead.
    use plotters::prelude::*;
    let root = BitMapBackend::new("layout.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Example Circuit Layout", ("sans-serif", 60))
        .unwrap();

    halo2::dev::CircuitLayout::default()
        // You can optionally render only a section of the circuit.
        .view_width(0..2)
        .view_height(0..16)
        // You can hide labels, which can be useful with smaller areas.
        .show_labels(false)
        // Render the circuit onto your area!
        // The first argument is the size parameter for the circuit.
        .render(5, &circuit, &root)
        .unwrap();
}
// ANCHOR_END: dev-graph
