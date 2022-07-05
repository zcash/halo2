//! Helper for single-row double-and-add.

use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{AssignedCell, Region, Value},
    plonk::{
        Advice, Assigned, Column, ConstraintSystem, Constraints, Error, Expression, VirtualCells,
    },
    poly::Rotation,
};

/// A helper struct for implementing single-row double-and-add using incomplete addition.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct DoubleAndAdd<C: CurveAffine> {
    // x-coordinate of the accumulator in each double-and-add iteration.
    pub(crate) x_a: Column<Advice>,
    // x-coordinate of the point being added in each double-and-add iteration.
    pub(crate) x_p: Column<Advice>,
    // lambda1 in each double-and-add iteration.
    pub(crate) lambda_1: Column<Advice>,
    // lambda2 in each double-and-add iteration.
    pub(crate) lambda_2: Column<Advice>,
    _marker: PhantomData<C>,
}

/// The x-coordinate of the accumulator in a double-and-add instance.
pub(crate) struct X<F: FieldExt>(pub AssignedCell<Assigned<F>, F>);

impl<F: FieldExt> From<AssignedCell<Assigned<F>, F>> for X<F> {
    fn from(cell_value: AssignedCell<Assigned<F>, F>) -> Self {
        X(cell_value)
    }
}

impl<F: FieldExt> std::ops::Deref for X<F> {
    type Target = AssignedCell<Assigned<F>, F>;

    fn deref(&self) -> &AssignedCell<Assigned<F>, F> {
        &self.0
    }
}

/// The y-coordinate of the accumulator in a double-and-add instance.
///
/// This is never actually witnessed until the last round, since it
/// can be derived from other variables. Thus it only exists as a field
/// element, not a `CellValue`.
pub(crate) struct Y<F: FieldExt>(pub Value<Assigned<F>>);

impl<F: FieldExt> From<Value<Assigned<F>>> for Y<F> {
    fn from(value: Value<Assigned<F>>) -> Self {
        Y(value)
    }
}

impl<F: FieldExt> std::ops::Deref for Y<F> {
    type Target = Value<Assigned<F>>;

    fn deref(&self) -> &Value<Assigned<F>> {
        &self.0
    }
}

impl<C: CurveAffine> DoubleAndAdd<C> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        x_a: Column<Advice>,
        x_p: Column<Advice>,
        lambda_1: Column<Advice>,
        lambda_2: Column<Advice>,
        secant_selector: &dyn Fn(&mut VirtualCells<C::Base>) -> Expression<C::Base>,
        gradient_selector: &dyn Fn(&mut VirtualCells<C::Base>) -> Expression<C::Base>,
    ) -> Self {
        meta.enable_equality(x_a);

        let config = Self {
            x_a,
            x_p,
            lambda_1,
            lambda_2,
            _marker: PhantomData,
        };

        config.secant_check(meta, secant_selector);
        config.gradient_check(meta, gradient_selector);

        config
    }

    /// Gate checking secant line on every step of the double-and-add.
    fn secant_check(
        &self,
        meta: &mut ConstraintSystem<C::Base>,
        selector: &dyn Fn(&mut VirtualCells<C::Base>) -> Expression<C::Base>,
    ) {
        meta.create_gate("secant check", |meta| {
            // x_{A,i}
            let x_a_cur = meta.query_advice(self.x_a, Rotation::cur());
            // x_{A,i-1}
            let x_a_next = meta.query_advice(self.x_a, Rotation::next());
            // λ_{2,i}
            let lambda2_cur = meta.query_advice(self.lambda_2, Rotation::cur());

            // λ_{2,i}^2 − x_{A,i-1} − x_{R,i} − x_{A,i} = 0
            let secant_line =
                lambda2_cur.square() - x_a_next - self.x_r(meta, Rotation::cur()) - x_a_cur;

            let selector = selector(meta);

            Constraints::with_selector(selector, [secant_line])
        });
    }

    /// Gate checking gradient in the steady state of the double-and-add
    /// (excluding the last step).
    fn gradient_check(
        &self,
        meta: &mut ConstraintSystem<C::Base>,
        selector: &dyn Fn(&mut VirtualCells<C::Base>) -> Expression<C::Base>,
    ) {
        meta.create_gate("gradient check", |meta| {
            // x_{A,i}
            let x_a_cur = meta.query_advice(self.x_a, Rotation::cur());
            // x_{A,i-1}
            let x_a_next = meta.query_advice(self.x_a, Rotation::next());
            // λ_{2,i}
            let lambda2_cur = meta.query_advice(self.lambda_2, Rotation::cur());

            let selector = selector(meta);
            let y_a_cur = self.y_a(meta, Rotation::cur());
            let y_a_next = self.y_a(meta, Rotation::next());

            // λ_{2,i}⋅(x_{A,i} − x_{A,i-1}) − y_{A,i} − y_{A,i-1} = 0
            let gradient_check = lambda2_cur * (x_a_cur - x_a_next) - y_a_cur - y_a_next;

            Constraints::with_selector(selector, [gradient_check])
        });
    }

    /// Derives the expression `x_r = lambda_1^2 - x_a - x_p`.
    pub(crate) fn x_r(
        &self,
        meta: &mut VirtualCells<C::Base>,
        rotation: Rotation,
    ) -> Expression<C::Base> {
        let x_a = meta.query_advice(self.x_a, rotation);
        let x_p = meta.query_advice(self.x_p, rotation);
        let lambda_1 = meta.query_advice(self.lambda_1, rotation);
        lambda_1.square() - x_a - x_p
    }

    /// Derives the expression `y_a = [(lambda_1 + lambda_2) * (x_a - x_r)] / 2`.
    #[allow(non_snake_case)]
    pub(crate) fn y_a(
        &self,
        meta: &mut VirtualCells<C::Base>,
        rotation: Rotation,
    ) -> Expression<C::Base> {
        let x_a = meta.query_advice(self.x_a, rotation);
        let lambda_1 = meta.query_advice(self.lambda_1, rotation);
        let lambda_2 = meta.query_advice(self.lambda_2, rotation);
        (lambda_1 + lambda_2) * (x_a - self.x_r(meta, rotation)) * C::Base::TWO_INV
    }

    /// Derives the expression `y_p = y_a - lambda1 * (x_a - x_p)`.
    #[allow(non_snake_case)]
    pub(crate) fn y_p(
        &self,
        meta: &mut VirtualCells<C::Base>,
        rotation: Rotation,
    ) -> Expression<C::Base> {
        let y_a = self.y_a(meta, rotation);
        let lambda_1 = meta.query_advice(self.lambda_1, rotation);
        let x_a = meta.query_advice(self.x_a, rotation);
        let x_p = meta.query_advice(self.x_p, rotation);

        y_a - lambda_1 * (x_a - x_p)
    }

    /// Assigns one double-and-add round in the steady state.
    ///
    /// The main selector must be enabled outside of this helper.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::type_complexity)]
    pub(crate) fn assign_region(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        (x_p, y_p): (Value<Assigned<C::Base>>, Value<Assigned<C::Base>>),
        x_a: X<C::Base>,
        y_a: Y<C::Base>,
    ) -> Result<(X<C::Base>, Y<C::Base>), Error> {
        // Handle exceptional cases
        x_a.value().zip(*y_a).zip(x_p.zip(y_p)).error_if_known_and(
            |((x_a, y_a), (x_p, y_p))| {
                // A is point at infinity
                (x_p.is_zero_vartime() && y_p.is_zero_vartime())
                // Q is point at infinity
                || (x_a.is_zero_vartime() && y_a.is_zero_vartime())
                // x_p = x_a
                || (x_p == *x_a)
            },
        )?;

        // Copy `x_a`
        x_a.0.copy_advice(|| "copy x_a", region, self.x_a, offset)?;

        // Assign `x_p`
        region.assign_advice(|| "x_p", self.x_p, offset, || x_p)?;

        // Compute and assign `lambda_1`
        let lambda_1 = {
            let lambda_1 = x_a
                .value()
                .zip(*y_a)
                .zip(x_p)
                .zip(y_p)
                .map(|(((x_a, y_a), x_p), y_p)| (y_a - y_p) * (*x_a - x_p).invert());

            // Assign lambda_1
            region.assign_advice(|| "lambda_1", self.lambda_1, offset, || lambda_1)?;

            lambda_1
        };

        // Compute `x_r`
        let x_r = lambda_1
            .zip(x_a.value())
            .zip(x_p)
            .map(|((lambda_1, x_a), x_p)| lambda_1.square() - *x_a - x_p);

        // Compute and assign `lambda_2`
        let lambda_2 = {
            let lambda_2 = x_a.value().zip(*y_a).zip(x_r).zip(lambda_1).map(
                |(((x_a, y_a), x_r), lambda_1)| {
                    y_a * C::Base::from(2) * (*x_a - x_r.evaluate()).invert() - lambda_1
                },
            );

            region.assign_advice(|| "lambda_2", self.lambda_2, offset, || lambda_2)?;

            lambda_2
        };

        // Compute and assign `x_a` for the next row.
        let x_a_new = {
            let x_a_new = lambda_2
                .zip(x_a.value())
                .zip(x_r)
                .map(|((lambda_2, x_a), x_r)| lambda_2.square() - *x_a - x_r);

            region
                .assign_advice(|| "x_a", self.x_a, offset + 1, || x_a_new)
                .map(X)?
        };

        // Compute y_a for the next row.
        let y_a_new = lambda_2
            .zip(x_a.value())
            .zip(x_a_new.value())
            .zip(*y_a)
            .map(|(((lambda_2, x_a), x_a_new), y_a)| lambda_2 * (*x_a - x_a_new.evaluate()) - y_a);

        Ok((x_a_new, Y(y_a_new)))
    }
}
