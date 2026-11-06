use group::ff::{BatchInvert, Field};
use pasta_curves::arithmetic::CurveExt;

use super::super::Error;
use super::{Params, MSM};
use crate::transcript::{EncodedChallenge, TranscriptRead};

#[cfg(any(test, feature = "batch"))]
use crate::arithmetic::BatchInvertAndScale;
use crate::arithmetic::{best_multiexp, CurveAffine};

/// A guard returned by the verifier
#[derive(Debug, Clone)]
pub struct Guard<'a, C: CurveAffine, E: EncodedChallenge<C>> {
    msm: MSM<'a, C>,
    g_0_scalar: C::Scalar,
    neg_c: C::Scalar,
    rounds: Vec<(C, C)>,
    u: Vec<C::Scalar>,
    u_packed: Vec<E>,
}

/// An accumulator instance consisting of an evaluation claim and a proof.
#[derive(Debug, Clone)]
pub struct Accumulator<C: CurveAffine, E: EncodedChallenge<C>> {
    /// The claimed output of the linear-time polycommit opening protocol
    pub g: C,

    /// A vector of challenges u_0, ..., u_{k - 1} sampled by the verifier, to
    /// be used in computing G'_0.
    pub u_packed: Vec<E>,
}

impl<'a, C: CurveAffine, E: EncodedChallenge<C>> Guard<'a, C, E> {
    /// Lets caller supply the challenges and obtain an MSM with updated
    /// scalars and points.
    pub fn use_challenges(mut self) -> MSM<'a, C> {
        self.append_round_terms();

        let mut s = compute_s(&self.u, self.neg_c);
        s[0] += self.g_0_scalar;
        self.msm.add_owned_g_scalars(s);

        self.msm
    }

    /// Applies `scale` to every term of this IPA verifier equation.
    ///
    /// The existing MSM terms, the deferred round terms, and the generator
    /// coefficients must all receive the same scale exactly once.
    #[cfg(any(test, feature = "batch"))]
    pub(crate) fn use_challenges_with_scale(mut self, scale: C::Scalar) -> MSM<'a, C> {
        // Scale P', U, W, and every commitment term assembled before the IPA
        // round terms.
        self.msm.scale(scale);

        // Append [scale * u_j^-1] L_j and [scale * u_j] R_j.
        self.append_round_terms_with_scale(scale);

        // Seeding the expansion with -c * scale applies the common factor to
        // all coefficients of G'_0 without a second pass over the vector.
        let mut s = compute_s(&self.u, self.neg_c * scale);
        s[0] += self.g_0_scalar * scale;
        self.msm.add_owned_g_scalars(s);

        self.msm
    }

    /// Lets caller supply the purported G point and simply appends
    /// [-c] G to return an updated MSM.
    pub fn use_g(mut self, g: C) -> (MSM<'a, C>, Accumulator<C, E>) {
        self.append_round_terms();
        self.msm.add_constant_term(self.g_0_scalar);
        self.msm.append_term(self.neg_c, g);

        let accumulator = Accumulator {
            g,
            u_packed: self.u_packed,
        };

        (self.msm, accumulator)
    }

    /// Computes G = ⟨s, params.g⟩
    pub fn compute_g(&self) -> C {
        let s = compute_s(&self.u, C::Scalar::ONE);

        best_multiexp(&s, &self.msm.params.g).to_affine_vartime()
    }

    fn append_round_terms(&mut self) {
        let mut u_inv = self.u.clone();
        u_inv.iter_mut().batch_invert();

        // This is the left-hand side of the verifier equation.
        // P' + \sum([u_j^{-1}] L_j) + \sum([u_j] R_j)
        for (((l, r), u_j), u_j_inv) in self.rounds.iter().zip(&self.u).zip(u_inv) {
            self.msm.append_term(u_j_inv, *l);
            self.msm.append_term(*u_j, *r);
        }
    }

    #[cfg(any(test, feature = "batch"))]
    fn append_round_terms_with_scale(&mut self, scale: C::Scalar) {
        let mut u_inv = self.u.clone();
        u_inv.iter_mut().batch_inverse_and_scale(scale);

        // This is the scaled left-hand side of the verifier equation.
        // [scale] P' + \sum([scale u_j^{-1}] L_j)
        // + \sum([scale u_j] R_j)
        for (((l, r), u_j), u_j_inv) in self.rounds.iter().zip(&self.u).zip(u_inv) {
            self.msm.append_term(u_j_inv, *l);
            self.msm.append_term(*u_j * scale, *r);
        }
    }
}

/// Checks to see if the proof represented within `transcript` is valid, and a
/// point `x` that the polynomial commitment `P` opens purportedly to the value
/// `v`. The provided `msm` should evaluate to the commitment `P` being opened.
pub fn verify_proof<'a, C: CurveAffine, E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
    params: &'a Params<C>,
    mut msm: MSM<'a, C>,
    transcript: &mut T,
    x: C::Scalar,
    v: C::Scalar,
) -> Result<Guard<'a, C, E>, Error> {
    let k = params.k as usize;

    // P' = P - [v] G_0 + [ξ] S. Defer the G_0 coefficient until the guard
    // expands the generator coefficients, avoiding a separate zero vector.
    let g_0_scalar = -v;
    let s_poly_commitment = transcript.read_point().map_err(|_| Error::OpeningError)?;
    let xi = *transcript.squeeze_challenge_scalar::<()>();
    msm.append_term(xi, s_poly_commitment);

    let z = *transcript.squeeze_challenge_scalar::<()>();

    let mut rounds = vec![];
    for _ in 0..k {
        // Read L and R from the proof and write them to the transcript
        let l = transcript.read_point().map_err(|_| Error::OpeningError)?;
        let r = transcript.read_point().map_err(|_| Error::OpeningError)?;

        let u_j_packed = transcript.squeeze_challenge();
        let u_j = *u_j_packed.as_challenge_scalar::<()>();

        rounds.push((l, r, /* to be inverted */ u_j, u_j_packed));
    }

    let mut u = Vec::with_capacity(k);
    let mut u_packed = Vec::with_capacity(k);
    let mut round_points = Vec::with_capacity(k);
    for (l, r, u_j, u_j_packed) in rounds {
        round_points.push((l, r));
        u.push(u_j);
        u_packed.push(u_j_packed);
    }

    // Our goal is to check that the left hand side of the verifier
    // equation
    //     P' + \sum([u_j^{-1}] L_j) + \sum([u_j] R_j)
    // equals (given b = \mathbf{b}_0, and the prover's values c, f),
    // the right-hand side
    //   = [c] (G'_0 + [b * z] U) + [f] W
    // Subtracting the right-hand side from both sides we get
    //   P' + \sum([u_j^{-1}] L_j) + \sum([u_j] R_j)
    //   + [-c] G'_0 + [-cbz] U + [-f] W
    //   = 0
    //
    // The guard's MSM defers both [-v] G_0 and [-c] G'_0 until the caller
    // chooses between supplying G'_0 and expanding its generator terms.

    let c = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let neg_c = -c;
    let f = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let b = compute_b(x, &u);

    msm.add_to_u_scalar(neg_c * &b * &z);
    msm.add_to_w_scalar(-f);

    let guard = Guard {
        msm,
        g_0_scalar,
        neg_c,
        rounds: round_points,
        u,
        u_packed,
    };

    Ok(guard)
}

/// Computes $\prod\limits_{i=0}^{k-1} (1 + u_{k - 1 - i} x^{2^i})$.
fn compute_b<F: Field>(x: F, u: &[F]) -> F {
    let mut tmp = F::ONE;
    let mut cur = x;
    for u_j in u.iter().rev() {
        tmp *= F::ONE + &(*u_j * &cur);
        cur *= cur;
    }
    tmp
}

/// Computes the coefficients of
/// $g(X) = \prod\limits_{i=0}^{k-1} (1 + u_{k - 1 - i} X^{2^i})$.
fn compute_s<F: Field>(u: &[F], init: F) -> Vec<F> {
    assert!(!u.is_empty());
    let capacity = 1 << u.len();
    let mut v = Vec::with_capacity(capacity);
    v.push(init);

    for u_j in u.iter().rev() {
        let len = v.len();
        for index in 0..len {
            v.push(v[index] * u_j);
        }
    }

    v
}

#[cfg(test)]
mod tests {
    use ff::Field;
    use group::{Curve, Group};
    use pasta_curves::{EpAffine, EqAffine, Fp, Fq};

    use super::{Guard, MSM, Params, compute_s};
    use crate::transcript::Challenge255;

    fn assert_sum<C: crate::arithmetic::CurveAffine>(mut msm: MSM<'_, C>, expected: C::Curve) {
        msm.append_term(-C::Scalar::ONE, expected.to_affine());
        assert!(msm.eval());
    }

    fn coefficient_products<F: Field>(u: &[F], init: F) -> Vec<F> {
        (0..1usize << u.len())
            .map(|index| {
                u.iter().rev().enumerate().fold(init, |value, (bit, u_j)| {
                    if index & (1 << bit) == 0 {
                        value
                    } else {
                        value * u_j
                    }
                })
            })
            .collect()
    }

    #[test]
    fn generator_coefficients_match_individual_products() {
        fn check<F: Field + From<u64>>() {
            for rounds in 1..=10 {
                let u = (0..rounds)
                    .map(|index| F::from(index as u64 + 2))
                    .collect::<Vec<_>>();
                for init in [F::ZERO, F::ONE, -F::ONE, F::from(41)] {
                    assert_eq!(compute_s(&u, init), coefficient_products(&u, init));
                    for zero in 0..rounds {
                        let mut with_zero = u.clone();
                        with_zero[zero] = F::ZERO;
                        assert_eq!(
                            compute_s(&with_zero, init),
                            coefficient_products(&with_zero, init),
                        );
                    }
                }
            }
        }

        check::<Fp>();
        check::<Fq>();
    }

    #[test]
    fn deferred_constant_preserves_every_guard_consumption() {
        macro_rules! check_curve {
            ($curve:ty, $scalar:ty) => {{
                let params = Params::<$curve>::new(4);
                let generator = <$curve as group::CurveAffine>::Curve::generator();
                let u = [2, 3, 5, 7].map(<$scalar>::from);
                let neg_c = -<$scalar>::from(11);
                let g_0_scalar = -<$scalar>::from(13);
                let rounds = (0..u.len())
                    .map(|index| {
                        (
                            (generator * <$scalar>::from(index as u64 + 17)).to_affine(),
                            (generator * <$scalar>::from(index as u64 + 23)).to_affine(),
                        )
                    })
                    .collect::<Vec<_>>();

                for existing in [false, true] {
                    let mut msm = MSM::new(&params);
                    msm.append_term(<$scalar>::from(29), generator.to_affine());
                    let mut existing_sum = generator * <$scalar>::from(29);
                    if existing {
                        let scalars = (0..params.n)
                            .map(|index| <$scalar>::from(u64::from(index) + 31))
                            .collect::<Vec<_>>();
                        msm.add_to_g_scalars(&scalars);
                        existing_sum += params.g.iter().zip(scalars).fold(
                            <$curve as group::CurveAffine>::Curve::identity(),
                            |sum, (base, scalar)| sum + *base * scalar,
                        );
                    }
                    let guard = Guard::<$curve, Challenge255<$curve>> {
                        msm,
                        g_0_scalar,
                        neg_c,
                        rounds: rounds.clone(),
                        u: u.to_vec(),
                        u_packed: vec![],
                    };
                    let round_sum = rounds.iter().zip(u).fold(
                        <$curve as group::CurveAffine>::Curve::identity(),
                        |sum, ((left, right), challenge)| {
                            sum + *left * challenge.invert().unwrap() + *right * challenge
                        },
                    );
                    let g = params
                        .g
                        .iter()
                        .zip(coefficient_products(&u, <$scalar>::ONE))
                        .fold(
                            <$curve as group::CurveAffine>::Curve::identity(),
                            |sum, (base, scalar)| sum + *base * scalar,
                        );
                    assert_eq!(guard.compute_g(), g.to_affine());

                    let expected = existing_sum + params.g[0] * g_0_scalar + round_sum + g * neg_c;
                    assert_sum(guard.clone().use_challenges(), expected);
                    for scale in [
                        <$scalar>::ZERO,
                        <$scalar>::ONE,
                        -<$scalar>::ONE,
                        <$scalar>::from(37),
                    ] {
                        assert_sum(
                            guard.clone().use_challenges_with_scale(scale),
                            expected * scale,
                        );
                    }
                    let supplied_g = (generator * <$scalar>::from(43)).to_affine();
                    let (actual, accumulator) = guard.use_g(supplied_g);
                    assert_eq!(accumulator.g, supplied_g);
                    assert_sum(
                        actual,
                        existing_sum + params.g[0] * g_0_scalar + round_sum + supplied_g * neg_c,
                    );
                }
            }};
        }

        check_curve!(EqAffine, Fp);
        check_curve!(EpAffine, Fq);
    }
}
