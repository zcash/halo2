use crate::multicore;
use crate::plonk::lookup::prover::Committed;
use crate::plonk::permutation::Argument;
use crate::plonk::{lookup, permutation, Any, ProvingKey};
use crate::poly::Basis;
use crate::{
    arithmetic::{eval_polynomial, parallelize, BaseExt, CurveAffine, FieldExt},
    poly::{
        commitment::Params, multiopen::ProverQuery, Coeff, EvaluationDomain, ExtendedLagrangeCoeff,
        LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{EncodedChallenge, TranscriptWrite},
};
use group::prime::PrimeCurve;
use group::{
    ff::{BatchInvert, Field},
    Curve,
};
use std::any::TypeId;
use std::convert::TryInto;
use std::num::ParseIntError;
use std::slice;
use std::{
    collections::BTreeMap,
    iter,
    ops::{Index, Mul, MulAssign},
};

use super::{ConstraintSystem, Expression};

/// Value for use in a calculation
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum ValueSource {
    /// This is a constant value
    Constant(usize),
    /// This is an intermediate value
    Intermediate(usize),
    /// This is a fixed column
    Fixed(usize, usize),
    /// This is an advice (witness) column
    Advice(usize, usize),
    /// This is an instance (external) column
    Instance(usize, usize),
}

impl ValueSource {
    /// Get the value for this source
    pub fn evaluate<F: Field, B: Basis>(
        &self,
        rotations: &[usize],
        constants: &[F],
        intermediates: &[F],
        fixed_values: &[Polynomial<F, B>],
        advice_values: &[Polynomial<F, B>],
        instance_values: &[Polynomial<F, B>],
    ) -> F {
        match self {
            ValueSource::Constant(idx) => constants[*idx],
            ValueSource::Intermediate(idx) => intermediates[*idx],
            ValueSource::Fixed(column_index, rotation) => {
                *fixed_values[*column_index].index(rotations[*rotation])
            }
            ValueSource::Advice(column_index, rotation) => {
                *advice_values[*column_index].index(rotations[*rotation])
            }
            ValueSource::Instance(column_index, rotation) => {
                *instance_values[*column_index].index(rotations[*rotation])
            }
        }
    }
}

/// Calculation
#[derive(Clone, Debug, PartialEq)]
pub enum Calculation {
    /// This is an addition
    Add(ValueSource, ValueSource),
    /// This is a subtraction
    Sub(ValueSource, ValueSource),
    /// This is a product
    Mul(ValueSource, ValueSource),
    /// This is a negation
    Negate(ValueSource),
    /// This is `(a + beta) * b`
    LcBeta(ValueSource, ValueSource),
    /// This is `a * theta + b`
    LcTheta(ValueSource, ValueSource),
    /// This is `a + gamma`
    AddGamma(ValueSource),
    /// This is a simple assignment
    Store(ValueSource),
}

impl Calculation {
    /// Get the resulting value of this calculation
    pub fn evaluate<F: Field, B: Basis>(
        &self,
        rotations: &[usize],
        constants: &[F],
        intermediates: &[F],
        fixed_values: &[Polynomial<F, B>],
        advice_values: &[Polynomial<F, B>],
        instance_values: &[Polynomial<F, B>],
        beta: &F,
        gamma: &F,
        theta: &F,
    ) -> F {
        match self {
            Calculation::Add(a, b) => {
                let a = a.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                a + b
            }
            Calculation::Sub(a, b) => {
                let a = a.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                a - b
            }
            Calculation::Mul(a, b) => {
                let a = a.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                a * b
            }
            Calculation::Negate(v) => -v.evaluate(
                rotations,
                constants,
                intermediates,
                fixed_values,
                advice_values,
                instance_values,
            ),
            Calculation::LcBeta(a, b) => {
                let a = a.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                (a + beta) * b
            }
            Calculation::LcTheta(a, b) => {
                let a = a.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                a * theta + b
            }
            Calculation::AddGamma(v) => {
                v.evaluate(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                ) + gamma
            }
            Calculation::Store(v) => v.evaluate(
                rotations,
                constants,
                intermediates,
                fixed_values,
                advice_values,
                instance_values,
            ),
        }
    }
}

/// EvaluationData
#[derive(Default, Debug)]
pub struct EvaluationData<F: Field> {
    /// Constants
    pub constants: Vec<F>,
    /// Rotations
    pub rotations: Vec<i32>,
    /// Calculations
    pub calculations: Vec<CalculationInfo>,
    /// Value parts
    pub value_parts: Vec<ValueSource>,
    /// Lookup results
    pub lookup_results: Vec<Calculation>,
}

/// CaluclationInfo
#[derive(Debug)]
pub struct CalculationInfo {
    /// Calculation
    pub calculation: Calculation,
    /// How many times this calculation is used
    pub counter: usize,
}

impl<F: Field> EvaluationData<F> {
    /// Creates a new evaluation structure
    pub fn new(cs: &ConstraintSystem<F>) -> Self {
        let mut ev = EvaluationData::default();
        ev.add_constant(&F::zero());
        ev.add_constant(&F::one());

        // Custom gates
        for gate in cs.gates.iter() {
            for poly in gate.polynomials().iter() {
                let vs = ev.add_expression(poly);
                ev.value_parts.push(vs);
            }
        }

        // Lookups
        for lookup in cs.lookups.iter() {
            let write_lc = |ev: &mut EvaluationData<_>, expressions: &Vec<Expression<_>>| {
                let parts = expressions
                    .iter()
                    .map(|expr| ev.add_expression(expr))
                    .collect::<Vec<_>>();
                let mut lc = parts[0];
                for part in parts.iter().skip(1) {
                    lc = ev.add_calculation(Calculation::LcTheta(lc, *part));
                }
                lc
            };
            // Input coset
            let compressed_input_coset = write_lc(&mut ev, &lookup.input_expressions);
            // table coset
            let compressed_table_coset = write_lc(&mut ev, &lookup.table_expressions);
            // z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
            let right_gamma = ev.add_calculation(Calculation::AddGamma(compressed_table_coset));
            ev.lookup_results
                .push(Calculation::LcBeta(compressed_input_coset, right_gamma));
        }

        ev
    }

    /// Adds a rotation
    fn add_rotation(&mut self, rotation: &Rotation) -> usize {
        let position = self.rotations.iter().position(|&c| c == rotation.0);
        match position {
            Some(pos) => pos,
            None => {
                self.rotations.push(rotation.0);
                self.rotations.len() - 1
            }
        }
    }

    /// Adds a constant
    fn add_constant(&mut self, constant: &F) -> ValueSource {
        let position = self.constants.iter().position(|&c| c == *constant);
        ValueSource::Constant(match position {
            Some(pos) => pos,
            None => {
                self.constants.push(*constant);
                self.constants.len() - 1
            }
        })
    }

    /// Adds a calculation
    /// Currently does the simplest thing possible: just stores the
    /// resulting value so the result can be reused  when that calculation
    /// is done multiple times.
    fn add_calculation(&mut self, calculation: Calculation) -> ValueSource {
        let position = self
            .calculations
            .iter()
            .position(|c| c.calculation == calculation);
        match position {
            Some(pos) => {
                self.calculations[pos].counter += 1;
                ValueSource::Intermediate(pos)
            }
            None => {
                self.calculations.push(CalculationInfo {
                    counter: 1,
                    calculation,
                });
                ValueSource::Intermediate(self.calculations.len() - 1)
            }
        }
    }

    /// Generates optimized evaluation for the expression
    fn add_expression(&mut self, expr: &Expression<F>) -> ValueSource {
        match expr {
            Expression::Constant(scalar) => self.add_constant(scalar),
            Expression::Selector(_selector) => unreachable!(),
            Expression::Fixed {
                query_index: _,
                column_index,
                rotation,
            } => {
                let rot_idx = self.add_rotation(rotation);
                self.add_calculation(Calculation::Store(ValueSource::Fixed(
                    *column_index,
                    rot_idx,
                )))
            }
            Expression::Advice {
                query_index: _,
                column_index,
                rotation,
            } => {
                let rot_idx = self.add_rotation(rotation);
                self.add_calculation(Calculation::Store(ValueSource::Advice(
                    *column_index,
                    rot_idx,
                )))
            }
            Expression::Instance {
                query_index: _,
                column_index,
                rotation,
            } => {
                let rot_idx = self.add_rotation(rotation);
                self.add_calculation(Calculation::Store(ValueSource::Instance(
                    *column_index,
                    rot_idx,
                )))
            }
            Expression::Negated(a) => match **a {
                Expression::Constant(scalar) => self.add_constant(&-scalar),
                _ => {
                    let result_a = self.add_expression(a);
                    match result_a {
                        ValueSource::Constant(0) => result_a,
                        _ => self.add_calculation(Calculation::Negate(result_a)),
                    }
                }
            },
            Expression::Sum(a, b) => {
                // Undo subtraction stored as a + (-b) in expressions
                match &**b {
                    Expression::Negated(b_int) => {
                        let result_a = self.add_expression(a);
                        let result_b = self.add_expression(b_int);
                        if result_a == ValueSource::Constant(0) {
                            result_b
                        } else if result_b == ValueSource::Constant(0) {
                            result_a
                        } else {
                            self.add_calculation(Calculation::Sub(result_a, result_b))
                        }
                    }
                    _ => {
                        let result_a = self.add_expression(a);
                        let result_b = self.add_expression(b);
                        if result_a == ValueSource::Constant(0) {
                            result_b
                        } else if result_b == ValueSource::Constant(0) {
                            result_a
                        } else if result_a <= result_b {
                            self.add_calculation(Calculation::Add(result_a, result_b))
                        } else {
                            self.add_calculation(Calculation::Add(result_b, result_a))
                        }
                    }
                }
            }
            Expression::Product(a, b) => {
                let result_a = self.add_expression(a);
                let result_b = self.add_expression(b);
                if result_a == ValueSource::Constant(0) || result_b == ValueSource::Constant(0) {
                    ValueSource::Constant(0)
                } else if result_a == ValueSource::Constant(1) {
                    result_b
                } else if result_b == ValueSource::Constant(1) {
                    result_a
                } else if result_a <= result_b {
                    self.add_calculation(Calculation::Mul(result_a, result_b))
                } else {
                    self.add_calculation(Calculation::Mul(result_b, result_a))
                }
            }
            Expression::Scaled(a, f) => {
                if *f == F::zero() {
                    ValueSource::Constant(0)
                } else if *f == F::one() {
                    self.add_expression(a)
                } else {
                    let cst = self.add_constant(f);
                    let result_a = self.add_expression(a);
                    self.add_calculation(Calculation::Mul(result_a, cst))
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ThreadBox<T>(*mut T, usize);
#[allow(unsafe_code)]
unsafe impl<T> Send for ThreadBox<T> {}
#[allow(unsafe_code)]
unsafe impl<T> Sync for ThreadBox<T> {}

/// Wraps a mutable slice so it can be passed into a thread without
/// hard to fix borrow checks caused by difficult data access patterns.
impl<T> ThreadBox<T> {
    fn wrap(data: &mut [T]) -> Self {
        Self(data.as_mut_ptr(), data.len())
    }

    fn unwrap(&mut self) -> &mut [T] {
        #[allow(unsafe_code)]
        unsafe {
            slice::from_raw_parts_mut(self.0, self.1)
        }
    }
}

/// Evaluate the polynomial
pub(in crate::plonk) fn evaluate_dynamic<C: CurveAffine, B: Basis>(
    size: usize,
    rot_scale: i32,
    fixed: &[Polynomial<C::ScalarExt, B>],
    advice: Vec<&Vec<Polynomial<C::ScalarExt, B>>>,
    instance: Vec<&Vec<Polynomial<C::ScalarExt, B>>>,
    y: C::ScalarExt,
    beta: C::ScalarExt,
    gamma: C::ScalarExt,
    theta: C::ScalarExt,
    pk: &ProvingKey<C>,
    lookups: &[Vec<lookup::prover::Committed<C>>],
    permutations: &[permutation::prover::Committed<C>],
    extended_omega: C::ScalarExt,
    ev: &EvaluationData<C::ScalarExt>,
) -> Vec<C::ScalarExt> {
    let num_lookups = pk.vk.cs.lookups.len();
    let isize = size as i32;
    let one = C::ScalarExt::one();
    let l0 = &pk.l0;
    let l_last = &pk.l_last;
    let l_active_row = &pk.l_active_row;
    let p = &pk.vk.cs.permutation;

    let mut values = vec![C::ScalarExt::zero(); size];
    let mut table_values = vec![C::Scalar::zero(); size * num_lookups];

    // Core evaluations
    let num_threads = multicore::current_num_threads();
    let mut table_values_box = ThreadBox::wrap(&mut table_values);
    for (((advice, instance), lookups), permutation) in advice
        .iter()
        .zip(instance.iter())
        .zip(lookups.iter())
        .zip(permutations.iter())
    {
        multicore::scope(|scope| {
            let chunk_size = (size + num_threads - 1) / num_threads;
            for (thread_idx, values) in values.chunks_mut(chunk_size).enumerate() {
                let start = thread_idx * chunk_size;
                scope.spawn(move |_| {
                    let table_values = table_values_box.unwrap();
                    let mut rotations = vec![0usize; ev.rotations.len()];
                    let mut intermediates: Vec<C::ScalarExt> =
                        vec![C::ScalarExt::zero(); ev.calculations.len()];
                    for (i, value) in values.iter_mut().enumerate() {
                        let idx = start + i;

                        // All rotation index values
                        for (rot_idx, rot) in ev.rotations.iter().enumerate() {
                            rotations[rot_idx] =
                                (((idx as i32) + (rot * rot_scale)).rem_euclid(isize)) as usize;
                        }

                        // All calculations
                        for (i_idx, calc) in ev.calculations.iter().enumerate() {
                            intermediates[i_idx] = calc.calculation.evaluate(
                                &rotations,
                                &ev.constants,
                                &intermediates,
                                fixed,
                                advice,
                                instance,
                                &beta,
                                &gamma,
                                &theta,
                            );
                        }

                        // Accumulate value parts
                        for value_part in ev.value_parts.iter() {
                            *value = *value * y
                                + value_part.evaluate(
                                    &rotations,
                                    &ev.constants,
                                    &intermediates,
                                    fixed,
                                    advice,
                                    instance,
                                );
                        }

                        // Values required for the lookups
                        for (t, table_result) in ev.lookup_results.iter().enumerate() {
                            table_values[t * size + idx] = table_result.evaluate(
                                &rotations,
                                &ev.constants,
                                &intermediates,
                                fixed,
                                advice,
                                instance,
                                &beta,
                                &gamma,
                                &theta,
                            );
                        }
                    }
                });
            }
        });

        // Permutations
        let sets = &permutation.sets;
        if !sets.is_empty() {
            let blinding_factors = pk.vk.cs.blinding_factors();
            let last_rotation = Rotation(-((blinding_factors + 1) as i32));
            let chunk_len = pk.vk.cs.degree() - 2;
            let delta_start = beta * &C::Scalar::ZETA;

            let first_set = sets.first().unwrap();
            let last_set = sets.last().unwrap();

            parallelize(&mut values, |values, start| {
                let mut beta_term = extended_omega.pow_vartime(&[start as u64, 0, 0, 0]);
                for (i, value) in values.iter_mut().enumerate() {
                    let idx = i + start;
                    let r_next = (((idx as i32) + rot_scale).rem_euclid(isize)) as usize;
                    let r_last =
                        (((idx as i32) + (last_rotation.0 * rot_scale)).rem_euclid(isize)) as usize;

                    // Permutation constraints
                    *value =
                        *value * y + ((one - first_set.permutation_product_coset[idx]) * l0[idx]);
                    *value = *value * y
                        + ((last_set.permutation_product_coset[idx]
                            * last_set.permutation_product_coset[idx]
                            - last_set.permutation_product_coset[idx])
                            * l_last[idx]);
                    for (set_idx, set) in sets.iter().enumerate() {
                        if set_idx != 0 {
                            *value = *value * y
                                + ((set.permutation_product_coset[idx]
                                    - permutation.sets[set_idx - 1].permutation_product_coset
                                        [r_last])
                                    * l0[idx]);
                        }
                    }

                    let mut current_delta = delta_start * beta_term;
                    for ((set, columns), cosets) in sets
                        .iter()
                        .zip(p.columns.chunks(chunk_len))
                        .zip(pk.permutation.cosets.chunks(chunk_len))
                    {
                        let mut left = set.permutation_product_coset[r_next];
                        for (values, permutation) in columns
                            .iter()
                            .map(|&column| match column.column_type() {
                                Any::Advice => &advice[column.index()],
                                Any::Fixed => &fixed[column.index()],
                                Any::Instance => &instance[column.index()],
                            })
                            .zip(cosets.iter())
                        {
                            left *= values[idx] + beta * permutation[idx] + gamma;
                        }

                        let mut right = set.permutation_product_coset[idx];
                        for values in columns.iter().map(|&column| match column.column_type() {
                            Any::Advice => &advice[column.index()],
                            Any::Fixed => &fixed[column.index()],
                            Any::Instance => &instance[column.index()],
                        }) {
                            right *= values[idx] + current_delta + gamma;
                            current_delta *= &C::Scalar::DELTA;
                        }

                        *value = *value * y + ((left - right) * l_active_row[idx]);
                    }
                    beta_term *= &extended_omega;
                }
            });
        }

        // Lookups
        for (n, lookup) in lookups.iter().enumerate() {
            // Polynomials required for this lookup
            // Calculated here so these only have to be kept in memory for the short time
            // they are actually needed
            let product_coset = pk.vk.domain.coeff_to_extended(lookup.product_poly.clone());
            let permuted_input_coset = pk
                .vk
                .domain
                .coeff_to_extended(lookup.permuted_input_poly.clone());
            let permuted_table_coset = pk
                .vk
                .domain
                .coeff_to_extended(lookup.permuted_table_poly.clone());

            // Lookup constraints
            let table = &table_values[n * size..(n + 1) * size];
            parallelize(&mut values, |values, start| {
                for (i, value) in values.iter_mut().enumerate() {
                    let idx = i + start;

                    let r_next = (((idx as i32) + rot_scale).rem_euclid(isize)) as usize;
                    let r_prev = (((idx as i32) - rot_scale).rem_euclid(isize)) as usize;

                    let a_minus_s = permuted_input_coset[idx] - permuted_table_coset[idx];
                    *value = *value * y + ((one - product_coset[idx]) * l0[idx]);
                    *value = *value * y
                        + ((product_coset[idx] * product_coset[idx] - product_coset[idx])
                            * l_last[idx]);
                    *value = *value * y
                        + ((product_coset[r_next]
                            * (permuted_input_coset[idx] + beta)
                            * (permuted_table_coset[idx] + gamma)
                            - product_coset[idx] * table[idx])
                            * l_active_row[idx]);
                    *value = *value * y + (a_minus_s * l0[idx]);
                    *value = *value * y
                        + (a_minus_s
                            * (permuted_input_coset[idx] - permuted_input_coset[r_prev])
                            * l_active_row[idx]);
                }
            });
        }
    }

    values
}

/// TODO: replace with optimized evaluation
pub fn evaluate<F: FieldExt, B: Basis>(
    expression: &Expression<F>,
    size: usize,
    rot_scale: i32,
    fixed: &[Polynomial<F, B>],
    advice: &[Polynomial<F, B>],
    instance: &[Polynomial<F, B>],
) -> Vec<F> {
    let mut values = vec![F::zero(); size];
    parallelize(&mut values, |values, start| {
        let isize = size as i32;
        for (i, value) in values.iter_mut().enumerate() {
            let idx = i + start;
            *value = expression.evaluate(
                &|scalar| scalar,
                &|_| panic!("virtual selectors are removed during optimization"),
                &|_, column_index, rotation| {
                    let new_rotation = rot_scale * rotation.0;
                    *fixed[column_index]
                        .index((((idx as i32) + new_rotation).rem_euclid(isize)) as usize)
                },
                &|_, column_index, rotation| {
                    let new_rotation = rot_scale * rotation.0;
                    *advice[column_index]
                        .index((((idx as i32) + new_rotation).rem_euclid(isize)) as usize)
                },
                &|_, column_index, rotation| {
                    let new_rotation = rot_scale * rotation.0;
                    *instance[column_index]
                        .index((((idx as i32) + new_rotation).rem_euclid(isize)) as usize)
                },
                &|a| -a,
                &|a, b| a + &b,
                &|a, b| a * b,
                &|a, scalar| a * scalar,
            );
        }
    });
    values
}
