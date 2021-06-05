use std::array;

use super::super::{copy, CellValue, EccConfig, EccPoint, Var};
use super::{INCOMPLETE_HI_RANGE, INCOMPLETE_LO_RANGE, X, Y, Z};
use ff::Field;
use halo2::{
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};

use pasta_curves::{arithmetic::FieldExt, pallas};

pub(super) struct Config {
    // Number of bits covered by this incomplete range.
    num_bits: usize,
    // Selector used to constrain the cells used in incomplete addition.
    pub(super) q_mul: Selector,
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
    // Permutation
    pub(super) perm: Permutation,
}

impl Config {
    // Columns used in processing the `hi` bits of the scalar.
    // `x_p, y_p` are shared across the `hi` and `lo` halves.
    pub(super) fn hi_config(ecc_config: &EccConfig) -> Self {
        Self {
            num_bits: INCOMPLETE_HI_RANGE.len(),
            q_mul: ecc_config.q_mul_hi,
            x_p: ecc_config.advices[0],
            y_p: ecc_config.advices[1],
            z: ecc_config.advices[9],
            x_a: ecc_config.advices[3],
            lambda1: ecc_config.advices[4],
            lambda2: ecc_config.advices[5],
            perm: ecc_config.perm.clone(),
        }
    }

    // Columns used in processing the `lo` bits of the scalar.
    // `x_p, y_p` are shared across the `hi` and `lo` halves.
    pub(super) fn lo_config(ecc_config: &EccConfig) -> Self {
        Self {
            num_bits: INCOMPLETE_LO_RANGE.len(),
            q_mul: ecc_config.q_mul_lo,
            x_p: ecc_config.advices[0],
            y_p: ecc_config.advices[1],
            z: ecc_config.advices[6],
            x_a: ecc_config.advices[7],
            lambda1: ecc_config.advices[8],
            lambda2: ecc_config.advices[2],
            perm: ecc_config.perm.clone(),
        }
    }

    // Gate for incomplete addition part of variable-base scalar multiplication.
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("Incomplete addition for variable-base scalar mul", |meta| {
            let q_mul = meta.query_selector(self.q_mul);
            let z_cur = meta.query_advice(self.z, Rotation::cur());
            let z_prev = meta.query_advice(self.z, Rotation::prev());
            let x_a_cur = meta.query_advice(self.x_a, Rotation::cur());
            let x_a_next = meta.query_advice(self.x_a, Rotation::next());
            let x_p_cur = meta.query_advice(self.x_p, Rotation::cur());
            let x_p_next = meta.query_advice(self.x_p, Rotation::next());
            let y_p_cur = meta.query_advice(self.y_p, Rotation::cur());
            let y_p_next = meta.query_advice(self.y_p, Rotation::next());
            let lambda1_cur = meta.query_advice(self.lambda1, Rotation::cur());
            let lambda2_cur = meta.query_advice(self.lambda2, Rotation::cur());
            let lambda1_next = meta.query_advice(self.lambda1, Rotation::next());
            let lambda2_next = meta.query_advice(self.lambda2, Rotation::next());

            // The current bit in the scalar decomposition, k_i = z_i - 2⋅z_{i+1}.
            // Recall that we assigned the cumulative variable `z_i` in descending order,
            // i from n down to 0. So z_{i+1} corresponds to the `z_prev` query.
            let k = z_cur - Expression::Constant(pallas::Base::from_u64(2)) * z_prev;

            // y_{A,i} = (λ_{1,i} + λ_{2,i})
            //           * (x_{A,i} - (λ_{1,i}^2 - x_{A,i} - x_{P,i})) / 2
            let y_a_cur = (lambda1_cur.clone() + lambda2_cur.clone())
                * (x_a_cur.clone()
                    - (lambda1_cur.clone() * lambda1_cur.clone()
                        - x_a_cur.clone()
                        - x_p_cur.clone()))
                * pallas::Base::TWO_INV;

            // y_{A,i+1} = (λ_{1,i+1} + λ_{2,i+1})
            //           * (x_{A,i+1} - (λ_{1,i+1}^2 - x_{A,i+1} - x_{P,i+1})) / 2
            let y_a_next = (lambda1_next.clone() + lambda2_next)
                * (x_a_next.clone()
                    - (lambda1_next.clone() * lambda1_next - x_a_next.clone() - x_p_next.clone()))
                * pallas::Base::TWO_INV;

            // Check booleanity of decomposition.
            let bool_check = k.clone() * (k.clone() + Expression::Constant(-pallas::Base::one()));

            // The base used in double-and-add remains constant. We check that its
            // x- and y- coordinates are the same throughout.
            let x_p_check = x_p_cur.clone() - x_p_next;
            let y_p_check = y_p_cur.clone() - y_p_next;

            // λ_{1,i}⋅(x_{A,i} − x_{P,i}) − y_{A,i} + (2k_i - 1) y_{P,i} = 0
            let poly1 = lambda1_cur.clone() * (x_a_cur.clone() - x_p_cur.clone()) - y_a_cur.clone()
                + (k * pallas::Base::from_u64(2) + Expression::Constant(-pallas::Base::one()))
                    * y_p_cur;

            // (λ_{1,i} + λ_{2,i})⋅(x_{A,i} − (λ_{1,i}^2 − x_{A,i} − x_{P,i})) − 2y_{A,i}) = 0
            let poly2 = {
                let lambda_neg = lambda1_cur.clone() + lambda2_cur.clone();
                let lambda1_expr =
                    lambda1_cur.clone() * lambda1_cur.clone() - x_a_cur.clone() - x_p_cur.clone();
                lambda_neg * (x_a_cur.clone() - lambda1_expr)
                    - Expression::Constant(pallas::Base::from_u64(2)) * y_a_cur.clone()
            };

            // λ_{2,i}^2 − x_{A,i+1} −(λ_{1,i}^2 − x_{A,i} − x_{P,i}) − x_{A,i} = 0
            let poly3 = lambda2_cur.clone() * lambda2_cur.clone()
                - x_a_next.clone()
                - (lambda1_cur.clone() * lambda1_cur)
                + x_p_cur;

            // λ_{2,i}⋅(x_{A,i} − x_{A,i+1}) − y_{A,i} − y_{A,i+1} = 0
            let poly4 = lambda2_cur * (x_a_cur - x_a_next) - y_a_cur - y_a_next;

            array::IntoIter::new([bool_check, x_p_check, y_p_check, poly1, poly2, poly3, poly4])
                .map(move |poly| q_mul.clone() * poly)
        });
    }

    // We perform incomplete addition on all but the last three bits of the
    // decomposed scalar.
    // We split the bits in the incomplete addition range into "hi" and "lo"
    // halves and process them side by side, using the same rows but with
    // non-overlapping columns.
    // Returns (x, y, z).
    #[allow(clippy::type_complexity)]
    pub(super) fn double_and_add(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        base: &EccPoint,
        bits: &[Option<bool>],
        acc: (X<pallas::Base>, Y<pallas::Base>, Z<pallas::Base>),
    ) -> Result<(X<pallas::Base>, Y<pallas::Base>, Vec<Z<pallas::Base>>), Error> {
        // Check that we have the correct number of bits for this double-and-add.
        assert_eq!(bits.len(), self.num_bits);

        // Handle exceptional cases
        let (x_p, y_p) = (base.x.value(), base.y.value());
        let (x_a, y_a) = (acc.0.value(), acc.1 .0);

        if let (Some(x_a), Some(y_a), Some(x_p), Some(y_p)) = (x_a, y_a, x_p, y_p) {
            // A is point at infinity
            if (x_p == pallas::Base::zero() && y_p == pallas::Base::zero())
            // Q is point at infinity
            || (x_a == pallas::Base::zero() && y_a == pallas::Base::zero())
            // x_p = x_a
            || (x_p == x_a)
            {
                return Err(Error::SynthesisError);
            }
        }

        // Enable `q_mul` on all but the last row of the incomplete range.
        for row in 1..(self.num_bits - 1) {
            self.q_mul.enable(region, offset + row)?;
        }

        // Initialise the running `z` sum for the scalar bits.
        let mut z = copy(region, || "starting z", self.z, offset, &acc.2, &self.perm)?;

        // Increase offset by 1; we used row 0 for initializing `z`.
        let offset = offset + 1;

        // Define `x_p`, `y_p`
        let x_p = base.x.value();
        let y_p = base.y.value();

        // Initialise acc
        let mut x_a = copy(
            region,
            || "starting x_a",
            self.x_a,
            offset,
            &acc.0,
            &self.perm,
        )?;
        let mut y_a = *acc.1;

        // Initialise vector to store all interstitial `z` running sum values.
        let mut zs: Vec<Z<pallas::Base>> = Vec::new();

        // Incomplete addition
        for (row, k) in bits.iter().enumerate() {
            // z_{i} = 2 * z_{i+1} + k_i
            let z_val = z.value().zip(k.as_ref()).map(|(z_val, k)| {
                pallas::Base::from_u64(2) * z_val + pallas::Base::from_u64(*k as u64)
            });
            let z_cell = region.assign_advice(
                || "z",
                self.z,
                row + offset,
                || z_val.ok_or(Error::SynthesisError),
            )?;
            z = CellValue::new(z_cell, z_val);
            zs.push(Z(z));

            // Assign `x_p`, `y_p`
            region.assign_advice(
                || "x_p",
                self.x_p,
                row + offset,
                || x_p.ok_or(Error::SynthesisError),
            )?;
            region.assign_advice(
                || "y_p",
                self.y_p,
                row + offset,
                || y_p.ok_or(Error::SynthesisError),
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
                || lambda1.ok_or(Error::SynthesisError),
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
                        pallas::Base::from_u64(2) * y_a * (x_a - x_r).invert().unwrap() - lambda1
                    });
            region.assign_advice(
                || "lambda2",
                self.lambda2,
                row + offset,
                || lambda2.ok_or(Error::SynthesisError),
            )?;

            // Compute and assign `x_a` for the next row
            let x_a_new = lambda2
                .zip(x_a.value())
                .zip(x_r)
                .map(|((lambda2, x_a), x_r)| lambda2 * lambda2 - x_a - x_r);
            y_a = lambda2
                .zip(x_a.value())
                .zip(x_a_new)
                .zip(y_a)
                .map(|(((lambda2, x_a), x_a_new), y_a)| lambda2 * (x_a - x_a_new) - y_a);
            let x_a_val = x_a_new;
            let x_a_cell = region.assign_advice(
                || "x_a",
                self.x_a,
                row + offset + 1,
                || x_a_val.ok_or(Error::SynthesisError),
            )?;
            x_a = CellValue::new(x_a_cell, x_a_val);
        }

        Ok((X(x_a), Y(y_a), zs))
    }
}
