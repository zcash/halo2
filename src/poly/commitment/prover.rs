use super::super::{Coeff, Polynomial};
use super::{Blind, OpeningProof, Params};
use crate::arithmetic::{
    best_multiexp, compute_inner_product, get_challenge_scalar, parallelize, Challenge, Curve,
    CurveAffine, Field,
};
use crate::transcript::Hasher;

impl<C: CurveAffine> OpeningProof<C> {
    /// Create a polynomial commitment opening proof for the polynomial defined
    /// by the coefficients `px`, the blinding factor `blind` used for the
    /// polynomial commitment, and the point `x` that the polynomial is
    /// evaluated at.
    ///
    /// This function will panic if the provided polynomial is too large with
    /// respect to the polynomial commitment parameters.
    ///
    /// **Important:** This function assumes that the provided `transcript` has
    /// already seen the common inputs: the polynomial commitment P, the claimed
    /// opening v, and the point x. It's probably also nice for the transcript
    /// to have seen the elliptic curve description and the SRS, if you want to
    /// be rigorous.
    pub fn create<H: Hasher<C::Base>>(
        params: &Params<C>,
        transcript: &mut H,
        px: &Polynomial<C::Scalar, Coeff>,
        blind: Blind<C::Scalar>,
        x: C::Scalar,
    ) -> Result<Self, ()> {
        let mut blind = blind.0;

        // We're limited to polynomials of degree n - 1.
        assert!(px.len() <= params.n as usize);

        let mut fork = 0;

        // TODO: remove this hack and force the caller to deal with it
        loop {
            let mut transcript = transcript.clone();
            transcript.absorb(C::Base::from_u64(fork as u64));
            let u_x = transcript.squeeze();
            // y^2 = x^3 + B
            let u_y2 = u_x.square() * &u_x + &C::b();
            let u_y = u_y2.deterministic_sqrt();

            if u_y.is_none() {
                fork += 1;
            } else {
                break;
            }
        }

        transcript.absorb(C::Base::from_u64(fork as u64));

        // Compute U
        let u = {
            let u_x = transcript.squeeze();
            // y^2 = x^3 + B
            let u_y2 = u_x.square() * &u_x + &C::b();
            let u_y = u_y2.deterministic_sqrt().unwrap();

            C::from_xy(u_x, u_y).unwrap()
        };

        // Initialize the vector `a` as the coefficients of the polynomial,
        // rounding up to the parameters.
        let mut a = px.to_vec();
        a.resize(params.n as usize, C::Scalar::zero());

        // Initialize the vector `b` as the powers of `x`. The inner product of
        // `a` and `b` is the evaluation of the polynomial at `x`.
        let mut b = Vec::with_capacity(1 << params.k);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..(1 << params.k) {
                b.push(cur);
                cur *= &x;
            }
        }

        // Initialize the vector `G` from the SRS. We'll be progressively
        // collapsing this vector into smaller and smaller vectors until it is
        // of length 1.
        let mut g = params.g.clone();

        // Perform the inner product argument, round by round.
        let mut rounds = Vec::with_capacity(params.k as usize);
        for k in (1..=params.k).rev() {
            let half = 1 << (k - 1); // half the length of `a`, `b`, `G`

            // Compute L, R
            //
            // TODO: If we modify multiexp to take "extra" bases, we could speed
            // this piece up a bit by combining the multiexps.
            let l = best_multiexp(&a[0..half], &g[half..]);
            let r = best_multiexp(&a[half..], &g[0..half]);
            let value_l = compute_inner_product(&a[0..half], &b[half..]);
            let value_r = compute_inner_product(&a[half..], &b[0..half]);
            let mut l_randomness = C::Scalar::random();
            let r_randomness = C::Scalar::random();
            let l = l + &best_multiexp(&[value_l, l_randomness], &[u, params.h]);
            let r = r + &best_multiexp(&[value_r, r_randomness], &[u, params.h]);
            let mut l = l.to_affine();
            let r = r.to_affine();

            let challenge = loop {
                // We'll fork the transcript and adjust our randomness
                // until the challenge is a square.
                let mut transcript = transcript.clone();

                // We expect these to not be points at infinity due to the randomness.
                let (l_x, l_y) = l.get_xy().unwrap();
                let (r_x, r_y) = r.get_xy().unwrap();

                // Feed L and R into the cloned transcript...
                transcript.absorb(l_x);
                transcript.absorb(l_y);
                transcript.absorb(r_x);
                transcript.absorb(r_y);

                // ... and get the squared challenge.
                let challenge_sq_packed = transcript.squeeze().get_lower_128();
                let challenge_sq: C::Scalar = get_challenge_scalar(Challenge(challenge_sq_packed));

                // There might be no square root, in which case we'll fork the
                // transcript.
                let challenge = challenge_sq.deterministic_sqrt();
                if let Some(challenge) = challenge {
                    break challenge;
                } else {
                    // Try again, with slightly different randomness
                    l = (l + params.h).to_affine();
                    l_randomness += &C::Scalar::one();
                }
            };

            // Challenge is unlikely to be zero.
            let challenge_inv = challenge.invert().unwrap();
            let challenge_sq_inv = challenge_inv.square();
            let challenge_sq = challenge.square();

            // Feed L and R into the real transcript
            let (l_x, l_y) = l.get_xy().unwrap();
            let (r_x, r_y) = r.get_xy().unwrap();
            transcript.absorb(l_x);
            transcript.absorb(l_y);
            transcript.absorb(r_x);
            transcript.absorb(r_y);

            // And obtain the challenge, even though we already have it, since
            // squeezing affects the transcript.
            {
                let challenge_sq_packed = transcript.squeeze().get_lower_128();
                let challenge_sq_expected = get_challenge_scalar(Challenge(challenge_sq_packed));
                assert_eq!(challenge_sq, challenge_sq_expected);
            }

            // Done with this round.
            rounds.push((l, r));

            // Collapse `a` and `b`.
            // TODO: parallelize
            for i in 0..half {
                a[i] = (a[i] * &challenge) + &(a[i + half] * &challenge_inv);
                b[i] = (b[i] * &challenge_inv) + &(b[i + half] * &challenge);
            }
            a.truncate(half);
            b.truncate(half);

            // Collapse `G`
            parallel_generator_collapse(&mut g, challenge, challenge_inv);
            g.truncate(half);

            // Update randomness (the synthetic blinding factor at the end)
            blind += &(l_randomness * &challenge_sq);
            blind += &(r_randomness * &challenge_sq_inv);
        }

        // We have fully collapsed `a`, `b`, `G`
        assert_eq!(a.len(), 1);
        let a = a[0];
        assert_eq!(b.len(), 1);
        let b = b[0];
        assert_eq!(g.len(), 1);
        let g = g[0];

        // Random nonces for the zero-knowledge opening
        let d = C::Scalar::random();
        let s = C::Scalar::random();

        let delta = best_multiexp(&[d, d * &b, s], &[g, u, params.h]).to_affine();

        let (delta_x, delta_y) = delta.get_xy().unwrap();

        // Feed delta into the transcript
        transcript.absorb(delta_x);
        transcript.absorb(delta_y);

        // Obtain the challenge c.
        let c_packed = transcript.squeeze().get_lower_128();
        let c: C::Scalar = get_challenge_scalar(Challenge(c_packed));

        // Compute z1 and z2 as described in the Halo paper.
        let z1 = a * &c + &d;
        let z2 = c * &blind + &s;

        Ok(OpeningProof {
            fork,
            rounds,
            delta,
            z1,
            z2,
        })
    }
}

fn parallel_generator_collapse<C: CurveAffine>(
    g: &mut [C],
    challenge: C::Scalar,
    challenge_inv: C::Scalar,
) {
    let len = g.len() / 2;
    let (mut g_lo, g_hi) = g.split_at_mut(len);

    parallelize(&mut g_lo, |g_lo, start| {
        let g_hi = &g_hi[start..];
        let mut tmp = Vec::with_capacity(g_lo.len());
        for (g_lo, g_hi) in g_lo.iter().zip(g_hi.iter()) {
            // TODO: could use multiexp
            tmp.push(((*g_lo) * challenge_inv) + &((*g_hi) * challenge));
        }
        C::Projective::batch_to_affine(&tmp, g_lo);
    });
}
