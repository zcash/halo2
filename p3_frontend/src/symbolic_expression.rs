//! `SymbolicExpression` copied from plonky3 and adapted for the Air to Plonkish usecase, at commit
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

use alloc::rc::Rc;
use core::fmt::{self, Debug};
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use p3_field::{AbstractField, Field};

use crate::symbolic_variable::SymbolicVariable;

#[derive(Clone, Copy, Debug)]
pub enum Location {
    FirstRow,
    LastRow,
    Transition,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FirstRow => write!(f, "fst"),
            Self::LastRow => write!(f, "lst"),
            Self::Transition => write!(f, "trn"),
        }
    }
}

/// An expression over `SymbolicVariable`s.
#[derive(Clone, Debug)]
pub enum SymbolicExpression<F: Field> {
    Variable(SymbolicVariable<F>),
    Location(Location),
    Constant(F),
    Add(Rc<Self>, Rc<Self>),
    Sub(Rc<Self>, Rc<Self>),
    Neg(Rc<Self>),
    Mul(Rc<Self>, Rc<Self>),
}

impl<F: Field> SymbolicExpression<F> {
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Constant(c) => c.is_zero(),
            _ => false,
        }
    }
    pub fn is_one(&self) -> bool {
        match self {
            Self::Constant(c) => c.is_one(),
            _ => false,
        }
    }
}

impl<F: Field> fmt::Display for SymbolicExpression<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(var) => write!(f, "{}", var),
            Self::Location(loc) => write!(f, "{}", loc),
            Self::Constant(c) => {
                if *c == F::zero() {
                    write!(f, "0")
                } else if *c == F::one() {
                    write!(f, "1")
                } else if *c == F::two() {
                    write!(f, "2")
                } else {
                    write!(f, "{}", c)
                }
            }
            Self::Add(lhs, rhs) => {
                if let Self::Neg(neg_rhs) = &**rhs {
                    write!(f, "({} - {})", lhs, neg_rhs)
                } else {
                    write!(f, "({} + {})", lhs, rhs)
                }
            }
            Self::Sub(lhs, rhs) => {
                write!(f, "({} - {})", lhs, rhs)
            }
            Self::Neg(neg) => write!(f, "-({})", neg),
            Self::Mul(lhs, rhs) => write!(f, "{} * {}", lhs, rhs),
        }
    }
}

impl<F: Field> Default for SymbolicExpression<F> {
    fn default() -> Self {
        Self::Constant(F::zero())
    }
}

impl<F: Field> From<F> for SymbolicExpression<F> {
    fn from(value: F) -> Self {
        Self::Constant(value)
    }
}

impl<F: Field> AbstractField for SymbolicExpression<F> {
    type F = F;

    fn zero() -> Self {
        Self::Constant(F::zero())
    }
    fn one() -> Self {
        Self::Constant(F::one())
    }
    fn two() -> Self {
        Self::Constant(F::two())
    }
    fn neg_one() -> Self {
        Self::Constant(F::neg_one())
    }

    #[inline]
    fn from_f(f: Self::F) -> Self {
        f.into()
    }

    fn from_bool(b: bool) -> Self {
        Self::Constant(F::from_bool(b))
    }

    fn from_canonical_u8(n: u8) -> Self {
        Self::Constant(F::from_canonical_u8(n))
    }

    fn from_canonical_u16(n: u16) -> Self {
        Self::Constant(F::from_canonical_u16(n))
    }

    fn from_canonical_u32(n: u32) -> Self {
        Self::Constant(F::from_canonical_u32(n))
    }

    fn from_canonical_u64(n: u64) -> Self {
        Self::Constant(F::from_canonical_u64(n))
    }

    fn from_canonical_usize(n: usize) -> Self {
        Self::Constant(F::from_canonical_usize(n))
    }

    fn from_wrapped_u32(n: u32) -> Self {
        Self::Constant(F::from_wrapped_u32(n))
    }

    fn from_wrapped_u64(n: u64) -> Self {
        Self::Constant(F::from_wrapped_u64(n))
    }

    fn generator() -> Self {
        Self::Constant(F::generator())
    }

    fn double(&self) -> Self {
        self.clone() * F::from_canonical_u64(2)
    }
}

impl<F: Field> Add for SymbolicExpression<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            self
        } else if self.is_zero() {
            rhs
        } else {
            Self::Add(Rc::new(self), Rc::new(rhs))
        }
    }
}

impl<F: Field> Add<F> for SymbolicExpression<F> {
    type Output = Self;

    fn add(self, rhs: F) -> Self {
        if rhs.is_zero() {
            self
        } else if self.is_zero() {
            Self::Constant(rhs)
        } else {
            self + Self::from(rhs)
        }
    }
}

impl<F: Field> AddAssign for SymbolicExpression<F> {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl<F: Field> AddAssign<F> for SymbolicExpression<F> {
    fn add_assign(&mut self, rhs: F) {
        *self += Self::from(rhs);
    }
}

impl<F: Field> Sum for SymbolicExpression<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|x, y| x + y).unwrap_or(Self::zero())
    }
}

impl<F: Field> Sum<F> for SymbolicExpression<F> {
    fn sum<I: Iterator<Item = F>>(iter: I) -> Self {
        iter.map(|x| Self::from(x)).sum()
    }
}

impl<F: Field> Sub for SymbolicExpression<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::Sub(Rc::new(self), Rc::new(rhs))
    }
}

impl<F: Field> Sub<F> for SymbolicExpression<F> {
    type Output = Self;

    fn sub(self, rhs: F) -> Self {
        self - Self::from(rhs)
    }
}

impl<F: Field> SubAssign for SymbolicExpression<F> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
    }
}

impl<F: Field> SubAssign<F> for SymbolicExpression<F> {
    fn sub_assign(&mut self, rhs: F) {
        *self -= Self::from(rhs);
    }
}

impl<F: Field> Neg for SymbolicExpression<F> {
    type Output = Self;

    fn neg(self) -> Self {
        Self::Neg(Rc::new(self))
    }
}

impl<F: Field> Mul for SymbolicExpression<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if rhs.is_zero() || self.is_zero() {
            Self::Constant(F::zero())
        } else if rhs.is_one() {
            self
        } else {
            Self::Mul(Rc::new(self), Rc::new(rhs))
        }
    }
}

impl<F: Field> Mul<F> for SymbolicExpression<F> {
    type Output = Self;

    fn mul(self, rhs: F) -> Self {
        if rhs.is_zero() || self.is_zero() {
            Self::Constant(F::zero())
        } else if rhs.is_one() {
            self
        } else {
            self * Self::from(rhs)
        }
    }
}

impl<F: Field> MulAssign for SymbolicExpression<F> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}

impl<F: Field> MulAssign<F> for SymbolicExpression<F> {
    fn mul_assign(&mut self, rhs: F) {
        *self *= Self::from(rhs);
    }
}

impl<F: Field> Product for SymbolicExpression<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|x, y| x * y).unwrap_or(Self::one())
    }
}

impl<F: Field> Product<F> for SymbolicExpression<F> {
    fn product<I: Iterator<Item = F>>(iter: I) -> Self {
        iter.map(|x| Self::from(x)).product()
    }
}

#[allow(clippy::bool_assert_comparison)]
#[cfg(test)]
mod test {
    use super::*;
    use crate::fwrap::FWrap;
    use halo2curves::bn256::Fr;

    type F = FWrap<Fr>;
    type V = SymbolicVariable<F>;
    type E = SymbolicExpression<F>;

    #[test]
    fn test_symbolic_expression() {
        assert_eq!(E::from(F::zero()).is_zero(), true);
        assert_eq!(E::from(F::one()).is_zero(), false);
        assert_eq!(E::from(F::one()).is_one(), true);
        assert_eq!(E::from(F::zero()).is_one(), false);

        assert_eq!(format!("{}", E::default()), "0");

        // AbstractField

        assert_eq!(format!("{}", E::zero()), "0");
        assert_eq!(format!("{}", E::one()), "1");
        assert_eq!(format!("{}", E::two()), "2");
        assert_eq!(
            format!("{}", E::neg_one()),
            "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000"
        );
        assert_eq!(format!("{}", E::from_f(F::two())), "2");
        assert_eq!(format!("{}", E::from_bool(true)), "1");
        assert_eq!(
            format!("{}", E::from_canonical_u8(0x12)),
            "0x0000000000000000000000000000000000000000000000000000000000000012"
        );
        assert_eq!(
            format!("{}", E::from_canonical_u16(0x1234)),
            "0x0000000000000000000000000000000000000000000000000000000000001234"
        );
        assert_eq!(
            format!("{}", E::from_canonical_u32(0x123456)),
            "0x0000000000000000000000000000000000000000000000000000000000123456"
        );
        assert_eq!(
            format!("{}", E::from_canonical_u64(0xfffffff12)),
            "0x0000000000000000000000000000000000000000000000000000000fffffff12"
        );
        assert_eq!(
            format!("{}", E::from_canonical_usize(0xfffffff12)),
            "0x0000000000000000000000000000000000000000000000000000000fffffff12"
        );
        assert_eq!(
            format!("{}", E::from_wrapped_u32(0x123456)),
            "0x0000000000000000000000000000000000000000000000000000000000123456"
        );
        assert_eq!(
            format!("{}", E::from_wrapped_u64(0xfffffff12)),
            "0x0000000000000000000000000000000000000000000000000000000fffffff12"
        );
        assert_eq!(
            format!("{}", E::generator()),
            "0x0000000000000000000000000000000000000000000000000000000000000007"
        );
        assert_eq!(format!("{}", E::two().double()), "2 * 2");

        // Arithmetic operators

        let w1 = E::from(V::new_query(false, 1));
        let w2 = E::from(V::new_query(false, 2));
        let f = F::two();
        assert_eq!(format!("{}", w1.clone() + w2.clone()), "(w1 + w2)");
        assert_eq!(format!("{}", w1.clone() + f), "(w1 + 2)");
        let mut v = w1.clone();
        v += w2.clone();
        v += f;
        assert_eq!(format!("{}", v), "((w1 + w2) + 2)");
        assert_eq!(
            format!("{}", [w1.clone(), w2.clone()].into_iter().sum::<E>()),
            "(w1 + w2)"
        );
        assert_eq!(format!("{}", [f, f].into_iter().sum::<E>()), "(2 + 2)");

        assert_eq!(format!("{}", w1.clone() - w2.clone()), "(w1 - w2)");
        assert_eq!(format!("{}", w1.clone() - f), "(w1 - 2)");
        let mut v = w1.clone();
        v -= w2.clone();
        v -= f;
        assert_eq!(format!("{}", v), "((w1 - w2) - 2)");
        assert_eq!(format!("{}", -w1.clone()), "-(w1)");

        assert_eq!(format!("{}", w1.clone() * w2.clone()), "w1 * w2");
        assert_eq!(format!("{}", w1.clone() * f), "w1 * 2");
        let mut v = w1.clone();
        v *= w2.clone();
        v *= f;
        assert_eq!(format!("{}", v), "w1 * w2 * 2");
        assert_eq!(
            format!("{}", [w1.clone(), w2.clone()].into_iter().product::<E>()),
            "w1 * w2"
        );
        assert_eq!(format!("{}", [f, f].into_iter().product::<E>()), "2 * 2");
    }
}
