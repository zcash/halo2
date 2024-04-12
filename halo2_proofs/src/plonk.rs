//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

mod error;
mod keygen;
mod prover;
mod verifier {
    pub use halo2_backend::plonk::verifier::verify_proof;
}

pub use keygen::{keygen_pk, keygen_pk_custom, keygen_vk, keygen_vk_custom};

pub use prover::{create_proof, create_proof_custom_with_engine, create_proof_with_engine};
pub use verifier::verify_proof;

pub use error::Error;
pub use halo2_backend::plonk::{Error as ErrorBack, ProvingKey, VerifyingKey};
pub use halo2_frontend::plonk::{
    Advice, Assigned, Challenge, Circuit, Column, ConstraintSystem, Error as ErrorFront,
    Expression, FirstPhase, Fixed, Instance, SecondPhase, Selector, TableColumn, ThirdPhase,
};
pub use halo2_middleware::circuit::{Any, ConstraintSystemMid};

use group::ff::FromUniformBytes;
use halo2_backend::helpers::{SerdeCurveAffine, SerdeFormat, SerdePrimeField};
use halo2_frontend::circuit::compile_circuit_cs;
use std::io;

/// Reads a verification key from a buffer.
///
/// Reads a curve element from the buffer and parses it according to the `format`:
/// - `Processed`: Reads a compressed curve element and decompresses it.
/// Reads a field element in standard form, with endianness specified by the
/// `PrimeField` implementation, and checks that the element is less than the modulus.
/// - `RawBytes`: Reads an uncompressed curve element with coordinates in Montgomery form.
/// Checks that field elements are less than modulus, and then checks that the point is on the curve.
/// - `RawBytesUnchecked`: Reads an uncompressed curve element with coordinates in Montgomery form;
/// does not perform any checks
pub fn vk_read<C: SerdeCurveAffine, R: io::Read, ConcreteCircuit: Circuit<C::Scalar>>(
    reader: &mut R,
    format: SerdeFormat,
    compress_selectors: bool,
    #[cfg(feature = "circuit-params")] params: ConcreteCircuit::Params,
) -> io::Result<VerifyingKey<C>>
where
    C::Scalar: SerdePrimeField + FromUniformBytes<64>,
{
    let (_, cs, _) = compile_circuit_cs::<_, ConcreteCircuit>(
        compress_selectors,
        #[cfg(feature = "circuit-params")]
        params,
    );
    let cs_mid: ConstraintSystemMid<_> = cs.into();
    VerifyingKey::read(reader, format, cs_mid.into())
}

/// Reads a proving key from a buffer.
/// Does so by reading verification key first, and then deserializing the rest of the file into the
/// remaining proving key data.
///
/// Reads a curve element from the buffer and parses it according to the `format`:
/// - `Processed`: Reads a compressed curve element and decompresses it.
/// Reads a field element in standard form, with endianness specified by the
/// `PrimeField` implementation, and checks that the element is less than the modulus.
/// - `RawBytes`: Reads an uncompressed curve element with coordinates in Montgomery form.
/// Checks that field elements are less than modulus, and then checks that the point is on the curve.
/// - `RawBytesUnchecked`: Reads an uncompressed curve element with coordinates in Montgomery form;
/// does not perform any checks
pub fn pk_read<C: SerdeCurveAffine, R: io::Read, ConcreteCircuit: Circuit<C::Scalar>>(
    reader: &mut R,
    format: SerdeFormat,
    compress_selectors: bool,
    #[cfg(feature = "circuit-params")] params: ConcreteCircuit::Params,
) -> io::Result<ProvingKey<C>>
where
    C::Scalar: SerdePrimeField + FromUniformBytes<64>,
{
    let (_, cs, _) = compile_circuit_cs::<_, ConcreteCircuit>(
        compress_selectors,
        #[cfg(feature = "circuit-params")]
        params,
    );
    let cs_mid: ConstraintSystemMid<_> = cs.into();
    ProvingKey::read(reader, format, cs_mid.into())
}
