use super::super::Error;
use super::{Guard, OpeningProof, Params, MSM};
use crate::transcript::Hasher;

use crate::arithmetic::{get_challenge_scalar, Challenge, CurveAffine, Field};

impl<C: CurveAffine> OpeningProof<C> {
    /// Checks to see if an [`OpeningProof`] is valid given the current
    /// `transcript`, and a point `x` that the polynomial commitment `p` opens
    /// purportedly to the value `v`.
    pub fn verify<'a, H: Hasher<C::Base>>(
        &self,
        params: &'a Params<C>,
        mut msm: MSM<'a, C>,
        transcript: &mut H,
        x: C::Scalar,
        p: &C,
        v: C::Scalar,
    ) -> Result<Guard<'a, C>, Error> {
        // Check for well-formedness
        if self.rounds.len() != params.k as usize {
            return Err(Error::OpeningError);
        }

        transcript.absorb(C::Base::from_u64(self.fork as u64));

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
            let l = round.0.get_xy();
            let r = round.1.get_xy();
            if bool::from(l.is_none() | r.is_none()) {
                return Err(Error::OpeningError);
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

        let delta = self.delta.get_xy();
        if bool::from(delta.is_none()) {
            return Err(Error::OpeningError);
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

        // Scale the MSM by a random factor to ensure that if the existing MSM
        // has is_zero() == false then this argument won't be able to interfere
        // with it to make it true. It's a way of keeping the MSM's linearly
        // independent.
        msm.scale(C::Scalar::random());

        for scalar in &mut extra_scalars {
            *scalar *= &c;
        }

        for (scalar, base) in extra_scalars.iter().zip(extra_bases.iter()) {
            msm.add_term(*scalar, *base);
        }

        let b = compute_b(x, &challenges, &challenges_inv);

        let neg_z1 = -self.z1;

        // [c] P
        msm.add_term(c, *p);

        // [c * v] U - [z1 * b] U
        msm.add_term((c * &v) + &(neg_z1 * &b), u);

        // delta
        msm.add_term(Field::one(), self.delta);

        // - [z2] H
        msm.add_to_h(-self.z2);

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
