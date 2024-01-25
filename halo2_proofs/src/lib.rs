//! Legacy halo2 API that wraps the frontend-backend split API.  This crate doesn't implement any
//! core functionality, it just imports from the other crates and offers the legacy API in the same
//! module structure so that projects depending on halo2 can update their dependency towards it
//! without breaking.

pub mod plonk;

pub mod circuit {
    pub use halo2_common::circuit::{Layouter, SimpleFloorPlanner};
}
pub use halo2_common::poly;
pub use halo2_common::transcript;
