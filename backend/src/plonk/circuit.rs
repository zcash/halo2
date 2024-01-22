use super::{lookup, permutation, shuffle, Queries};
// use crate::dev::metadata;
use core::cmp::max;
use core::ops::{Add, Mul};
use halo2_common::plonk::{ConstraintSystem, Expression};
use halo2_middleware::circuit::{
    Advice, AdviceQueryMid, Any, Challenge, Column, ConstraintSystemV2Backend, ExpressionMid,
    Fixed, FixedQueryMid, GateV2Backend, Instance, InstanceQueryMid,
};
use halo2_middleware::ff::Field;
use halo2_middleware::metadata;
use halo2_middleware::poly::Rotation;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::{Product, Sum};
use std::{
    convert::TryFrom,
    ops::{Neg, Sub},
};

/// Represents an index into a vector where each entry corresponds to a distinct
/// point that polynomials are queried at.
#[derive(Copy, Clone, Debug)]
pub(crate) struct PointIndex(pub usize);

/// An individual polynomial constraint.
///
/// These are returned by the closures passed to `ConstraintSystem::create_gate`.
#[derive(Debug)]
pub struct Constraint<F: Field> {
    name: String,
    poly: Expression<F>,
}

impl<F: Field> From<Expression<F>> for Constraint<F> {
    fn from(poly: Expression<F>) -> Self {
        Constraint {
            name: "".to_string(),
            poly,
        }
    }
}

impl<F: Field, S: AsRef<str>> From<(S, Expression<F>)> for Constraint<F> {
    fn from((name, poly): (S, Expression<F>)) -> Self {
        Constraint {
            name: name.as_ref().to_string(),
            poly,
        }
    }
}

impl<F: Field> From<Expression<F>> for Vec<Constraint<F>> {
    fn from(poly: Expression<F>) -> Self {
        vec![Constraint {
            name: "".to_string(),
            poly,
        }]
    }
}

/// A set of polynomial constraints with a common selector.
///
/// ```
/// use halo2_backend::{plonk::{Constraints, Expression}, poly::Rotation};
/// use halo2curves::pasta::Fp;
/// # use halo2_backend::plonk::ConstraintSystem;
///
/// # let mut meta = ConstraintSystem::<Fp>::default();
/// let a = meta.advice_column();
/// let b = meta.advice_column();
/// let c = meta.advice_column();
/// let s = meta.selector();
///
/// meta.create_gate("foo", |meta| {
///     let next = meta.query_advice(a, Rotation::next());
///     let a = meta.query_advice(a, Rotation::cur());
///     let b = meta.query_advice(b, Rotation::cur());
///     let c = meta.query_advice(c, Rotation::cur());
///     let s_ternary = meta.query_selector(s);
///
///     let one_minus_a = Expression::Constant(Fp::one()) - a.clone();
///
///     Constraints::with_selector(
///         s_ternary,
///         std::array::IntoIter::new([
///             ("a is boolean", a.clone() * one_minus_a.clone()),
///             ("next == a ? b : c", next - (a * b + one_minus_a * c)),
///         ]),
///     )
/// });
/// ```
///
/// Note that the use of `std::array::IntoIter::new` is only necessary if you need to
/// support Rust 1.51 or 1.52. If your minimum supported Rust version is 1.53 or greater,
/// you can pass an array directly.
#[derive(Debug)]
pub struct Constraints<F: Field, C: Into<Constraint<F>>, Iter: IntoIterator<Item = C>> {
    selector: Expression<F>,
    constraints: Iter,
}

impl<F: Field, C: Into<Constraint<F>>, Iter: IntoIterator<Item = C>> Constraints<F, C, Iter> {
    /// Constructs a set of constraints that are controlled by the given selector.
    ///
    /// Each constraint `c` in `iterator` will be converted into the constraint
    /// `selector * c`.
    pub fn with_selector(selector: Expression<F>, constraints: Iter) -> Self {
        Constraints {
            selector,
            constraints,
        }
    }
}

fn apply_selector_to_constraint<F: Field, C: Into<Constraint<F>>>(
    (selector, c): (Expression<F>, C),
) -> Constraint<F> {
    let constraint: Constraint<F> = c.into();
    Constraint {
        name: constraint.name,
        poly: selector * constraint.poly,
    }
}

type ApplySelectorToConstraint<F, C> = fn((Expression<F>, C)) -> Constraint<F>;
type ConstraintsIterator<F, C, I> = std::iter::Map<
    std::iter::Zip<std::iter::Repeat<Expression<F>>, I>,
    ApplySelectorToConstraint<F, C>,
>;

impl<F: Field, C: Into<Constraint<F>>, Iter: IntoIterator<Item = C>> IntoIterator
    for Constraints<F, C, Iter>
{
    type Item = Constraint<F>;
    type IntoIter = ConstraintsIterator<F, C, Iter::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::repeat(self.selector)
            .zip(self.constraints)
            .map(apply_selector_to_constraint)
    }
}

#[cfg(test)]
mod tests {
    use super::Expression;
    use halo2curves::bn256::Fr;

    #[test]
    fn iter_sum() {
        let exprs: Vec<Expression<Fr>> = vec![
            Expression::Constant(1.into()),
            Expression::Constant(2.into()),
            Expression::Constant(3.into()),
        ];
        let happened: Expression<Fr> = exprs.into_iter().sum();
        let expected: Expression<Fr> = Expression::Sum(
            Box::new(Expression::Sum(
                Box::new(Expression::Constant(1.into())),
                Box::new(Expression::Constant(2.into())),
            )),
            Box::new(Expression::Constant(3.into())),
        );

        assert_eq!(happened, expected);
    }

    #[test]
    fn iter_product() {
        let exprs: Vec<Expression<Fr>> = vec![
            Expression::Constant(1.into()),
            Expression::Constant(2.into()),
            Expression::Constant(3.into()),
        ];
        let happened: Expression<Fr> = exprs.into_iter().product();
        let expected: Expression<Fr> = Expression::Product(
            Box::new(Expression::Product(
                Box::new(Expression::Constant(1.into())),
                Box::new(Expression::Constant(2.into())),
            )),
            Box::new(Expression::Constant(3.into())),
        );

        assert_eq!(happened, expected);
    }
}
