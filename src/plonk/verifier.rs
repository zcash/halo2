use ff::Field;
use std::iter;

use super::{Error, Proof, VerifyingKey};
use crate::arithmetic::{get_challenge_scalar, Challenge, CurveAffine, FieldExt};
use crate::poly::{
    commitment::{Guard, Params, MSM},
    multiopen::VerifierQuery,
};
use crate::transcript::{Hasher, Transcript};

impl<'a, C: CurveAffine> Proof<C> {
    /// Returns a boolean indicating whether or not the proof is valid
    pub fn verify<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        &'a self,
        params: &'a Params<C>,
        vk: &'a VerifyingKey<C>,
        msm: MSM<'a, C>,
        aux_commitments: &'a [C],
    ) -> Result<Guard<'a, C>, Error> {
        self.check_lengths(vk, aux_commitments)?;

        // Check that aux_commitments matches the expected number of aux_columns
        // and self.aux_evals
        if aux_commitments.len() != vk.cs.num_aux_columns
            || self.aux_evals.len() != vk.cs.num_aux_columns
        {
            return Err(Error::IncompatibleParams);
        }

        // Create a transcript for obtaining Fiat-Shamir challenges.
        let mut transcript = Transcript::<C, HBase, HScalar>::new();

        // Hash the aux (external) commitments into the transcript
        for commitment in aux_commitments {
            transcript
                .absorb_point(commitment)
                .map_err(|_| Error::TranscriptError)?;
        }

        // Hash the prover's advice commitments into the transcript
        for commitment in &self.advice_commitments {
            transcript
                .absorb_point(commitment)
                .map_err(|_| Error::TranscriptError)?;
        }

        // Sample x_0 challenge
        let x_0: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Sample x_1 challenge
        let x_1: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Hash each permutation product commitment
        if let Some(p) = &self.permutations {
            p.absorb_commitments(&mut transcript)?;
        }

        // Sample x_2 challenge, which keeps the gates linearly independent.
        let x_2: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Obtain a commitment to h(X) in the form of multiple pieces of degree n - 1
        for c in &self.h_commitments {
            transcript
                .absorb_point(c)
                .map_err(|_| Error::TranscriptError)?;
        }

        // Sample x_3 challenge, which is used to ensure the circuit is
        // satisfied with high probability.
        let x_3: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // This check ensures the circuit is satisfied so long as the polynomial
        // commitments open to the correct values.
        self.check_hx(params, vk, x_0, x_1, x_2, x_3)?;

        for eval in self
            .advice_evals
            .iter()
            .chain(self.aux_evals.iter())
            .chain(self.fixed_evals.iter())
            .chain(self.h_evals.iter())
            .chain(
                self.permutations
                    .as_ref()
                    .map(|p| p.evals())
                    .into_iter()
                    .flatten(),
            )
        {
            transcript.absorb_scalar(*eval);
        }

        let queries =
            iter::empty()
                .chain(vk.cs.advice_queries.iter().enumerate().map(
                    |(query_index, &(column, at))| VerifierQuery {
                        point: vk.domain.rotate_omega(x_3, at),
                        commitment: &self.advice_commitments[column.index()],
                        eval: self.advice_evals[query_index],
                    },
                ))
                .chain(
                    vk.cs
                        .aux_queries
                        .iter()
                        .enumerate()
                        .map(|(query_index, &(column, at))| VerifierQuery {
                            point: vk.domain.rotate_omega(x_3, at),
                            commitment: &aux_commitments[column.index()],
                            eval: self.aux_evals[query_index],
                        }),
                )
                .chain(vk.cs.fixed_queries.iter().enumerate().map(
                    |(query_index, &(column, at))| VerifierQuery {
                        point: vk.domain.rotate_omega(x_3, at),
                        commitment: &vk.fixed_commitments[column.index()],
                        eval: self.fixed_evals[query_index],
                    },
                ))
                .chain(
                    self.h_commitments
                        .iter()
                        .enumerate()
                        .zip(self.h_evals.iter())
                        .map(|((idx, _), &eval)| VerifierQuery {
                            point: x_3,
                            commitment: &self.h_commitments[idx],
                            eval,
                        }),
                );

        // We are now convinced the circuit is satisfied so long as the
        // polynomial commitments open to the correct values.
        self.multiopening
            .verify(
                params,
                &mut transcript,
                queries.chain(
                    self.permutations
                        .as_ref()
                        .map(|p| p.queries(vk, x_3))
                        .into_iter()
                        .flatten(),
                ),
                msm,
            )
            .map_err(|_| Error::OpeningError)
    }

    /// Checks that the lengths of vectors are consistent with the constraint
    /// system
    fn check_lengths(&self, vk: &VerifyingKey<C>, aux_commitments: &[C]) -> Result<(), Error> {
        // Check that aux_commitments matches the expected number of aux_columns
        // and self.aux_evals
        if aux_commitments.len() != vk.cs.num_aux_columns
            || self.aux_evals.len() != vk.cs.num_aux_columns
        {
            return Err(Error::IncompatibleParams);
        }

        // TODO: check h_evals

        if self.fixed_evals.len() != vk.cs.fixed_queries.len() {
            return Err(Error::IncompatibleParams);
        }

        if self.advice_evals.len() != vk.cs.advice_queries.len() {
            return Err(Error::IncompatibleParams);
        }

        self.permutations
            .as_ref()
            .map(|p| p.check_lengths(vk))
            .transpose()?;

        // TODO: check h_commitments

        if self.advice_commitments.len() != vk.cs.num_advice_columns {
            return Err(Error::IncompatibleParams);
        }

        Ok(())
    }

    /// Checks that this proof's h_evals are correct, and thus that all of the
    /// rules are satisfied.
    fn check_hx(
        &self,
        params: &'a Params<C>,
        vk: &VerifyingKey<C>,
        x_0: C::Scalar,
        x_1: C::Scalar,
        x_2: C::Scalar,
        x_3: C::Scalar,
    ) -> Result<(), Error> {
        // x_3^n
        let x_3n = x_3.pow(&[params.n as u64, 0, 0, 0]);

        // TODO: bubble this error up
        // l_0(x_3)
        let l_0 = (x_3 - &C::Scalar::one()).invert().unwrap() // 1 / (x_3 - 1)
            * &(x_3n - &C::Scalar::one()) // (x_3^n - 1) / (x_3 - 1)
            * &vk.domain.get_barycentric_weight(); // l_0(x_3)

        // Compute the expected value of h(x_3)
        let expected_h_eval = std::iter::empty()
            // Evaluate the circuit using the custom gates provided
            .chain(vk.cs.gates.iter().map(|poly| {
                poly.evaluate(
                    &|index| self.fixed_evals[index],
                    &|index| self.advice_evals[index],
                    &|index| self.aux_evals[index],
                    &|a, b| a + &b,
                    &|a, b| a * &b,
                    &|a, scalar| a * &scalar,
                )
            }))
            .chain(
                self.permutations
                    .as_ref()
                    .map(|p| p.expressions(vk, &self.advice_evals, l_0, x_0, x_1, x_3))
                    .into_iter()
                    .flatten(),
            )
            .fold(C::Scalar::zero(), |h_eval, v| h_eval * &x_2 + &v);

        // Compute h(x_3) from the prover
        let h_eval = self
            .h_evals
            .iter()
            .rev()
            .fold(C::Scalar::zero(), |acc, eval| acc * &x_3n + eval);

        // Did the prover commit to the correct polynomial?
        if expected_h_eval != (h_eval * &(x_3n - &C::Scalar::one())) {
            return Err(Error::ConstraintSystemFailure);
        }

        Ok(())
    }
}
