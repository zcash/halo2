use super::{CellValue, EccConfig, EccPoint, Var};

use halo2::{
    arithmetic::CurveAffine,
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Config<C: CurveAffine> {
    q_point: Selector,
    // x-coordinate
    pub x: Column<Advice>,
    // y-coordinate
    pub y: Column<Advice>,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> From<&EccConfig<C>> for Config<C> {
    fn from(ecc_config: &EccConfig<C>) -> Self {
        Self {
            q_point: ecc_config.q_point,
            x: ecc_config.advices[0],
            y: ecc_config.advices[1],
            _marker: PhantomData,
        }
    }
}

impl<C: CurveAffine> Config<C> {
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        meta.create_gate("witness point", |meta| {
            let q_point = meta.query_selector(self.q_point);
            let x = meta.query_advice(self.x, Rotation::cur());
            let y = meta.query_advice(self.y, Rotation::cur());

            // Check that y^2 = x^3 + b, where b = 5 in the Pallas equation
            vec![
                q_point
                    * (y.clone() * y - (x.clone() * x.clone() * x) - Expression::Constant(C::b())),
            ]
        });
    }

    pub(super) fn assign_region(
        &self,
        value: Option<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        // Enable `q_point` selector
        self.q_point.enable(region, offset)?;

        let value = value.map(|value| value.coordinates().unwrap());

        // Assign `x` value
        let x_val = value.map(|value| *value.x());
        let x_var = region.assign_advice(
            || "x",
            self.x,
            offset,
            || x_val.ok_or(Error::SynthesisError),
        )?;

        // Assign `y` value
        let y_val = value.map(|value| *value.y());
        let y_var = region.assign_advice(
            || "y",
            self.y,
            offset,
            || y_val.ok_or(Error::SynthesisError),
        )?;

        Ok(EccPoint {
            x: CellValue::<C::Base>::new(x_var, x_val),
            y: CellValue::<C::Base>::new(y_var, y_val),
        })
    }
}
