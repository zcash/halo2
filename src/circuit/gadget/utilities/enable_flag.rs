use super::{copy, CellValue, UtilitiesInstructions};
use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use pasta_curves::arithmetic::FieldExt;
use std::marker::PhantomData;

pub trait EnableFlagInstructions<F: FieldExt>: UtilitiesInstructions<F> {
    /// Given a `value` and an `enable_flag`, check that either `value = 0`
    /// or `enable_flag = 1`.
    fn enable_flag(
        &self,
        layouter: impl Layouter<F>,
        value: Self::Var,
        enable_flag: Option<bool>,
    ) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct EnableFlagConfig {
    q_enable: Selector,
    value: Column<Advice>,
    enable_flag: Column<Advice>,
    perm: Permutation,
}

/// A chip implementing an enable flag.
#[derive(Clone, Debug)]
pub struct EnableFlagChip<F> {
    config: EnableFlagConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for EnableFlagChip<F> {
    type Config = EnableFlagConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> UtilitiesInstructions<F> for EnableFlagChip<F> {
    type Var = CellValue<F>;
}

impl<F: FieldExt> EnableFlagInstructions<F> for EnableFlagChip<F> {
    fn enable_flag(
        &self,
        mut layouter: impl Layouter<F>,
        value: Self::Var,
        enable_flag: Option<bool>,
    ) -> Result<(), Error> {
        let config = self.config().clone();
        layouter.assign_region(
            || "enable flag",
            |mut region| {
                // Enable `q_enable` selector
                config.q_enable.enable(&mut region, 0)?;

                // Witness `enable_flag` value
                let enable_flag_val = enable_flag.map(|flag| F::from_u64(flag as u64));
                region.assign_advice(
                    || "enable_flag",
                    config.enable_flag,
                    0,
                    || enable_flag_val.ok_or(Error::SynthesisError),
                )?;

                // Copy `value`
                copy(
                    &mut region,
                    || "copy value",
                    config.value,
                    0,
                    &value,
                    &config.perm,
                )?;

                Ok(())
            },
        )
    }
}

impl<F: FieldExt> EnableFlagChip<F> {
    /// Configures this chip for use in a circuit.
    ///
    /// `perm` must cover `advices[0]`, as well as any columns that will be
    /// passed to this chip.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advices: [Column<Advice>; 2],
        perm: Permutation,
    ) -> EnableFlagConfig {
        let q_enable = meta.selector();

        let config = EnableFlagConfig {
            q_enable,
            value: advices[0],
            enable_flag: advices[1],
            perm,
        };

        meta.create_gate("Enable flag", |meta| {
            let q_enable = meta.query_selector(config.q_enable, Rotation::cur());
            let value = meta.query_advice(config.value, Rotation::cur());
            let enable_flag = meta.query_advice(config.enable_flag, Rotation::cur());

            vec![q_enable * (Expression::Constant(F::one()) - enable_flag) * value]
        });

        config
    }

    pub fn construct(config: EnableFlagConfig) -> Self {
        EnableFlagChip {
            config,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::UtilitiesInstructions;
    use super::{EnableFlagChip, EnableFlagConfig, EnableFlagInstructions};
    use halo2::{
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::{MockProver, VerifyFailure},
        plonk::{Any, Assignment, Circuit, Column, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas::Base};

    #[test]
    fn enable_flag() {
        struct MyCircuit<F: FieldExt> {
            value: Option<F>,
            enable_flag: Option<bool>,
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
            type Config = EnableFlagConfig;

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let advices = [meta.advice_column(), meta.advice_column()];

                let perm = meta.permutation(
                    &advices
                        .iter()
                        .map(|advice| (*advice).into())
                        .collect::<Vec<Column<Any>>>(),
                );

                EnableFlagChip::<F>::configure(meta, advices, perm)
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                config: Self::Config,
            ) -> Result<(), Error> {
                let mut layouter = SingleChipLayouter::new(cs)?;
                let chip = EnableFlagChip::<F>::construct(config.clone());

                // Load the value and the enable flag into the circuit.
                let value =
                    chip.load_private(layouter.namespace(|| "value"), config.value, self.value)?;

                // Run the enable flag logic.
                chip.enable_flag(layouter.namespace(|| "swap"), value, self.enable_flag)?;

                Ok(())
            }
        }

        // Test value = 1, flag = 1 case (success)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::one()),
                enable_flag: Some(true),
            };
            let prover = MockProver::<Base>::run(1, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test value = 0, flag = 0 case (success)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::zero()),
                enable_flag: Some(false),
            };
            let prover = MockProver::<Base>::run(1, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test value = 0, flag = 1 case (success)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::zero()),
                enable_flag: Some(true),
            };
            let prover = MockProver::<Base>::run(1, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test value = 1, flag = 0 case (error)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::one()),
                enable_flag: Some(false),
            };
            let prover = MockProver::<Base>::run(1, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Err(vec![VerifyFailure::Gate {
                    gate_index: 0,
                    gate_name: "Enable flag",
                    row: 1,
                }])
            );
        }
    }
}
