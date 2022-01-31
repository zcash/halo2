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
            // One side is directly zero.
            (Self::Zero, _) => rhs,
            (_, Self::Zero) => self,

            // One side is x/0 which maps to zero.
            (Self::Rational(_, denominator), other) | (other, Self::Rational(_, denominator))
                if denominator.is_zero_vartime() =>
            {
                other
            }

            // Okay, we need to do some actual math...
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

    /// Inverts this assigned value (taking the inverse of zero to be zero).
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
    use pairing::bn256::Fr as Fp;

    use super::Assigned;
    // We use (numerator, denominator) in the comments below to denote a rational.
    #[test]
    fn add_trivial_to_inv0_rational() {
        // a = 2
        // b = (1,0)
        let a = Assigned::Trivial(Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // 2 + (1,0) = 2 + 0 = 2
        // This fails if addition is implemented using normal rules for rationals.
        assert_eq!((a + b).evaluate(), a.evaluate());
        assert_eq!((b + a).evaluate(), a.evaluate());
    }

    #[test]
    fn add_rational_to_inv0_rational() {
        // a = (1,2)
        // b = (1,0)
        let a = Assigned::Rational(Fp::one(), Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // (1,2) + (1,0) = (1,2) + 0 = (1,2)
        // This fails if addition is implemented using normal rules for rationals.
        assert_eq!((a + b).evaluate(), a.evaluate());
        assert_eq!((b + a).evaluate(), a.evaluate());
    }

    #[test]
    fn sub_trivial_from_inv0_rational() {
        // a = 2
        // b = (1,0)
        let a = Assigned::Trivial(Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // (1,0) - 2 = 0 - 2 = -2
        // This fails if subtraction is implemented using normal rules for rationals.
        assert_eq!((b - a).evaluate(), (-a).evaluate());

        // 2 - (1,0) = 2 - 0 = 2
        assert_eq!((a - b).evaluate(), a.evaluate());
    }

    #[test]
    fn sub_rational_from_inv0_rational() {
        // a = (1,2)
        // b = (1,0)
        let a = Assigned::Rational(Fp::one(), Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // (1,0) - (1,2) = 0 - (1,2) = -(1,2)
        // This fails if subtraction is implemented using normal rules for rationals.
        assert_eq!((b - a).evaluate(), (-a).evaluate());

        // (1,2) - (1,0) = (1,2) - 0 = (1,2)
        assert_eq!((a - b).evaluate(), a.evaluate());
    }

    #[test]
    fn mul_rational_by_inv0_rational() {
        // a = (1,2)
        // b = (1,0)
        let a = Assigned::Rational(Fp::one(), Fp::from(2));
        let b = Assigned::Rational(Fp::one(), Fp::zero());

        // (1,2) * (1,0) = (1,2) * 0 = 0
        assert_eq!((a * b).evaluate(), Fp::zero());

        // (1,0) * (1,2) = 0 * (1,2) = 0
        assert_eq!((b * a).evaluate(), Fp::zero());
    }
}

#[cfg(test)]
mod proptests {
    use std::{
        convert::TryFrom,
        ops::{Add, Mul, Sub},
    };

    use pairing::{arithmetic::FieldExt, bn256::Fr as Fp};
    use proptest::{collection::vec, prelude::*, sample::select};

    use super::Assigned;

    #[derive(Clone, Debug)]
    enum Operation {
        Add,
        Sub,
        Mul,
    }

    const OPERATIONS: &[Operation] = &[Operation::Add, Operation::Sub, Operation::Mul];

    impl Operation {
        fn apply<F: Add<Output = F> + Sub<Output = F> + Mul<Output = F>>(&self, a: F, b: F) -> F {
            match self {
                Self::Add => a + b,
                Self::Sub => a - b,
                Self::Mul => a * b,
            }
        }
    }

    prop_compose! {
        /// Use narrow that can be easily reduced.
        fn arb_element()(val in any::<u64>()) -> Fp {
            Fp::from(val)
        }
    }

    prop_compose! {
        fn arb_trivial()(element in arb_element()) -> Assigned<Fp> {
            Assigned::Trivial(element)
        }
    }

    prop_compose! {
        /// Generates half of the denominators as zero to represent a deferred inversion.
        fn arb_rational()(
            numerator in arb_element(),
            denominator in prop_oneof![Just(Fp::zero()), arb_element()],
        ) -> Assigned<Fp> {
            Assigned::Rational(numerator, denominator)
        }
    }

    prop_compose! {
        fn arb_testcase()(
            num_operations in 1usize..5,
        )(
            values in vec(
                prop_oneof![Just(Assigned::Zero), arb_trivial(), arb_rational()],
                num_operations + 1),
            operations in vec(select(OPERATIONS), num_operations),
        ) -> (Vec<Assigned<Fp>>, Vec<Operation>) {
            (values, operations)
        }
    }

    proptest! {
        #[test]
        fn operation_commutativity((values, operations) in arb_testcase()) {
            // Evaluate the values at the start.
            let elements: Vec<_> = values.iter().cloned().map(|v| v.evaluate()).collect();

            // Apply the operations to both the deferred and evaluated values.
            let deferred_result = {
                let mut ops = operations.iter();
                values.into_iter().reduce(|a, b| ops.next().unwrap().apply(a, b)).unwrap()
            };
            let evaluated_result = {
                let mut ops = operations.iter();
                elements.into_iter().reduce(|a, b| ops.next().unwrap().apply(a, b)).unwrap()
            };

            // The two should be equal, i.e. deferred inversion should commute with the
            // list of operations.
            assert_eq!(deferred_result.evaluate(), evaluated_result);
        }
    }
}
