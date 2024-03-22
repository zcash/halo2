//! Verifying/Proving key of a permutation argument, with its serialization.

use crate::helpers::{SerdeCurveAffine, SerdeFormat, SerdePrimeField};
use crate::{
    arithmetic::CurveAffine,
    helpers::{polynomial_slice_byte_length, read_polynomial_vec, write_polynomial_slice},
    poly::{Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial},
};
// TODO: Remove the renaming
pub use halo2_middleware::permutation::ArgumentMid as Argument;

use std::io;

pub mod keygen;
pub mod prover;
pub mod verifier;

/// The verifying key for a single permutation argument.
#[derive(Clone, Debug)]
pub struct VerifyingKey<C: CurveAffine> {
    commitments: Vec<C>,
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Returns commitments of sigma polynomials
    pub fn commitments(&self) -> &Vec<C> {
        &self.commitments
    }

    pub(crate) fn write<W: io::Write>(&self, writer: &mut W, format: SerdeFormat) -> io::Result<()>
    where
        C: SerdeCurveAffine,
    {
        for commitment in &self.commitments {
            commitment.write(writer, format)?;
        }
        Ok(())
    }

    pub(crate) fn read<R: io::Read>(
        reader: &mut R,
        argument: &Argument,
        format: SerdeFormat,
    ) -> io::Result<Self>
    where
        C: SerdeCurveAffine,
    {
        let commitments = (0..argument.columns.len())
            .map(|_| C::read(reader, format))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(VerifyingKey { commitments })
    }

    pub(crate) fn bytes_length(&self, format: SerdeFormat) -> usize
    where
        C: SerdeCurveAffine,
    {
        self.commitments.len() * C::byte_length(format)
    }
}

/// The proving key for a single permutation argument.
#[derive(Clone, Debug)]
pub(crate) struct ProvingKey<C: CurveAffine> {
    permutations: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    polys: Vec<Polynomial<C::Scalar, Coeff>>,
    pub(super) cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
}

impl<C: SerdeCurveAffine> ProvingKey<C>
where
    C::Scalar: SerdePrimeField,
{
    /// Reads proving key for a single permutation argument from buffer using `Polynomial::read`.  
    pub(super) fn read<R: io::Read>(reader: &mut R, format: SerdeFormat) -> io::Result<Self> {
        let permutations = read_polynomial_vec(reader, format)?;
        let polys = read_polynomial_vec(reader, format)?;
        let cosets = read_polynomial_vec(reader, format)?;
        Ok(ProvingKey {
            permutations,
            polys,
            cosets,
        })
    }

    /// Writes proving key for a single permutation argument to buffer using `Polynomial::write`.  
    pub(super) fn write<W: io::Write>(
        &self,
        writer: &mut W,
        format: SerdeFormat,
    ) -> io::Result<()> {
        write_polynomial_slice(&self.permutations, writer, format)?;
        write_polynomial_slice(&self.polys, writer, format)?;
        write_polynomial_slice(&self.cosets, writer, format)?;
        Ok(())
    }
}

impl<C: CurveAffine> ProvingKey<C> {
    /// Gets the total number of bytes in the serialization of `self`
    pub(super) fn bytes_length(&self) -> usize {
        polynomial_slice_byte_length(&self.permutations)
            + polynomial_slice_byte_length(&self.polys)
            + polynomial_slice_byte_length(&self.cosets)
    }
}
