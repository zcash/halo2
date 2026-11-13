use group::ff::Field;
#[cfg(any(test, feature = "batch"))]
use pasta_curves::arithmetic::VartimeField;
use pasta_curves::arithmetic::{CurveExt, VartimeBatchInvert};

use super::super::Error;
use super::{Params, MSM};
use crate::transcript::{EncodedChallenge, TranscriptRead};

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

    /// Applies the same batch weight to every term, seeding the generator
    /// expansion with it instead of scaling the coefficient vector afterward.
    #[cfg(any(test, feature = "batch"))]
    pub(crate) fn use_challenges_with_scale(mut self, scale: C::Scalar) -> MSM<'a, C> {
        self.msm.scale(scale);
        let mut u_inv = self.u.clone();
        batch_inverse_and_scale(&mut u_inv, scale);
        for (((l, r), u_j), u_j_inv) in self.rounds.iter().zip(&self.u).zip(u_inv) {
            self.msm.append_term(u_j_inv, *l);
            self.msm.append_term(*u_j * scale, *r);
        }
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
        u_inv.iter_mut().batch_invert_vartime();
        for (((l, r), u_j), u_j_inv) in self.rounds.iter().zip(&self.u).zip(u_inv) {
            self.msm.append_term(u_j_inv, *l);
            self.msm.append_term(*u_j, *r);
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

    // P' = P - [v] G_0 + [ξ] S. Defer the G_0 term until the guard
    // expands the generator coefficients, avoiding a separate zero vector.
    let g_0_scalar = -v;
    let s_poly_commitment = transcript.read_point().map_err(|_| Error::OpeningError)?;
    let xi = *transcript.squeeze_challenge_scalar::<()>();
    msm.append_term(xi, s_poly_commitment);

    let z = *transcript.squeeze_challenge_scalar::<()>();

    let mut rounds = Vec::with_capacity(k);
    let mut u = Vec::with_capacity(k);
    let mut u_packed = Vec::with_capacity(k);
    for _ in 0..k {
        // Read L and R from the proof and write them to the transcript
        let l = transcript.read_point().map_err(|_| Error::OpeningError)?;
        let r = transcript.read_point().map_err(|_| Error::OpeningError)?;

        let u_j_packed = transcript.squeeze_challenge();
        let u_j = *u_j_packed.as_challenge_scalar::<()>();

        rounds.push((l, r));
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
    // The guard defers [-v] G_0, the L/R round terms, and [-c] G'_0
    // until its caller chooses how to consume the equation.

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
        rounds,
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

/// Computes the coefficients of $g(X) = \prod\limits_{i=0}^{k-1} (1 + u_{k - 1 - i} X^{2^i})$.
fn compute_s<F: Field>(u: &[F], init: F) -> Vec<F> {
    assert!(!u.is_empty());
    let mut v = Vec::with_capacity(1 << u.len());
    v.push(init);
    for u_j in u.iter().rev() {
        let len = v.len();
        for index in 0..len {
            v.push(v[index] * u_j);
        }
    }

    v
}

/// Inverts public verifier challenges and applies a common scale during the
/// backward pass. Like batch inversion, zero inputs remain zero.
#[cfg(any(test, feature = "batch"))]
fn batch_inverse_and_scale<F: VartimeField>(values: &mut [F], scale: F) {
    let Some((first, rest)) = values.split_first_mut() else {
        return;
    };
    let first_is_zero = first.is_zero_vartime();
    let mut acc = if first_is_zero { F::ONE } else { *first };
    let mut products = Vec::with_capacity(rest.len());
    for value in rest.iter() {
        products.push(acc);
        if !value.is_zero_vartime() {
            acc *= value;
        }
    }
    // The product excludes zeros, so it always has an inverse.
    acc = acc.invert_vartime().unwrap() * scale;
    for (value, product) in rest.iter_mut().zip(products).rev() {
        if !value.is_zero_vartime() {
            let inverse = product * acc;
            acc *= *value;
            *value = inverse;
        }
    }
    if !first_is_zero {
        *first = acc;
    }
}

#[cfg(test)]
mod tests;
