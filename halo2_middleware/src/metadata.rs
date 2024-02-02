use crate::circuit::{self, Any};
use std::fmt::{self, Debug};

// TODO: Could we replace this by circuit::Column<Any>? at least for the middleware?
/// Metadata about a column within a circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Column {
    /// The type of the column.
    pub column_type: Any,
    /// The index of the column.
    pub index: usize,
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

impl From<circuit::ColumnMid> for Column {
    fn from(column: circuit::ColumnMid) -> Self {
        Column {
            column_type: column.column_type,
            index: column.index,
        }
    }
}
