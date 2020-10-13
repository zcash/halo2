//! This module contains an implementation of the polynomial commitment scheme
//! described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use super::{Coeff, LagrangeCoeff, Polynomial};
use crate::arithmetic::{best_fft, best_multiexp, parallelize, Curve, CurveAffine, Field};
use crate::transcript::Hasher;
use std::ops::{Add, AddAssign, Mul, MulAssign};

mod msm;
mod prover;
mod verifier;

pub use msm::MSM;
pub use verifier::{Accumulator, Guard};

/// These are the public parameters for the polynomial commitment scheme.
#[derive(Debug)]
pub struct Params<C: CurveAffine> {
    pub(crate) k: u32,
    pub(crate) n: u64,
    pub(crate) g: Vec<C>,
    pub(crate) g_lagrange: Vec<C>,
    pub(crate) h: C,
}

/// This is a proof object for the polynomial commitment scheme opening.
#[derive(Debug, Clone)]
pub struct Proof<C: CurveAffine> {
    rounds: Vec<(C, C)>,
    delta: C,
    z1: C::Scalar,
    z2: C::Scalar,
}

impl<C: CurveAffine> Params<C> {
    /// Initializes parameters for the curve, given a random oracle to draw
    /// points from.
    pub fn new<H: Hasher<C::Base>>(k: u32) -> Self {
        // This is usually a limitation on the curve, but we also want 32-bit
        // architectures to be supported.
        assert!(k < 32);
        // No goofy hardware please.
        assert!(core::mem::size_of::<usize>() >= 4);

        let n: u64 = 1 << k;

        let g = {
            let hasher = &H::init(C::Base::zero());

            let mut g = Vec::with_capacity(n as usize);
            g.resize(n as usize, C::zero());

            parallelize(&mut g, move |g, start| {
                let mut cur_value = C::Base::from(start as u64);
                for g in g.iter_mut() {
                    let mut hasher = hasher.clone();
                    hasher.absorb(cur_value);
                    cur_value += &C::Base::one();
                    loop {
                        let x = hasher.squeeze().to_bytes();
                        let p = C::from_bytes(&x);
                        if bool::from(p.is_some()) {
                            *g = p.unwrap();
                            break;
                        }
                    }
                }
            });

            g
        };

        // Let's evaluate all of the Lagrange basis polynomials
        // using an inverse FFT.
        let mut alpha_inv = C::Scalar::ROOT_OF_UNITY_INV;
        for _ in k..C::Scalar::S {
            alpha_inv = alpha_inv.square();
        }
        let mut g_lagrange_projective = g.iter().map(|g| g.to_projective()).collect::<Vec<_>>();
        best_fft(&mut g_lagrange_projective, alpha_inv, k);
        let minv = C::Scalar::TWO_INV.pow_vartime(&[k as u64, 0, 0, 0]);
        parallelize(&mut g_lagrange_projective, |g, _| {
            for g in g.iter_mut() {
                *g *= minv;
            }
        });

        let g_lagrange = {
            let mut g_lagrange = vec![C::zero(); n as usize];
            parallelize(&mut g_lagrange, |g_lagrange, starts| {
                C::Projective::batch_to_affine(
                    &g_lagrange_projective[starts..(starts + g_lagrange.len())],
                    g_lagrange,
                );
            });
            drop(g_lagrange_projective);
            g_lagrange
        };

        let h = {
            let mut hasher = H::init(C::Base::zero());
            hasher.absorb(-C::Base::one());
            let x = hasher.squeeze().to_bytes();
            let p = C::from_bytes(&x);
            p.unwrap()
        };

        Params {
            k,
            n,
            g,
            g_lagrange,
            h,
        }
    }

    /// This computes a commitment to a polynomial described by the provided
    /// slice of coefficients. The commitment will be blinded by the blinding
    /// factor `r`.
    pub fn commit(
        &self,
        poly: &Polynomial<C::Scalar, Coeff>,
        r: Blind<C::Scalar>,
    ) -> C::Projective {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r.0);

        tmp_bases.extend(self.g.iter());
        tmp_bases.push(self.h);

        best_multiexp::<C>(&tmp_scalars, &tmp_bases)
    }

    /// This commits to a polynomial using its evaluations over the $2^k$ size
    /// evaluation domain. The commitment will be blinded by the blinding factor
    /// `r`.
    pub fn commit_lagrange(
        &self,
        poly: &Polynomial<C::Scalar, LagrangeCoeff>,
        r: Blind<C::Scalar>,
    ) -> C::Projective {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r.0);

        tmp_bases.extend(self.g_lagrange.iter());
        tmp_bases.push(self.h);

        best_multiexp::<C>(&tmp_scalars, &tmp_bases)
    }

    /// Generates an empty multiscalar multiplication struct using the
    /// appropriate params.
    pub fn empty_msm(&self) -> MSM<C> {
        MSM::new(self)
    }

    /// Getter for g generators
    pub fn get_g(&self) -> Vec<C> {
        self.g.clone()
    }
}

/// Wrapper type around a blinding factor.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Blind<F>(pub F);

impl<F: Field> Default for Blind<F> {
    fn default() -> Self {
        Blind(F::one())
    }
}

impl<F: Field> Add for Blind<F> {
    type Output = Self;

    fn add(self, rhs: Blind<F>) -> Self {
        Blind(self.0 + rhs.0)
    }
}

impl<F: Field> Mul for Blind<F> {
    type Output = Self;

    fn mul(self, rhs: Blind<F>) -> Self {
        Blind(self.0 * rhs.0)
    }
}

impl<F: Field> AddAssign for Blind<F> {
    fn add_assign(&mut self, rhs: Blind<F>) {
        self.0 += rhs.0;
    }
}

impl<F: Field> MulAssign for Blind<F> {
    fn mul_assign(&mut self, rhs: Blind<F>) {
        self.0 *= rhs.0;
    }
}

impl<F: Field> AddAssign<F> for Blind<F> {
    fn add_assign(&mut self, rhs: F) {
        self.0 += rhs;
    }
}

impl<F: Field> MulAssign<F> for Blind<F> {
    fn mul_assign(&mut self, rhs: F) {
        self.0 *= rhs;
    }
}

#[test]
fn test_commit_lagrange() {
    const K: u32 = 6;

    use crate::arithmetic::{EpAffine, Fp, Fq};
    use crate::transcript::DummyHash;
    let params = Params::<EpAffine>::new::<DummyHash<Fp>>(K);
    let domain = super::EvaluationDomain::new(1, K);

    let mut a = domain.empty_lagrange();

    for (i, a) in a.iter_mut().enumerate() {
        *a = Fq::from(i as u64);
    }

    let b = domain.lagrange_to_coeff(a.clone());

    let alpha = Blind(Fq::random());

    assert_eq!(params.commit(&b, alpha), params.commit_lagrange(&a, alpha));
}

#[test]
fn test_opening_proof() {
    const K: u32 = 6;

    use super::{
        commitment::{Blind, Params},
        EvaluationDomain,
    };
    use crate::arithmetic::{
        eval_polynomial, get_challenge_scalar, Challenge, Curve, EpAffine, Field, Fp, Fq,
    };
    use crate::transcript::{DummyHash, Hasher};

    let params = Params::<EpAffine>::new::<DummyHash<Fp>>(K);
    let domain = EvaluationDomain::new(1, K);

    let mut px = domain.empty_coeff();

    for (i, a) in px.iter_mut().enumerate() {
        *a = Fq::from(i as u64);
    }

    let blind = Blind(Fq::random());

    let p = params.commit(&px, blind).to_affine();

    let mut transcript = DummyHash::init(Field::one());
    let (p_x, p_y) = p.get_xy().unwrap();
    transcript.absorb(p_x);
    transcript.absorb(p_y);
    let x_packed = transcript.squeeze().get_lower_128();
    let x: Fq = get_challenge_scalar(Challenge(x_packed));

    // Evaluate the polynomial
    let v = eval_polynomial(&px, x);

    transcript.absorb(Fp::from_bytes(&v.to_bytes()).unwrap()); // unlikely to fail since p ~ q

    loop {
        let transcript_dup = transcript.clone();

        let opening_proof = Proof::create(&params, &mut transcript, &px, blind, x);
        if opening_proof.is_err() {
            transcript = transcript_dup;
            transcript.absorb(Field::one());
        } else {
            let opening_proof = opening_proof.unwrap();
            // Verify the opening proof
            let mut commitment_msm = params.empty_msm();
            commitment_msm.add_term(Field::one(), p);
            let guard = opening_proof
                .verify(
                    &params,
                    params.empty_msm(),
                    &mut transcript_dup.clone(),
                    x,
                    commitment_msm,
                    v,
                )
                .unwrap();

            // Test guard behavior prior to checking another proof
            {
                // Test use_challenges()
                let msm_challenges = guard.clone().use_challenges();
                assert!(msm_challenges.eval());

                // Test use_g()
                let g = guard.compute_g();
                let (msm_g, _accumulator) = guard.clone().use_g(g);
                assert!(msm_g.eval());
            }

            // Check another proof to populate `msm.g_scalars`
            let msm = guard.use_challenges();
            let mut commitment_msm = params.empty_msm();
            commitment_msm.add_term(Field::one(), p);
            let guard = opening_proof
                .verify(
                    &params,
                    msm,
                    &mut transcript_dup.clone(),
                    x,
                    commitment_msm,
                    v,
                )
                .unwrap();

            // Test use_challenges()
            let msm_challenges = guard.clone().use_challenges();
            assert!(msm_challenges.eval());

            // Test use_g()
            let g = guard.compute_g();
            let (msm_g, _accumulator) = guard.clone().use_g(g);
            assert!(msm_g.eval());

            break;
        }
    }
}
