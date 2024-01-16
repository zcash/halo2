//! Metadata about circuits.

use crate::plonk::{self, Any};
use std::fmt::{self, Debug};
/// Metadata about a column within a circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Column {
    /// The type of the column.
    pub(super) column_type: Any,
    /// The index of the column.
    pub(super) index: usize,
}

impl Column {
    /// Return the column type.
    pub fn column_type(&self) -> Any {
        self.column_type
    }
    /// Return the column index.
    pub fn index(&self) -> usize {
        self.index
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Column('{:?}', {})", self.column_type, self.index)
    }
}

impl From<(Any, usize)> for Column {
    fn from((column_type, index): (Any, usize)) -> Self {
        Column { column_type, index }
    }
}

impl From<plonk::Column<Any>> for Column {
    fn from(column: plonk::Column<Any>) -> Self {
        Column {
            column_type: *column.column_type(),
            index: column.index(),
        }
    }
}
