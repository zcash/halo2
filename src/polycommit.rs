//! This module contains an implementation of the polynomial commitment scheme
//! described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use crate::arithmetic::{
    best_fft, best_multiexp, compute_inner_product, get_challenge_scalar, parallelize, Challenge,
    Curve, CurveAffine, Field,
};
use crate::transcript::Hasher;

/// This is a proof object for the polynomial commitment scheme opening.
#[derive(Debug, Clone)]
pub struct OpeningProof<C: CurveAffine> {
    fork: u8,
    rounds: Vec<(C, C)>,
    delta: C,
    z1: C::Scalar,
    z2: C::Scalar,
}

/// These are the public parameters for the polynomial commitment scheme.
#[derive(Debug)]
pub struct Params<C: CurveAffine> {
    pub(crate) k: u32,
    pub(crate) n: u64,
    pub(crate) g: Vec<C>,
    pub(crate) g_lagrange: Vec<C>,
    pub(crate) h: C,
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
    pub fn commit(&self, poly: &[C::Scalar], r: C::Scalar) -> C::Projective {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r);

        tmp_bases.extend(self.g.iter());
        tmp_bases.push(self.h);

        best_multiexp::<C>(&tmp_scalars, &tmp_bases)
    }

    /// This commits to a polynomial using its evaluations over the $2^k$ size
    /// evaluation domain. The commitment will be blinded by the blinding factor
    /// `r`.
    pub fn commit_lagrange(&self, poly: &[C::Scalar], r: C::Scalar) -> C::Projective {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r);

        tmp_bases.extend(self.g_lagrange.iter());
        tmp_bases.push(self.h);

        best_multiexp::<C>(&tmp_scalars, &tmp_bases)
    }

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
    pub fn create_proof<H: Hasher<C::Base>>(
        &self,
        transcript: &mut H,
        px: &[C::Scalar],
        mut blind: C::Scalar,
        x: C::Scalar,
    ) -> Result<OpeningProof<C>, ()> {
        // We're limited to polynomials of degree n - 1.
        assert!(px.len() <= self.n as usize);

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
        a.resize(self.n as usize, C::Scalar::zero());

        // Initialize the vector `b` as the powers of `x`. The inner product of
        // `a` and `b` is the evaluation of the polynomial at `x`.
        let mut b = Vec::with_capacity(1 << self.k);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..(1 << self.k) {
                b.push(cur);
                cur *= &x;
            }
        }

        // Initialize the vector `G` from the SRS. We'll be progressively
        // collapsing this vector into smaller and smaller vectors until it is
        // of length 1.
        let mut g = self.g.clone();

        // Perform the inner product argument, round by round.
        let mut rounds = Vec::with_capacity(self.k as usize);
        for k in (1..=self.k).rev() {
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
            let l = l + &best_multiexp(&[value_l, l_randomness], &[u, self.h]);
            let r = r + &best_multiexp(&[value_r, r_randomness], &[u, self.h]);
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
                    l = (l + self.h).to_affine();
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

        // We have fully colapsed `a`, `b`, `G`
        assert_eq!(a.len(), 1);
        let a = a[0];
        assert_eq!(b.len(), 1);
        let b = b[0];
        assert_eq!(g.len(), 1);
        let g = g[0];

        // Random nonces for the zero-knowledge opening
        let d = C::Scalar::random();
        let s = C::Scalar::random();

        let delta = best_multiexp(&[d, d * &b, s], &[g, u, self.h]).to_affine();

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

    /// Checks to see if an [`OpeningProof`] is valid given the current
    /// `transcript`, and a point `x` that the polynomial commitment `p` opens
    /// purportedly to the value `v`.
    pub fn verify_proof<H: Hasher<C::Base>>(
        &self,
        proof: &OpeningProof<C>,
        transcript: &mut H,
        x: C::Scalar,
        p: &C,
        v: C::Scalar,
    ) -> bool {
        // Check for well-formedness
        if proof.rounds.len() != self.k as usize {
            return false;
        }

        transcript.absorb(C::Base::from_u64(proof.fork as u64));

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

        let mut extra_scalars = Vec::with_capacity(proof.rounds.len() * 2 + 4 + self.n as usize);
        let mut extra_bases = Vec::with_capacity(proof.rounds.len() * 2 + 4 + self.n as usize);

        // Data about the challenges from each of the rounds.
        let mut challenges = Vec::with_capacity(proof.rounds.len());
        let mut challenges_inv = Vec::with_capacity(proof.rounds.len());
        let mut challenges_sq = Vec::with_capacity(proof.rounds.len());
        let mut allinv = Field::one();

        for round in &proof.rounds {
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

        let delta = proof.delta.get_xy();
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

        let neg_z1 = -proof.z1;

        // [c] P
        extra_bases.push(*p);
        extra_scalars.push(c);

        // [c * v] U - [z1 * b] U
        extra_bases.push(u);
        extra_scalars.push((c * &v) + &(neg_z1 * &b));

        // delta
        extra_bases.push(proof.delta);
        extra_scalars.push(Field::one());

        // - [z2] H
        extra_bases.push(self.h);
        extra_scalars.push(-proof.z2);

        // - [z1] G
        extra_bases.extend(&self.g);
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

#[test]
fn test_commit_lagrange() {
    const K: u32 = 6;

    use crate::arithmetic::{EpAffine, Fp, Fq};
    use crate::transcript::DummyHash;
    let params = Params::<EpAffine>::new::<DummyHash<Fp>>(K);

    let a = (0..(1 << K)).map(|l| Fq::from(l)).collect::<Vec<_>>();
    let mut b = a.clone();
    let mut alpha = Fq::ROOT_OF_UNITY;
    for _ in K..Fq::S {
        alpha = alpha.square();
    }
    best_fft(&mut b, alpha, K);

    assert_eq!(params.commit(&a, alpha), params.commit_lagrange(&b, alpha));
}

#[test]
fn test_opening_proof() {
    const K: u32 = 6;

    use crate::arithmetic::{eval_polynomial, EpAffine, Fp, Fq};
    use crate::transcript::DummyHash;
    let params = Params::<EpAffine>::new::<DummyHash<Fp>>(K);

    let px = (0..(1 << K))
        .map(|l| Fq::from(l + 1) * Fq::ZETA)
        .collect::<Vec<_>>();
    let blind = Fq::random();

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
        let mut transcript_dup = transcript.clone();

        let opening_proof = params.create_proof(&mut transcript, &px, blind, x);
        if opening_proof.is_err() {
            transcript = transcript_dup;
            transcript.absorb(Field::one());
        } else {
            let opening_proof = opening_proof.unwrap();
            assert!(params.verify_proof(&opening_proof, &mut transcript_dup, x, &p, v));
            break;
        }
    }
}
