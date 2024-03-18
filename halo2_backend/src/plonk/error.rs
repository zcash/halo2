use std::error;
use std::fmt;
use std::io;

use halo2_middleware::circuit::ColumnMid;

/// This is an error that could occur during proving.
#[derive(Debug)]
pub enum Error {
    /// The provided instances do not match the circuit parameters.
    InvalidInstances,
    /// The constraint system is not satisfied.
    ConstraintSystemFailure,
    /// Out of bounds index passed to a backend
    BoundsFailure,
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
    /// The instance sets up a copy constraint involving a column that has not been
    /// included in the permutation.
    ColumnNotInPermutation(ColumnMid),
    /// Generic error not covered by previous cases
    Other(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        // The only place we can get io::Error from is the transcript.
        Error::Transcript(error)
    }
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
            Error::InvalidInstances => write!(f, "Provided instances do not match the circuit"),
            Error::ConstraintSystemFailure => write!(f, "The constraint system is not satisfied"),
            Error::BoundsFailure => write!(f, "An out-of-bounds index was passed to the backend"),
            Error::Opening => write!(f, "Multi-opening proof was invalid"),
            Error::Transcript(e) => write!(f, "Transcript error: {e}"),
            Error::NotEnoughRowsAvailable { current_k } => write!(
                f,
                "k = {current_k} is too small for the given circuit. Try using a larger value of k",
            ),
            Error::InstanceTooLarge => write!(f, "Instance vectors are larger than the circuit"),
            Error::ColumnNotInPermutation(column) => {
                write!(f, "Column {column:?} must be included in the permutation",)
            }
            Error::Other(error) => write!(f, "Other: {error}"),
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
