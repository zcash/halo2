//! Legacy halo2 API that wraps the frontend-backend split API.  This crate doesn't implement any
//! core functionality, it just imports from the other crates and offers the legacy API in the same
//! module structure so that projects depending on halo2 can update their dependency towards it
//! without breaking.

pub mod plonk;

pub mod circuit {
    pub use halo2_common::circuit::{Cell, Layouter, SimpleFloorPlanner, Value};
}
pub mod arithmetic {
    pub use halo2_common::arithmetic::Field;
}
pub mod dev {
    pub use halo2_frontend::dev::MockProver;
}
pub mod poly {
    pub use halo2_backend::poly::VerificationStrategy;
    pub use halo2_common::poly::commitment;
    pub use halo2_common::poly::ipa;
    pub use halo2_common::poly::kzg;
    pub use halo2_middleware::poly::Rotation;
}
pub mod transcript {
    pub use halo2_common::transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, TranscriptReadBuffer,
        TranscriptWriterBuffer,
    };
}
