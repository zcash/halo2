use std::fmt;

use super::TableColumn;
use crate::plonk::Column;
use halo2_middleware::circuit::Any;

/// This is an error that could occur during circuit synthesis.
#[derive(Debug)]
pub enum Error {
    /// This is an error that can occur during synthesis of the circuit, for
    /// example, when the witness is not present.
    Synthesis,
    /// Out of bounds index passed to a backend
    BoundsFailure,
    /// `k` is too small for the given circuit.
    NotEnoughRowsAvailable {
        /// The current value of `k` being used.
        current_k: u32,
    },
    /// Circuit synthesis requires global constants, but circuit configuration did not
    /// call [`ConstraintSystem::enable_constant`] on fixed columns with sufficient space.
    ///
    /// [`ConstraintSystem::enable_constant`]: crate::plonk::ConstraintSystem::enable_constant
    NotEnoughColumnsForConstants,
    /// The instance sets up a copy constraint involving a column that has not been
    /// included in the permutation.
    ColumnNotInPermutation(Column<Any>),
    /// An error relating to a lookup table.
    TableError(TableError),
    /// Generic error not covered by previous cases
    Other(String),
}

impl Error {
    /// Constructs an `Error::NotEnoughRowsAvailable`.
    pub fn not_enough_rows_available(current_k: u32) -> Self {
        Error::NotEnoughRowsAvailable { current_k }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Synthesis => write!(f, "General synthesis error"),
            Error::BoundsFailure => write!(f, "An out-of-bounds index was passed to the backend"),
            Error::NotEnoughRowsAvailable { current_k } => write!(
                f,
                "k = {current_k} is too small for the given circuit. Try using a larger value of k",
            ),
            Error::NotEnoughColumnsForConstants => {
                write!(
                    f,
                    "Too few fixed columns are enabled for global constants usage"
                )
            }
            Error::ColumnNotInPermutation(column) => write!(
                f,
                "Column {column:?} must be included in the permutation. Help: try applying `meta.enable_equalty` on the column",
            ),
            Error::TableError(error) => write!(f, "{error}"),
            Error::Other(error) => write!(f, "Other: {error}"),
        }
    }
}

/// This is an error that could occur during table synthesis.
#[derive(Debug)]
pub enum TableError {
    /// A `TableColumn` has not been assigned.
    ColumnNotAssigned(TableColumn),
    /// A Table has columns of uneven lengths.
    UnevenColumnLengths((TableColumn, usize), (TableColumn, usize)),
    /// Attempt to assign a used `TableColumn`
    UsedColumn(TableColumn),
    /// Attempt to overwrite a default value
    OverwriteDefault(TableColumn, String, String),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::ColumnNotAssigned(col) => {
                write!(
                    f,
                    "{col:?} not fully assigned. Help: assign a value at offset 0.",
                )
            }
            TableError::UnevenColumnLengths((col, col_len), (table, table_len)) => write!(
                f,
                "{col:?} has length {col_len} while {table:?} has length {table_len}",
            ),
            TableError::UsedColumn(col) => {
                write!(f, "{col:?} has already been used")
            }
            TableError::OverwriteDefault(col, default, val) => {
                write!(
                    f,
                    "Attempted to overwrite default value {default} with {val} in {col:?}",
                )
            }
        }
    }
}
