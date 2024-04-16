use core::cmp::max;
use core::ops::{Add, Mul, Neg, Sub};
use ff::Field;
use std::iter::{Product, Sum};

pub trait Variable: Clone + Copy + std::fmt::Debug + Eq + PartialEq {
    /// Degree that an expression would have if it was only this variable.
    fn degree(&self) -> usize;

    /// Approximate the computational complexity an expression would have if it was only this
    /// variable.
    fn complexity(&self) -> usize {
        0
    }

    /// Write an identifier of the variable.  If two variables have the same identifier, they must
    /// be the same variable.
    fn write_identifier<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

/// Low-degree expression representing an identity that must hold over the committed columns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression<F, V: Variable> {
    /// This is a constant polynomial
    Constant(F),
    /// This is a variable
    Var(V),
    /// This is a negated polynomial
    Negated(Box<Expression<F, V>>),
    /// This is the sum of two polynomials
    Sum(Box<Expression<F, V>>, Box<Expression<F, V>>),
    /// This is the product of two polynomials
    Product(Box<Expression<F, V>>, Box<Expression<F, V>>),
}

impl<F: Field, V: Variable> Expression<F, V> {
    /// Evaluate the polynomial using the provided closures to perform the
    /// operations.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<T>(
        &self,
        constant: &impl Fn(F) -> T,
        var: &impl Fn(V) -> T,
        negated: &impl Fn(T) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
    ) -> T {
        match self {
            Expression::Constant(scalar) => constant(*scalar),
            Expression::Var(v) => var(*v),
            Expression::Negated(a) => {
                let a = a.evaluate(constant, var, negated, sum, product);
                negated(a)
            }
            Expression::Sum(a, b) => {
                let a = a.evaluate(constant, var, negated, sum, product);
                let b = b.evaluate(constant, var, negated, sum, product);
                sum(a, b)
            }
            Expression::Product(a, b) => {
                let a = a.evaluate(constant, var, negated, sum, product);
                let b = b.evaluate(constant, var, negated, sum, product);
                product(a, b)
            }
        }
    }

    pub fn write_identifier<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Expression::Constant(scalar) => write!(writer, "{scalar:?}"),
            Expression::Var(v) => v.write_identifier(writer),
            Expression::Negated(a) => {
                writer.write_all(b"(-")?;
                a.write_identifier(writer)?;
                writer.write_all(b")")
            }
            Expression::Sum(a, b) => {
                writer.write_all(b"(")?;
                a.write_identifier(writer)?;
                writer.write_all(b"+")?;
                b.write_identifier(writer)?;
                writer.write_all(b")")
            }
            Expression::Product(a, b) => {
                writer.write_all(b"(")?;
                a.write_identifier(writer)?;
                writer.write_all(b"*")?;
                b.write_identifier(writer)?;
                writer.write_all(b")")
            }
        }
    }

    /// Identifier for this expression. Expressions with identical identifiers
    /// do the same calculation (but the expressions don't need to be exactly equal
    /// in how they are composed e.g. `1 + 2` and `2 + 1` can have the same identifier).
    pub fn identifier(&self) -> String {
        let mut cursor = std::io::Cursor::new(Vec::new());
        self.write_identifier(&mut cursor).unwrap();
        String::from_utf8(cursor.into_inner()).unwrap()
    }

    /// Compute the degree of this polynomial
    pub fn degree(&self) -> usize {
        use Expression::*;
        match self {
            Constant(_) => 0,
            Var(v) => v.degree(),
            Negated(poly) => poly.degree(),
            Sum(a, b) => max(a.degree(), b.degree()),
            Product(a, b) => a.degree() + b.degree(),
        }
    }

    /// Approximate the computational complexity of this expression.
    pub fn complexity(&self) -> usize {
        match self {
            Expression::Constant(_) => 0,
            Expression::Var(v) => v.complexity(),
            Expression::Negated(poly) => poly.complexity() + 5,
            Expression::Sum(a, b) => a.complexity() + b.complexity() + 15,
            Expression::Product(a, b) => a.complexity() + b.complexity() + 30,
        }
    }
}

impl<F: Field, V: Variable> Neg for Expression<F, V> {
    type Output = Expression<F, V>;
    fn neg(self) -> Self::Output {
        Expression::Negated(Box::new(self))
    }
}

impl<F: Field, V: Variable> Add for Expression<F, V> {
    type Output = Expression<F, V>;
    fn add(self, rhs: Expression<F, V>) -> Expression<F, V> {
        Expression::Sum(Box::new(self), Box::new(rhs))
    }
}

impl<F: Field, V: Variable> Sub for Expression<F, V> {
    type Output = Expression<F, V>;
    fn sub(self, rhs: Expression<F, V>) -> Expression<F, V> {
        Expression::Sum(Box::new(self), Box::new(-rhs))
    }
}

impl<F: Field, V: Variable> Mul for Expression<F, V> {
    type Output = Expression<F, V>;
    fn mul(self, rhs: Expression<F, V>) -> Expression<F, V> {
        Expression::Product(Box::new(self), Box::new(rhs))
    }
}

impl<F: Field, V: Variable> Mul<F> for Expression<F, V> {
    type Output = Expression<F, V>;
    fn mul(self, rhs: F) -> Expression<F, V> {
        Expression::Product(Box::new(self), Box::new(Expression::Constant(rhs)))
    }
}

impl<F: Field, V: Variable> Sum<Self> for Expression<F, V> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x)
            .unwrap_or(Expression::Constant(F::ZERO))
    }
}

impl<F: Field, V: Variable> Product<Self> for Expression<F, V> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc * x)
            .unwrap_or(Expression::Constant(F::ONE))
    }
}
