use ff::Field;

use super::super::Error;
use super::{Params, MSM};
use crate::transcript::{Challenge, ChallengeScalar, TranscriptRead};

use crate::arithmetic::{best_multiexp, Curve, CurveAffine, FieldExt};

use std::io::Read;

/// A guard returned by the verifier
#[derive(Debug, Clone)]
pub struct Guard<'a, C: CurveAffine> {
    msm: MSM<'a, C>,
    neg_a: C::Scalar,
    allinv: C::Scalar,
    challenges_sq: Vec<C::Scalar>,
    challenges_sq_packed: Vec<Challenge>,
}

/// An accumulator instance consisting of an evaluation claim and a proof.
#[derive(Debug, Clone)]
pub struct Accumulator<C: CurveAffine> {
    /// The claimed output of the linear-time polycommit opening protocol
    pub g: C,

    /// A vector of 128-bit challenges sampled by the verifier, to be used in
    /// computing g.
    pub challenges_sq_packed: Vec<Challenge>,
}

impl<'a, C: CurveAffine> Guard<'a, C> {
    /// Lets caller supply the challenges and obtain an MSM with updated
    /// scalars and points.
    pub fn use_challenges(mut self) -> MSM<'a, C> {
        let s = compute_s(&self.challenges_sq, self.allinv * &self.neg_a);
        self.msm.add_to_g_scalars(&s);
        self.msm.add_to_h_scalar(self.neg_a);

        self.msm
    }

    /// Lets caller supply the purported G point and simply appends
    /// [-a] G to return an updated MSM.
    pub fn use_g(mut self, g: C) -> (MSM<'a, C>, Accumulator<C>) {
        self.msm.append_term(self.neg_a, g);

        let accumulator = Accumulator {
            g,
            challenges_sq_packed: self.challenges_sq_packed,
        };

        (self.msm, accumulator)
    }

    /// Computes G + H, where G = ⟨s, params.g⟩ and H is used for blinding
    pub fn compute_g(&self) -> C {
        let s = compute_s(&self.challenges_sq, self.allinv);

        metrics::increment_counter!("multiexp", "size" => format!("{}", s.len()), "fn" => "compute_g");
        let mut tmp = best_multiexp(&s, &self.msm.params.g);
        tmp += self.msm.params.h;
        tmp.to_affine()
    }
}

/// Checks to see if an [`Proof`] is valid given the current `transcript`, and a
/// point `x` that the polynomial commitment `P` opens purportedly to the value
/// `v`. The provided `msm` should evaluate to the commitment `P` being opened.
pub fn verify_proof<'a, C: CurveAffine, R: Read, T: TranscriptRead<R, C>>(
    params: &'a Params<C>,
    mut msm: MSM<'a, C>,
    transcript: &mut T,
    x: C::Scalar,
    v: C::Scalar,
) -> Result<Guard<'a, C>, Error> {
    let k = params.k as usize;

    //     P - [v] G_0 + S * iota
    //   + \sum(L_i * u_i^2) + \sum(R_i * u_i^-2)
    msm.add_constant_term(-v);
    let s_poly_commitment = transcript.read_point().map_err(|_| Error::OpeningError)?;

    let iota = *ChallengeScalar::<C, ()>::get(transcript);
    msm.append_term(iota, s_poly_commitment);

    let z = *ChallengeScalar::<C, ()>::get(transcript);

    // Data about the challenges from each of the rounds.
    let mut challenges = Vec::with_capacity(k);
    let mut challenges_inv = Vec::with_capacity(k);
    let mut challenges_sq = Vec::with_capacity(k);
    let mut challenges_sq_packed: Vec<Challenge> = Vec::with_capacity(k);
    let mut allinv = C::Scalar::one();

    for _ in 0..k {
        // Read L and R from the proof and write them to the transcript
        let l = transcript.read_point().map_err(|_| Error::OpeningError)?;
        let r = transcript.read_point().map_err(|_| Error::OpeningError)?;

        let challenge_sq_packed = Challenge::get(transcript);
        let challenge_sq = *ChallengeScalar::<C, ()>::from(challenge_sq_packed);

        let challenge = challenge_sq.deterministic_sqrt();
        if challenge.is_none() {
            // We didn't sample a square.
            return Err(Error::OpeningError);
        }
        let challenge = challenge.unwrap();

        let challenge_inv = challenge.invert();
        if bool::from(challenge_inv.is_none()) {
            // We sampled zero for some reason, unlikely to happen by
            // chance.
            return Err(Error::OpeningError);
        }
        let challenge_inv = challenge_inv.unwrap();
        allinv *= &challenge_inv;

        let challenge_sq_inv = challenge_inv.square();

        msm.append_term(challenge_sq, l);
        msm.append_term(challenge_sq_inv, r);

        challenges.push(challenge);
        challenges_inv.push(challenge_inv);
        challenges_sq.push(challenge_sq);
        challenges_sq_packed.push(challenge_sq_packed);
    }

    // Our goal is to open
    //     msm - [v] G_0 + random_poly_commitment * iota
    //   + \sum(L_i * u_i^2) + \sum(R_i * u_i^-2)
    // at x to 0, by asking the prover to supply (a, h) such that it equals
    //   = [a] (G + [b * z] U) + [h] H
    // except that we wish for the prover to supply G as Commit(g(X); 1) so
    // we must substitute to get
    //   = [a] ((G - H) + [b * z] U) + [h] H
    //   = [a] G + [-a] H + [abz] U + [h] H
    //   = [a] G + [abz] U + [h - a] H
    // but subtracting to get the desired equality
    //   ... + [-a] G + [-abz] U + [a - h] H = 0

    let a = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let neg_a = -a;
    let h = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let b = compute_b(x, &challenges, &challenges_inv);

    msm.add_to_u_scalar(neg_a * &b * &z);
    msm.add_to_h_scalar(a - &h);

    let guard = Guard {
        msm,
        neg_a,
        allinv,
        challenges_sq,
        challenges_sq_packed,
    };

    Ok(guard)
}

fn compute_b<F: Field>(x: F, challenges: &[F], challenges_inv: &[F]) -> F {
    assert!(!challenges.is_empty());
    assert_eq!(challenges.len(), challenges_inv.len());
    if challenges.len() == 1 {
        *challenges_inv.last().unwrap() + *challenges.last().unwrap() * x
    } else {
        (*challenges_inv.last().unwrap() + *challenges.last().unwrap() * x)
            * compute_b(
                x.square(),
                &challenges[0..(challenges.len() - 1)],
                &challenges_inv[0..(challenges.len() - 1)],
            )
    }
}

// TODO: parallelize
fn compute_s<F: Field>(challenges_sq: &[F], allinv: F) -> Vec<F> {
    let lg_n = challenges_sq.len();
    let n = 1 << lg_n;

    let mut s = Vec::with_capacity(n);
    s.push(allinv);
    for i in 1..n {
        let lg_i = (32 - 1 - (i as u32).leading_zeros()) as usize;
        let k = 1 << lg_i;
        let u_lg_i_sq = challenges_sq[(lg_n - 1) - lg_i];
        s.push(s[i - k] * u_lg_i_sq);
    }

    s
}
