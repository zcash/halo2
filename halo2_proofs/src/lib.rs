//! Legacy halo2 API that wraps the frontend-backend split API.  This crate doesn't implement any
//! core functionality, it just imports from the other crates and offers the legacy API in the same
//! module structure so that projects depending on halo2 can update their dependency towards it
//! without breaking.

pub mod plonk;

pub mod circuit {
    pub use halo2_common::circuit::floor_planner;
    pub use halo2_common::circuit::{
        AssignedCell, Cell, Chip, Layouter, Region, SimpleFloorPlanner, Value,
    };
}
pub mod arithmetic {
    pub use halo2_common::arithmetic::{
        best_fft, parallelize, small_multiexp, CurveAffine, CurveExt, Field,
    };
}
pub mod dev {
    pub use halo2_frontend::dev::{metadata, FailureLocation, MockProver, VerifyFailure};
}
pub mod poly {
    pub use halo2_backend::poly::VerificationStrategy;
    pub use halo2_common::poly::{commitment, ipa, kzg};
    pub use halo2_middleware::poly::Rotation;
}
pub mod transcript {
    pub use halo2_common::transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, TranscriptReadBuffer,
        TranscriptWriterBuffer,
    };
}
pub mod helpers {
    pub use halo2_common::helpers::SerdeFormat;
}

pub use crate::helpers::SerdeFormat;

pub use halo2curves;
