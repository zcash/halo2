use ff::Field;

use super::super::{Coeff, Polynomial};
use super::{Blind, Params};
use crate::arithmetic::{
    best_multiexp, compute_inner_product, eval_polynomial, parallelize, CurveAffine, FieldExt,
};
use crate::transcript::{EncodedChallenge, TranscriptWrite};

use group::Curve;
use std::io;

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
/// to have seen the elliptic curve description and the URS, if you want to
/// be rigorous.
pub fn create_proof<C: CurveAffine, E: EncodedChallenge<C>, T: TranscriptWrite<C, E>>(
    params: &Params<C>,
    transcript: &mut T,
    px: &Polynomial<C::Scalar, Coeff>,
    blind: Blind<C::Scalar>,
    x: C::Scalar,
) -> io::Result<()> {
    // We're limited to polynomials of degree n - 1.
    assert!(px.len() <= params.n as usize);

    // Sample a random polynomial (of same degree) that has a root at x, first
    // by setting all coefficients to random values.
    let mut s_poly = (*px).clone();
    for coeff in s_poly.iter_mut() {
        *coeff = C::Scalar::rand();
    }
    // Evaluate the random polynomial at x
    let v_prime = eval_polynomial(&s_poly[..], x);
    // Subtract constant coefficient to get a random polynomial with a root at x
    s_poly[0] = s_poly[0] - &v_prime;
    // And sample a random blind
    let s_poly_blind = Blind(C::Scalar::rand());

    // Write a commitment to the random polynomial to the transcript
    let s_poly_commitment = params.commit(&s_poly, s_poly_blind).to_affine();
    transcript.write_point(s_poly_commitment)?;

    // Challenge that will ensure that the prover cannot change P but can only
    // witness a random polynomial commitment that agrees with P at x, with high
    // probability.
    let iota = *transcript.squeeze_challenge_scalar::<()>()?;

    // Challenge that ensures that the prover did not interfere with the U term
    // in their commitments.
    let z = *transcript.squeeze_challenge_scalar::<()>()?;

    // We'll be opening `s_poly_commitment * iota + P - [v] G_0` to ensure it
    // has a root at zero.
    let mut final_poly = s_poly * iota + px;
    let v = eval_polynomial(&final_poly, x);
    final_poly[0] = final_poly[0] - &v;
    let blind = s_poly_blind * Blind(iota) + blind;
    let mut blind = blind.0;

    // Initialize the vector `a` as the coefficients of the polynomial,
    // rounding up to the parameters.
    let mut a = final_poly.values;
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

    // Initialize the vector `G` from the URS. We'll be progressively collapsing
    // this vector into smaller and smaller vectors until it is of length 1.
    let mut g = params.g.clone();

    // Perform the inner product argument, round by round.
    for k in (1..=params.k).rev() {
        let half = 1 << (k - 1); // half the length of `a`, `b`, `G`

        // Compute L, R
        //
        // TODO: If we modify multiexp to take "extra" bases, we could speed
        // this piece up a bit by combining the multiexps.
        let l = best_multiexp(&a[half..], &g[0..half]);
        let r = best_multiexp(&a[0..half], &g[half..]);
        let value_l = compute_inner_product(&a[half..], &b[0..half]);
        let value_r = compute_inner_product(&a[0..half], &b[half..]);
        let l_randomness = C::Scalar::rand();
        let r_randomness = C::Scalar::rand();
        let l = l + &best_multiexp(&[value_l * &z, l_randomness], &[params.u, params.h]);
        let r = r + &best_multiexp(&[value_r * &z, r_randomness], &[params.u, params.h]);
        let l = l.to_affine();
        let r = r.to_affine();

        // Feed L and R into the real transcript
        transcript.write_point(l)?;
        transcript.write_point(r)?;

        let challenge = *transcript.squeeze_challenge_scalar::<()>()?;
        let challenge_inv = challenge.invert().unwrap(); // TODO, bubble this up

        // Collapse `a` and `b`.
        // TODO: parallelize
        for i in 0..half {
            a[i] = a[i] + &(a[i + half] * &challenge_inv);
            b[i] = b[i] + &(b[i + half] * &challenge);
        }
        a.truncate(half);
        b.truncate(half);

        // Collapse `G`
        parallel_generator_collapse(&mut g, challenge);
        g.truncate(half);

        // Update randomness (the synthetic blinding factor at the end)
        blind += &(l_randomness * &challenge_inv);
        blind += &(r_randomness * &challenge);
    }

    // We have fully collapsed `a`, `b`, `G`
    assert_eq!(a.len(), 1);
    let a = a[0];

    transcript.write_scalar(a)?;
    transcript.write_scalar(blind)?; // \xi

    Ok(())
}

fn parallel_generator_collapse<C: CurveAffine>(g: &mut [C], challenge: C::Scalar) {
    let len = g.len() / 2;
    let (mut g_lo, g_hi) = g.split_at_mut(len);

    parallelize(&mut g_lo, |g_lo, start| {
        let g_hi = &g_hi[start..];
        let mut tmp = Vec::with_capacity(g_lo.len());
        for (g_lo, g_hi) in g_lo.iter().zip(g_hi.iter()) {
            tmp.push(g_lo.to_curve() + &(*g_hi * challenge));
        }
        C::Curve::batch_normalize(&tmp, g_lo);
    });
}
