use crate::{
    circuit::Layouter,
    plonk::{Error, TableColumn},
};

use super::keygen::Params;
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

/// tag, x, y
/// where tag = a + 2b + 2^2 * window
#[derive(Debug, Clone)]
pub struct Config<C: CurveAffine, const N: usize> {
    pub(super) params: Params<C, N>,
    pub(super) tag: TableColumn,
    pub(super) x: TableColumn,
    pub(super) y: TableColumn,
}

impl<C: CurveAffine, const N: usize> Config<C, N> {
    pub(super) fn load(&self, mut layouter: impl Layouter<C::Base>) -> Result<(), Error> {
        layouter.assign_table(
            || "lookup table",
            |mut region| {
                // pedersen_windows: [[C; 4]; N]
                //
                // Each window value is stored in a lookup table with a tag constructed
                // such that the least two bits correspond to the position within the
                // window, and the remaining bits correspond to the window index:
                //     tag = a + 2b + 2^2 * window
                //
                // Because we use 2-bit windows, enumerating a flat map will produce the
                // correct tags.
                for (tag, value) in self
                    .params
                    .pedersen_windows
                    .iter()
                    .flat_map(|window| window.iter())
                    .enumerate()
                {
                    region.assign_cell(
                        || format!("tag: {}", tag),
                        self.tag,
                        tag,
                        || Ok(C::Base::from_u64(tag as u64)),
                    )?;

                    // No base is the identity.
                    let value = value.coordinates().unwrap();
                    region.assign_cell(|| "x", self.x, tag, || Ok(*value.x()))?;
                    region.assign_cell(|| "y", self.y, tag, || Ok(*value.y()))?;
                }

                Ok(())
            },
        )
    }
}
