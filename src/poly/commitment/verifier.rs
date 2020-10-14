use super::super::Error;
use super::{Params, Proof, MSM};
use crate::transcript::{Hasher, Transcript};

use crate::arithmetic::{
    best_multiexp, get_challenge_scalar, Challenge, Curve, CurveAffine, Field,
};

/// A guard returned by the verifier
#[derive(Debug, Clone)]
pub struct Guard<'a, C: CurveAffine> {
    msm: MSM<'a, C>,
    neg_z1: C::Scalar,
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
        let s = compute_s(&self.challenges_sq, self.allinv * &self.neg_z1);
        self.msm.add_to_g(&s);
        self.msm.add_to_h(self.neg_z1);

        self.msm
    }

    /// Lets caller supply the purported G point and simply appends it to
    /// return an updated MSM.
    pub fn use_g(mut self, g: C) -> (MSM<'a, C>, Accumulator<C>) {
        self.msm.add_term(self.neg_z1, g);

        let accumulator = Accumulator {
            g,
            challenges_sq_packed: self.challenges_sq_packed,
        };

        (self.msm, accumulator)
    }

    /// Computes the g value when given a potential scalar as input.
    pub fn compute_g(&self) -> C {
        let s = compute_s(&self.challenges_sq, self.allinv);

        let mut tmp = best_multiexp(&s, &self.msm.params.g);
        tmp += self.msm.params.h;
        tmp.to_affine()
    }
}

impl<C: CurveAffine> Proof<C> {
    /// Checks to see if an [`Proof`] is valid given the current `transcript`,
    /// and a point `x` that the polynomial commitment `p` opens purportedly to
    /// the value `v`.
    pub fn verify<'a, HBase, HScalar>(
        &self,
        params: &'a Params<C>,
        mut msm: MSM<'a, C>,
        transcript: &mut Transcript<C, HBase, HScalar>,
        x: C::Scalar,
        mut commitment_msm: MSM<'a, C>,
        v: C::Scalar,
    ) -> Result<Guard<'a, C>, Error>
    where
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    {
        // Check for well-formedness
        if self.rounds.len() != params.k as usize {
            return Err(Error::OpeningError);
        }

        // Compute U
        let u = {
            let u_x = transcript.squeeze();
            // y^2 = x^3 + B
            let u_y2 = u_x.square() * &u_x + &C::b();
            let u_y = u_y2.deterministic_sqrt();
            if u_y.is_none() {
                return Err(Error::OpeningError);
            }
            let u_y = u_y.unwrap();

            C::from_xy(u_x, u_y).unwrap()
        };

        let mut extra_scalars = Vec::with_capacity(self.rounds.len() * 2 + 4 + params.n as usize);
        let mut extra_bases = Vec::with_capacity(self.rounds.len() * 2 + 4 + params.n as usize);

        // Data about the challenges from each of the rounds.
        let mut challenges = Vec::with_capacity(self.rounds.len());
        let mut challenges_inv = Vec::with_capacity(self.rounds.len());
        let mut challenges_sq = Vec::with_capacity(self.rounds.len());
        let mut challenges_sq_packed: Vec<Challenge> = Vec::with_capacity(self.rounds.len());
        let mut allinv = C::Scalar::one();

        for round in &self.rounds {
            // Feed L and R into the transcript.
            let l = round.0;
            let r = round.1;
            if bool::from(l.get_xy().is_none() | r.get_xy().is_none()) {
                return Err(Error::OpeningError);
            }
            transcript.absorb_point(&l).ok();
            transcript.absorb_point(&r).ok();
            let challenge_sq_packed = transcript.squeeze().get_lower_128();
            let challenge_sq: C::Scalar = get_challenge_scalar(Challenge(challenge_sq_packed));

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

            extra_scalars.push(challenge_sq);
            extra_bases.push(round.0);
            extra_scalars.push(challenge_sq_inv);
            extra_bases.push(round.1);

            challenges.push(challenge);
            challenges_inv.push(challenge_inv);
            challenges_sq.push(challenge_sq);
            challenges_sq_packed.push(Challenge(challenge_sq_packed));
        }

        let delta = self.delta;
        if bool::from(delta.get_xy().is_none()) {
            return Err(Error::OpeningError);
        }

        // Feed delta into the transcript
        transcript.absorb_point(&delta).ok();

        // Get the challenge `c`
        let c_packed = transcript.squeeze().get_lower_128();
        let c: C::Scalar = get_challenge_scalar(Challenge(c_packed));

        // Check
        // [c] P + [c * v] U + [c] sum(L_i * u_i^2) + [c] sum(R_i * u_i^-2) + delta - [z1] G - [z1 * b] U - [z1 - z2] H
        // = 0

        let b = compute_b(x, &challenges, &challenges_inv);

        let neg_z1 = -self.z1;

        // [c] P
        commitment_msm.scale(c);
        msm.add_msm(&commitment_msm);

        for scalar in &mut extra_scalars {
            *scalar *= &c;
        }

        for (scalar, base) in extra_scalars.iter().zip(extra_bases.iter()) {
            msm.add_term(*scalar, *base);
        }

        // [c * v] U - [z1 * b] U
        msm.add_term((c * &v) + &(neg_z1 * &b), u);

        // delta
        msm.add_term(Field::one(), self.delta);

        // - [z1 - z2] H
        msm.add_to_h(self.z1 - &self.z2);

        let guard = Guard {
            msm,
            neg_z1,
            allinv,
            challenges_sq,
            challenges_sq_packed,
        };

        Ok(guard)
    }
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
