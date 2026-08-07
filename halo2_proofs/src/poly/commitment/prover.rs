use ff::Field;
use rand_core::RngCore;

use super::super::{Coeff, Polynomial};
use super::{Blind, Params};
use crate::arithmetic::{
    best_multiexp, compute_inner_product, eval_polynomial, parallelize, CurveAffine, CurveExt,
};
use crate::transcript::{EncodedChallenge, TranscriptWrite};

use group::{Curve, Group};
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
pub fn create_proof<
    C: CurveAffine,
    E: EncodedChallenge<C>,
    R: RngCore,
    T: TranscriptWrite<C, E>,
>(
    params: &Params<C>,
    mut rng: R,
    transcript: &mut T,
    p_poly: &Polynomial<C::Scalar, Coeff>,
    p_blind: Blind<C::Scalar>,
    x_3: C::Scalar,
) -> io::Result<()> {
    // We're limited to polynomials of degree n - 1.
    assert_eq!(p_poly.len(), params.n as usize);

    // Sample a random polynomial (of same degree) that has a root at x_3, first
    // by setting all coefficients to random values.
    let mut s_poly = (*p_poly).clone();
    for coeff in s_poly.iter_mut() {
        *coeff = C::Scalar::random(&mut rng);
    }
    // Evaluate the random polynomial at x_3
    let s_at_x3 = eval_polynomial(&s_poly[..], x_3);
    // Subtract constant coefficient to get a random polynomial with a root at x_3
    s_poly[0] -= &s_at_x3;
    // And sample a random blind
    let s_poly_blind = Blind(C::Scalar::random(&mut rng));

    // Write a commitment to the random polynomial to the transcript
    let s_poly_commitment = params.commit(&s_poly, s_poly_blind).to_affine();
    transcript.write_point(s_poly_commitment)?;

    // Challenge that will ensure that the prover cannot change P but can only
    // witness a random polynomial commitment that agrees with P at x_3, with high
    // probability.
    let xi = *transcript.squeeze_challenge_scalar::<()>();

    // Challenge that ensures that the prover did not interfere with the U term
    // in their commitments.
    let z = *transcript.squeeze_challenge_scalar::<()>();

    // We'll be opening `P' = P - [v] G_0 + [ξ] S` to ensure it has a root at
    // zero.
    let mut p_prime_poly = s_poly * xi + p_poly;
    let v = eval_polynomial(&p_prime_poly, x_3);
    p_prime_poly[0] -= &v;
    let p_prime_blind = s_poly_blind * Blind(xi) + p_blind;

    // This accumulates the synthetic blinding factor `f` starting
    // with the blinding factor for `P'`.
    let mut f = p_prime_blind.0;

    // Initialize the vector `p_prime` as the coefficients of the polynomial.
    let mut p_prime = p_prime_poly.values;
    assert_eq!(p_prime.len(), params.n as usize);

    // Initialize the vector `b` as the powers of `x_3`. The inner product of
    // `p_prime` and `b` is the evaluation of the polynomial at `x_3`.
    let mut b = Vec::with_capacity(1 << params.k);
    {
        let mut cur = C::Scalar::ONE;
        for _ in 0..(1 << params.k) {
            b.push(cur);
            cur *= &x_3;
        }
    }

    // Initialize the vector `G'` from the URS. We'll be progressively collapsing
    // this vector into smaller and smaller vectors until it is of length 1.
    let mut g_prime = params.g.clone();

    // Perform the inner product argument, round by round.
    for j in 0..params.k {
        let half = 1 << (params.k - j - 1); // half the length of `p_prime`, `b`, `G'`

        // Compute L, R
        //
        // TODO: If we modify multiexp to take "extra" bases, we could speed
        // this piece up a bit by combining the multiexps.
        let l_j = best_multiexp(&p_prime[half..], &g_prime[0..half]);
        let r_j = best_multiexp(&p_prime[0..half], &g_prime[half..]);
        let value_l_j = compute_inner_product(&p_prime[half..], &b[0..half]);
        let value_r_j = compute_inner_product(&p_prime[0..half], &b[half..]);
        let l_j_randomness = C::Scalar::random(&mut rng);
        let r_j_randomness = C::Scalar::random(&mut rng);
        let l_j = l_j + &best_multiexp(&[value_l_j * &z, l_j_randomness], &[params.u, params.w]);
        let r_j = r_j + &best_multiexp(&[value_r_j * &z, r_j_randomness], &[params.u, params.w]);
        let l_j = l_j.to_affine();
        let r_j = r_j.to_affine();

        // Feed L and R into the real transcript
        transcript.write_point(l_j)?;
        transcript.write_point(r_j)?;

        let u_j = *transcript.squeeze_challenge_scalar::<()>();
        let u_j_inv = u_j.invert().unwrap(); // TODO, bubble this up

        // Collapse `p_prime` and `b`.
        // TODO: parallelize
        #[allow(clippy::assign_op_pattern)]
        for i in 0..half {
            p_prime[i] = p_prime[i] + &(p_prime[i + half] * &u_j_inv);
            b[i] = b[i] + &(b[i + half] * &u_j);
        }
        p_prime.truncate(half);
        b.truncate(half);

        // Collapse `G'`
        parallel_generator_collapse(&mut g_prime, u_j);
        g_prime.truncate(half);

        // Update randomness (the synthetic blinding factor at the end)
        f += &(l_j_randomness * &u_j_inv);
        f += &(r_j_randomness * &u_j);
    }

    // We have fully collapsed `p_prime`, `b`, `G'`
    assert_eq!(p_prime.len(), 1);
    let c = p_prime[0];

    transcript.write_scalar(c)?;
    transcript.write_scalar(f)?;

    Ok(())
}

fn parallel_generator_collapse<C: CurveAffine>(g: &mut [C], challenge: C::Scalar) {
    let len = g.len() / 2;
    let (g_lo, g_hi) = g.split_at_mut(len);

    parallelize(g_lo, |g_lo, start| {
        let g_hi = &g_hi[start..start + g_lo.len()];
        let mut scaled = vec![C::Curve::identity(); g_hi.len()];
        // The Fiat-Shamir challenge is public, so variable-time scalar
        // multiplication is appropriate here.
        <C::Curve as CurveExt>::batch_mul_same_scalar_vartime(g_hi, &challenge, &mut scaled);
        for (scaled, g_lo) in scaled.iter_mut().zip(g_lo.iter()) {
            *scaled += *g_lo;
        }
        C::Curve::batch_normalize(&scaled, g_lo);
    });
}

#[cfg(test)]
mod tests {
    use super::parallel_generator_collapse;
    use crate::arithmetic::CurveAffine;
    use ff::Field;
    use group::{Curve, Group};
    use pasta_curves::{pallas, vesta};

    fn generator_collapse_matches_native<C>()
    where
        C: CurveAffine + core::fmt::Debug,
    {
        let points: Vec<C> = (0..16)
            .map(|i| {
                if i == 3 || i == 12 {
                    C::identity()
                } else {
                    C::from(C::Curve::generator() * C::Scalar::from(i as u64 + 1))
                }
            })
            .collect();
        let full_width = (C::Scalar::from(0x9E37_79B9_7F4A_7C15u64).square()
            + C::Scalar::from(0x0123_4567_89AB_CDEFu64))
        .square();
        let challenges = [
            C::Scalar::ZERO,
            C::Scalar::ONE,
            -C::Scalar::ONE,
            C::Scalar::from(2),
            full_width,
        ];

        for challenge in challenges {
            let half = points.len() / 2;
            let expected_projective: Vec<C::Curve> = points[..half]
                .iter()
                .zip(points[half..].iter())
                .map(|(g_lo, g_hi)| g_lo.to_curve() + (*g_hi * challenge))
                .collect();
            let mut expected = vec![C::identity(); half];
            C::Curve::batch_normalize(&expected_projective, &mut expected);

            let mut actual = points.clone();
            parallel_generator_collapse(&mut actual, challenge);
            assert_eq!(&actual[..half], expected);
        }
    }

    #[test]
    fn generator_collapse_matches_native_pallas() {
        generator_collapse_matches_native::<pallas::Affine>();
    }

    #[test]
    fn generator_collapse_matches_native_vesta() {
        generator_collapse_matches_native::<vesta::Affine>();
    }
}
