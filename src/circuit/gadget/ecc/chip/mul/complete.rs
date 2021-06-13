use super::super::{add, copy, CellValue, EccConfig, EccPoint, Var};
use super::{COMPLETE_RANGE, X, Y, Z};

use halo2::{
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};

use pasta_curves::{arithmetic::FieldExt, pallas};

pub struct Config {
    // Selector used to constrain the cells used in complete addition.
    q_mul_z: Selector,
    // Advice column used to decompose scalar in complete addition.
    pub z_complete: Column<Advice>,
    // Permutation
    perm: Permutation,
    // Configuration used in complete addition
    add_config: add::Config,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        let config = Self {
            q_mul_z: ecc_config.q_mul_z,
            z_complete: ecc_config.advices[9],
            perm: ecc_config.perm.clone(),
            add_config: ecc_config.into(),
        };

        let add_config_advices = config.add_config.advice_columns();
        assert!(
            !add_config_advices.contains(&config.z_complete),
            "z_complete cannot overlap with complete addition columns."
        );

        config
    }
}

impl Config {
    /// Gate used to check scalar decomposition is correct.
    /// This is used to check the bits used in complete addition, since the incomplete
    /// addition gate (controlled by `q_mul`) already checks scalar decomposition for
    /// the other bits.
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate(
            "Decompose scalar for complete bits of variable-base mul",
            |meta| {
                let q_mul_z = meta.query_selector(self.q_mul_z);
                let z_cur = meta.query_advice(self.z_complete, Rotation::cur());
                let z_prev = meta.query_advice(self.z_complete, Rotation::prev());

                // k_{i} = z_{i} - 2⋅z_{i+1}
                let k = z_cur - Expression::Constant(pallas::Base::from_u64(2)) * z_prev;
                // (k_i) ⋅ (k_i - 1) = 0
                let bool_check = k.clone() * (k + Expression::Constant(-pallas::Base::one()));

                vec![q_mul_z * bool_check]
            },
        );
    }

    #[allow(clippy::type_complexity)]
    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn assign_region(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        bits: &[Option<bool>],
        base: &EccPoint,
        x_a: X<pallas::Base>,
        y_a: Y<pallas::Base>,
        z: Z<pallas::Base>,
    ) -> Result<(EccPoint, Vec<Z<pallas::Base>>), Error> {
        // Make sure we have the correct number of bits for the complete addition
        // part of variable-base scalar mul.
        assert_eq!(bits.len(), COMPLETE_RANGE.len());

        // Enable selectors for complete range
        for row in 0..COMPLETE_RANGE.len() {
            // Each iteration uses 2 rows (two complete additions)
            let row = 2 * row;
            // Check scalar decomposition for each iteration. Since the gate enabled by
            // `q_mul_z` queries the previous row, we enable the selector on
            // `row + offset + 1` (instead of `row + offset`).
            self.q_mul_z.enable(region, row + offset + 1)?;
        }

        // Use x_a, y_a output from incomplete addition
        let mut acc = {
            // Copy in x_a output from incomplete addition
            let x_a = copy(
                region,
                || "x_a output from incomplete addition",
                self.add_config.x_qr,
                offset,
                &x_a.0,
                &self.perm,
            )?;

            // Assign final `y_a` output from incomplete addition
            let y_a_cell = region.assign_advice(
                || "y_a",
                self.add_config.y_qr,
                offset,
                || y_a.ok_or(Error::SynthesisError),
            )?;

            EccPoint {
                x: x_a,
                y: CellValue::<pallas::Base>::new(y_a_cell, *y_a),
            }
        };

        // Copy running sum `z` from incomplete addition
        let mut z = {
            let z = copy(
                region,
                || "Copy `z` running sum from incomplete addition",
                self.z_complete,
                offset,
                &z,
                &self.perm,
            )?;
            Z(z)
        };

        // Store interstitial running sum `z`s in vector
        let mut zs: Vec<Z<pallas::Base>> = Vec::new();

        // Complete addition
        for (iter, k) in bits.iter().enumerate() {
            // Each iteration uses 2 rows (two complete additions)
            let row = 2 * iter;

            // Copy `z` running sum from previous iteration.
            copy(
                region,
                || "Copy `z` running sum from previous iteration",
                self.z_complete,
                row + offset,
                &z,
                &self.perm,
            )?;

            // Update `z`.
            z = {
                // z_next = z_cur * 2 + k_next
                let z_val = z.value().zip(k.as_ref()).map(|(z_val, k)| {
                    pallas::Base::from_u64(2) * z_val + pallas::Base::from_u64(*k as u64)
                });
                let z_cell = region.assign_advice(
                    || "z",
                    self.z_complete,
                    row + offset + 1,
                    || z_val.ok_or(Error::SynthesisError),
                )?;
                Z(CellValue::new(z_cell, z_val))
            };
            zs.push(z);

            // Assign `x_p` for complete addition
            let x_p = {
                let x_p_val = base.x.value();
                let x_p_cell = region.assign_advice(
                    || "x_p",
                    self.add_config.x_p,
                    row + offset,
                    || x_p_val.ok_or(Error::SynthesisError),
                )?;
                CellValue::<pallas::Base>::new(x_p_cell, x_p_val)
            };

            // Assign `y_p` for complete addition.
            let y_p = {
                let y_p = base.y.value();

                // If the bit is set, use `y`; if the bit is not set, use `-y`
                let y_p = y_p
                    .zip(k.as_ref())
                    .map(|(y_p, k)| if !k { -y_p } else { y_p });

                let y_p_cell = region.assign_advice(
                    || "y_p",
                    self.add_config.y_p,
                    row + offset,
                    || y_p.ok_or(Error::SynthesisError),
                )?;
                CellValue::<pallas::Base>::new(y_p_cell, y_p)
            };

            // U = P if the bit is set; U = -P is the bit is not set.
            let U = EccPoint { x: x_p, y: y_p };

            // Acc + U
            let tmp_acc = self
                .add_config
                .assign_region(&U, &acc, row + offset, region)?;

            // Copy acc from `x_a`, `y_a` over to `x_p`, `y_p` on the next row
            acc = {
                let acc_x = copy(
                    region,
                    || "copy acc x_a",
                    self.add_config.x_p,
                    row + offset + 1,
                    &acc.x,
                    &self.perm,
                )?;
                let acc_y = copy(
                    region,
                    || "copy acc y_a",
                    self.add_config.y_p,
                    row + offset + 1,
                    &acc.y,
                    &self.perm,
                )?;

                EccPoint { x: acc_x, y: acc_y }
            };

            // Acc + U + Acc
            acc = self
                .add_config
                .assign_region(&acc, &tmp_acc, row + offset + 1, region)?;
        }
        Ok((acc, zs))
    }
}
