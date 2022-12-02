//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use blake2b_simd::Params as Blake2bParams;
use ff::PrimeField;
use group::ff::Field;

use crate::arithmetic::{CurveAffine, FieldExt};
use crate::helpers::{
    polynomial_slice_byte_length, read_polynomial_vec, write_polynomial_slice, CurveRead,
    SerdePrimeField,
};
use crate::poly::{
    commitment::Params, Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff,
    PinnedEvaluationDomain, Polynomial,
};
use crate::transcript::{ChallengeScalar, EncodedChallenge, Transcript};

mod assigned;
mod circuit;
mod error;
mod evaluation;
mod keygen;
mod lookup;
pub(crate) mod permutation;
mod vanishing;

mod prover;
mod verifier;

pub use assigned::*;
pub use circuit::*;
pub use error::*;
pub use keygen::*;
pub use prover::*;
pub use verifier::*;

use evaluation::Evaluator;
use std::io;

/// This is a verifying key which allows for the verification of proofs for a
/// particular circuit.
#[derive(Clone, Debug)]
pub struct VerifyingKey<C: CurveAffine> {
    domain: EvaluationDomain<C::Scalar>,
    fixed_commitments: Vec<C>,
    permutation: permutation::VerifyingKey<C>,
    cs: ConstraintSystem<C::Scalar>,
    /// Cached maximum degree of `cs` (which doesn't change after construction).
    cs_degree: usize,
    /// The representative of this `VerifyingKey` in transcripts.
    transcript_repr: C::Scalar,
    selectors: Vec<Vec<bool>>,
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Writes a verifying key to a buffer.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&(self.fixed_commitments.len() as u32).to_be_bytes())?;
        for commitment in &self.fixed_commitments {
            writer.write_all(commitment.to_bytes().as_ref())?;
        }
        self.permutation.write(writer)?;

        // write self.selectors
        for selector in &self.selectors {
            // since `selector` is filled with `bool`, we pack them 8 at a time into bytes and then write
            for bits in selector.chunks(8) {
                writer.write_all(&[crate::helpers::pack(bits)])?;
            }
        }
        Ok(())
    }

    /// Reads a verification key from a buffer.
    pub fn read<'params, R: io::Read, ConcreteCircuit: Circuit<C::Scalar>>(
        reader: &mut R,
        params: &impl Params<'params, C>,
    ) -> io::Result<Self> {
        let (domain, cs, _) = keygen::create_domain::<C, ConcreteCircuit>(params.k());
        let mut num_fixed_columns_be_bytes = [0u8; 4];
        reader.read_exact(&mut num_fixed_columns_be_bytes)?;
        let num_fixed_columns = u32::from_be_bytes(num_fixed_columns_be_bytes);

        let fixed_commitments: Vec<_> = (0..num_fixed_columns)
            .map(|_| C::read(reader))
            .collect::<Result<_, _>>()?;

        let permutation = permutation::VerifyingKey::read(reader, &cs.permutation)?;

        // read selectors
        let selectors: Vec<Vec<bool>> = vec![vec![false; params.n() as usize]; cs.num_selectors]
            .into_iter()
            .map(|mut selector| {
                let mut selector_bytes = vec![0u8; (selector.len() + 7) / 8];
                reader.read_exact(&mut selector_bytes)?;
                for (bits, byte) in selector.chunks_mut(8).into_iter().zip(selector_bytes) {
                    crate::helpers::unpack(byte, bits);
                }
                Ok(selector)
            })
            .collect::<io::Result<Vec<Vec<bool>>>>()
            .unwrap();
        let (cs, _) = cs.compress_selectors(selectors.clone());

        Ok(Self::from_parts(
            domain,
            fixed_commitments,
            permutation,
            cs,
            selectors,
        ))
    }

    /// Writes a verifying key to a vector of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::with_capacity(self.bytes_length());
        Self::write(self, &mut bytes).expect("Writing to vector should not fail");
        bytes
    }

    /// Reads a verification key from a slice of bytes.
    pub fn from_bytes<'params, ConcreteCircuit: Circuit<C::Scalar>>(
        mut bytes: &[u8],
        params: &impl Params<'params, C>,
    ) -> io::Result<Self> {
        Self::read::<_, ConcreteCircuit>(&mut bytes, params)
    }

    fn bytes_length(&self) -> usize {
        4 + (self.fixed_commitments.len() * C::default().to_bytes().as_ref().len())
            + self.permutation.bytes_length()
            + self.selectors.len()
                * (self
                    .selectors
                    .get(0)
                    .map(|selector| selector.len() / 8 + 1)
                    .unwrap_or(0))
    }

    fn from_parts(
        domain: EvaluationDomain<C::Scalar>,
        fixed_commitments: Vec<C>,
        permutation: permutation::VerifyingKey<C>,
        cs: ConstraintSystem<C::Scalar>,
        selectors: Vec<Vec<bool>>,
    ) -> Self {
        // Compute cached values.
        let cs_degree = cs.degree();

        let mut vk = Self {
            domain,
            fixed_commitments,
            permutation,
            cs,
            cs_degree,
            // Temporary, this is not pinned.
            transcript_repr: C::Scalar::zero(),
            selectors,
        };

        let mut hasher = Blake2bParams::new()
            .hash_length(64)
            .personal(b"Halo2-Verify-Key")
            .to_state();

        let s = format!("{:?}", vk.pinned());

        hasher.update(&(s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());

        // Hash in final Blake2bState
        vk.transcript_repr = C::Scalar::from_bytes_wide(hasher.finalize().as_array());

        vk
    }

    /// Hashes a verification key into a transcript.
    pub fn hash_into<E: EncodedChallenge<C>, T: Transcript<C, E>>(
        &self,
        transcript: &mut T,
    ) -> io::Result<()> {
        transcript.common_scalar(self.transcript_repr)?;

        Ok(())
    }

    /// Obtains a pinned representation of this verification key that contains
    /// the minimal information necessary to reconstruct the verification key.
    pub fn pinned(&self) -> PinnedVerificationKey<'_, C> {
        PinnedVerificationKey {
            base_modulus: C::Base::MODULUS,
            scalar_modulus: C::Scalar::MODULUS,
            domain: self.domain.pinned(),
            fixed_commitments: &self.fixed_commitments,
            permutation: &self.permutation,
            cs: self.cs.pinned(),
        }
    }

    /// Returns commitments of fixed polynomials
    pub fn fixed_commitments(&self) -> &Vec<C> {
        &self.fixed_commitments
    }

    /// Returns `VerifyingKey` of permutation
    pub fn permutation(&self) -> &permutation::VerifyingKey<C> {
        &self.permutation
    }

    /// Returns `ConstraintSystem`
    pub fn cs(&self) -> &ConstraintSystem<C::Scalar> {
        &self.cs
    }
}

/// Minimal representation of a verification key that can be used to identify
/// its active contents.
#[allow(dead_code)]
#[derive(Debug)]
pub struct PinnedVerificationKey<'a, C: CurveAffine> {
    base_modulus: &'static str,
    scalar_modulus: &'static str,
    domain: PinnedEvaluationDomain<'a, C::Scalar>,
    cs: PinnedConstraintSystem<'a, C::Scalar>,
    fixed_commitments: &'a Vec<C>,
    permutation: &'a permutation::VerifyingKey<C>,
}
/// This is a proving key which allows for the creation of proofs for a
/// particular circuit.
#[derive(Clone, Debug)]
pub struct ProvingKey<C: CurveAffine> {
    vk: VerifyingKey<C>,
    l0: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    l_last: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    l_active_row: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    fixed_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    fixed_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    fixed_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutation: permutation::ProvingKey<C>,
    ev: Evaluator<C>,
}

impl<C: CurveAffine> ProvingKey<C> {
    /// Get the underlying [`VerifyingKey`].
    pub fn get_vk(&self) -> &VerifyingKey<C> {
        &self.vk
    }

    /// Writes a proving key to a buffer.
    /// Does so by first writing the verifying key and then serializing the rest of the data (in the form of field polynomials)
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.vk.write(writer)?;
        self.l0.write(writer)?;
        self.l_last.write(writer)?;
        self.l_active_row.write(writer)?;
        write_polynomial_slice(&self.fixed_values, writer)?;
        write_polynomial_slice(&self.fixed_polys, writer)?;
        write_polynomial_slice(&self.fixed_cosets, writer)?;
        self.permutation.write(writer)?;
        Ok(())
    }

    /// Reads a proving key from a buffer.
    /// Does so by reading verification key first, and then deserializing the rest of the file into the remaining proving key data.
    pub fn read<'params, R: io::Read, ConcreteCircuit: Circuit<C::Scalar>>(
        reader: &mut R,
        params: &impl Params<'params, C>,
    ) -> io::Result<Self> {
        let vk = VerifyingKey::<C>::read::<R, ConcreteCircuit>(reader, params)?;
        let l0 = Polynomial::read(reader)?;
        let l_last = Polynomial::read(reader)?;
        let l_active_row = Polynomial::read(reader)?;
        let fixed_values = read_polynomial_vec(reader)?;
        let fixed_polys = read_polynomial_vec(reader)?;
        let fixed_cosets = read_polynomial_vec(reader)?;
        let permutation = permutation::ProvingKey::read(reader)?;
        let ev = Evaluator::new(vk.cs());
        Ok(Self {
            vk,
            l0,
            l_last,
            l_active_row,
            fixed_values,
            fixed_polys,
            fixed_cosets,
            permutation,
            ev,
        })
    }

    /// Writes a proving key to a vector of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::with_capacity(self.bytes_length());
        Self::write(self, &mut bytes).expect("Writing to vector should not fail");
        bytes
    }

    /// Reads a proving key from a slice of bytes.
    pub fn from_bytes<'params, ConcreteCircuit: Circuit<C::Scalar>>(
        mut bytes: &[u8],
        params: &impl Params<'params, C>,
    ) -> io::Result<Self> {
        Self::read::<_, ConcreteCircuit>(&mut bytes, params)
    }

    /// Gets the total number of bytes in the serialization of `self`
    fn bytes_length(&self) -> usize {
        let scalar_len = C::Scalar::default().to_repr().as_ref().len();
        self.vk.bytes_length()
            + 12
            + scalar_len * (self.l0.len() + self.l_last.len() + self.l_active_row.len())
            + polynomial_slice_byte_length(&self.fixed_values)
            + polynomial_slice_byte_length(&self.fixed_polys)
            + polynomial_slice_byte_length(&self.fixed_cosets)
            + self.permutation.bytes_length()
    }
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Get the underlying [`EvaluationDomain`].
    pub fn get_domain(&self) -> &EvaluationDomain<C::Scalar> {
        &self.domain
    }
}

#[derive(Clone, Copy, Debug)]
struct Theta;
type ChallengeTheta<F> = ChallengeScalar<F, Theta>;

#[derive(Clone, Copy, Debug)]
struct Beta;
type ChallengeBeta<F> = ChallengeScalar<F, Beta>;

#[derive(Clone, Copy, Debug)]
struct Gamma;
type ChallengeGamma<F> = ChallengeScalar<F, Gamma>;

#[derive(Clone, Copy, Debug)]
struct Y;
type ChallengeY<F> = ChallengeScalar<F, Y>;

#[derive(Clone, Copy, Debug)]
struct X;
type ChallengeX<F> = ChallengeScalar<F, X>;
