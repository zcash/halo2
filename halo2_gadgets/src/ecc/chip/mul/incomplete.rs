use super::super::NonIdentityEccPoint;
use super::{X, Y, Z};
use crate::utilities::bool_check;
use ff::Field;
use halo2_proofs::{
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector, VirtualCells},
    poly::Rotation,
};

use pasta_curves::{arithmetic::FieldExt, pallas};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct Config<const NUM_BITS: usize> {
    // Selectors used to constrain the cells used in incomplete addition.
    pub(super) q_mul: (Selector, Selector, Selector),
    // Cumulative sum used to decompose the scalar.
    pub(super) z: Column<Advice>,
    // x-coordinate of the accumulator in each double-and-add iteration.
    pub(super) x_a: Column<Advice>,
    // x-coordinate of the point being added in each double-and-add iteration.
    pub(super) x_p: Column<Advice>,
    // y-coordinate of the point being added in each double-and-add iteration.
    pub(super) y_p: Column<Advice>,
    // lambda1 in each double-and-add iteration.
    pub(super) lambda1: Column<Advice>,
    // lambda2 in each double-and-add iteration.
    pub(super) lambda2: Column<Advice>,
}

impl<const NUM_BITS: usize> Config<NUM_BITS> {
    pub(super) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        z: Column<Advice>,
        x_a: Column<Advice>,
        x_p: Column<Advice>,
        y_p: Column<Advice>,
        lambda1: Column<Advice>,
        lambda2: Column<Advice>,
    ) -> Self {
        meta.enable_equality(z);
        meta.enable_equality(lambda1);

        let config = Self {
            q_mul: (meta.selector(), meta.selector(), meta.selector()),
            z,
            x_a,
            x_p,
            y_p,
            lambda1,
            lambda2,
        };

        config.create_gate(meta);

        config
    }

    // Gate for incomplete addition part of variable-base scalar multiplication.
    fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        // Closure to compute x_{R,i} = λ_{1,i}^2 - x_{A,i} - x_{P,i}
        let x_r = |meta: &mut VirtualCells<pallas::Base>, rotation: Rotation| {
            let x_a = meta.query_advice(self.x_a, rotation);
            let x_p = meta.query_advice(self.x_p, rotation);
            let lambda_1 = meta.query_advice(self.lambda1, rotation);
            lambda_1.square() - x_a - x_p
        };

        // Closure to compute y_{A,i} = (λ_{1,i} + λ_{2,i}) * (x_{A,i} - x_{R,i}) / 2
        let y_a = |meta: &mut VirtualCells<pallas::Base>, rotation: Rotation| {
            let x_a = meta.query_advice(self.x_a, rotation);
            let lambda_1 = meta.query_advice(self.lambda1, rotation);
            let lambda_2 = meta.query_advice(self.lambda2, rotation);

            (lambda_1 + lambda_2) * (x_a - x_r(meta, rotation)) * pallas::Base::TWO_INV
        };

        // Constraints used for q_mul_{2, 3} == 1
        let for_loop = |meta: &mut VirtualCells<pallas::Base>,
                        q_mul: Expression<pallas::Base>,
                        y_a_next: Expression<pallas::Base>| {
            let one = Expression::Constant(pallas::Base::one());

            // z_i
            let z_cur = meta.query_advice(self.z, Rotation::cur());
            // z_{i+1}
            let z_prev = meta.query_advice(self.z, Rotation::prev());
            // x_{A,i}
            let x_a_cur = meta.query_advice(self.x_a, Rotation::cur());
            // x_{A,i-1}
            let x_a_next = meta.query_advice(self.x_a, Rotation::next());
            // x_{P,i}
            let x_p_cur = meta.query_advice(self.x_p, Rotation::cur());
            // y_{P,i}
            let y_p_cur = meta.query_advice(self.y_p, Rotation::cur());
            // λ_{1,i}
            let lambda1_cur = meta.query_advice(self.lambda1, Rotation::cur());
            // λ_{2,i}
            let lambda2_cur = meta.query_advice(self.lambda2, Rotation::cur());

            let y_a_cur = y_a(meta, Rotation::cur());

            // The current bit in the scalar decomposition, k_i = z_i - 2⋅z_{i+1}.
            // Recall that we assigned the cumulative variable `z_i` in descending order,
            // i from n down to 0. So z_{i+1} corresponds to the `z_prev` query.
            let k = z_cur - z_prev * pallas::Base::from(2);
            // Check booleanity of decomposition.
            let bool_check = bool_check(k.clone());

            // λ_{1,i}⋅(x_{A,i} − x_{P,i}) − y_{A,i} + (2k_i - 1) y_{P,i} = 0
            let gradient_1 = lambda1_cur * (x_a_cur.clone() - x_p_cur) - y_a_cur.clone()
                + (k * pallas::Base::from(2) - one) * y_p_cur;

            // λ_{2,i}^2 − x_{A,i-1} − x_{R,i} − x_{A,i} = 0
            let secant_line = lambda2_cur.clone().square()
                - x_a_next.clone()
                - x_r(meta, Rotation::cur())
                - x_a_cur.clone();

            // λ_{2,i}⋅(x_{A,i} − x_{A,i-1}) − y_{A,i} − y_{A,i-1} = 0
            let gradient_2 = lambda2_cur * (x_a_cur - x_a_next) - y_a_cur - y_a_next;

            std::iter::empty()
                .chain(Some(("bool_check", q_mul.clone() * bool_check)))
                .chain(Some(("gradient_1", q_mul.clone() * gradient_1)))
                .chain(Some(("secant_line", q_mul.clone() * secant_line)))
                .chain(Some(("gradient_2", q_mul * gradient_2)))
        };

        // q_mul_1 == 1 checks
        meta.create_gate("q_mul_1 == 1 checks", |meta| {
            let q_mul_1 = meta.query_selector(self.q_mul.0);

            let y_a_next = y_a(meta, Rotation::next());
            let y_a_witnessed = meta.query_advice(self.lambda1, Rotation::cur());
            vec![("init y_a", q_mul_1 * (y_a_witnessed - y_a_next))]
        });

        // q_mul_2 == 1 checks
        meta.create_gate("q_mul_2 == 1 checks", |meta| {
            let q_mul_2 = meta.query_selector(self.q_mul.1);

            let y_a_next = y_a(meta, Rotation::next());

            // x_{P,i}
            let x_p_cur = meta.query_advice(self.x_p, Rotation::cur());
            // x_{P,i-1}
            let x_p_next = meta.query_advice(self.x_p, Rotation::next());
            // y_{P,i}
            let y_p_cur = meta.query_advice(self.y_p, Rotation::cur());
            // y_{P,i-1}
            let y_p_next = meta.query_advice(self.y_p, Rotation::next());

            // The base used in double-and-add remains constant. We check that its
            // x- and y- coordinates are the same throughout.
            let x_p_check = x_p_cur - x_p_next;
            let y_p_check = y_p_cur - y_p_next;

            std::iter::empty()
                .chain(Some(("x_p_check", q_mul_2.clone() * x_p_check)))
                .chain(Some(("y_p_check", q_mul_2.clone() * y_p_check)))
                .chain(for_loop(meta, q_mul_2, y_a_next))
        });

        // q_mul_3 == 1 checks
        meta.create_gate("q_mul_3 == 1 checks", |meta| {
            let q_mul_3 = meta.query_selector(self.q_mul.2);
            let y_a_final = meta.query_advice(self.lambda1, Rotation::next());
            for_loop(meta, q_mul_3, y_a_final)
        });
    }

    /// We perform incomplete addition on all but the last three bits of the
    /// decomposed scalar.
    /// We split the bits in the incomplete addition range into "hi" and "lo"
    /// halves and process them side by side, using the same rows but with
    /// non-overlapping columns. The base is never the identity point even at
    /// the boundary between halves.
    /// Returns (x, y, z).
    #[allow(clippy::type_complexity)]
    pub(super) fn double_and_add(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: &NonIdentityEccPoint,
        bits: &[Option<bool>],
        acc: (X<pallas::Base>, Y<pallas::Base>, Z<pallas::Base>),
    ) -> Result<(X<pallas::Base>, Y<pallas::Base>, Vec<Z<pallas::Base>>), Error> {
        // Check that we have the correct number of bits for this double-and-add.
        assert_eq!(bits.len(), NUM_BITS);

        // Handle exceptional cases
        let (x_p, y_p) = (base.x.value().cloned(), base.y.value().cloned());
        let (x_a, y_a) = (acc.0.value().cloned(), acc.1.value().cloned());

        if let (Some(x_a), Some(y_a), Some(x_p), Some(y_p)) = (x_a, y_a, x_p, y_p) {
            // A is point at infinity
            if (x_p.is_zero_vartime() && y_p.is_zero_vartime())
            // Q is point at infinity
            || (x_a.is_zero_vartime() && y_a.is_zero_vartime())
            // x_p = x_a
            || (x_p == x_a)
            {
                return Err(Error::Synthesis);
            }
        }

        // Set q_mul values
        {
            // q_mul_1 = 1 on offset 0
            self.q_mul.0.enable(region, offset)?;

            let offset = offset + 1;
            // q_mul_2 = 1 on all rows after offset 0, excluding the last row.
            for idx in 0..(NUM_BITS - 1) {
                self.q_mul.1.enable(region, offset + idx)?;
            }

            // q_mul_3 = 1 on the last row.
            self.q_mul.2.enable(region, offset + NUM_BITS - 1)?;
        }

        // Initialise double-and-add
        let (mut x_a, mut y_a, mut z) = {
            // Initialise the running `z` sum for the scalar bits.
            let z = acc.2.copy_advice(|| "starting z", region, self.z, offset)?;

            // Initialise acc
            let x_a = acc
                .0
                .copy_advice(|| "starting x_a", region, self.x_a, offset + 1)?;
            let y_a = acc
                .1
                .copy_advice(|| "starting y_a", region, self.lambda1, offset)?;

            (x_a, y_a.value().cloned(), z)
        };

        // Increase offset by 1; we used row 0 for initializing `z`.
        let offset = offset + 1;

        // Initialise vector to store all interstitial `z` running sum values.
        let mut zs: Vec<Z<pallas::Base>> = Vec::with_capacity(bits.len());

        // Incomplete addition
        for (row, k) in bits.iter().enumerate() {
            // z_{i} = 2 * z_{i+1} + k_i
            let z_val = z
                .value()
                .zip(k.as_ref())
                .map(|(z_val, k)| pallas::Base::from(2) * z_val + pallas::Base::from(*k as u64));
            z = region.assign_advice(
                || "z",
                self.z,
                row + offset,
                || z_val.ok_or(Error::Synthesis),
            )?;
            zs.push(Z(z.clone()));

            // Assign `x_p`, `y_p`
            region.assign_advice(
                || "x_p",
                self.x_p,
                row + offset,
                || x_p.ok_or(Error::Synthesis),
            )?;
            region.assign_advice(
                || "y_p",
                self.y_p,
                row + offset,
                || y_p.ok_or(Error::Synthesis),
            )?;

            // If the bit is set, use `y`; if the bit is not set, use `-y`
            let y_p = y_p
                .zip(k.as_ref())
                .map(|(y_p, k)| if !k { -y_p } else { y_p });

            // Compute and assign λ1⋅(x_A − x_P) = y_A − y_P
            let lambda1 = y_a
                .zip(y_p)
                .zip(x_a.value())
                .zip(x_p)
                .map(|(((y_a, y_p), x_a), x_p)| (y_a - y_p) * (x_a - x_p).invert().unwrap());
            region.assign_advice(
                || "lambda1",
                self.lambda1,
                row + offset,
                || lambda1.ok_or(Error::Synthesis),
            )?;

            // x_R = λ1^2 - x_A - x_P
            let x_r = lambda1
                .zip(x_a.value())
                .zip(x_p)
                .map(|((lambda1, x_a), x_p)| lambda1 * lambda1 - x_a - x_p);

            // λ2 = (2(y_A) / (x_A - x_R)) - λ1
            let lambda2 =
                lambda1
                    .zip(y_a)
                    .zip(x_a.value())
                    .zip(x_r)
                    .map(|(((lambda1, y_a), x_a), x_r)| {
                        pallas::Base::from(2) * y_a * (x_a - x_r).invert().unwrap() - lambda1
                    });
            region.assign_advice(
                || "lambda2",
                self.lambda2,
                row + offset,
                || lambda2.ok_or(Error::Synthesis),
            )?;

            // Compute and assign `x_a` for the next row
            let x_a_new = lambda2
                .zip(x_a.value())
                .zip(x_r)
                .map(|((lambda2, x_a), x_r)| lambda2.square() - x_a - x_r);
            y_a = lambda2
                .zip(x_a.value().cloned())
                .zip(x_a_new)
                .zip(y_a)
                .map(|(((lambda2, x_a), x_a_new), y_a)| lambda2 * (x_a - x_a_new) - y_a);
            let x_a_val = x_a_new;
            x_a = region.assign_advice(
                || "x_a",
                self.x_a,
                row + offset + 1,
                || x_a_val.ok_or(Error::Synthesis),
            )?;
        }

        // Witness final y_a
        let y_a = region.assign_advice(
            || "y_a",
            self.lambda1,
            offset + NUM_BITS,
            || y_a.ok_or(Error::Synthesis),
        )?;

        Ok((X(x_a), Y(y_a), zs))
    }
}
