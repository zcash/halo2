use halo2::{
    arithmetic::FieldExt,
    dev::circuit_layout,
    pasta::Fp,
    plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed, Permutation},
    poly::Rotation,
};
use plotters::prelude::*;
use std::marker::PhantomData;

#[allow(clippy::many_single_char_names)]
fn main() {
    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    #[derive(Clone)]
    struct PLONKConfig {
        a: Column<Advice>,
        b: Column<Advice>,
        c: Column<Advice>,
        d: Column<Advice>,
        e: Column<Advice>,

        sa: Column<Fixed>,
        sb: Column<Fixed>,
        sc: Column<Fixed>,
        sm: Column<Fixed>,
        sp: Column<Fixed>,
        sl: Column<Fixed>,
        sl2: Column<Fixed>,

        perm: Permutation,
        perm2: Permutation,
    }

    trait StandardCS<FF: FieldExt> {
        fn raw_multiply<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>;
        fn raw_add<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>;
        fn copy(&mut self, a: Variable, b: Variable) -> Result<(), Error>;
        fn public_input<F>(&mut self, f: F) -> Result<Variable, Error>
        where
            F: FnOnce() -> Result<FF, Error>;
        fn lookup_table(&mut self, values: &[Vec<FF>]) -> Result<(), Error>;
    }

    struct MyCircuit<F: FieldExt> {
        a: Option<F>,
        lookup_tables: Vec<Vec<F>>,
    }

    struct StandardPLONK<'a, F: FieldExt, CS: Assignment<F> + 'a> {
        cs: &'a mut CS,
        config: PLONKConfig,
        current_gate: usize,
        _marker: PhantomData<F>,
    }

    impl<'a, FF: FieldExt, CS: Assignment<FF>> StandardPLONK<'a, FF, CS> {
        fn new(cs: &'a mut CS, config: PLONKConfig) -> Self {
            StandardPLONK {
                cs,
                config,
                current_gate: 0,
                _marker: PhantomData,
            }
        }

        fn enter_region<NR, N>(&mut self, name_fn: N)
        where
            NR: Into<String>,
            N: FnOnce() -> NR,
        {
            self.cs.enter_region(name_fn);
        }

        fn exit_region(&mut self) {
            self.cs.exit_region();
        }
    }

    impl<'a, FF: FieldExt, CS: Assignment<FF>> StandardCS<FF> for StandardPLONK<'a, FF, CS> {
        fn raw_multiply<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>,
        {
            let index = self.current_gate;
            self.current_gate += 1;
            let mut value = None;
            self.cs.assign_advice(
                || "lhs",
                self.config.a,
                index,
                || {
                    value = Some(f()?);
                    Ok(value.ok_or(Error::SynthesisError)?.0)
                },
            )?;
            self.cs.assign_advice(
                || "lhs^4",
                self.config.d,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.0.square().square()),
            )?;
            self.cs.assign_advice(
                || "rhs",
                self.config.b,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1),
            )?;
            self.cs.assign_advice(
                || "rhs^4",
                self.config.e,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1.square().square()),
            )?;
            self.cs.assign_advice(
                || "out",
                self.config.c,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.2),
            )?;

            self.cs
                .assign_fixed(|| "a", self.config.sa, index, || Ok(FF::zero()))?;
            self.cs
                .assign_fixed(|| "b", self.config.sb, index, || Ok(FF::zero()))?;
            self.cs
                .assign_fixed(|| "c", self.config.sc, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "a * b", self.config.sm, index, || Ok(FF::one()))?;
            Ok((
                Variable(self.config.a, index),
                Variable(self.config.b, index),
                Variable(self.config.c, index),
            ))
        }
        fn raw_add<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>,
        {
            let index = self.current_gate;
            self.current_gate += 1;
            let mut value = None;
            self.cs.assign_advice(
                || "lhs",
                self.config.a,
                index,
                || {
                    value = Some(f()?);
                    Ok(value.ok_or(Error::SynthesisError)?.0)
                },
            )?;
            self.cs.assign_advice(
                || "lhs^4",
                self.config.d,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.0.square().square()),
            )?;
            self.cs.assign_advice(
                || "rhs",
                self.config.b,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1),
            )?;
            self.cs.assign_advice(
                || "rhs^4",
                self.config.e,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1.square().square()),
            )?;
            self.cs.assign_advice(
                || "out",
                self.config.c,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.2),
            )?;

            self.cs
                .assign_fixed(|| "a", self.config.sa, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "b", self.config.sb, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "c", self.config.sc, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "a * b", self.config.sm, index, || Ok(FF::zero()))?;
            Ok((
                Variable(self.config.a, index),
                Variable(self.config.b, index),
                Variable(self.config.c, index),
            ))
        }
        fn copy(&mut self, left: Variable, right: Variable) -> Result<(), Error> {
            self.cs.copy(
                &self.config.perm,
                left.0.into(),
                left.1,
                right.0.into(),
                right.1,
            )?;
            self.cs.copy(
                &self.config.perm2,
                left.0.into(),
                left.1,
                right.0.into(),
                right.1,
            )
        }
        fn public_input<F>(&mut self, f: F) -> Result<Variable, Error>
        where
            F: FnOnce() -> Result<FF, Error>,
        {
            let index = self.current_gate;
            self.current_gate += 1;
            self.cs
                .assign_advice(|| "value", self.config.a, index, || f())?;
            self.cs
                .assign_fixed(|| "public", self.config.sp, index, || Ok(FF::one()))?;

            Ok(Variable(self.config.a, index))
        }
        fn lookup_table(&mut self, values: &[Vec<FF>]) -> Result<(), Error> {
            for (&value_0, &value_1) in values[0].iter().zip(values[1].iter()) {
                let index = self.current_gate;

                self.current_gate += 1;
                self.cs
                    .assign_fixed(|| "table col 1", self.config.sl, index, || Ok(value_0))?;
                self.cs
                    .assign_fixed(|| "table col 2", self.config.sl2, index, || Ok(value_1))?;
            }
            Ok(())
        }
    }

    impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
        type Config = PLONKConfig;

        fn configure(meta: &mut ConstraintSystem<F>) -> PLONKConfig {
            let e = meta.advice_column();
            let a = meta.advice_column();
            let b = meta.advice_column();
            let sf = meta.fixed_column();
            let c = meta.advice_column();
            let d = meta.advice_column();
            let p = meta.instance_column();

            let perm = meta.permutation(&[a.into(), b.into(), c.into()]);
            let perm2 = meta.permutation(&[a.into(), b.into(), c.into()]);

            let sm = meta.fixed_column();
            let sa = meta.fixed_column();
            let sb = meta.fixed_column();
            let sc = meta.fixed_column();
            let sp = meta.fixed_column();
            let sl = meta.fixed_column();
            let sl2 = meta.fixed_column();

            /*
             *   A         B      ...  sl        sl2
             * [
             *   instance  0      ...  0         0
             *   a         a      ...  0         0
             *   a         a^2    ...  0         0
             *   a         a      ...  0         0
             *   a         a^2    ...  0         0
             *   ...       ...    ...  ...       ...
             *   ...       ...    ...  instance  0
             *   ...       ...    ...  a         a
             *   ...       ...    ...  a         a^2
             *   ...       ...    ...  0         0
             * ]
             */
            let a_ = meta.query_any(a.into(), Rotation::cur());
            let b_ = meta.query_any(b.into(), Rotation::cur());
            let sl_ = meta.query_any(sl.into(), Rotation::cur());
            let sl2_ = meta.query_any(sl2.into(), Rotation::cur());
            meta.lookup(&[a_.clone()], &[sl_.clone()]);
            meta.lookup(&[a_, b_], &[sl_, sl2_]);

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

                a.clone() * sa + b.clone() * sb + a * b * sm + (c * sc * (-F::one())) + sf * (d * e)
            });

            meta.create_gate("Public input", |meta| {
                let a = meta.query_advice(a, Rotation::cur());
                let p = meta.query_instance(p, Rotation::cur());
                let sp = meta.query_fixed(sp, Rotation::cur());

                sp * (a + p * (-F::one()))
            });

            PLONKConfig {
                a,
                b,
                c,
                d,
                e,
                sa,
                sb,
                sc,
                sm,
                sp,
                sl,
                sl2,
                perm,
                perm2,
            }
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<F>,
            config: PLONKConfig,
        ) -> Result<(), Error> {
            let mut cs = StandardPLONK::new(cs, config);

            cs.enter_region(|| "input");
            let _ = cs.public_input(|| Ok(F::one() + F::one()))?;
            cs.exit_region();

            for i in 0..10 {
                cs.enter_region(|| format!("region_{}", i));
                let mut a_squared = None;
                let (a0, _, c0) = cs.raw_multiply(|| {
                    a_squared = self.a.map(|a| a.square());
                    Ok((
                        self.a.ok_or(Error::SynthesisError)?,
                        self.a.ok_or(Error::SynthesisError)?,
                        a_squared.ok_or(Error::SynthesisError)?,
                    ))
                })?;
                let (a1, b1, _) = cs.raw_add(|| {
                    let fin = a_squared.and_then(|a2| self.a.map(|a| a + a2));
                    Ok((
                        self.a.ok_or(Error::SynthesisError)?,
                        a_squared.ok_or(Error::SynthesisError)?,
                        fin.ok_or(Error::SynthesisError)?,
                    ))
                })?;
                cs.copy(a0, a1)?;
                cs.copy(b1, c0)?;
                cs.exit_region();
            }

            cs.enter_region(|| "lookup table");
            cs.lookup_table(&self.lookup_tables)?;
            cs.exit_region();

            Ok(())
        }
    }

    let a = Fp::rand();
    let a_squared = a * a;
    let instance = Fp::one() + Fp::one();
    let lookup_table = vec![instance, a, a, Fp::zero()];
    let lookup_table_2 = vec![Fp::zero(), a, a_squared, Fp::zero()];

    let circuit: MyCircuit<Fp> = MyCircuit {
        a: None,
        lookup_tables: vec![lookup_table, lookup_table_2],
    };

    let root = BitMapBackend::new("example-circuit-layout.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Example Circuit Layout", ("sans-serif", 60))
        .unwrap();

    circuit_layout(&circuit, &root).unwrap();
}
