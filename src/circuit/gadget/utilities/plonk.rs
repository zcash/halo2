use super::{copy, CellValue, UtilitiesInstructions};
use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation},
    poly::Rotation,
};
use pasta_curves::arithmetic::FieldExt;
use std::marker::PhantomData;

#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::too_many_arguments)]
pub trait PLONKInstructions<F: FieldExt>: UtilitiesInstructions<F> {
    // Checks that a * sm * b = c * sc
    fn mul(
        &self,
        layouter: impl Layouter<F>,
        a: Self::Var,
        b: Self::Var,
        c: Self::Var,
        sc: Option<F>,
        sm: Option<F>,
    ) -> Result<(), Error>;
    // Checks that a * sa + b * sb = c * sc
    fn add(
        &self,
        layouter: impl Layouter<F>,
        a: Self::Var,
        b: Self::Var,
        c: Self::Var,
        sa: Option<F>,
        sb: Option<F>,
        sc: Option<F>,
    ) -> Result<(), Error>;
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub struct PLONKConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,

    sa: Column<Fixed>,
    sb: Column<Fixed>,
    sc: Column<Fixed>,
    sm: Column<Fixed>,

    perm: Permutation,
}

#[allow(clippy::upper_case_acronyms)]
pub struct PLONKChip<F: FieldExt> {
    config: PLONKConfig,
    _marker: PhantomData<F>,
}

#[allow(clippy::upper_case_acronyms)]
impl<F: FieldExt> Chip<F> for PLONKChip<F> {
    type Config = PLONKConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

#[allow(clippy::upper_case_acronyms)]
impl<F: FieldExt> UtilitiesInstructions<F> for PLONKChip<F> {
    type Var = CellValue<F>;
}

#[allow(clippy::upper_case_acronyms)]
impl<F: FieldExt> PLONKInstructions<F> for PLONKChip<F> {
    fn mul(
        &self,
        mut layouter: impl Layouter<F>,
        a: Self::Var,
        b: Self::Var,
        c: Self::Var,
        sc: Option<F>,
        sm: Option<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "mul",
            |mut region| {
                let config = self.config().clone();

                // Copy in `a`
                copy(&mut region, || "copy a", config.a, 0, &a, &config.perm)?;

                // Copy in `b`
                copy(&mut region, || "copy b", config.b, 0, &b, &config.perm)?;

                // Copy in `c`
                copy(&mut region, || "copy c", config.c, 0, &c, &config.perm)?;

                // Assign fixed columns
                region.assign_fixed(|| "sc", config.sc, 0, || sc.ok_or(Error::SynthesisError))?;
                region.assign_fixed(
                    || "a * (sm) * b",
                    config.sm,
                    0,
                    || sm.ok_or(Error::SynthesisError),
                )?;

                #[cfg(test)]
                // Checks that a * sm * b = c * sc
                {
                    if let (Some(a), Some(b), Some(c), Some(sm), Some(sc)) =
                        (a.value, b.value, c.value, sm, sc)
                    {
                        assert_eq!(a * sm * b, c * sc);
                    }
                }

                Ok(())
            },
        )
    }

    fn add(
        &self,
        mut layouter: impl Layouter<F>,
        a: Self::Var,
        b: Self::Var,
        c: Self::Var,
        sa: Option<F>,
        sb: Option<F>,
        sc: Option<F>,
    ) -> Result<(), Error> {
        let config = self.config().clone();

        layouter.assign_region(
            || "add",
            |mut region| {
                // Copy in `a`
                copy(&mut region, || "copy a", config.a, 0, &a, &config.perm)?;

                // Copy in `b`
                copy(&mut region, || "copy b", config.b, 0, &b, &config.perm)?;

                // Copy in `c`
                copy(&mut region, || "copy c", config.c, 0, &c, &config.perm)?;

                // Assign fixed columns
                region.assign_fixed(|| "a", config.sa, 0, || sa.ok_or(Error::SynthesisError))?;
                region.assign_fixed(|| "b", config.sb, 0, || sb.ok_or(Error::SynthesisError))?;
                region.assign_fixed(|| "c", config.sc, 0, || sc.ok_or(Error::SynthesisError))?;

                #[cfg(test)]
                // Checks that a * sa + b * sb = c * sc
                {
                    if let (Some(a), Some(b), Some(c), Some(sa), Some(sb), Some(sc)) =
                        (a.value, b.value, c.value, sa, sb, sc)
                    {
                        assert_eq!(a * sa + b * sb, c * sc);
                    }
                }

                Ok(())
            },
        )
    }
}

#[allow(clippy::upper_case_acronyms)]
impl<F: FieldExt> PLONKChip<F> {
    /// Configures this chip for use in a circuit.
    ///
    /// `perm` must cover `advices`, as well as any columns that will be passed
    /// to this chip.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advices: [Column<Advice>; 3],
        perm: Permutation,
    ) -> PLONKConfig {
        let a = advices[0];
        let b = advices[1];
        let c = advices[2];

        let sa = meta.fixed_column();
        let sb = meta.fixed_column();
        let sc = meta.fixed_column();
        let sm = meta.fixed_column();

        meta.create_gate("Combined add-mult", |meta| {
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());

            let sa = meta.query_fixed(sa, Rotation::cur());
            let sb = meta.query_fixed(sb, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());
            let sm = meta.query_fixed(sm, Rotation::cur());

            vec![a.clone() * sa + b.clone() * sb + a * b * sm + (c * sc * (-F::one()))]
        });

        PLONKConfig {
            a,
            b,
            c,
            sa,
            sb,
            sc,
            sm,
            perm,
        }
    }

    pub fn construct(config: PLONKConfig) -> Self {
        PLONKChip {
            config,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::UtilitiesInstructions;
    use super::{PLONKChip, PLONKConfig, PLONKInstructions};
    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Any, Circuit, Column, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas::Base};

    #[test]
    fn plonk_util() {
        #[derive(Default)]
        struct MyCircuit<F: FieldExt> {
            a: Option<F>,
            b: Option<F>,
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
            type Config = PLONKConfig;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self::default()
            }

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let advices = [
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                ];

                let perm = meta.permutation(
                    &advices
                        .iter()
                        .map(|advice| (*advice).into())
                        .collect::<Vec<Column<Any>>>(),
                );

                PLONKChip::<F>::configure(meta, advices, perm)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<F>,
            ) -> Result<(), Error> {
                let chip = PLONKChip::<F>::construct(config.clone());

                let a = chip.load_private(layouter.namespace(|| "a"), config.a, self.a)?;
                let b = chip.load_private(layouter.namespace(|| "b"), config.b, self.b)?;

                // a + b = c
                {
                    let c = self.a.zip(self.b).map(|(a, b)| a + b);
                    let c = chip.load_private(layouter.namespace(|| "c"), config.c, c)?;
                    chip.add(
                        layouter.namespace(|| "a + b = c"),
                        a,
                        b,
                        c,
                        Some(F::one()),
                        Some(F::one()),
                        Some(F::one()),
                    )?;
                }

                // a * b = c
                {
                    let c = self.a.zip(self.b).map(|(a, b)| a * b);
                    let c = chip.load_private(layouter.namespace(|| "c"), config.c, c)?;
                    chip.mul(
                        layouter.namespace(|| "a * b = c"),
                        a,
                        b,
                        c,
                        Some(F::one()),
                        Some(F::one()),
                    )?;
                }

                // 2a + 3b = c
                {
                    let c = self
                        .a
                        .zip(self.b)
                        .map(|(a, b)| a * F::from_u64(2) + b * F::from_u64(3));
                    let c = chip.load_private(layouter.namespace(|| "c"), config.c, c)?;
                    chip.add(
                        layouter.namespace(|| "2a + 3b = c"),
                        a,
                        b,
                        c,
                        Some(F::from_u64(2)),
                        Some(F::from_u64(3)),
                        Some(F::one()),
                    )?;
                }

                // 4 * a * b = 2 * c => c = 2ab
                {
                    let c = self.a.zip(self.b).map(|(a, b)| a * b * F::from_u64(2));
                    let c = chip.load_private(layouter.namespace(|| "c"), config.c, c)?;
                    chip.mul(
                        layouter.namespace(|| "4 * a * b = 2 * c"),
                        a,
                        b,
                        c,
                        Some(F::from_u64(2)),
                        Some(F::from_u64(4)),
                    )?;
                }

                Ok(())
            }
        }

        let circuit: MyCircuit<Base> = MyCircuit {
            a: Some(Base::rand()),
            b: Some(Base::rand()),
        };
        let prover = MockProver::<Base>::run(3, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}
