//! `SymbolicVariable` copied from plonky3 and adapted for the Air to Plonkish usecase, at commit
//! `7b5b8a69f633bc61c530f3722701e5f701b11963`.

// The MIT License (MIT)
//
// Copyright (c) 2022 The Plonky3 Authors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, Mul, Sub};

use p3_field::Field;

use crate::symbolic_expression::SymbolicExpression;

/// A variable within the evaluation window, i.e. a column in either the local or next row.
#[derive(Copy, Clone, Debug)]
pub struct SymbolicVariable<F: Field>(pub Var, pub PhantomData<F>);

impl<F: Field> fmt::Display for SymbolicVariable<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Var::Query(q) => write!(f, "w{}{}", q.column, if q.is_next { "'" } else { "" }),
            Var::Public(p) => write!(f, "p{}", p.index),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Var {
    Query(Query),
    Public(Public),
}

#[derive(Copy, Clone, Debug)]
pub struct Query {
    pub is_next: bool,
    pub column: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Public {
    pub index: usize,
}

impl<F: Field> SymbolicVariable<F> {
    pub fn new_query(is_next: bool, column: usize) -> Self {
        Self(Var::Query(Query { is_next, column }), PhantomData)
    }
    pub fn new_public(index: usize) -> Self {
        Self(Var::Public(Public { index }), PhantomData)
    }
}

impl<F: Field> From<SymbolicVariable<F>> for SymbolicExpression<F> {
    fn from(value: SymbolicVariable<F>) -> Self {
        SymbolicExpression::Variable(value)
    }
}

impl<F: Field> Add for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn add(self, rhs: Self) -> Self::Output {
        SymbolicExpression::from(self) + SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Add<F> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn add(self, rhs: F) -> Self::Output {
        SymbolicExpression::from(self) + SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Add<SymbolicExpression<F>> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn add(self, rhs: SymbolicExpression<F>) -> Self::Output {
        SymbolicExpression::from(self) + rhs
    }
}

impl<F: Field> Add<SymbolicVariable<F>> for SymbolicExpression<F> {
    type Output = Self;

    fn add(self, rhs: SymbolicVariable<F>) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl<F: Field> Sub for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn sub(self, rhs: Self) -> Self::Output {
        SymbolicExpression::from(self) - SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Sub<F> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn sub(self, rhs: F) -> Self::Output {
        SymbolicExpression::from(self) - SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Sub<SymbolicExpression<F>> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn sub(self, rhs: SymbolicExpression<F>) -> Self::Output {
        SymbolicExpression::from(self) - rhs
    }
}

impl<F: Field> Sub<SymbolicVariable<F>> for SymbolicExpression<F> {
    type Output = Self;

    fn sub(self, rhs: SymbolicVariable<F>) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl<F: Field> Mul for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn mul(self, rhs: Self) -> Self::Output {
        SymbolicExpression::from(self) * SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Mul<F> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn mul(self, rhs: F) -> Self::Output {
        SymbolicExpression::from(self) * SymbolicExpression::from(rhs)
    }
}

impl<F: Field> Mul<SymbolicExpression<F>> for SymbolicVariable<F> {
    type Output = SymbolicExpression<F>;

    fn mul(self, rhs: SymbolicExpression<F>) -> Self::Output {
        SymbolicExpression::from(self) * rhs
    }
}

impl<F: Field> Mul<SymbolicVariable<F>> for SymbolicExpression<F> {
    type Output = Self;

    fn mul(self, rhs: SymbolicVariable<F>) -> Self::Output {
        self * Self::from(rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fwrap::FWrap;
    use halo2curves::bn256::Fr;
    use p3_field::AbstractField;

    type F = FWrap<Fr>;
    type V = SymbolicVariable<F>;
    type E = SymbolicExpression<F>;

    #[test]
    fn test_symbolic_variable() {
        assert_eq!(format!("{}", V::new_query(false, 1)), "w1");
        assert_eq!(format!("{}", V::new_query(true, 1)), "w1'");
        assert_eq!(format!("{}", V::new_public(1)), "p1");

        let w1 = V::new_query(false, 1);
        let w2 = V::new_query(false, 2);
        let f = F::two();
        assert_eq!(format!("{}", E::from(w1)), "w1");

        // Arithmetic operators

        assert_eq!(format!("{}", w1 + w2), "(w1 + w2)");
        assert_eq!(format!("{}", w1 + E::from(w2)), "(w1 + w2)");
        assert_eq!(format!("{}", E::from(w1) + w2), "(w1 + w2)");
        assert_eq!(format!("{}", w1 + f), "(w1 + 2)");

        assert_eq!(format!("{}", w1 - w2), "(w1 - w2)");
        assert_eq!(format!("{}", w1 - E::from(w2)), "(w1 - w2)");
        assert_eq!(format!("{}", E::from(w1) - w2), "(w1 - w2)");
        assert_eq!(format!("{}", w1 - f), "(w1 - 2)");

        assert_eq!(format!("{}", w1 * w2), "w1 * w2");
        assert_eq!(format!("{}", w1 * E::from(w2)), "w1 * w2");
        assert_eq!(format!("{}", E::from(w1) * w2), "w1 * w2");
        assert_eq!(format!("{}", w1 * f), "w1 * 2");
    }
}
