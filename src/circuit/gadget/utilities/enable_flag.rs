use super::{copy, CellValue, UtilitiesInstructions, Var};
use halo2::{
    circuit::{Cell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use pasta_curves::arithmetic::FieldExt;
use std::marker::PhantomData;

pub trait EnableFlagInstructions<F: FieldExt>: UtilitiesInstructions<F> {
    /// Variable representing cell with a certain value in the circuit.
    type Var: Var<F>;

    /// Variable representing an `enable` boolean flag.
    type Flag: From<<Self as EnableFlagInstructions<F>>::Var>;

    /// Given a `value` and an `enable_flag`, check that either `value = 0`
    /// or `enable_flag = 1`.
    fn enable_flag(
        &self,
        layouter: impl Layouter<F>,
        value: <Self as EnableFlagInstructions<F>>::Var,
        enable_flag: <Self as EnableFlagInstructions<F>>::Flag,
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

/// A variable representing an `enable` boolean flag.
#[derive(Copy, Clone)]
pub struct Flag {
    cell: Cell,
    value: Option<bool>,
}

impl<F: FieldExt> From<CellValue<F>> for Flag {
    fn from(var: CellValue<F>) -> Self {
        let value = var.value.map(|value| {
            let zero = value == F::zero();
            let one = value == F::one();
            if zero {
                false
            } else if one {
                true
            } else {
                panic!("Value must be boolean.")
            }
        });
        Flag {
            cell: var.cell,
            value,
        }
    }
}

impl<F: FieldExt> UtilitiesInstructions<F> for EnableFlagChip<F> {
    type Var = CellValue<F>;
}

impl<F: FieldExt> EnableFlagInstructions<F> for EnableFlagChip<F> {
    type Var = CellValue<F>;
    type Flag = Flag;

    fn enable_flag(
        &self,
        mut layouter: impl Layouter<F>,
        value: <Self as EnableFlagInstructions<F>>::Var,
        enable_flag: <Self as EnableFlagInstructions<F>>::Flag,
    ) -> Result<(), Error> {
        let config = self.config().clone();
        layouter.assign_region(
            || "enable flag",
            |mut region| {
                // Enable `q_enable` selector
                config.q_enable.enable(&mut region, 0)?;

                // Copy in `enable_flag` value
                let enable_flag_val = enable_flag.value;
                let enable_flag_cell = region.assign_advice(
                    || "enable_flag",
                    config.enable_flag,
                    0,
                    || {
                        enable_flag_val
                            .map(|enable_flag| F::from_u64(enable_flag as u64))
                            .ok_or(Error::SynthesisError)
                    },
                )?;
                region.constrain_equal(&config.perm, enable_flag_cell, enable_flag.cell)?;

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
            enable_flag: Option<F>,
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
                let enable_flag = chip.load_private(
                    layouter.namespace(|| "enable_flag"),
                    config.enable_flag,
                    self.enable_flag,
                )?;

                // Run the enable flag logic.
                chip.enable_flag(layouter.namespace(|| "swap"), value, enable_flag.into())?;

                Ok(())
            }
        }

        // Test value = 1, flag = 1 case (success)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::one()),
                enable_flag: Some(Base::one()),
            };
            let prover = match MockProver::<Base>::run(1, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test value = 0, flag = 0 case (success)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::zero()),
                enable_flag: Some(Base::zero()),
            };
            let prover = match MockProver::<Base>::run(1, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test value = 0, flag = 1 case (success)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::zero()),
                enable_flag: Some(Base::one()),
            };
            let prover = match MockProver::<Base>::run(1, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test value = 1, flag = 0 case (error)
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                value: Some(Base::one()),
                enable_flag: Some(Base::zero()),
            };
            let prover = match MockProver::<Base>::run(1, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
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
