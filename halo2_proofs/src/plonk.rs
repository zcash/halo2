//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

mod keygen;
mod prover;
mod verifier {
    pub use halo2_backend::plonk::verifier::verify_proof;
}

pub use keygen::{keygen_pk, keygen_vk};

pub use prover::create_proof;
pub use verifier::verify_proof;

pub use halo2_backend::plonk::{ProvingKey, VerifyingKey};
pub use halo2_common::plonk::{
    circuit::{Challenge, Column},
    Assigned, Circuit, ConstraintSystem, Error, Expression, FirstPhase, SecondPhase, Selector,
    TableColumn, ThirdPhase,
};
pub use halo2_middleware::circuit::{Advice, Fixed, Instance};
