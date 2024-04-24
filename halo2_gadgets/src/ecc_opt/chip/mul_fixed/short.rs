use crate::ecc::chip::EccPoint;

use super::super::FixedPoints;

use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    plonk::Error,
};
use pasta_curves::pallas;

use crate::ecc::chip::mul_fixed::short::Config;

impl<Fixed: FixedPoints<pallas::Affine>> Config<Fixed> {
    /// Multiply the point by sign, using the q_mul_fixed_short gate.
    /// Constraints `sign` in {-1, 1}
    pub fn assign_scalar_sign(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        sign: &AssignedCell<pallas::Base, pallas::Base>,
        point: &EccPoint,
    ) -> Result<EccPoint, Error> {
        let signed_point = layouter.assign_region(
            || "Signed point",
            |mut region| {
                let offset = 0;

                // Enable mul_fixed_short selector to check the sign logic.
                self.q_mul_fixed_short.enable(&mut region, offset)?;

                // Set "last window" to 0 (this field is irrelevant here).
                region.assign_advice_from_constant(
                    || "u=0",
                    self.super_config.u,
                    offset,
                    pallas::Base::zero(),
                )?;

                // Copy sign to `window` column
                sign.copy_advice(|| "sign", &mut region, self.super_config.window, offset)?;

                // Assign the input y-coordinate.
                point.y.copy_advice(
                    || "unsigned y",
                    &mut region,
                    self.super_config.add_config.y_qr,
                    offset,
                )?;

                // Conditionally negate y-coordinate according to the value of sign
                let signed_y_val = sign.value().and_then(|sign| {
                    if sign == &-pallas::Base::one() {
                        -point.y.value()
                    } else {
                        point.y.value().cloned()
                    }
                });

                // Assign the output signed y-coordinate.
                let signed_y = region.assign_advice(
                    || "signed y",
                    self.super_config.add_config.y_p,
                    offset,
                    || signed_y_val,
                )?;

                Ok(EccPoint {
                    x: point.x.clone(),
                    y: signed_y,
                })
            },
        )?;

        Ok(signed_point)
    }
}

#[cfg(test)]
pub mod tests {
    use group::{ff::PrimeField, Curve, Group};
    use halo2_proofs::{
        arithmetic::CurveAffine,
        circuit::{AssignedCell, Chip, Layouter, Value},
        plonk::{Any, Error},
    };
    use pasta_curves::pallas;

    use crate::utilities::lookup_range_check::{DefaultLookupRangeCheck, LookupRangeCheck};
    use crate::utilities_opt::lookup_range_check::LookupRangeCheckConfigOptimized;
    use crate::{
        ecc::{
            chip::{EccChip},
            tests::{Short, TestFixedBases},
            Point, ScalarFixedShort,
        },
        utilities::UtilitiesInstructions,
    };

    #[test]
    fn invalid_magnitude_sign() {
        use crate::{
            ecc::chip::{EccConfig, FixedPoint},
            utilities::UtilitiesInstructions,
        };
        use halo2_proofs::{
            circuit::{Layouter, SimpleFloorPlanner},
            dev::{FailureLocation, MockProver, VerifyFailure},
            plonk::{Circuit, ConstraintSystem, Error},
        };

        #[derive(Default)]
        struct MyCircuit {
            magnitude: Value<pallas::Base>,
            sign: Value<pallas::Base>,
            // For test checking
            magnitude_error: Value<pallas::Base>,
        }

        impl UtilitiesInstructions<pallas::Base> for MyCircuit {
            type Var = AssignedCell<pallas::Base, pallas::Base>;
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = EccConfig<
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >;
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
                let lookup_table = meta.lookup_table_column();
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

                // Shared fixed column for loading constants
                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                let range_check =
                    LookupRangeCheckConfigOptimized::configure(meta, advices[9], lookup_table);
                EccChip::<
                    TestFixedBases,
                    LookupRangeCheckConfigOptimized<
                        pallas::Base,
                        { crate::sinsemilla::primitives::K },
                    >,
                >::configure(meta, advices, lagrange_coeffs, range_check)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                let column = config.advices[0];

                let short_config = config.mul_fixed_short.clone();
                let magnitude_sign = {
                    let magnitude = self.load_private(
                        layouter.namespace(|| "load magnitude"),
                        column,
                        self.magnitude,
                    )?;
                    let sign =
                        self.load_private(layouter.namespace(|| "load sign"), column, self.sign)?;
                    ScalarFixedShort::new(
                        EccChip::construct(config),
                        layouter.namespace(|| "signed short scalar"),
                        (magnitude, sign),
                    )?
                };

                short_config.assign(layouter, &magnitude_sign.inner, &Short)?;

                Ok(())
            }
        }

        // Copied from halo2_proofs::dev::util
        fn format_value(v: pallas::Base) -> String {
            use ff::Field;
            if v.is_zero_vartime() {
                "0".into()
            } else if v == pallas::Base::one() {
                "1".into()
            } else if v == -pallas::Base::one() {
                "-1".into()
            } else {
                // Format value as hex.
                let s = format!("{:?}", v);
                // Remove leading zeroes.
                let s = s.strip_prefix("0x").unwrap();
                let s = s.trim_start_matches('0');
                format!("0x{}", s)
            }
        }

        // Magnitude larger than 64 bits should fail
        {
            let circuits = [
                // 2^64
                MyCircuit {
                    magnitude: Value::known(pallas::Base::from_u128(1 << 64)),
                    sign: Value::known(pallas::Base::one()),
                    magnitude_error: Value::known(pallas::Base::from(1 << 1)),
                },
                // -2^64
                MyCircuit {
                    magnitude: Value::known(pallas::Base::from_u128(1 << 64)),
                    sign: Value::known(-pallas::Base::one()),
                    magnitude_error: Value::known(pallas::Base::from(1 << 1)),
                },
                // 2^66
                MyCircuit {
                    magnitude: Value::known(pallas::Base::from_u128(1 << 66)),
                    sign: Value::known(pallas::Base::one()),
                    magnitude_error: Value::known(pallas::Base::from(1 << 3)),
                },
                // -2^66
                MyCircuit {
                    magnitude: Value::known(pallas::Base::from_u128(1 << 66)),
                    sign: Value::known(-pallas::Base::one()),
                    magnitude_error: Value::known(pallas::Base::from(1 << 3)),
                },
                // 2^254
                MyCircuit {
                    magnitude: Value::known(pallas::Base::from_u128(1 << 127).square()),
                    sign: Value::known(pallas::Base::one()),
                    magnitude_error: Value::known(
                        pallas::Base::from_u128(1 << 95).square() * pallas::Base::from(2),
                    ),
                },
                // -2^254
                MyCircuit {
                    magnitude: Value::known(pallas::Base::from_u128(1 << 127).square()),
                    sign: Value::known(-pallas::Base::one()),
                    magnitude_error: Value::known(
                        pallas::Base::from_u128(1 << 95).square() * pallas::Base::from(2),
                    ),
                },
            ];

            for circuit in circuits.iter() {
                let prover = MockProver::<pallas::Base>::run(11, circuit, vec![]).unwrap();
                circuit.magnitude_error.assert_if_known(|magnitude_error| {
                    assert_eq!(
                        prover.verify(),
                        Err(vec![
                            VerifyFailure::ConstraintNotSatisfied {
                                constraint: (
                                    (17, "Short fixed-base mul gate").into(),
                                    0,
                                    "last_window_check",
                                )
                                    .into(),
                                location: FailureLocation::InRegion {
                                    region: (3, "Short fixed-base mul (most significant word)")
                                        .into(),
                                    offset: 1,
                                },
                                cell_values: vec![(
                                    ((Any::Advice, 5).into(), 0).into(),
                                    format_value(*magnitude_error),
                                )],
                            },
                            VerifyFailure::Permutation {
                                column: (Any::Fixed, 10).into(),
                                location: FailureLocation::OutsideRegion { row: 0 },
                            },
                            VerifyFailure::Permutation {
                                column: (Any::Advice, 4).into(),
                                location: FailureLocation::InRegion {
                                    region: (2, "Short fixed-base mul (incomplete addition)")
                                        .into(),
                                    offset: 22,
                                },
                            },
                        ])
                    );
                    true
                });
            }
        }

        // Sign that is not +/- 1 should fail
        {
            let magnitude_u64 = rand::random::<u64>();
            let circuit = MyCircuit {
                magnitude: Value::known(pallas::Base::from(magnitude_u64)),
                sign: Value::known(pallas::Base::zero()),
                magnitude_error: Value::unknown(),
            };

            let negation_check_y = {
                *(Short.generator() * pallas::Scalar::from(magnitude_u64))
                    .to_affine()
                    .coordinates()
                    .unwrap()
                    .y()
            };

            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Err(vec![
                    VerifyFailure::ConstraintNotSatisfied {
                        constraint: ((17, "Short fixed-base mul gate").into(), 1, "sign_check")
                            .into(),
                        location: FailureLocation::InRegion {
                            region: (3, "Short fixed-base mul (most significant word)").into(),
                            offset: 1,
                        },
                        cell_values: vec![(((Any::Advice, 4).into(), 0).into(), "0".to_string())],
                    },
                    VerifyFailure::ConstraintNotSatisfied {
                        constraint: (
                            (17, "Short fixed-base mul gate").into(),
                            3,
                            "negation_check"
                        )
                            .into(),
                        location: FailureLocation::InRegion {
                            region: (3, "Short fixed-base mul (most significant word)").into(),
                            offset: 1,
                        },
                        cell_values: vec![
                            (
                                ((Any::Advice, 1).into(), 0).into(),
                                format_value(negation_check_y),
                            ),
                            (
                                ((Any::Advice, 3).into(), 0).into(),
                                format_value(negation_check_y),
                            ),
                            (((Any::Advice, 4).into(), 0).into(), "0".to_string()),
                        ],
                    }
                ])
            );
        }
    }

    pub(crate) fn test_mul_sign<LookupRangeCheckConfig: DefaultLookupRangeCheck>(
        chip: EccChip<TestFixedBases, LookupRangeCheckConfig>,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        // Generate a random non-identity point P
        let p_val = pallas::Point::random(rand::rngs::OsRng).to_affine();
        let p = Point::new(
            chip.clone(),
            layouter.namespace(|| "P"),
            Value::known(p_val),
        )?;

        // Create -P
        let p_neg_val = -p_val;
        let p_neg = Point::new(
            chip.clone(),
            layouter.namespace(|| "-P"),
            Value::known(p_neg_val),
        )?;

        // Create the identity point
        let identity = Point::new(
            chip.clone(),
            layouter.namespace(|| "identity"),
            Value::known(pallas::Point::identity().to_affine()),
        )?;

        // Create -1 and 1 scalars
        let pos_sign = chip.load_private(
            layouter.namespace(|| "positive sign"),
            chip.config().advices[0],
            Value::known(pallas::Base::one()),
        )?;
        let neg_sign = chip.load_private(
            layouter.namespace(|| "negative sign"),
            chip.config().advices[1],
            Value::known(-pallas::Base::one()),
        )?;

        // [1] P == P
        {
            let result = p.mul_sign(layouter.namespace(|| "[1] P"), &pos_sign)?;
            result.constrain_equal(layouter.namespace(|| "constrain [1] P"), &p)?;
        }

        // [-1] P == -P
        {
            let result = p.mul_sign(layouter.namespace(|| "[1] P"), &neg_sign)?;
            result.constrain_equal(layouter.namespace(|| "constrain [1] P"), &p_neg)?;
        }

        // [1] 0 == 0
        {
            let result = identity.mul_sign(layouter.namespace(|| "[1] O"), &pos_sign)?;
            result.constrain_equal(layouter.namespace(|| "constrain [1] 0"), &identity)?;
        }

        // [-1] 0 == 0
        {
            let result = identity.mul_sign(layouter.namespace(|| "[-1] O"), &neg_sign)?;
            result.constrain_equal(layouter.namespace(|| "constrain [1] 0"), &identity)?;
        }

        Ok(())
    }

    #[test]
    fn invalid_sign_in_mul_sign() {
        use crate::{ecc::chip::EccConfig, utilities::UtilitiesInstructions};
        use halo2_proofs::{
            circuit::{Layouter, SimpleFloorPlanner},
            dev::{FailureLocation, MockProver, VerifyFailure},
            plonk::{Circuit, ConstraintSystem, Error},
        };

        #[derive(Default)]
        struct MyCircuit {
            base: Value<pallas::Affine>,
            sign: Value<pallas::Base>,
        }

        impl UtilitiesInstructions<pallas::Base> for MyCircuit {
            type Var = AssignedCell<pallas::Base, pallas::Base>;
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = EccConfig<
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >;
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
                let lookup_table = meta.lookup_table_column();
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

                // Shared fixed column for loading constants
                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                let range_check =
                    LookupRangeCheckConfigOptimized::configure(meta, advices[9], lookup_table);
                EccChip::<
                    TestFixedBases,
                    LookupRangeCheckConfigOptimized<
                        pallas::Base,
                        { crate::sinsemilla::primitives::K },
                    >,
                >::configure(meta, advices, lagrange_coeffs, range_check)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                let chip = EccChip::construct(config.clone());

                let column = config.advices[0];

                //let short_config = config.mul_fixed_short.clone();
                let base = Point::new(chip, layouter.namespace(|| "load base"), self.base)?;

                let sign =
                    self.load_private(layouter.namespace(|| "load sign"), column, self.sign)?;

                base.mul_sign(layouter.namespace(|| "[sign] base"), &sign)?;

                Ok(())
            }
        }

        // Copied from halo2_proofs::dev::util
        fn format_value(v: pallas::Base) -> String {
            use ff::Field;
            if v.is_zero_vartime() {
                "0".into()
            } else if v == pallas::Base::one() {
                "1".into()
            } else if v == -pallas::Base::one() {
                "-1".into()
            } else {
                // Format value as hex.
                let s = format!("{:?}", v);
                // Remove leading zeroes.
                let s = s.strip_prefix("0x").unwrap();
                let s = s.trim_start_matches('0');
                format!("0x{}", s)
            }
        }

        // Sign that is not +/- 1 should fail
        // Generate a random non-identity point
        let point = pallas::Point::random(rand::rngs::OsRng);
        let circuit = MyCircuit {
            base: Value::known(point.to_affine()),
            sign: Value::known(pallas::Base::zero()),
        };

        let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![
                VerifyFailure::ConstraintNotSatisfied {
                    constraint: ((17, "Short fixed-base mul gate").into(), 1, "sign_check").into(),
                    location: FailureLocation::InRegion {
                        region: (2, "Signed point").into(),
                        offset: 0,
                    },
                    cell_values: vec![(((Any::Advice, 4).into(), 0).into(), "0".to_string())],
                },
                VerifyFailure::ConstraintNotSatisfied {
                    constraint: (
                        (17, "Short fixed-base mul gate").into(),
                        3,
                        "negation_check"
                    )
                        .into(),
                    location: FailureLocation::InRegion {
                        region: (2, "Signed point").into(),
                        offset: 0,
                    },
                    cell_values: vec![
                        (
                            ((Any::Advice, 1).into(), 0).into(),
                            format_value(*point.to_affine().coordinates().unwrap().y()),
                        ),
                        (
                            ((Any::Advice, 3).into(), 0).into(),
                            format_value(*point.to_affine().coordinates().unwrap().y()),
                        ),
                        (((Any::Advice, 4).into(), 0).into(), "0".to_string()),
                    ],
                }
            ])
        );
    }
}
