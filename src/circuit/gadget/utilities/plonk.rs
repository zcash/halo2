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
    type Var;

    // Checks that a * sm * b = c * sc
    fn mul(
        &self,
        layouter: impl Layouter<F>,
        a: <Self as PLONKInstructions<F>>::Var,
        b: <Self as PLONKInstructions<F>>::Var,
        c: <Self as PLONKInstructions<F>>::Var,
        sc: Option<F>,
        sm: Option<F>,
    ) -> Result<(), Error>;
    // Checks that a * sa + b * sb = c * sc
    fn add(
        &self,
        layouter: impl Layouter<F>,
        a: <Self as PLONKInstructions<F>>::Var,
        b: <Self as PLONKInstructions<F>>::Var,
        c: <Self as PLONKInstructions<F>>::Var,
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
    type Var = CellValue<F>;

    fn mul(
        &self,
        mut layouter: impl Layouter<F>,
        a: <Self as PLONKInstructions<F>>::Var,
        b: <Self as PLONKInstructions<F>>::Var,
        c: <Self as PLONKInstructions<F>>::Var,
        sc: Option<F>,
        sm: Option<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "mul",
            |mut region| {
                let config = self.config().clone();
                let sc = sc.unwrap_or_else(F::one);
                let sm = sm.unwrap_or_else(F::one);

                // Copy in `a`
                copy(&mut region, || "copy a", config.a, 0, &a, &config.perm)?;

                // Copy in `b`
                copy(&mut region, || "copy b", config.b, 0, &b, &config.perm)?;

                // Copy in `c`
                copy(&mut region, || "copy c", config.c, 0, &c, &config.perm)?;

                // Assign fixed columns
                region.assign_fixed(|| "sc", config.sc, 0, || Ok(sc))?;
                region.assign_fixed(|| "a * (sm) * b", config.sm, 0, || Ok(sm))?;

                #[cfg(test)]
                // Checks that a * sm * b = c * sc
                {
                    let a = a.value.unwrap();
                    let b = b.value.unwrap();
                    let c = c.value.unwrap();
                    assert_eq!(a * sm * b, c * sc);
                }

                Ok(())
            },
        )
    }

    fn add(
        &self,
        mut layouter: impl Layouter<F>,
        a: <Self as PLONKInstructions<F>>::Var,
        b: <Self as PLONKInstructions<F>>::Var,
        c: <Self as PLONKInstructions<F>>::Var,
        sa: Option<F>,
        sb: Option<F>,
        sc: Option<F>,
    ) -> Result<(), Error> {
        let config = self.config().clone();
        let sa = sa.unwrap_or_else(F::one);
        let sb = sb.unwrap_or_else(F::one);
        let sc = sc.unwrap_or_else(F::one);

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
                region.assign_fixed(|| "a", config.sa, 0, || Ok(sa))?;
                region.assign_fixed(|| "b", config.sb, 0, || Ok(sb))?;
                region.assign_fixed(|| "c", config.sc, 0, || Ok(sc))?;

                #[cfg(test)]
                // Checks that a * sa + b * sb = c * sc
                {
                    let a = a.value.unwrap();
                    let b = b.value.unwrap();
                    let c = c.value.unwrap();
                    assert_eq!(a * sa + b * sb, c * sc);
                }

                Ok(())
            },
        )
    }
}

#[allow(clippy::upper_case_acronyms)]
impl<F: FieldExt> PLONKChip<F> {
    /// Configures this chip for use in a circuit.
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
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::MockProver,
        plonk::{Any, Assignment, Circuit, Column, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas::Base};

    #[test]
    fn plonk_util() {
        struct MyCircuit<F: FieldExt> {
            a: Option<F>,
            b: Option<F>,
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
            type Config = PLONKConfig;

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
                cs: &mut impl Assignment<F>,
                config: Self::Config,
            ) -> Result<(), Error> {
                let mut layouter = SingleChipLayouter::new(cs)?;
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
                        None,
                        None,
                        None,
                    )?;
                }

                // a * b = c
                {
                    let c = self.a.zip(self.b).map(|(a, b)| a * b);
                    let c = chip.load_private(layouter.namespace(|| "c"), config.c, c)?;
                    chip.mul(layouter.namespace(|| "a * b = c"), a, b, c, None, None)?;
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
                        None,
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
        let prover = match MockProver::<Base>::run(3, &circuit, vec![]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));
    }
}
