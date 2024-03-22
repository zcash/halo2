//! Legacy halo2 API that wraps the frontend-backend split API.  This crate doesn't implement any
//! core functionality, it just imports from the other crates and offers the legacy API in the same
//! module structure so that projects depending on halo2 can update their dependency towards it
//! without breaking.

#![cfg_attr(docsrs, feature(doc_cfg))]
// The actual lints we want to disable.
#![allow(clippy::op_ref, clippy::many_single_char_names)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod plonk;

/// Traits and structs for implementing circuit components.
pub mod circuit {
    pub use halo2_frontend::circuit::floor_planner;
    pub use halo2_frontend::circuit::{
        AssignedCell, Cell, Chip, Layouter, Region, SimpleFloorPlanner, Value,
    };
}
/// This module provides common utilities, traits and structures for group,
/// field and polynomial arithmetic.
pub mod arithmetic {
    pub use halo2_backend::arithmetic::{parallelize, CurveAffine, CurveExt, Field};
}
/// Tools for developing circuits.
pub mod dev {
    pub use halo2_frontend::dev::{metadata, FailureLocation, MockProver, VerifyFailure};

    #[cfg(feature = "cost-estimator")]
    pub use halo2_frontend::dev::cost_model;

    #[cfg(feature = "dev-graph")]
    pub use halo2_frontend::dev::{circuit_dot_graph, CircuitLayout};
}
/// Contains utilities for performing arithmetic over univariate polynomials in
/// various forms, including computing commitments to them and provably opening
/// the committed polynomials at arbitrary points.
pub mod poly {
    pub use halo2_backend::poly::VerificationStrategy;
    pub use halo2_backend::poly::{commitment, ipa, kzg};
    pub use halo2_middleware::poly::Rotation;
}
/// This module contains utilities and traits for dealing with Fiat-Shamir
/// transcripts.
pub mod transcript {
    pub use halo2_backend::transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, TranscriptRead,
        TranscriptReadBuffer, TranscriptWrite, TranscriptWriterBuffer,
    };
}
mod helpers {
    pub use halo2_backend::helpers::SerdeFormat;
}

pub use crate::helpers::SerdeFormat;

pub use halo2curves;
