use ff::Field;
use std::iter;

use super::{
    ChallengeBeta, ChallengeGamma, ChallengeTheta, ChallengeX, ChallengeY, Error, Proof,
    VerifyingKey,
};
use crate::arithmetic::{CurveAffine, FieldExt};
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

        // Sample theta challenge for keeping lookup columns linearly independent
        let theta = ChallengeTheta::get(&mut transcript);

        // Hash each lookup permuted commitment
        for lookup in &self.lookups {
            lookup.absorb_permuted_commitments(&mut transcript)?;
        }

        // Sample beta challenge
        let beta = ChallengeBeta::get(&mut transcript);

        // Sample gamma challenge
        let gamma = ChallengeGamma::get(&mut transcript);

        // Hash each permutation product commitment
        if let Some(p) = &self.permutations {
            p.absorb_commitments(&mut transcript)?;
        }

        // Hash each lookup product commitment
        for lookup in &self.lookups {
            lookup.absorb_product_commitment(&mut transcript)?;
        }

        // Sample y challenge, which keeps the gates linearly independent.
        let y = ChallengeY::get(&mut transcript);

        self.vanishing.absorb_commitments(&mut transcript)?;

        // Sample x challenge, which is used to ensure the circuit is
        // satisfied with high probability.
        let x = ChallengeX::get(&mut transcript);

        // This check ensures the circuit is satisfied so long as the polynomial
        // commitments open to the correct values.
        self.check_hx(params, vk, theta, beta, gamma, y, x)?;

        for eval in self
            .advice_evals
            .iter()
            .chain(self.aux_evals.iter())
            .chain(self.fixed_evals.iter())
            .chain(self.vanishing.evals())
            .chain(
                self.permutations
                    .as_ref()
                    .map(|p| p.evals())
                    .into_iter()
                    .flatten(),
            )
            .chain(self.lookups.iter().map(|p| p.evals()).into_iter().flatten())
        {
            transcript.absorb_scalar(*eval);
        }

        let queries =
            iter::empty()
                .chain(vk.cs.advice_queries.iter().enumerate().map(
                    |(query_index, &(column, at))| VerifierQuery {
                        point: vk.domain.rotate_omega(*x, at),
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
                            point: vk.domain.rotate_omega(*x, at),
                            commitment: &aux_commitments[column.index()],
                            eval: self.aux_evals[query_index],
                        }),
                )
                .chain(vk.cs.fixed_queries.iter().enumerate().map(
                    |(query_index, &(column, at))| VerifierQuery {
                        point: vk.domain.rotate_omega(*x, at),
                        commitment: &vk.fixed_commitments[column.index()],
                        eval: self.fixed_evals[query_index],
                    },
                ))
                .chain(self.vanishing.queries(x));

        // We are now convinced the circuit is satisfied so long as the
        // polynomial commitments open to the correct values.
        self.multiopening
            .verify(
                params,
                &mut transcript,
                queries
                    .chain(
                        self.permutations
                            .as_ref()
                            .map(|p| p.queries(vk, x))
                            .into_iter()
                            .flatten(),
                    )
                    .chain(
                        self.lookups
                            .iter()
                            .map(|p| p.queries(vk, x))
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

        self.vanishing.check_lengths(vk)?;

        if self.lookups.len() != vk.cs.lookups.len() {
            return Err(Error::IncompatibleParams);
        }

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
        theta: ChallengeTheta<C::Scalar>,
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
        y: ChallengeY<C::Scalar>,
        x: ChallengeX<C::Scalar>,
    ) -> Result<(), Error> {
        // x^n
        let xn = x.pow(&[params.n as u64, 0, 0, 0]);

        // TODO: bubble this error up
        // l_0(x)
        let l_0 = (*x - &C::Scalar::one()).invert().unwrap() // 1 / (x - 1)
            * &(xn - &C::Scalar::one()) // (x^n - 1) / (x - 1)
            * &vk.domain.get_barycentric_weight(); // l_0(x)

        // Compute the expected value of h(x)
        let expressions = std::iter::empty()
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
                    .map(|p| p.expressions(vk, &self.advice_evals, l_0, beta, gamma, x))
                    .into_iter()
                    .flatten(),
            )
            .chain(
                self.lookups
                    .iter()
                    .zip(vk.cs.lookups.iter())
                    .map(|(p, argument)| {
                        p.expressions(
                            vk,
                            l_0,
                            argument,
                            theta,
                            beta,
                            gamma,
                            &self.advice_evals,
                            &self.fixed_evals,
                            &self.aux_evals,
                        )
                    })
                    .into_iter()
                    .flatten(),
            );

        self.vanishing.verify(expressions, y, xn)
    }
}
