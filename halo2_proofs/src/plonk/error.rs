use std::cmp;
use std::error;
use std::fmt;
use std::io;

use super::{Any, Column};

/// This is an error that could occur during proving or circuit synthesis.
// TODO: these errors need to be cleaned up
#[derive(Debug)]
pub enum Error {
    /// This is an error that can occur during synthesis of the circuit, for
    /// example, when the witness is not present.
    Synthesis(String),
    /// The provided instances do not match the circuit parameters.
    InvalidInstances,
    /// The constraint system is not satisfied.
    ConstraintSystemFailure,
    /// Out of bounds query passed to a backend.
    QueryOutOfBounds {
        /// Column index of the query
        col: usize,
        /// Row index of the query
        row: usize,
    },
    /// Not enough usable rows for circuit assignment.
    AssignOutOfBounds,
    /// Opening error
    Opening,
    /// Transcript error
    Transcript(io::Error),
    /// `k` is too small for the given circuit.
    NotEnoughRowsAvailable {
        /// The current value of `k` being used.
        current_k: u32,
    },
    /// Instance provided exceeds number of available rows
    InstanceTooLarge,
    /// Circuit synthesis requires global constants, but circuit configuration did not
    /// call [`ConstraintSystem::enable_constant`] on fixed columns with sufficient space.
    ///
    /// [`ConstraintSystem::enable_constant`]: crate::plonk::ConstraintSystem::enable_constant
    NotEnoughColumnsForConstants,
    /// The instance sets up a copy constraint involving a column that has not been
    /// included in the permutation.
    ColumnNotInPermutation(Column<Any>),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        // The only place we can get io::Error from is the transcript.
        Error::Transcript(error)
    }
}

impl Error {
    /// Constructs an `Error::NotEnoughRowsAvailable`.
    pub(crate) fn not_enough_rows_available(current_k: u32) -> Self {
        Error::NotEnoughRowsAvailable { current_k }
    }
    /// Constructs an `Error::QueryOutOfBounds`.
    pub(crate) fn query_out_of_bounds(col: usize, row: usize) -> Self {
        Error::QueryOutOfBounds { col, row }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Synthesis(err) => write!(f, "Synthesis error: {:?}", err),
            Error::InvalidInstances => write!(f, "Provided instances do not match the circuit"),
            Error::ConstraintSystemFailure => write!(f, "The constraint system is not satisfied"),
            Error::QueryOutOfBounds { row, col} => write!(
                f,
                "Query at col:{} row:{} is out of bounds",
                row,
                col,
            ),
            Error::AssignOutOfBounds => write!(f, "Not enough rows for circuit assignments. Try using a larger value of k"),
            Error::Opening => write!(f, "Multi-opening proof was invalid"),
            Error::Transcript(e) => write!(f, "Transcript error: {}", e),
            Error::NotEnoughRowsAvailable { current_k } => write!(
                f,
                "k = {} is too small for the given circuit. Try using a larger value of k",
                current_k,
            ),
            Error::InstanceTooLarge => write!(f, "Instance vectors are larger than the circuit"),
            Error::NotEnoughColumnsForConstants => {
                write!(
                    f,
                    "Too few fixed columns are enabled for global constants usage"
                )
            }
            Error::ColumnNotInPermutation(column) => write!(
                f,
                "Column {:?} must be included in the permutation. Help: try applying `meta.enable_equalty` on the column",
                column
            ),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Transcript(e) => Some(e),
            _ => None,
        }
    }
}
