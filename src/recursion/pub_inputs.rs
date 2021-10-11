//! Gadget to pack and unpack public inputs.

use crate::{
    circuit::{AllocatedCell, Cell, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

use ff::Field;
use pasta_curves::arithmetic::{CurveAffine, FieldExt};
use std::convert::TryInto;

mod keygen;
mod pedersen;
mod table;

use keygen::Params;

#[derive(Copy, Clone, Debug)]
struct AllocatedPoint<C: CurveAffine> {
    x: AllocatedCell<C::Base, C::Base>,
    y: AllocatedCell<C::Base, C::Base>,
}

impl<C: CurveAffine> AllocatedPoint<C> {
    fn point(&self) -> Option<C> {
        self.x
            .value()
            .zip(self.y.value())
            .map(|(x, y)| C::from_xy(x, y).unwrap())
    }
}

#[derive(Debug, Clone)]
struct PubInputsConfig<C: CurveAffine, const N: usize> {
    params: Params<C, N>,
    q_bool: Selector,
    q_init: Selector,
    q_point: Selector,
    window: Column<Advice>,
    bits: [Column<Advice>; 4], // little-endian bits[0], bits[1], bits[2], bits[3]
    point: (Column<Advice>, Column<Advice>),
    acc: (Column<Advice>, Column<Advice>),
    lookup: table::Config<C, N>,
}

impl<C: CurveAffine, const N: usize> PubInputsConfig<C, N> {
    fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        advices: [Column<Advice>; 9],
        params: Params<C, N>,
    ) -> Self {
        let lookup = table::Config::<C, N> {
            params: params.clone(),
            tag: meta.lookup_table_column(),
            x: meta.lookup_table_column(),
            y: meta.lookup_table_column(),
        };

        let config = Self {
            params,
            q_bool: meta.selector(),
            q_init: meta.selector(),
            q_point: meta.selector(),
            window: advices[0],
            bits: advices[1..5].try_into().unwrap(),
            point: (advices[5], advices[6]),
            acc: (advices[7], advices[8]),
            lookup,
        };

        config.create_gate(meta);

        config
    }

    fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        let one = Expression::Constant(C::Base::one());

        // y^2 = x^3 + b
        let curve_eqn = |x: Expression<C::Base>, y: Expression<C::Base>| {
            y.square() - (x.clone().square() * x) - Expression::Constant(C::b())
        };

        // bit * (1 - bit) = 0
        let bool_check = |bit: Expression<C::Base>| bit.clone() * (one.clone() - bit);

        meta.create_gate("public inputs", |meta| {
            let q_point = meta.query_selector(self.q_point);
            let point_x = meta.query_advice(self.point.0, Rotation::cur());
            let point_y = meta.query_advice(self.point.1, Rotation::cur());
            let acc_x = meta.query_advice(self.acc.0, Rotation::cur());
            let acc_y = meta.query_advice(self.acc.1, Rotation::cur());

            // Check `point` is on curve
            let point_check = q_point.clone() * curve_eqn(point_x.clone(), point_y.clone());

            // Check `acc` is on curve
            let acc_check = q_point * curve_eqn(acc_x.clone(), acc_y.clone());

            // Check `point` == `acc` when initialising accumulator
            let init_checks = {
                let q_init = meta.query_selector(self.q_init);
                let x_check = q_init.clone() * (point_x.clone() - acc_x.clone());
                let y_check = q_init.clone() * (point_y.clone() - acc_y.clone());

                vec![("x_check", x_check), ("y_check", y_check)]
            };

            // Check all bits are boolean
            let bool_checks = self
                .bits
                .iter()
                .map(move |bit| {
                    let q_bool = meta.query_selector(self.q_bool);
                    let bit = meta.query_advice(*bit, Rotation::cur());
                    ("bool check", q_bool * bool_check(bit.clone()))
                })
                .collect::<Vec<_>>();

            bool_checks
                .into_iter()
                .chain(init_checks.into_iter())
                .chain(Some(("point check", point_check)))
                .chain(Some(("acc check", acc_check)))
        });

        meta.lookup(|meta| {
            // tag = bit0 + bit1 * 2 + window * 4
            let tag = {
                let window = meta.query_advice(self.window, Rotation::cur());
                let bit0 = meta.query_advice(self.bits[0], Rotation::cur());
                let bit1 = meta.query_advice(self.bits[1], Rotation::cur());

                bit0 + bit1 * C::Base::from_u64(2) + window * C::Base::from_u64(4)
            };

            let x = meta.query_advice(self.point.0, Rotation::cur());
            let y = meta.query_advice(self.point.1, Rotation::cur());

            // We are looking up x_ab, so we have to reverse-engineer `x_ab`
            // from `x`:
            //      - expect x = endo(x_ab) if bits[3] = 1
            //      - expect x = x_ab if bits[3] = 0
            // x = (1 - bits[3]) * x_ab + bits[3] * inv_endo(x_ab)
            let lookup_x = {
                let bit3 = meta.query_advice(self.bits[3], Rotation::cur());
                (one.clone() - bit3.clone()) * x.clone()
                    + bit3 * x * Expression::Constant(C::Base::ZETA.invert().unwrap())
            };

            // We are looking up y_ab, so we have to reverse-engineer `y_ab`
            // from `y`:
            //      - expect y = -y_ab if bits[2] = 1
            //      - expect y = y_ab if bits[2] = 0
            // y = (1 - bits[2]) * y_ab + bits[2] * (-y_ab)
            let lookup_y = {
                let bit2 = meta.query_advice(self.bits[2], Rotation::cur());
                (one.clone() - bit2.clone()) * y.clone() + bit2 * (-y)
            };

            vec![
                (tag, self.lookup.tag),
                (lookup_x, self.lookup.x),
                (lookup_y, self.lookup.y),
            ]
        });
    }

    fn assign_point(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        point: (Option<C::Base>, Option<C::Base>),
    ) -> Result<AllocatedPoint<C>, Error> {
        self.q_point.enable(region, offset)?;

        let x = {
            let value = point.0;
            let cell = region.assign_advice(
                || "point x",
                self.point.0,
                offset,
                || point.0.ok_or(Error::SynthesisError),
            )?;
            AllocatedCell::new(value, cell)
        };

        let y = {
            let value = point.1;
            let cell = region.assign_advice(
                || "point y",
                self.point.1,
                offset,
                || point.1.ok_or(Error::SynthesisError),
            )?;
            AllocatedCell::new(value, cell)
        };

        Ok(AllocatedPoint { x, y })
    }

    fn assign_acc(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        acc: (Option<C::Base>, Option<C::Base>),
    ) -> Result<AllocatedPoint<C>, Error> {
        let x = {
            let value = acc.0;
            let cell = region.assign_advice(
                || "acc x",
                self.acc.0,
                offset,
                || acc.0.ok_or(Error::SynthesisError),
            )?;
            AllocatedCell::new(value, cell)
        };

        let y = {
            let value = acc.1;
            let cell = region.assign_advice(
                || "acc y",
                self.acc.1,
                offset,
                || acc.1.ok_or(Error::SynthesisError),
            )?;
            AllocatedCell::new(value, cell)
        };

        Ok(AllocatedPoint { x, y })
    }

    fn assign_bits(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        bits: [Option<bool>; 4],
    ) -> Result<[AllocatedCell<C::Base, bool>; 4], Error> {
        // Check that each bit is boolean.
        self.q_bool.enable(region, offset)?;

        let four_bit: Result<Vec<_>, _> = bits
            .iter()
            .enumerate()
            .map(
                |(index, bit)| -> Result<AllocatedCell<C::Base, bool>, Error> {
                    let cell = region.assign_advice(
                        || format!("bit {}", index),
                        self.bits[index],
                        offset,
                        || {
                            bit.map(|bit| C::Base::from_u64(bit as u64))
                                .ok_or(Error::SynthesisError)
                        },
                    )?;
                    Ok(AllocatedCell::new(*bit, cell))
                },
            )
            .collect();

        Ok(four_bit?.try_into().unwrap())
    }

    fn init(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        bits: [Option<bool>; 4],
        window: [C; 4],
    ) -> Result<AllocatedPoint<C>, Error> {
        self.q_init.enable(region, offset)?;
        let point = self.process_bits(region, offset, &bits, window, 0)?;

        // Copy point to `acc` columns
        point
            .x
            .copy_advice(|| "Copy point x to acc x", region, self.acc.0, offset)?;
        point
            .y
            .copy_advice(|| "Copy point y to acc y", region, self.acc.1, offset)?;

        Ok(point)
    }

    fn process_bits(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        bits: &[Option<bool>; 4],
        window: [C; 4],
        idx: usize,
    ) -> Result<AllocatedPoint<C>, Error> {
        // TODO: Explain this
        let closure = |constants: Vec<C::Base>, assignment: &mut [C::Base]| {
            for (i, constant) in constants.into_iter().enumerate() {
                let cur = constant - assignment[i];
                assignment[i] = cur;
                for (j, eval) in assignment.iter_mut().enumerate().skip(i + 1) {
                    if j & i == i {
                        *eval += cur;
                    }
                }
            }
        };

        let linear_comb =
            |coeffs: [C::Base; 4], bit0: Option<bool>, bit1: Option<bool>| -> Option<C::Base> {
                let bits_01 = bit0.zip(bit1).map(|(bit0, bit1)| bit0 & bit1);

                if let (Some(bit0), Some(bit1), Some(bits_01)) = (bit0, bit1, bits_01) {
                    let point = coeffs
                        .iter()
                        .zip([true, bit0, bit1, bits_01].iter())
                        .fold(C::Base::zero(), |acc, (coeff, bit)| {
                            acc + *coeff * C::Base::from_u64(*bit as u64)
                        });

                    Some(point)
                } else {
                    None
                }
            };

        let mut x_coeffs = [C::Base::zero(); 4];
        let mut y_coeffs = [C::Base::zero(); 4];

        region.assign_advice(
            || format!("window {}", idx),
            self.window,
            offset,
            || Ok(C::Base::from_u64(idx as u64)),
        )?;
        self.assign_bits(region, offset, *bits)?;

        let window = window.iter().map(|point| point.coordinates().unwrap());

        closure(
            window.clone().map(|point| *point.x()).collect(),
            &mut x_coeffs,
        );
        closure(window.map(|point| *point.y()).collect(), &mut y_coeffs);

        let x_lc = linear_comb(x_coeffs, bits[0], bits[1]);
        let y_lc = linear_comb(y_coeffs, bits[0], bits[1]);

        // Conditional endo on bits[3]
        let x2 = x_lc
            .zip(bits[3])
            .map(|(x, bit3)| if bit3 { x * C::Base::ZETA } else { x });

        // Conditional negation on bits[2]
        let y2 = y_lc.zip(bits[2]).map(|(y, bit2)| if bit2 { -y } else { y });

        self.assign_point(region, offset, (x2, y2))
    }

    pub(crate) fn pack_bits(
        &self,
        mut layouter: impl Layouter<C::Base>,
        bits: Vec<[Option<bool>; 4]>,
    ) -> Result<AllocatedPoint<C>, Error> {
        layouter.assign_region(
            || "pack bits",
            |mut region| {
                let mut offset = 0;
                let acc = {
                    let point = self.init(
                        &mut region,
                        offset,
                        bits[0],
                        self.params.pedersen_windows[0],
                    )?;
                    point
                };
                let (mut x1, mut y1) = (acc.x.value(), acc.y.value());
                let mut acc = vec![acc];

                offset += 1;

                for (idx, (bits, window)) in bits
                    .iter()
                    .zip(self.params.pedersen_windows.iter())
                    .enumerate()
                    .skip(1)
                {
                    let point = self.process_bits(&mut region, offset, bits, *window, idx)?;

                    let x2 = point.x.value();
                    let y2 = point.y.value();

                    // TODO: use Orchard incomplete addition gadget

                    // lambda = (y2 - y1) / (x2 - x1)
                    let lambda = x1
                        .zip(y1)
                        .zip(x2)
                        .zip(y2)
                        .map(|(((x1, y1), x2), y2)| (y2 - y1) * (x2 - x1).invert().unwrap());
                    // x3 = lambda^2 - x1 - x2
                    let x3 = lambda
                        .zip(x1)
                        .zip(x2)
                        .map(|((lambda, x1), x2)| lambda.square() - x1 - x2);

                    // y3 = lambda * (x1 - x3) - y1
                    let y3 = lambda
                        .zip(x1)
                        .zip(x3)
                        .zip(y1)
                        .map(|(((lambda, x1), x3), y1)| lambda * (x1 - x3) - y1);

                    x1 = x3;
                    y1 = y3;

                    acc.push(self.assign_acc(&mut region, offset, (x1, y1))?);

                    offset += 1;
                }

                Ok(*acc.last().unwrap())
            },
        )
    }
}

#[test]
fn test_pub_inputs() {
    use crate::{circuit::SimpleFloorPlanner, dev::MockProver, plonk::Circuit};
    use pasta_curves::pallas;

    struct MyCircuit<const N_DIV_FOUR: usize> {
        bits: [[Option<bool>; 4]; N_DIV_FOUR],
    }

    impl<const N_DIV_FOUR: usize> Default for MyCircuit<N_DIV_FOUR> {
        fn default() -> Self {
            Self {
                bits: [[None; 4]; N_DIV_FOUR],
            }
        }
    }

    impl Circuit<pallas::Base> for MyCircuit<2> {
        type Config = PubInputsConfig<pallas::Affine, 8>;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
            let params: Params<pallas::Affine, 8> = Params::init();
            let advices: [Column<Advice>; 9] = (0..9)
                .map(|_| meta.advice_column())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();

            PubInputsConfig::<pallas::Affine, 8>::configure(meta, advices, params)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            config
                .lookup
                .load(layouter.namespace(|| "load generators table"))?;
            let point = config.pack_bits(layouter, self.bits.to_vec())?.point();

            let expected_point = {
                let bits: Vec<Option<bool>> = self
                    .bits
                    .iter()
                    .flat_map(|bits| bits.iter().map(|bit| *bit))
                    .collect();
                let bits: Option<Vec<bool>> = bits.into_iter().collect();
                bits.map(|bits| config.params.commit(&bits))
            };

            if let (Some(point), Some(expected_point)) = (point, expected_point) {
                assert_eq!(point, expected_point);
            }

            Ok(())
        }
    }

    let circuit: MyCircuit<2> = MyCircuit {
        bits: [
            [Some(false), Some(false), Some(true), Some(true)],
            [Some(true), Some(true), Some(false), Some(false)],
        ],
    };

    let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
