use super::{ErrorBack, ErrorFront};
use std::fmt;

/// This is an error that could occur during proving or circuit synthesis.
#[derive(Debug)]
pub enum Error {
    /// Frontend error case
    Frontend(ErrorFront),
    /// Backend error case
    Backend(ErrorBack),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Frontend(err) => write!(f, "Frontend: {err}"),
            Error::Backend(err) => write!(f, "Backend: {err}"),
        }
    }
}

impl From<ErrorFront> for Error {
    fn from(err: ErrorFront) -> Self {
        Error::Frontend(err)
    }
}

impl From<ErrorBack> for Error {
    fn from(err: ErrorBack) -> Self {
        Error::Backend(err)
    }
}
