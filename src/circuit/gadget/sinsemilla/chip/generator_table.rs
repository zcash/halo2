use crate::primitives::sinsemilla::{self, sinsemilla_s_generators, S_PERSONALIZATION};
use halo2::{
    circuit::Layouter,
    plonk::{Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

use pasta_curves::{
    arithmetic::{CurveAffine, CurveExt, FieldExt},
    pallas,
};

use group::Curve;

/// Table containing independent generators S[0..2^k]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct GeneratorTableConfig {
    pub table_idx: Column<Fixed>,
    pub table_x: Column<Fixed>,
    pub table_y: Column<Fixed>,
}

impl GeneratorTableConfig {
    #[allow(clippy::too_many_arguments)]
    #[allow(non_snake_case)]
    // Even though the lookup table can be used in other parts of the circuit,
    // this specific configuration sets up Sinsemilla-specific constraints
    // controlled by `q_sinsemilla`, and would likely not apply to other chips.
    pub fn configure(meta: &mut ConstraintSystem<pallas::Base>, config: super::SinsemillaConfig) {
        let (table_idx, table_x, table_y) = (
            config.generator_table.table_idx,
            config.generator_table.table_x,
            config.generator_table.table_y,
        );

        meta.lookup(|meta| {
            let q_s1 = meta.query_selector(config.q_sinsemilla1);
            let q_s2 = meta.query_fixed(config.q_sinsemilla2, Rotation::cur());

            let table_idx_cur = meta.query_fixed(table_idx, Rotation::cur());
            let table_x_cur = meta.query_fixed(table_x, Rotation::cur());
            let table_y_cur = meta.query_fixed(table_y, Rotation::cur());

            // m_{i+1} = z_{i} - 2^K * q_s2 * z_{i + 1}
            // Note that the message words m_i's are 1-indexed while the
            // running sum z_i's are 0-indexed.
            let word = {
                let z_cur = meta.query_advice(config.bits, Rotation::cur());
                let z_next = meta.query_advice(config.bits, Rotation::next());
                z_cur - (q_s2 * z_next * pallas::Base::from_u64(1 << sinsemilla::K))
            };

            let x_p = meta.query_advice(config.x_p, Rotation::cur());

            // y_{p,i} = (Y_{A,i} / 2) - lambda1 * (x_{A,i} - x_{R,i}),
            // where Y_{A,i} = (lambda1_i + lambda2_i) * (x_{A,i} - x_{R,i}),
            //       x_{R,i} = lambda1^2 - x_{A,i} - x_{P,i}
            //
            let y_p = {
                let lambda1 = meta.query_advice(config.lambda_1, Rotation::cur());
                let lambda2 = meta.query_advice(config.lambda_2, Rotation::cur());
                let x_a = meta.query_advice(config.x_a, Rotation::cur());

                let x_r = lambda1.clone().square() - x_a.clone() - x_p.clone();
                let Y_A = (lambda1.clone() + lambda2) * (x_a.clone() - x_r.clone());

                (Y_A * pallas::Base::TWO_INV) * (lambda1 * (x_a - x_r))
            };

            // Lookup expressions default to the first entry when `q_s1`
            // is not enabled.
            let (init_x, init_y) = {
                let init_p = get_s_by_idx(0).to_affine().coordinates().unwrap();
                (*init_p.x(), *init_p.y())
            };
            let not_q_s1 = Expression::Constant(pallas::Base::one()) - q_s1.clone();

            let m = q_s1.clone() * word; // The first table index is 0.
            let x_p = q_s1.clone() * x_p + not_q_s1.clone() * init_x;
            let y_p = q_s1 * y_p + not_q_s1 * init_y;

            vec![(m, table_idx_cur), (x_p, table_x_cur), (y_p, table_y_cur)]
        });
    }

    pub fn load(&self, layouter: &mut impl Layouter<pallas::Base>) -> Result<(), Error> {
        layouter.assign_region(
            || "generator_table",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                let mut rows = sinsemilla_s_generators();

                for index in 0..(1 << sinsemilla::K) {
                    let mut row = None;
                    gate.assign_fixed(
                        || "table_idx",
                        self.table_idx,
                        index,
                        || {
                            row = rows.next();
                            Ok(pallas::Base::from_u64(index as u64))
                        },
                    )?;
                    gate.assign_fixed(
                        || "table_x",
                        self.table_x,
                        index,
                        || row.map(|(x, _)| x).ok_or(Error::SynthesisError),
                    )?;
                    gate.assign_fixed(
                        || "table_y",
                        self.table_y,
                        index,
                        || row.map(|(_, y)| y).ok_or(Error::SynthesisError),
                    )?;
                }
                Ok(())
            },
        )
    }
}

/// Get generator S by index
pub fn get_s_by_idx(idx: u32) -> pallas::Point {
    let hash = pallas::Point::hash_to_curve(S_PERSONALIZATION);
    hash(&idx.to_le_bytes())
}
