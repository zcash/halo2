use ff::Field;

use super::super::Error;
use super::{Params, MSM};
use crate::transcript::{Challenge, ChallengeScalar, TranscriptRead};

use crate::arithmetic::{best_multiexp, BatchInvert, Curve, CurveAffine};

/// A guard returned by the verifier
#[derive(Debug, Clone)]
pub struct Guard<'a, C: CurveAffine> {
    msm: MSM<'a, C>,
    neg_a: C::Scalar,
    challenges: Vec<C::Scalar>,
    challenges_packed: Vec<Challenge>,
}

/// An accumulator instance consisting of an evaluation claim and a proof.
#[derive(Debug, Clone)]
pub struct Accumulator<C: CurveAffine> {
    /// The claimed output of the linear-time polycommit opening protocol
    pub g: C,

    /// A vector of 128-bit challenges sampled by the verifier, to be used in
    /// computing g.
    pub challenges_packed: Vec<Challenge>,
}

impl<'a, C: CurveAffine> Guard<'a, C> {
    /// Lets caller supply the challenges and obtain an MSM with updated
    /// scalars and points.
    pub fn use_challenges(mut self) -> MSM<'a, C> {
        let s = compute_s(&self.challenges, self.neg_a);
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
            challenges_packed: self.challenges_packed,
        };

        (self.msm, accumulator)
    }

    /// Computes G + H, where G = ⟨s, params.g⟩ and H is used for blinding
    pub fn compute_g(&self) -> C {
        let s = compute_s(&self.challenges, C::Scalar::one());

        metrics::increment_counter!("multiexp", "size" => format!("{}", s.len()), "fn" => "compute_g");
        let mut tmp = best_multiexp(&s, &self.msm.params.g);
        tmp += self.msm.params.h;
        tmp.to_affine()
    }
}

/// Checks to see if an [`Proof`] is valid given the current `transcript`, and a
/// point `x` that the polynomial commitment `P` opens purportedly to the value
/// `v`. The provided `msm` should evaluate to the commitment `P` being opened.
pub fn verify_proof<'a, C: CurveAffine, T: TranscriptRead<C>>(
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

    let mut rounds = vec![];
    for _ in 0..k {
        // Read L and R from the proof and write them to the transcript
        let l = transcript.read_point().map_err(|_| Error::OpeningError)?;
        let r = transcript.read_point().map_err(|_| Error::OpeningError)?;

        let challenge_packed = Challenge::get(transcript);
        let challenge = *ChallengeScalar::<C, ()>::from(challenge_packed);

        rounds.push((
            l,
            r,
            challenge,
            /* to be inverted */ challenge,
            challenge_packed,
        ));
    }

    rounds
        .iter_mut()
        .map(|&mut (_, _, _, ref mut challenge, _)| challenge)
        .batch_invert();

    let mut challenges = Vec::with_capacity(k);
    let mut challenges_packed: Vec<Challenge> = Vec::with_capacity(k);
    for (l, r, challenge, challenge_inv, challenge_packed) in rounds {
        msm.append_term(challenge_inv, l);
        msm.append_term(challenge, r);

        challenges.push(challenge);
        challenges_packed.push(challenge_packed);
    }

    // Our goal is to open
    //     msm - [v] G_0 + random_poly_commitment * iota
    //   + \sum(L_i * u_i^2) + \sum(R_i * u_i^-2)
    // at x to 0, by asking the prover to supply (a, \xi) such that it equals
    //   = [a] (G + [b * z] U) + [\xi] H
    // except that we wish for the prover to supply G as Commit(g(X); 1) so
    // we must substitute to get
    //   = [a] ((G - H) + [b * z] U) + [\xi] H
    //   = [a] G + [-a] H + [abz] U + [\xi] H
    //   = [a] G + [abz] U + [\xi - a] H
    // but subtracting to get the desired equality
    //   ... + [-a] G + [-abz] U + [a - \xi] H = 0

    let a = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let neg_a = -a;
    let xi = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let b = compute_b(x, &challenges);

    msm.add_to_u_scalar(neg_a * &b * &z);
    msm.add_to_h_scalar(a - &xi);

    let guard = Guard {
        msm,
        neg_a,
        challenges,
        challenges_packed,
    };

    Ok(guard)
}

/// Computes $\prod\limits_{i=0}^{k-1} (1 + u_i x^{2^i})$.
fn compute_b<F: Field>(x: F, challenges: &[F]) -> F {
    let mut tmp = F::one();
    let mut cur = x;
    for challenge in challenges.iter().rev() {
        tmp *= F::one() + &(*challenge * &cur);
        cur *= cur;
    }
    tmp
}

/// Computes the coefficients of $g(X) = \prod\limits_{i=0}^{k-1} (1 + u_i X^{2^i})$.
fn compute_s<F: Field>(challenges: &[F], init: F) -> Vec<F> {
    assert!(!challenges.is_empty());
    let mut v = vec![F::zero(); 1 << challenges.len()];
    v[0] = init;

    for (len, challenge) in challenges
        .iter()
        .rev()
        .enumerate()
        .map(|(i, challenge)| (1 << i, challenge))
    {
        let (left, right) = v.split_at_mut(len);
        let right = &mut right[0..len];
        right.copy_from_slice(&left);
        for v in right {
            *v *= challenge;
        }
    }

    v
}
