use std::io;

/// This is an error that could occur during proving or circuit synthesis.
// TODO: these errors need to be cleaned up
#[derive(Debug)]
pub enum Error {
    /// This is an error that can occur during synthesis of the circuit, for
    /// example, when the witness is not present.
    Synthesis,
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
    /// Instance provided has more rows than supported by circuit
    NotEnoughRowsAvailable,
    /// Instance provided exceeds number of available rows
    InstanceTooLarge,
    /// Circuit synthesis requires global constants, but circuit configuration did not
    /// call [`ConstraintSystem::enable_constant`] on fixed columns with sufficient space.
    NotEnoughColumnsForConstants,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        // The only place we can get io::Error from is the transcript.
        Error::Transcript(error)
    }
}
