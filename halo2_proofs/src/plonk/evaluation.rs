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

/// Return the index in the polynomial of size `isize` after rotation `rot`.
fn get_rotation_idx(idx: usize, rot: i32, rot_scale: i32, isize: i32) -> usize {
    (((idx as i32) + (rot * rot_scale)).rem_euclid(isize)) as usize
}

/// Value used in a calculation
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
    pub fn get<F: Field, B: Basis>(
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
                fixed_values[*column_index][rotations[*rotation]]
            }
            ValueSource::Advice(column_index, rotation) => {
                advice_values[*column_index][rotations[*rotation]]
            }
            ValueSource::Instance(column_index, rotation) => {
                instance_values[*column_index][rotations[*rotation]]
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
                let a = a.get(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.get(
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
                let a = a.get(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.get(
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
                let a = a.get(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.get(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                a * b
            }
            Calculation::Negate(v) => -v.get(
                rotations,
                constants,
                intermediates,
                fixed_values,
                advice_values,
                instance_values,
            ),
            Calculation::LcBeta(a, b) => {
                let a = a.get(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.get(
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
                let a = a.get(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                );
                let b = b.get(
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
                v.get(
                    rotations,
                    constants,
                    intermediates,
                    fixed_values,
                    advice_values,
                    instance_values,
                ) + gamma
            }
            Calculation::Store(v) => v.get(
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
pub struct Evaluator<C: CurveAffine> {
    /// Constants
    pub constants: Vec<C::ScalarExt>,
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

impl<C: CurveAffine> Evaluator<C> {
    /// Creates a new evaluation structure
    pub fn new(cs: &ConstraintSystem<C::ScalarExt>) -> Self {
        let mut ev = Evaluator::default();
        ev.add_constant(&C::ScalarExt::zero());
        ev.add_constant(&C::ScalarExt::one());

        // Custom gates
        for gate in cs.gates.iter() {
            for poly in gate.polynomials().iter() {
                let vs = ev.add_expression(poly);
                ev.value_parts.push(vs);
            }
        }

        // Lookups
        for lookup in cs.lookups.iter() {
            let evaluate_lc = |ev: &mut Evaluator<_>, expressions: &Vec<Expression<_>>| {
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
            let compressed_input_coset = evaluate_lc(&mut ev, &lookup.input_expressions);
            // table coset
            let compressed_table_coset = evaluate_lc(&mut ev, &lookup.table_expressions);
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
    fn add_constant(&mut self, constant: &C::ScalarExt) -> ValueSource {
        let position = self.constants.iter().position(|&c| c == *constant);
        ValueSource::Constant(match position {
            Some(pos) => pos,
            None => {
                self.constants.push(*constant);
                self.constants.len() - 1
            }
        })
    }

    /// Adds a calculation.
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

    /// Generates an optimized evaluation for the expression
    fn add_expression(&mut self, expr: &Expression<C::ScalarExt>) -> ValueSource {
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
                if *f == C::ScalarExt::zero() {
                    ValueSource::Constant(0)
                } else if *f == C::ScalarExt::one() {
                    self.add_expression(a)
                } else {
                    let cst = self.add_constant(f);
                    let result_a = self.add_expression(a);
                    self.add_calculation(Calculation::Mul(result_a, cst))
                }
            }
        }
    }

    /// Evaluate h poly
    pub(in crate::plonk) fn evaluate_h(
        &self,
        pk: &ProvingKey<C>,
        advice: Vec<&Vec<Polynomial<C::ScalarExt, ExtendedLagrangeCoeff>>>,
        instance: Vec<&Vec<Polynomial<C::ScalarExt, ExtendedLagrangeCoeff>>>,
        y: C::ScalarExt,
        beta: C::ScalarExt,
        gamma: C::ScalarExt,
        theta: C::ScalarExt,
        lookups: &[Vec<lookup::prover::Committed<C>>],
        permutations: &[permutation::prover::Committed<C>],
    ) -> Polynomial<C::ScalarExt, ExtendedLagrangeCoeff> {
        let domain = &pk.vk.domain;
        let size = domain.extended_len();
        let rot_scale = 1 << (domain.extended_k() - domain.k());
        let fixed = &pk.fixed_cosets[..];
        let extended_omega = domain.get_extended_omega();
        let num_lookups = pk.vk.cs.lookups.len();
        let isize = size as i32;
        let one = C::ScalarExt::one();
        let l0 = &pk.l0;
        let l_last = &pk.l_last;
        let l_active_row = &pk.l_active_row;
        let p = &pk.vk.cs.permutation;

        let mut values = domain.empty_extended();
        let mut lookup_values = vec![C::Scalar::zero(); size * num_lookups];

        // Core expression evaluations
        let num_threads = multicore::current_num_threads();
        let mut table_values_box = ThreadBox::wrap(&mut lookup_values);
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
                        let mut rotations = vec![0usize; self.rotations.len()];
                        let mut intermediates: Vec<C::ScalarExt> =
                            vec![C::ScalarExt::zero(); self.calculations.len()];
                        for (i, value) in values.iter_mut().enumerate() {
                            let idx = start + i;

                            // All rotation index values
                            for (rot_idx, rot) in self.rotations.iter().enumerate() {
                                rotations[rot_idx] = get_rotation_idx(idx, *rot, rot_scale, isize);
                            }

                            // All calculations, with cached intermediate results
                            for (i_idx, calc) in self.calculations.iter().enumerate() {
                                intermediates[i_idx] = calc.calculation.evaluate(
                                    &rotations,
                                    &self.constants,
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
                            for value_part in self.value_parts.iter() {
                                *value = *value * y
                                    + value_part.get(
                                        &rotations,
                                        &self.constants,
                                        &intermediates,
                                        fixed,
                                        advice,
                                        instance,
                                    );
                            }

                            // Values required for the lookups
                            for (t, table_result) in self.lookup_results.iter().enumerate() {
                                table_values[t * size + idx] = table_result.evaluate(
                                    &rotations,
                                    &self.constants,
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

                // Permutation constraints
                parallelize(&mut values, |values, start| {
                    let mut beta_term = extended_omega.pow_vartime(&[start as u64, 0, 0, 0]);
                    for (i, value) in values.iter_mut().enumerate() {
                        let idx = start + i;
                        let r_next = get_rotation_idx(idx, 1, rot_scale, isize);
                        let r_last = get_rotation_idx(idx, last_rotation.0, rot_scale, isize);

                        // Enforce only for the first set.
                        // l_0(X) * (1 - z_0(X)) = 0
                        *value = *value * y
                            + ((one - first_set.permutation_product_coset[idx]) * l0[idx]);
                        // Enforce only for the last set.
                        // l_last(X) * (z_l(X)^2 - z_l(X)) = 0
                        *value = *value * y
                            + ((last_set.permutation_product_coset[idx]
                                * last_set.permutation_product_coset[idx]
                                - last_set.permutation_product_coset[idx])
                                * l_last[idx]);
                        // Except for the first set, enforce.
                        // l_0(X) * (z_i(X) - z_{i-1}(\omega^(last) X)) = 0
                        for (set_idx, set) in sets.iter().enumerate() {
                            if set_idx != 0 {
                                *value = *value * y
                                    + ((set.permutation_product_coset[idx]
                                        - permutation.sets[set_idx - 1].permutation_product_coset
                                            [r_last])
                                        * l0[idx]);
                            }
                        }
                        // And for all the sets we enforce:
                        // (1 - (l_last(X) + l_blind(X))) * (
                        //   z_i(\omega X) \prod_j (p(X) + \beta s_j(X) + \gamma)
                        // - z_i(X) \prod_j (p(X) + \delta^j \beta X + \gamma)
                        // )
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
                // Polynomials required for this lookup.
                // Calculated here so these only have to be kept in memory for the short time
                // they are actually needed.
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
                let table = &lookup_values[n * size..(n + 1) * size];
                parallelize(&mut values, |values, start| {
                    for (i, value) in values.iter_mut().enumerate() {
                        let idx = start + i;

                        let r_next = get_rotation_idx(idx, 1, rot_scale, isize);
                        let r_prev = get_rotation_idx(idx, -1, rot_scale, isize);

                        let a_minus_s = permuted_input_coset[idx] - permuted_table_coset[idx];
                        // l_0(X) * (1 - z(X)) = 0
                        *value = *value * y + ((one - product_coset[idx]) * l0[idx]);
                        // l_last(X) * (z(X)^2 - z(X)) = 0
                        *value = *value * y
                            + ((product_coset[idx] * product_coset[idx] - product_coset[idx])
                                * l_last[idx]);
                        // (1 - (l_last(X) + l_blind(X))) * (
                        //   z(\omega X) (a'(X) + \beta) (s'(X) + \gamma)
                        //   - z(X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta)
                        //          (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
                        // ) = 0
                        *value = *value * y
                            + ((product_coset[r_next]
                                * (permuted_input_coset[idx] + beta)
                                * (permuted_table_coset[idx] + gamma)
                                - product_coset[idx] * table[idx])
                                * l_active_row[idx]);
                        // Check that the first values in the permuted input expression and permuted
                        // fixed expression are the same.
                        // l_0(X) * (a'(X) - s'(X)) = 0
                        *value = *value * y + (a_minus_s * l0[idx]);
                        // Check that each value in the permuted lookup input expression is either
                        // equal to the value above it, or the value at the same index in the
                        // permuted table expression.
                        // (1 - (l_last + l_blind)) * (a′(X) − s′(X))⋅(a′(X) − a′(\omega^{-1} X)) = 0
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

/// Simple evaluation of an expression
pub fn evaluate<F: FieldExt, B: Basis>(
    expression: &Expression<F>,
    size: usize,
    rot_scale: i32,
    fixed: &[Polynomial<F, B>],
    advice: &[Polynomial<F, B>],
    instance: &[Polynomial<F, B>],
) -> Vec<F> {
    let mut values = vec![F::zero(); size];
    let isize = size as i32;
    parallelize(&mut values, |values, start| {
        for (i, value) in values.iter_mut().enumerate() {
            let idx = start + i;
            *value = expression.evaluate(
                &|scalar| scalar,
                &|_| panic!("virtual selectors are removed during optimization"),
                &|_, column_index, rotation| {
                    fixed[column_index][get_rotation_idx(idx, rotation.0, rot_scale, isize)]
                },
                &|_, column_index, rotation| {
                    advice[column_index][get_rotation_idx(idx, rotation.0, rot_scale, isize)]
                },
                &|_, column_index, rotation| {
                    instance[column_index][get_rotation_idx(idx, rotation.0, rot_scale, isize)]
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
