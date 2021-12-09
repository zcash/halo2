use std::ops::{Add, Mul, Neg, Sub};

use group::ff::Field;

/// A value assigned to a cell within a circuit.
///
/// Stored as a fraction, so the backend can use batch inversion.
///
/// A denominator of zero maps to an assigned value of zero.
#[derive(Clone, Copy, Debug)]
pub enum Assigned<F> {
    /// The field element zero.
    Zero,
    /// A value that does not require inversion to evaluate.
    Trivial(F),
    /// A value stored as a fraction to enable batch inversion.
    Rational(F, F),
}

impl<F: Field> From<&F> for Assigned<F> {
    fn from(numerator: &F) -> Self {
        Assigned::Trivial(*numerator)
    }
}

impl<F: Field> From<F> for Assigned<F> {
    fn from(numerator: F) -> Self {
        Assigned::Trivial(numerator)
    }
}

impl<F: Field> From<(F, F)> for Assigned<F> {
    fn from((numerator, denominator): (F, F)) -> Self {
        Assigned::Rational(numerator, denominator)
    }
}

impl<F: Field> Neg for Assigned<F> {
    type Output = Assigned<F>;
    fn neg(self) -> Self::Output {
        match self {
            Self::Zero => Self::Zero,
            Self::Trivial(numerator) => Self::Trivial(-numerator),
            Self::Rational(numerator, denominator) => Self::Rational(-numerator, denominator),
        }
    }
}

impl<F: Field> Add for Assigned<F> {
    type Output = Assigned<F>;
    fn add(self, rhs: Assigned<F>) -> Assigned<F> {
        match (self, rhs) {
            (Self::Zero, _) => rhs,
            (_, Self::Zero) => self,
            (Self::Trivial(lhs), Self::Trivial(rhs)) => Self::Trivial(lhs + rhs),
            (Self::Rational(numerator, denominator), Self::Trivial(other))
            | (Self::Trivial(other), Self::Rational(numerator, denominator)) => {
                Self::Rational(numerator + denominator * other, denominator)
            }
            (
                Self::Rational(lhs_numerator, lhs_denominator),
                Self::Rational(rhs_numerator, rhs_denominator),
            ) => Self::Rational(
                lhs_numerator * rhs_denominator + lhs_denominator * rhs_numerator,
                lhs_denominator * rhs_denominator,
            ),
        }
    }
}

impl<F: Field> Add<F> for Assigned<F> {
    type Output = Assigned<F>;
    fn add(self, rhs: F) -> Assigned<F> {
        self + Self::Trivial(rhs)
    }
}

impl<F: Field> Sub for Assigned<F> {
    type Output = Assigned<F>;
    fn sub(self, rhs: Assigned<F>) -> Assigned<F> {
        self + (-rhs)
    }
}

impl<F: Field> Sub<F> for Assigned<F> {
    type Output = Assigned<F>;
    fn sub(self, rhs: F) -> Assigned<F> {
        self + (-rhs)
    }
}

impl<F: Field> Mul for Assigned<F> {
    type Output = Assigned<F>;
    fn mul(self, rhs: Assigned<F>) -> Assigned<F> {
        match (self, rhs) {
            (Self::Zero, _) | (_, Self::Zero) => Self::Zero,
            (Self::Trivial(lhs), Self::Trivial(rhs)) => Self::Trivial(lhs * rhs),
            (Self::Rational(numerator, denominator), Self::Trivial(other))
            | (Self::Trivial(other), Self::Rational(numerator, denominator)) => {
                Self::Rational(numerator * other, denominator)
            }
            (
                Self::Rational(lhs_numerator, lhs_denominator),
                Self::Rational(rhs_numerator, rhs_denominator),
            ) => Self::Rational(
                lhs_numerator * rhs_numerator,
                lhs_denominator * rhs_denominator,
            ),
        }
    }
}

impl<F: Field> Mul<F> for Assigned<F> {
    type Output = Assigned<F>;
    fn mul(self, rhs: F) -> Assigned<F> {
        self * Self::Trivial(rhs)
    }
}

impl<F: Field> Assigned<F> {
    /// Returns the numerator.
    pub fn numerator(&self) -> F {
        match self {
            Self::Zero => F::zero(),
            Self::Trivial(x) => *x,
            Self::Rational(numerator, _) => *numerator,
        }
    }

    /// Returns the denominator, if non-trivial.
    pub fn denominator(&self) -> Option<F> {
        match self {
            Self::Zero => None,
            Self::Trivial(_) => None,
            Self::Rational(_, denominator) => Some(*denominator),
        }
    }

    /// Inverts this assigned value.
    pub fn invert(&self) -> Self {
        match self {
            Self::Zero => Self::Zero,
            Self::Trivial(x) => Self::Rational(F::one(), *x),
            Self::Rational(numerator, denominator) => Self::Rational(*denominator, *numerator),
        }
    }

    /// Evaluates this assigned value directly, performing an unbatched inversion if
    /// necessary.
    ///
    /// If the denominator is zero, this returns zero.
    pub fn evaluate(self) -> F {
        match self {
            Self::Zero => F::zero(),
            Self::Trivial(x) => x,
            Self::Rational(numerator, denominator) => {
                if denominator == F::one() {
                    numerator
                } else {
                    numerator * denominator.invert().unwrap_or(F::zero())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pasta_curves::Fp;

    use super::Assigned;

    #[test]
    fn add_trivial_to_inv0_rational() {
        // a = 2
        // b = inv0(1/0)
        let a = Assigned::Trivial(Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // 2 + inv0(1/0) = 2 + 0 = 1/2
        // This fails if addition is implemented using normal rules for rationals.
        assert_eq!((a + b).evaluate(), a.evaluate());
    }

    #[test]
    fn add_rational_to_inv0_rational() {
        // a = inv0(1/2)
        // b = inv0(1/0)
        let a = Assigned::Rational(Fp::one(), Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // inv0(1/2) + inv0(1/0) = 1/2 + 0 = 1/2
        // This fails if addition is implemented using normal rules for rationals.
        assert_eq!((a + b).evaluate(), a.evaluate());
    }

    #[test]
    fn sub_trivial_from_inv0_rational() {
        // a = 2
        // b = inv0(1/0)
        let a = Assigned::Trivial(Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // inv0(1/0) - 2 = 0 - 2 = -2
        // This fails if subtraction is implemented using normal rules for rationals.
        assert_eq!((b - a).evaluate(), (-a).evaluate());
    }

    #[test]
    fn sub_rational_from_inv0_rational() {
        // a = inv0(1/2)
        // b = inv0(1/0)
        let a = Assigned::Rational(Fp::one(), Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // inv0(1/0) - inv0(1/2) = 0 - 1/2 = -1/2
        // This fails if subtraction is implemented using normal rules for rationals.
        assert_eq!((b - a).evaluate(), (-a).evaluate());
    }

    #[test]
    fn mul_rational_by_inv0_rational() {
        // a = inv0(1/2)
        // b = inv0(1/0)
        let a = Assigned::Rational(Fp::one(), Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // inv0(1/2) * inv0(1/0) = 1/2 * 0 = 0
        assert_eq!((a * b).evaluate(), Fp::zero());
    }
}
