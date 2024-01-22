use group::ff::{BatchInvert, Field};

use super::ParamsIPA;
use crate::{arithmetic::CurveAffine, poly::ipa::strategy::GuardIPA};
use crate::{
    poly::{commitment::MSM, ipa::msm::MSMIPA, Error},
    transcript::{EncodedChallenge, TranscriptRead},
};

/// Checks to see if the proof represented within `transcript` is valid, and a
/// point `x` that the polynomial commitment `P` opens purportedly to the value
/// `v`. The provided `msm` should evaluate to the commitment `P` being opened.
pub fn verify_proof<'params, C: CurveAffine, E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
    params: &'params ParamsIPA<C>,
    mut msm: MSMIPA<'params, C>,
    transcript: &mut T,
    x: C::Scalar,
    v: C::Scalar,
) -> Result<GuardIPA<'params, C>, Error> {
    let k = params.k as usize;

    // P' = P - [v] G_0 + [ξ] S
    msm.add_constant_term(-v); // add [-v] G_0
    let s_poly_commitment = transcript.read_point().map_err(|_| Error::OpeningError)?;
    let xi = *transcript.squeeze_challenge_scalar::<()>();
    msm.append_term(xi, s_poly_commitment.into());

    let z = *transcript.squeeze_challenge_scalar::<()>();

    let mut rounds = vec![];
    for _ in 0..k {
        // Read L and R from the proof and write them to the transcript
        let l = transcript.read_point().map_err(|_| Error::OpeningError)?;
        let r = transcript.read_point().map_err(|_| Error::OpeningError)?;

        let u_j_packed = transcript.squeeze_challenge();
        let u_j = *u_j_packed.as_challenge_scalar::<()>();

        rounds.push((l, r, u_j, /* to be inverted */ u_j, u_j_packed));
    }

    rounds
        .iter_mut()
        .map(|&mut (_, _, _, ref mut u_j, _)| u_j)
        .batch_invert();

    // This is the left-hand side of the verifier equation.
    // P' + \sum([u_j^{-1}] L_j) + \sum([u_j] R_j)
    let mut u = Vec::with_capacity(k);
    let mut u_packed: Vec<C::Scalar> = Vec::with_capacity(k);
    for (l, r, u_j, u_j_inv, u_j_packed) in rounds {
        msm.append_term(u_j_inv, l.into());
        msm.append_term(u_j, r.into());

        u.push(u_j);
        u_packed.push(u_j_packed.get_scalar());
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
    // Note that the guard returned from this function does not include
    // the [-c]G'_0 term.

    let c = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let neg_c = -c;
    let f = transcript.read_scalar().map_err(|_| Error::SamplingError)?;
    let b = compute_b(x, &u);

    msm.add_to_u_scalar(neg_c * &b * &z);
    msm.add_to_w_scalar(-f);

    let guard = GuardIPA {
        msm,
        neg_c,
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
