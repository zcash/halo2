//! Implementations of common table layouters.

use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use halo2_middleware::ff::Field;

use crate::plonk::Assigned;
use crate::plonk::{Assignment, Error, TableColumn, TableError};

use super::Value;

/// Helper trait for implementing a custom [`Layouter`].
///
/// This trait is used for implementing table assignments.
///
/// [`Layouter`]: super::Layouter
pub trait TableLayouter<F: Field>: std::fmt::Debug {
    /// Assigns a fixed value to a table cell.
    ///
    /// Returns an error if the table cell has already been assigned to.
    fn assign_cell<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: TableColumn,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Value<Assigned<F>> + 'v),
    ) -> Result<(), Error>;
}

/// The default value to fill a table column with.
///
/// - The outer `Option` tracks whether the value in row 0 of the table column has been
///   assigned yet. This will always be `Some` once a valid table has been completely
///   assigned.
/// - The inner `Value` tracks whether the underlying `Assignment` is evaluating
///   witnesses or not.
type DefaultTableValue<F> = Option<Value<Assigned<F>>>;

/// A table layouter that can be used to assign values to a table.
pub struct SimpleTableLayouter<'r, 'a, F: Field, CS: Assignment<F> + 'a> {
    cs: &'a mut CS,
    used_columns: &'r [TableColumn],
    /// maps from a fixed column to a pair (default value, vector saying which rows are assigned)
    pub default_and_assigned: HashMap<TableColumn, (DefaultTableValue<F>, Vec<bool>)>,
}

impl<'r, 'a, F: Field, CS: Assignment<F> + 'a> fmt::Debug for SimpleTableLayouter<'r, 'a, F, CS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SimpleTableLayouter")
            .field("used_columns", &self.used_columns)
            .field("default_and_assigned", &self.default_and_assigned)
            .finish()
    }
}

impl<'r, 'a, F: Field, CS: Assignment<F> + 'a> SimpleTableLayouter<'r, 'a, F, CS> {
    /// Returns a new SimpleTableLayouter
    pub fn new(cs: &'a mut CS, used_columns: &'r [TableColumn]) -> Self {
        SimpleTableLayouter {
            cs,
            used_columns,
            default_and_assigned: HashMap::default(),
        }
    }
}

impl<'r, 'a, F: Field, CS: Assignment<F> + 'a> TableLayouter<F>
    for SimpleTableLayouter<'r, 'a, F, CS>
{
    fn assign_cell<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: TableColumn,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Value<Assigned<F>> + 'v),
    ) -> Result<(), Error> {
        if self.used_columns.contains(&column) {
            return Err(Error::TableError(TableError::UsedColumn(column)));
        }

        let entry = self.default_and_assigned.entry(column).or_default();

        let mut value = Value::unknown();
        self.cs.assign_fixed(
            annotation,
            column.inner(),
            offset, // tables are always assigned starting at row 0
            || {
                let res = to();
                value = res;
                res
            },
        )?;

        match (entry.0.is_none(), offset) {
            // Use the value at offset 0 as the default value for this table column.
            (true, 0) => entry.0 = Some(value),
            // Since there is already an existing default value for this table column,
            // the caller should not be attempting to assign another value at offset 0.
            (false, 0) => {
                return Err(Error::TableError(TableError::OverwriteDefault(
                    column,
                    format!("{:?}", entry.0.unwrap()),
                    format!("{value:?}"),
                )))
            }
            _ => (),
        }
        if entry.1.len() <= offset {
            entry.1.resize(offset + 1, false);
        }
        entry.1[offset] = true;

        Ok(())
    }
}

pub(crate) fn compute_table_lengths<F: Debug>(
    default_and_assigned: &HashMap<TableColumn, (DefaultTableValue<F>, Vec<bool>)>,
) -> Result<usize, Error> {
    let column_lengths: Result<Vec<_>, Error> = default_and_assigned
        .iter()
        .map(|(col, (default_value, assigned))| {
            if default_value.is_none() || assigned.is_empty() {
                return Err(Error::TableError(TableError::ColumnNotAssigned(*col)));
            }
            if assigned.iter().all(|b| *b) {
                // All values in the column have been assigned
                Ok((col, assigned.len()))
            } else {
                Err(Error::TableError(TableError::ColumnNotAssigned(*col)))
            }
        })
        .collect();
    let column_lengths = column_lengths?;
    column_lengths
        .into_iter()
        .try_fold((None, 0), |acc, (col, col_len)| {
            if acc.1 == 0 || acc.1 == col_len {
                Ok((Some(*col), col_len))
            } else {
                let mut cols = [(*col, col_len), (acc.0.unwrap(), acc.1)];
                cols.sort();
                Err(Error::TableError(TableError::UnevenColumnLengths(
                    cols[0], cols[1],
                )))
            }
        })
        .map(|col_len| col_len.1)
}
