use super::{copy, CellValue, UtilitiesInstructions};
use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use pasta_curves::arithmetic::FieldExt;
use std::marker::PhantomData;

pub trait CondSwapInstructions<F: FieldExt>: UtilitiesInstructions<F> {
    #[allow(clippy::type_complexity)]
    /// Given an input pair (x,y) and a `swap` boolean flag, return
    /// (y,x) if `swap` is set, else (x,y) if `swap` is not set.
    fn swap(
        &self,
        layouter: impl Layouter<F>,
        pair: (Self::Var, Self::Var),
        swap: Option<bool>,
    ) -> Result<(Self::Var, Self::Var), Error>;
}

/// A chip implementing a conditional swap.
#[derive(Clone, Debug)]
pub struct CondSwapChip<F> {
    config: CondSwapConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Chip<F> for CondSwapChip<F> {
    type Config = CondSwapConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

#[derive(Clone, Debug)]
pub struct CondSwapConfig {
    pub q_swap: Selector,
    pub x: Column<Advice>,
    pub y: Column<Advice>,
    pub x_swapped: Column<Advice>,
    pub y_swapped: Column<Advice>,
    pub swap: Column<Advice>,
    pub perm: Permutation,
}

impl<F: FieldExt> UtilitiesInstructions<F> for CondSwapChip<F> {
    type Var = CellValue<F>;
}

impl<F: FieldExt> CondSwapInstructions<F> for CondSwapChip<F> {
    #[allow(clippy::type_complexity)]
    fn swap(
        &self,
        mut layouter: impl Layouter<F>,
        pair: (Self::Var, Self::Var),
        swap: Option<bool>,
    ) -> Result<(Self::Var, Self::Var), Error> {
        let config = self.config();

        layouter.assign_region(
            || "swap",
            |mut region| {
                // Enable `q_swap` selector
                config.q_swap.enable(&mut region, 0)?;

                // Copy in `x` value
                let x = copy(&mut region, || "copy x", config.x, 0, &pair.0, &config.perm)?;

                // Copy in `y` value
                let y = copy(&mut region, || "copy y", config.y, 0, &pair.1, &config.perm)?;

                // Witness `swap` value
                let swap_val = swap.map(|swap| F::from_u64(swap as u64));
                region.assign_advice(
                    || "swap",
                    config.swap,
                    0,
                    || swap_val.ok_or(Error::SynthesisError),
                )?;

                // Conditionally swap x
                let x_swapped = {
                    let x_swapped = x
                        .value
                        .zip(y.value)
                        .zip(swap)
                        .map(|((x, y), swap)| if swap { y } else { x });
                    let x_swapped_cell = region.assign_advice(
                        || "x_swapped",
                        config.x_swapped,
                        0,
                        || x_swapped.ok_or(Error::SynthesisError),
                    )?;
                    CellValue {
                        cell: x_swapped_cell,
                        value: x_swapped,
                    }
                };

                // Conditionally swap y
                let y_swapped = {
                    let y_swapped = x
                        .value
                        .zip(y.value)
                        .zip(swap)
                        .map(|((x, y), swap)| if swap { x } else { y });
                    let y_swapped_cell = region.assign_advice(
                        || "y_swapped",
                        config.y_swapped,
                        0,
                        || y_swapped.ok_or(Error::SynthesisError),
                    )?;
                    CellValue {
                        cell: y_swapped_cell,
                        value: y_swapped,
                    }
                };

                // Return swapped pair
                Ok((x_swapped, y_swapped))
            },
        )
    }
}

impl<F: FieldExt> CondSwapChip<F> {
    /// Configures this chip for use in a circuit.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advices: [Column<Advice>; 5],
        perm: Permutation,
    ) -> CondSwapConfig {
        let q_swap = meta.selector();

        let config = CondSwapConfig {
            q_swap,
            x: advices[0],
            y: advices[1],
            x_swapped: advices[2],
            y_swapped: advices[3],
            swap: advices[4],
            perm,
        };

        // TODO: optimise shape of gate for Merkle path validation

        meta.create_gate("x' = y ⋅ swap + x ⋅ (1-swap)", |meta| {
            let q_swap = meta.query_selector(q_swap, Rotation::cur());

            let x = meta.query_advice(config.x, Rotation::cur());
            let y = meta.query_advice(config.y, Rotation::cur());
            let x_swapped = meta.query_advice(config.x_swapped, Rotation::cur());
            let y_swapped = meta.query_advice(config.y_swapped, Rotation::cur());
            let swap = meta.query_advice(config.swap, Rotation::cur());

            let one = Expression::Constant(F::one());

            // x_swapped - y ⋅ swap - x ⋅ (1-swap) = 0
            // This checks that `x_swapped` is equal to `y` when `swap` is set,
            // but remains as `x` when `swap` is not set.
            let x_check =
                x_swapped - y.clone() * swap.clone() - x.clone() * (one.clone() - swap.clone());

            // y_swapped - x ⋅ swap - y ⋅ (1-swap) = 0
            // This checks that `y_swapped` is equal to `x` when `swap` is set,
            // but remains as `y` when `swap` is not set.
            let y_check = y_swapped - x * swap.clone() - y * (one.clone() - swap.clone());

            // Check `swap` is boolean.
            let bool_check = swap.clone() * (one - swap);

            [x_check, y_check, bool_check]
                .iter()
                .map(|poly| q_swap.clone() * poly.clone())
                .collect()
        });

        config
    }

    pub fn construct(config: CondSwapConfig) -> Self {
        CondSwapChip {
            config,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::UtilitiesInstructions;
    use super::{CondSwapChip, CondSwapConfig, CondSwapInstructions};
    use halo2::{
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::MockProver,
        plonk::{Any, Assignment, Circuit, Column, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas::Base};

    #[test]
    fn cond_swap() {
        struct MyCircuit<F: FieldExt> {
            x: Option<F>,
            y: Option<F>,
            swap: Option<bool>,
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
            type Config = CondSwapConfig;

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let advices = [
                    meta.advice_column(),
                    meta.advice_column(),
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

                CondSwapChip::<F>::configure(meta, advices, perm)
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                config: Self::Config,
            ) -> Result<(), Error> {
                let mut layouter = SingleChipLayouter::new(cs)?;
                let chip = CondSwapChip::<F>::construct(config.clone());

                // Load the pair and the swap flag into the circuit.
                let x = chip.load_private(layouter.namespace(|| "x"), config.x, self.x)?;
                let y = chip.load_private(layouter.namespace(|| "y"), config.y, self.y)?;
                // Return the swapped pair.
                let swapped_pair =
                    chip.swap(layouter.namespace(|| "swap"), (x, y).into(), self.swap)?;

                if let Some(swap) = self.swap {
                    if swap {
                        // Check that `x` and `y` have been swapped
                        assert_eq!(swapped_pair.0.value.unwrap(), y.value.unwrap());
                        assert_eq!(swapped_pair.1.value.unwrap(), x.value.unwrap());
                    } else {
                        // Check that `x` and `y` have not been swapped
                        assert_eq!(swapped_pair.0.value.unwrap(), x.value.unwrap());
                        assert_eq!(swapped_pair.1.value.unwrap(), y.value.unwrap());
                    }
                }

                Ok(())
            }
        }

        // Test swap case
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                x: Some(Base::rand()),
                y: Some(Base::rand()),
                swap: Some(true),
            };
            let prover = match MockProver::<Base>::run(3, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test non-swap case
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                x: Some(Base::rand()),
                y: Some(Base::rand()),
                swap: Some(false),
            };
            let prover = match MockProver::<Base>::run(3, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
