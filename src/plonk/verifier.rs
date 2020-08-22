use super::{hash_point, Proof, SRS};
use crate::arithmetic::{get_challenge_scalar, Challenge, Curve, CurveAffine, Field};
use crate::polycommit::Params;
use crate::transcript::Hasher;

impl<C: CurveAffine> Proof<C> {
    /// Returns
    pub fn verify<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &self,
        params: &Params<C>,
        srs: &SRS<C>,
    ) -> bool {
        // Create a transcript for obtaining Fiat-Shamir challenges.
        let mut transcript = HBase::init(C::Base::one());

        hash_point(&mut transcript, &self.a_commitment)
            .expect("proof cannot contain points at infinity");
        hash_point(&mut transcript, &self.b_commitment)
            .expect("proof cannot contain points at infinity");
        hash_point(&mut transcript, &self.c_commitment)
            .expect("proof cannot contain points at infinity");
        hash_point(&mut transcript, &self.d_commitment)
            .expect("proof cannot contain points at infinity");

        for commitment in &self.advice_commitments {
            hash_point(&mut transcript, commitment)
                .expect("proof cannot contain points at infinity");
        }

        for c in &self.h_commitments {
            hash_point(&mut transcript, c).expect("proof cannot contain points at infinity");
        }

        let x: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // We set up a second transcript on the scalar field to hash in openings of
        // our polynomial commitments.
        let mut transcript_scalar = HScalar::init(C::Scalar::one());
        transcript_scalar.absorb(self.a_eval_x);
        transcript_scalar.absorb(self.b_eval_x);
        transcript_scalar.absorb(self.c_eval_x);
        transcript_scalar.absorb(self.d_eval_x);
        transcript_scalar.absorb(self.sa_eval_x);
        transcript_scalar.absorb(self.sb_eval_x);
        transcript_scalar.absorb(self.sc_eval_x);
        transcript_scalar.absorb(self.sd_eval_x);
        transcript_scalar.absorb(self.sm_eval_x);

        for eval in &self.h_evals_x {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let y: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut q_commitment = self.h_commitments[0].clone().to_projective();
        let mut expected_opening = self.h_evals_x[0];
        {
            let mut accumulate = |commitment: C, opening: C::Scalar| {
                q_commitment = commitment.to_projective() + &(q_commitment * y);
                expected_opening = opening + &(expected_opening * &y);
            };

            for (commitment, eval) in self.h_commitments.iter().zip(self.h_evals_x.iter()).skip(1) {
                accumulate(*commitment, *eval);
            }

            accumulate(self.a_commitment, self.a_eval_x);
            accumulate(self.b_commitment, self.b_eval_x);
            accumulate(self.c_commitment, self.c_eval_x);
            accumulate(self.d_commitment, self.d_eval_x);
            accumulate(srs.sa_commitment, self.sa_eval_x);
            accumulate(srs.sb_commitment, self.sb_eval_x);
            accumulate(srs.sc_commitment, self.sc_eval_x);
            accumulate(srs.sd_commitment, self.sd_eval_x);
            accumulate(srs.sm_commitment, self.sm_eval_x);
        }
        let q_commitment = q_commitment.to_affine();

        let xn = x.pow(&[params.n as u64, 0, 0, 0]);

        // Compute the expected h(x) value
        let mut h_eval_x = C::Scalar::zero();
        let mut cur = C::Scalar::one();
        for eval in &self.h_evals_x {
            h_eval_x += &(cur * eval);
            cur *= &xn;
        }

        // Check that the circuit is satisfied.
        // (a * sa) + (b * sb) + (a * sm * b) + (d * sd) - (c * sc)
        if self.a_eval_x * &self.sa_eval_x
            + &(self.b_eval_x * &self.sb_eval_x)
            + &(self.a_eval_x * &self.sm_eval_x * &self.b_eval_x)
            + &(self.d_eval_x * &self.sd_eval_x)
            - &(self.c_eval_x * &self.sc_eval_x)
            != h_eval_x * &(xn - &C::Scalar::one())
        {
            return false;
        }

        params.verify_proof(
            &self.opening,
            &mut transcript,
            x,
            &q_commitment,
            expected_opening,
        )
    }
}
