//! Gadget and chip for a conditional swap utility.

use group::ff::{Field, PrimeField};
use pasta_curves::pallas;

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter},
    plonk::{self, Error},
};

use crate::ecc::chip::{EccPoint, NonIdentityEccPoint};

use crate::utilities::cond_swap::{CondSwapChip, CondSwapInstructions};

/// Instructions for a conditional swap gadget.
pub trait CondSwapInstructionsOptimized<F: Field>: CondSwapInstructions<F> {
    /// Given an input `(choice, left, right)` where `choice` is a boolean flag,
    /// returns `left` if `choice` is not set and `right` if `choice` is set.
    fn mux(
        &self,
        layouter: &mut impl Layouter<F>,
        choice: Self::Var,
        left: Self::Var,
        right: Self::Var,
    ) -> Result<Self::Var, Error>;
}

impl<F: PrimeField> CondSwapInstructionsOptimized<F> for CondSwapChip<F> {
    fn mux(
        &self,
        layouter: &mut impl Layouter<F>,
        choice: Self::Var,
        left: Self::Var,
        right: Self::Var,
    ) -> Result<Self::Var, Error> {
        let config = self.config();

        layouter.assign_region(
            || "mux",
            |mut region| {
                // Enable `q_swap` selector
                config.q_swap.enable(&mut region, 0)?;

                // Copy in `a` value
                let left = left.copy_advice(|| "copy left", &mut region, config.a, 0)?;

                // Copy in `b` value
                let right = right.copy_advice(|| "copy right", &mut region, config.b, 0)?;

                // Copy `choice` value
                let choice = choice.copy_advice(|| "copy choice", &mut region, config.swap, 0)?;

                let a_swapped = left
                    .value()
                    .zip(right.value())
                    .zip(choice.value())
                    .map(|((left, right), choice)| {
                        if *choice == F::from(0_u64) {
                            left
                        } else {
                            right
                        }
                    })
                    .cloned();
                let b_swapped = left
                    .value()
                    .zip(right.value())
                    .zip(choice.value())
                    .map(|((left, right), choice)| {
                        if *choice == F::from(0_u64) {
                            right
                        } else {
                            left
                        }
                    })
                    .cloned();

                region.assign_advice(|| "out b_swap", self.config.b_swapped, 0, || b_swapped)?;
                region.assign_advice(|| "out a_swap", self.config.a_swapped, 0, || a_swapped)
            },
        )
    }
}

impl CondSwapChip<pallas::Base> {
    /// Given an input `(choice, left, right)` where `choice` is a boolean flag and `left` and `right` are `EccPoint`,
    /// returns `left` if `choice` is not set and `right` if `choice` is set.
    pub fn mux_on_points(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        choice: &AssignedCell<pallas::Base, pallas::Base>,
        left: &EccPoint,
        right: &EccPoint,
    ) -> Result<EccPoint, plonk::Error> {
        let x_cell = self.mux(&mut layouter, choice.clone(), left.x(), right.x())?;
        let y_cell = self.mux(&mut layouter, choice.clone(), left.y(), right.y())?;
        Ok(EccPoint::from_coordinates_unchecked(
            x_cell.into(),
            y_cell.into(),
        ))
    }

    /// Given an input `(choice, left, right)` where `choice` is a boolean flag and `left` and `right` are
    /// `NonIdentityEccPoint`, returns `left` if `choice` is not set and `right` if `choice` is set.
    pub fn mux_on_non_identity_points(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        choice: &AssignedCell<pallas::Base, pallas::Base>,
        left: &NonIdentityEccPoint,
        right: &NonIdentityEccPoint,
    ) -> Result<NonIdentityEccPoint, plonk::Error> {
        let x_cell = self.mux(&mut layouter, choice.clone(), left.x(), right.x())?;
        let y_cell = self.mux(&mut layouter, choice.clone(), left.y(), right.y())?;
        Ok(NonIdentityEccPoint::from_coordinates_unchecked(
            x_cell.into(),
            y_cell.into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::utilities::cond_swap::{CondSwapChip, CondSwapConfig};
    use crate::utilities_opt::lookup_range_check::LookupRangeCheckConfigOptimized;

    #[test]
    fn test_mux() {
        use crate::{
            ecc::{
                chip::{EccChip, EccConfig},
                tests::TestFixedBases,
                NonIdentityPoint, Point,
            },
            utilities::lookup_range_check::{LookupRangeCheck, LookupRangeCheckConfig},
        };

        use group::{cofactor::CofactorCurveAffine, Curve, Group};
        use halo2_proofs::{
            circuit::{Layouter, SimpleFloorPlanner, Value},
            dev::MockProver,
            plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
        };
        use pasta_curves::arithmetic::CurveAffine;
        use pasta_curves::{pallas, EpAffine};

        use rand::rngs::OsRng;

        #[derive(Clone, Debug)]
        pub struct MyConfig {
            primary: Column<Instance>,
            advice: Column<Advice>,
            cond_swap_config: CondSwapConfig,
            ecc_config: EccConfig<
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >,
        }

        #[derive(Default)]
        struct MyCircuit {
            left_point: Value<EpAffine>,
            right_point: Value<EpAffine>,
            choice: Value<pallas::Base>,
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = MyConfig;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self::default()
            }

            fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
                let advices = [
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                ];

                for advice in advices.iter() {
                    meta.enable_equality(*advice);
                }

                // Instance column used for public inputs
                let primary = meta.instance_column();
                meta.enable_equality(primary);

                let cond_swap_config =
                    CondSwapChip::configure(meta, advices[0..5].try_into().unwrap());

                let table_idx = meta.lookup_table_column();

                let lagrange_coeffs = [
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                ];
                meta.enable_constant(lagrange_coeffs[0]);

                let range_check =
                    LookupRangeCheckConfigOptimized::configure(meta, advices[9], table_idx);

                let ecc_config =
                    EccChip::<
                        TestFixedBases,
                        LookupRangeCheckConfigOptimized<
                            pallas::Base,
                            { crate::sinsemilla::primitives::K },
                        >,
                    >::configure(meta, advices, lagrange_coeffs, range_check);

                MyConfig {
                    primary,
                    advice: advices[0],
                    cond_swap_config,
                    ecc_config,
                }
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                // Construct a CondSwap chip
                let cond_swap_chip = CondSwapChip::construct(config.cond_swap_config);

                // Construct an ECC chip
                let ecc_chip = EccChip::construct(config.ecc_config);

                // Assign choice
                let choice = layouter.assign_region(
                    || "load private",
                    |mut region| {
                        region.assign_advice(|| "load private", config.advice, 0, || self.choice)
                    },
                )?;

                // Test mux on non identity points
                // Assign left point
                let left_non_identity_point = NonIdentityPoint::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "left point"),
                    self.left_point.map(|left_point| left_point),
                )?;

                // Assign right point
                let right_non_identity_point = NonIdentityPoint::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "right point"),
                    self.right_point.map(|right_point| right_point),
                )?;

                // Apply mux
                let result_non_identity_point = cond_swap_chip.mux_on_non_identity_points(
                    layouter.namespace(|| "MUX"),
                    &choice,
                    left_non_identity_point.inner(),
                    right_non_identity_point.inner(),
                )?;

                // Check equality with instance
                layouter.constrain_instance(
                    result_non_identity_point.x().cell(),
                    config.primary,
                    0,
                )?;
                layouter.constrain_instance(
                    result_non_identity_point.y().cell(),
                    config.primary,
                    1,
                )?;

                // Test mux on points
                // Assign left point
                let left_point = Point::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "left point"),
                    self.left_point.map(|left_point| left_point),
                )?;

                // Assign right point
                let right_point = Point::new(
                    ecc_chip,
                    layouter.namespace(|| "right point"),
                    self.right_point.map(|right_point| right_point),
                )?;

                // Apply mux
                let result = cond_swap_chip.mux_on_points(
                    layouter.namespace(|| "MUX"),
                    &choice,
                    left_point.inner(),
                    right_point.inner(),
                )?;

                // Check equality with instance
                layouter.constrain_instance(result.x().cell(), config.primary, 0)?;
                layouter.constrain_instance(result.y().cell(), config.primary, 1)
            }
        }

        // Test different circuits
        let mut circuits = vec![];
        let mut instances = vec![];
        for choice in [false, true] {
            let choice_value = if choice {
                pallas::Base::one()
            } else {
                pallas::Base::zero()
            };
            let left_point = pallas::Point::random(OsRng).to_affine();
            let right_point = pallas::Point::random(OsRng).to_affine();
            circuits.push(MyCircuit {
                left_point: Value::known(left_point),
                right_point: Value::known(right_point),
                choice: Value::known(choice_value),
            });
            let expected_output = if choice { right_point } else { left_point };
            let (expected_x, expected_y) = if bool::from(expected_output.is_identity()) {
                (pallas::Base::zero(), pallas::Base::zero())
            } else {
                let coords = expected_output.coordinates().unwrap();
                (*coords.x(), *coords.y())
            };
            instances.push([[expected_x, expected_y]]);
        }

        for (circuit, instance) in circuits.iter().zip(instances.iter()) {
            let prover = MockProver::<pallas::Base>::run(
                5,
                circuit,
                instance.iter().map(|p| p.to_vec()).collect(),
            )
            .unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
