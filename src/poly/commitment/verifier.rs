use super::{OpeningProof, Params};
use crate::transcript::Hasher;

use crate::arithmetic::{
    best_multiexp, get_challenge_scalar, Challenge, Curve, CurveAffine, Field,
};

impl<C: CurveAffine> OpeningProof<C> {
    /// Checks to see if an [`OpeningProof`] is valid given the current
    /// `transcript`, and a point `x` that the polynomial commitment `p` opens
    /// purportedly to the value `v`.
    pub fn verify<H: Hasher<C::Base>>(
        &self,
        params: &Params<C>,
        transcript: &mut H,
        x: C::Scalar,
        p: &C,
        v: C::Scalar,
    ) -> bool {
        // Check for well-formedness
        if self.rounds.len() != params.k as usize {
            return false;
        }

        transcript.absorb(C::Base::from_u64(self.fork as u64));

        // Compute U
        let u = {
            let u_x = transcript.squeeze();
            // y^2 = x^3 + B
            let u_y2 = u_x.square() * &u_x + &C::b();
            let u_y = u_y2.deterministic_sqrt();
            if u_y.is_none() {
                return false;
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
        let mut allinv = Field::one();

        for round in &self.rounds {
            // Feed L and R into the transcript.
            let l = round.0.get_xy();
            let r = round.1.get_xy();
            if bool::from(l.is_none() | r.is_none()) {
                return false;
            }
            let l = l.unwrap();
            let r = r.unwrap();
            transcript.absorb(l.0);
            transcript.absorb(l.1);
            transcript.absorb(r.0);
            transcript.absorb(r.1);
            let challenge_sq_packed = transcript.squeeze().get_lower_128();
            let challenge_sq: C::Scalar = get_challenge_scalar(Challenge(challenge_sq_packed));

            let challenge = challenge_sq.deterministic_sqrt();
            if challenge.is_none() {
                // We didn't sample a square.
                return false;
            }
            let challenge = challenge.unwrap();

            let challenge_inv = challenge.invert();
            if bool::from(challenge_inv.is_none()) {
                // We sampled zero for some reason, unlikely to happen by
                // chance.
                return false;
            }
            let challenge_inv = challenge_inv.unwrap();
            allinv *= challenge_inv;

            let challenge_sq_inv = challenge_inv.square();

            extra_scalars.push(challenge_sq);
            extra_bases.push(round.0);
            extra_scalars.push(challenge_sq_inv);
            extra_bases.push(round.1);

            challenges.push(challenge);
            challenges_inv.push(challenge_inv);
            challenges_sq.push(challenge_sq);
        }

        let delta = self.delta.get_xy();
        if bool::from(delta.is_none()) {
            return false;
        }
        let delta = delta.unwrap();

        // Feed delta into the transcript
        transcript.absorb(delta.0);
        transcript.absorb(delta.1);

        // Get the challenge `c`
        let c_packed = transcript.squeeze().get_lower_128();
        let c: C::Scalar = get_challenge_scalar(Challenge(c_packed));

        // Check
        // [c] P + [c * v] U + [c] sum(L_i * u_i^2) + [c] sum(R_i * u_i^-2) + delta - [z1] G - [z1 * b] U - [z2] H
        // = 0

        for scalar in &mut extra_scalars {
            *scalar *= &c;
        }

        let b = compute_b(x, &challenges, &challenges_inv);

        let neg_z1 = -self.z1;

        // [c] P
        extra_bases.push(*p);
        extra_scalars.push(c);

        // [c * v] U - [z1 * b] U
        extra_bases.push(u);
        extra_scalars.push((c * &v) + &(neg_z1 * &b));

        // delta
        extra_bases.push(self.delta);
        extra_scalars.push(Field::one());

        // - [z2] H
        extra_bases.push(params.h);
        extra_scalars.push(-self.z2);

        // - [z1] G
        extra_bases.extend(&params.g);
        let mut s = compute_s(&challenges_sq, allinv);
        // TODO: parallelize
        for s in &mut s {
            *s *= &neg_z1;
        }
        extra_scalars.extend(s);

        bool::from(best_multiexp(&extra_scalars, &extra_bases).is_zero())
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
