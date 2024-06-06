//! This module contains an implementation of the polynomial commitment scheme
//! described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use crate::arithmetic::{g_to_lagrange, parallelize, CurveAffine, CurveExt};
use crate::helpers::CurveRead;
use crate::poly::commitment::{Blind, CommitmentScheme, Params, ParamsProver, ParamsVerifier};
use crate::poly::ipa::msm::MSMIPA;
use crate::poly::{Coeff, LagrangeCoeff, Polynomial};

use group::{Curve, Group};
use halo2_middleware::zal::traits::MsmAccel;
use rand_core::RngCore;
use std::marker::PhantomData;

mod prover;
mod verifier;

pub use prover::create_proof_with_engine;
pub use verifier::verify_proof;

use std::io;

/// Public parameters for IPA commitment scheme
#[derive(Debug, Clone)]
pub struct ParamsIPA<C: CurveAffine> {
    pub(crate) k: u32,
    pub(crate) n: u64,
    pub(crate) g: Vec<C>,
    pub(crate) g_lagrange: Vec<C>,
    pub(crate) w: C,
    pub(crate) u: C,
}

/// Concrete IPA commitment scheme
#[derive(Debug)]
pub struct IPACommitmentScheme<C: CurveAffine> {
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> CommitmentScheme for IPACommitmentScheme<C> {
    type Scalar = C::ScalarExt;
    type Curve = C;

    type ParamsProver = ParamsIPA<C>;
    type ParamsVerifier = ParamsVerifierIPA<C>;

    fn new_params(k: u32, rng: impl RngCore) -> Self::ParamsProver {
        ParamsIPA::new(k, rng)
    }

    fn read_params<R: io::Read>(reader: &mut R) -> io::Result<Self::ParamsProver> {
        ParamsIPA::read(reader)
    }
}

/// Verifier parameters
pub type ParamsVerifierIPA<C> = ParamsIPA<C>;

impl<'params, C: CurveAffine> ParamsVerifier<'params, C> for ParamsIPA<C> {
    type MSM = MSMIPA<'params, C>;

    // IPA params always support commitment.
    const COMMIT_INSTANCE: bool = true;

    fn empty_msm(&self) -> MSMIPA<C> {
        MSMIPA::new(self)
    }
}

impl<C: CurveAffine> Params<C> for ParamsIPA<C> {
    fn k(&self) -> u32 {
        self.k
    }

    fn n(&self) -> u64 {
        self.n
    }

    fn downsize(&mut self, k: u32) {
        assert!(k <= self.k);

        self.k = k;
        self.n = 1 << k;
        self.g.truncate(self.n as usize);
        self.g_lagrange = g_to_lagrange(self.g.iter().map(|g| g.to_curve()).collect(), k);
    }

    /// This commits to a polynomial using its evaluations over the $2^k$ size
    /// evaluation domain. The commitment will be blinded by the blinding factor
    /// `r`.
    fn commit_lagrange(
        &self,
        engine: &impl MsmAccel<C>,
        poly: &Polynomial<C::Scalar, LagrangeCoeff>,
        r: Blind<C::Scalar>,
    ) -> C::Curve {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r.0);

        tmp_bases.extend(self.g_lagrange.iter());
        tmp_bases.push(self.w);

        engine.msm(&tmp_scalars, &tmp_bases)
    }

    /// Writes params to a buffer.
    fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.k.to_le_bytes())?;
        for g_element in &self.g {
            writer.write_all(g_element.to_bytes().as_ref())?;
        }
        for g_lagrange_element in &self.g_lagrange {
            writer.write_all(g_lagrange_element.to_bytes().as_ref())?;
        }
        writer.write_all(self.w.to_bytes().as_ref())?;
        writer.write_all(self.u.to_bytes().as_ref())?;

        Ok(())
    }

    /// Reads params from a buffer.
    fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut k = [0u8; 4];
        reader.read_exact(&mut k[..])?;
        let k = u32::from_le_bytes(k);

        let n: u64 = 1 << k;

        let g: Vec<_> = (0..n).map(|_| C::read(reader)).collect::<Result<_, _>>()?;
        let g_lagrange: Vec<_> = (0..n).map(|_| C::read(reader)).collect::<Result<_, _>>()?;

        let w = C::read(reader)?;
        let u = C::read(reader)?;

        Ok(Self {
            k,
            n,
            g,
            g_lagrange,
            w,
            u,
        })
    }
}

impl<C: CurveAffine> ParamsProver<C> for ParamsIPA<C> {
    /// Initializes parameters for the curve, given a random oracle to draw
    /// points from.
    fn new(k: u32, _: impl RngCore) -> Self {
        // This is usually a limitation on the curve, but we also want 32-bit
        // architectures to be supported.
        assert!(k < 32);

        // In src/arithmetic/fields.rs we ensure that usize is at least 32 bits.

        let n: u64 = 1 << k;

        let g_projective = {
            let mut g = Vec::with_capacity(n as usize);
            g.resize(n as usize, C::Curve::identity());

            parallelize(&mut g, move |g, start| {
                let hasher = C::CurveExt::hash_to_curve("Halo2-Parameters");

                for (i, g) in g.iter_mut().enumerate() {
                    let i = (i + start) as u32;

                    let mut message = [0u8; 5];
                    message[1..5].copy_from_slice(&i.to_le_bytes());

                    *g = hasher(&message);
                }
            });

            g
        };

        let g = {
            let mut g = vec![C::identity(); n as usize];
            parallelize(&mut g, |g, starts| {
                C::Curve::batch_normalize(&g_projective[starts..(starts + g.len())], g);
            });
            g
        };

        // Let's evaluate all of the Lagrange basis polynomials
        // using an inverse FFT.
        let g_lagrange = g_to_lagrange(g_projective, k);

        let hasher = C::CurveExt::hash_to_curve("Halo2-Parameters");

        let [w, u] = {
            let projectives = vec![hasher(&[1]), hasher(&[2])];
            let mut affines = [C::identity(); 2];
            C::CurveExt::batch_normalize(&projectives, &mut affines);
            affines
        };

        ParamsIPA {
            k,
            n,
            g,
            g_lagrange,
            w,
            u,
        }
    }

    /// This computes a commitment to a polynomial described by the provided
    /// slice of coefficients. The commitment will be blinded by the blinding
    /// factor `r`.
    fn commit(
        &self,
        engine: &impl MsmAccel<C>,
        poly: &Polynomial<C::Scalar, Coeff>,
        r: Blind<C::Scalar>,
    ) -> C::Curve {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r.0);

        tmp_bases.extend(self.g.iter());
        tmp_bases.push(self.w);

        engine.msm(&tmp_scalars, &tmp_bases)
    }
}

#[cfg(test)]
mod test {
    use crate::poly::commitment::ParamsProver;
    use crate::poly::commitment::{Blind, Params, MSM};
    use crate::poly::ipa::commitment::{create_proof_with_engine, verify_proof, ParamsIPA};
    use crate::poly::ipa::msm::MSMIPA;

    use group::Curve;
    use halo2_middleware::ff::Field;
    use halo2_middleware::zal::impls::H2cEngine;

    #[test]
    fn test_commit_lagrange_epaffine() {
        const K: u32 = 6;

        use rand_core::OsRng;

        use crate::poly::EvaluationDomain;
        use halo2curves::pasta::{EpAffine, Fq};

        let engine = H2cEngine::new();
        let params = ParamsIPA::<EpAffine>::new(K, OsRng);
        let domain = EvaluationDomain::new(1, K);

        let mut a = domain.empty_lagrange();

        for (i, a) in a.iter_mut().enumerate() {
            *a = Fq::from(i as u64);
        }

        let b = domain.lagrange_to_coeff(a.clone());

        let alpha = Blind(Fq::random(OsRng));

        assert_eq!(
            params.commit(&engine, &b, alpha),
            params.commit_lagrange(&engine, &a, alpha)
        );
    }

    #[test]
    fn test_commit_lagrange_eqaffine() {
        const K: u32 = 6;

        use rand_core::OsRng;

        use crate::poly::EvaluationDomain;
        use halo2curves::pasta::{EqAffine, Fp};

        let engine = H2cEngine::new();
        let params: ParamsIPA<EqAffine> = ParamsIPA::<EqAffine>::new(K, OsRng);
        let domain = EvaluationDomain::new(1, K);

        let mut a = domain.empty_lagrange();

        for (i, a) in a.iter_mut().enumerate() {
            *a = Fp::from(i as u64);
        }

        let b = domain.lagrange_to_coeff(a.clone());

        let alpha = Blind(Fp::random(OsRng));

        assert_eq!(
            params.commit(&engine, &b, alpha),
            params.commit_lagrange(&engine, &a, alpha)
        );
    }

    #[test]
    fn test_opening_proof() {
        const K: u32 = 6;

        use halo2_middleware::ff::Field;
        use rand_core::OsRng;

        use super::super::commitment::{Blind, Params};
        use crate::arithmetic::eval_polynomial;
        use crate::poly::EvaluationDomain;
        use crate::transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, Transcript, TranscriptRead, TranscriptWrite,
        };
        use halo2curves::pasta::{EpAffine, Fq};

        use crate::transcript::TranscriptReadBuffer;
        use crate::transcript::TranscriptWriterBuffer;

        let rng = OsRng;

        let engine = H2cEngine::new();
        let params = ParamsIPA::<EpAffine>::new(K, OsRng);
        let mut params_buffer = vec![];
        <ParamsIPA<_> as Params<_>>::write(&params, &mut params_buffer).unwrap();
        let params: ParamsIPA<EpAffine> = Params::read::<_>(&mut &params_buffer[..]).unwrap();

        let domain = EvaluationDomain::new(1, K);

        let mut px = domain.empty_coeff();

        for (i, a) in px.iter_mut().enumerate() {
            *a = Fq::from(i as u64);
        }

        let blind = Blind(Fq::random(rng));

        let p = params.commit(&engine, &px, blind).to_affine();

        let mut transcript =
            Blake2bWrite::<Vec<u8>, EpAffine, Challenge255<EpAffine>>::init(vec![]);
        transcript.write_point(p).unwrap();
        let x = transcript.squeeze_challenge_scalar::<()>();
        // Evaluate the polynomial
        let v = eval_polynomial(&px, *x);
        transcript.write_scalar(v).unwrap();

        let (proof, ch_prover) = {
            create_proof_with_engine(&engine, &params, rng, &mut transcript, &px, blind, *x)
                .unwrap();
            let ch_prover = transcript.squeeze_challenge();
            (transcript.finalize(), ch_prover)
        };

        // Verify the opening proof
        let mut transcript =
            Blake2bRead::<&[u8], EpAffine, Challenge255<EpAffine>>::init(&proof[..]);
        let p_prime = transcript.read_point().unwrap();
        assert_eq!(p, p_prime);
        let x_prime = transcript.squeeze_challenge_scalar::<()>();
        assert_eq!(*x, *x_prime);
        let v_prime = transcript.read_scalar().unwrap();
        assert_eq!(v, v_prime);

        let mut commitment_msm = MSMIPA::new(&params);
        commitment_msm.append_term(Fq::one(), p.into());

        let guard = verify_proof(commitment_msm, &mut transcript, *x, v).unwrap();
        let ch_verifier = transcript.squeeze_challenge();
        assert_eq!(*ch_prover, *ch_verifier);

        // Test guard behavior prior to checking another proof
        {
            // Test use_challenges()
            let msm_challenges = guard.clone().use_challenges();
            assert!(msm_challenges.check(&engine));

            // Test use_g()
            let g = guard.compute_g(&engine);
            let (msm_g, _accumulator) = guard.clone().use_g(g);
            assert!(msm_g.check(&engine));
        }
    }
}
