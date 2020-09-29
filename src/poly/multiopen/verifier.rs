use super::super::commitment::{Params, MSM};
use super::Proof;
use crate::arithmetic::{get_challenge_scalar, Challenge, CurveAffine, Field};
use crate::plonk::hash_point;
use crate::transcript::Hasher;

impl<'a, C: CurveAffine> Proof<C> {
    /// Verify a multi-opening proof
    pub fn verify<I, HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &self,
        params: &'a Params<C>,
        transcript: &mut HBase,
        transcript_scalar: &mut HScalar,
        points: Vec<C::Scalar>,
        instances: I,
    ) -> (C::Scalar, MSM<'a, C>, C::Scalar)
    where
        I: IntoIterator<Item = (usize, C, C::Scalar)> + Clone,
    {
        // Sample x_4 for compressing openings at the same points together
        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Compress the commitments and expected evaluations at x_3 together
        // using the challenge x_4
        let mut q_commitments: Vec<_> = vec![params.empty_msm(); points.len()];
        let mut q_evals: Vec<_> = vec![C::Scalar::zero(); points.len()];
        {
            let mut accumulate = |point_index: usize, new_commitment, eval| {
                q_commitments[point_index].scale(x_4);
                q_commitments[point_index].add_term(C::Scalar::one(), new_commitment);
                q_evals[point_index] *= &x_4;
                q_evals[point_index] += &eval;
            };

            for instance in instances.clone() {
                accumulate(
                    instance.0, // point_index,
                    instance.1, // commitment,
                    instance.2, // eval,
                );
            }
        }

        // Sample a challenge x_5 for keeping the multi-point quotient
        // polynomial terms linearly independent.
        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Obtain the commitment to the multi-point quotient polynomial f(X).
        hash_point(transcript, &self.f_commitment).unwrap();

        // Sample a challenge x_6 for checking that f(X) was committed to
        // correctly.
        let x_6: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        for eval in self.q_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        // We can compute the expected msm_eval at x_6 using the q_evals provided
        // by the prover and from x_5
        let mut msm_eval = C::Scalar::zero();
        for (point_index, point) in points.iter().enumerate() {
            let mut eval = self.q_evals[point_index];

            eval = eval - &q_evals[point_index];
            eval = eval * &(x_6 - &point).invert().unwrap();

            msm_eval *= &x_5;
            msm_eval += &eval;
        }

        // Sample a challenge x_7 that we will use to collapse the openings of
        // the various remaining polynomials at x_6 together.
        let x_7: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Compute the final commitment that has to be opened
        let mut commitment_msm = params.empty_msm();
        commitment_msm.add_term(C::Scalar::one(), self.f_commitment);
        for (point_index, _) in points.iter().enumerate() {
            commitment_msm.scale(x_7);
            commitment_msm.add_msm(&q_commitments[point_index]);
            msm_eval *= &x_7;
            msm_eval += &self.q_evals[point_index];
        }

        (x_6, commitment_msm, msm_eval)
    }
}
