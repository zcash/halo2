use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{Error, TableColumn},
};
use std::fmt::Debug;

use pasta_curves::pallas;

use crate::sinsemilla::chip::generator_table::{DefaultGeneratorTable, GeneratorTable};
use crate::sinsemilla::{
    chip::generator_table::GeneratorTableConfig,
    primitives::{K, SINSEMILLA_S},
};
use crate::utilities::lookup_range_check::{DefaultLookupRangeCheck, LookupRangeCheckConfig};
use crate::utilities_opt::lookup_range_check::LookupRangeCheckConfigOptimized;

/// Table containing independent generators S[0..2^k]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct GeneratorTableConfigOptimized {
    pub(crate) base: GeneratorTableConfig,
    pub(crate) table_range_check_tag: TableColumn,
}
impl GeneratorTable for GeneratorTableConfigOptimized {
    fn config(&self) -> &GeneratorTableConfig {
        &self.base
    }

    /// Load the generator table into the circuit.
    ///
    /// | table_idx |     table_x    |     table_y    | table_range_check_tag |
    /// -------------------------------------------------------------------
    /// |     0     |    X(S\[0\])   |    Y(S\[0\])   |           0           |
    /// |     1     |    X(S\[1\])   |    Y(S\[1\])   |           0           |
    /// |    ...    |      ...       |       ...      |           0           |
    /// |   2^10-1  | X(S\[2^10-1\]) | Y(S\[2^10-1\]) |           0           |
    /// |     0     |    X(S\[0\])   |    Y(S\[0\])   |           4           |
    /// |     1     |    X(S\[1\])   |    Y(S\[1\])   |           4           |
    /// |    ...    |       ...      |       ...      |           4           |
    /// |   2^4-1   | X(S\[2^4-1\])  | Y(S\[2^4-1\])  |           4           |
    /// |     0     |    X(S\[0\])   |    Y(S\[0\])   |           5           |
    /// |     1     |    X(S\[1\])   |    Y(S\[1\])   |           5           |
    /// |    ...    |       ...      |       ...      |           5           |
    /// |   2^5-1   | X(S\[2^5-1\])  | Y(S\[2^5-1\])  |           5           |
    fn load(&self, layouter: &mut impl Layouter<pallas::Base>) -> Result<(), Error> {
        layouter.assign_table(
            || "generator_table",
            |mut table| {
                for (index, (x, y)) in SINSEMILLA_S.iter().enumerate() {
                    table.assign_cell(
                        || "table_idx",
                        self.config().table_idx,
                        index,
                        || Value::known(pallas::Base::from(index as u64)),
                    )?;
                    table.assign_cell(
                        || "table_x",
                        self.config().table_x,
                        index,
                        || Value::known(*x),
                    )?;
                    table.assign_cell(
                        || "table_y",
                        self.config().table_y,
                        index,
                        || Value::known(*y),
                    )?;
                    table.assign_cell(
                        || "table_range_check_tag",
                        self.table_range_check_tag,
                        index,
                        || Value::known(pallas::Base::zero()),
                    )?;
                    if index < (1 << 4) {
                        let new_index = index + (1 << K);
                        table.assign_cell(
                            || "table_idx",
                            self.config().table_idx,
                            new_index,
                            || Value::known(pallas::Base::from(index as u64)),
                        )?;
                        table.assign_cell(
                            || "table_x",
                            self.config().table_x,
                            new_index,
                            || Value::known(*x),
                        )?;
                        table.assign_cell(
                            || "table_y",
                            self.config().table_y,
                            new_index,
                            || Value::known(*y),
                        )?;
                        table.assign_cell(
                            || "table_range_check_tag",
                            self.table_range_check_tag,
                            new_index,
                            || Value::known(pallas::Base::from(4_u64)),
                        )?;
                    }
                    if index < (1 << 5) {
                        let new_index = index + (1 << 10) + (1 << 4);
                        table.assign_cell(
                            || "table_idx",
                            self.config().table_idx,
                            new_index,
                            || Value::known(pallas::Base::from(index as u64)),
                        )?;
                        table.assign_cell(
                            || "table_x",
                            self.config().table_x,
                            new_index,
                            || Value::known(*x),
                        )?;
                        table.assign_cell(
                            || "table_y",
                            self.config().table_y,
                            new_index,
                            || Value::known(*y),
                        )?;
                        table.assign_cell(
                            || "table_range_check_tag",
                            self.table_range_check_tag,
                            new_index,
                            || Value::known(pallas::Base::from(5_u64)),
                        )?;
                    }
                }
                Ok(())
            },
        )
    }
}

pub(crate) type DefaultGeneratorTableConfigOptimized = GeneratorTableConfigOptimized;

impl DefaultGeneratorTable for DefaultGeneratorTableConfigOptimized {}
