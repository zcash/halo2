//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use blake2b_simd::Params as Blake2bParams;
use group::ff::{Field, FromUniformBytes, PrimeField};

use crate::arithmetic::CurveAffine;
use crate::poly::{
    commitment::Params, Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff,
    PinnedEvaluationDomain, Polynomial,
};
use crate::transcript::{ChallengeScalar, EncodedChallenge, Transcript};

mod assigned;
mod circuit;
mod error;
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

use crate::helpers::CurveRead;
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

impl<C: CurveAffine> VerifyingKey<C>
where
    C::Scalar: FromUniformBytes<64>,
{
    /// Writes a verifying key to a buffer.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        // Version byte that will be checked on read.
        writer.write_all(&[0x01])?;

        writer.write_all(&(u32::try_from(self.fixed_commitments.len()).unwrap()).to_le_bytes())?;
        for commitment in &self.fixed_commitments {
            writer.write_all(commitment.to_bytes().as_ref())?;
        }
        self.permutation.write(writer)?;

        // write self.selectors
        writer.write_all(&(u32::try_from(self.selectors.len()).unwrap()).to_le_bytes())?;
        for selector in &self.selectors {
            let mut selector_bytes = vec![0u8; (selector.len() + 7) / 8];
            for (i, selector_idx) in selector.iter().enumerate() {
                let byte_index = i / 8;
                let bit_index = i % 8;
                selector_bytes[byte_index] |= (*selector_idx as u8) << bit_index;
            }
            writer.write_all(&selector_bytes)?;
        }

        Ok(())
    }

    /// Reads a verification key from a buffer.
    pub fn read<R: io::Read, ConcreteCircuit: Circuit<C::Scalar>>(
        reader: &mut R,
        params: &Params<C>,
    ) -> io::Result<Self> {
        let (domain, cs, _) = keygen::create_domain::<C, ConcreteCircuit>(params);

        let mut version_byte = [0u8; 1];
        reader.read_exact(&mut version_byte)?;
        if 0x01 != version_byte[0] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected version byte",
            ));
        }

        let mut num_fixed_columns_le_bytes = [0u8; 4];
        reader.read_exact(&mut num_fixed_columns_le_bytes)?;
        let num_fixed_columns = u32::from_le_bytes(num_fixed_columns_le_bytes);

        let fixed_commitments: Vec<_> = (0..num_fixed_columns)
            .map(|_| C::read(reader))
            .collect::<io::Result<_>>()?;

        let permutation = permutation::VerifyingKey::read(reader, &cs.permutation)?;

        // read selectors
        let mut num_selectors_le_bytes = [0u8; 4];
        reader.read_exact(&mut num_selectors_le_bytes)?;
        let num_selectors = u32::from_le_bytes(num_selectors_le_bytes);
        if cs.num_selectors != num_selectors.try_into().unwrap() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected number of selectors",
            ));
        }
        let selectors: Vec<Vec<bool>> = vec![vec![false; params.n as usize]; cs.num_selectors]
            .into_iter()
            .map(|mut selector| {
                let mut selector_bytes = vec![0u8; (selector.len() + 7) / 8];
                reader.read_exact(&mut selector_bytes)?;
                for (i, selector_idx) in selector.iter_mut().enumerate() {
                    let byte_index = i / 8;
                    let bit_index = i % 8;
                    *selector_idx = (selector_bytes[byte_index] >> bit_index) & 1 == 1;
                }
                Ok(selector)
            })
            .collect::<io::Result<_>>()?;

        let (cs, _) = cs.compress_selectors(selectors.clone());

        Ok(Self::from_parts(
            domain,
            fixed_commitments,
            permutation,
            cs,
            selectors,
        ))
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
            transcript_repr: C::Scalar::ZERO,
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
        vk.transcript_repr = C::Scalar::from_uniform_bytes(hasher.finalize().as_array());

        vk
    }
}

impl<C: CurveAffine> VerifyingKey<C> {
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
    l_blind: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    l_last: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    fixed_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    fixed_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    fixed_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutation: permutation::ProvingKey<C>,
}

impl<C: CurveAffine> ProvingKey<C> {
    /// Get the underlying [`VerifyingKey`].
    pub fn get_vk(&self) -> &VerifyingKey<C> {
        &self.vk
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
