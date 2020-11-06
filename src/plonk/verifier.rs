use super::{hash_point, Error, Proof, VerifyingKey};
use crate::arithmetic::{get_challenge_scalar, Challenge, CurveAffine, Field};
use crate::poly::{
    commitment::{Guard, Params, MSM},
    multiopen::VerifierQuery,
    Rotation,
};
use crate::transcript::Hasher;

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
        let mut transcript = HBase::init(C::Base::one());

        // Hash the aux (external) commitments into the transcript
        for commitment in aux_commitments {
            hash_point(&mut transcript, commitment)?;
        }

        // Hash the prover's advice commitments into the transcript
        for commitment in &self.advice_commitments {
            hash_point(&mut transcript, commitment)?;
        }

        // Sample x_0 challenge
        let x_0: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Sample x_1 challenge
        let x_1: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Hash each permutation product commitment
        for c in &self.permutation_product_commitments {
            hash_point(&mut transcript, c)?;
        }

        // Sample x_2 challenge, which keeps the gates linearly independent.
        let x_2: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Obtain a commitment to h(X) in the form of multiple pieces of degree n - 1
        for c in &self.h_commitments {
            hash_point(&mut transcript, c)?;
        }

        // Sample x_3 challenge, which is used to ensure the circuit is
        // satisfied with high probability.
        let x_3: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // This check ensures the circuit is satisfied so long as the polynomial
        // commitments open to the correct values.
        self.check_hx(params, vk, x_0, x_1, x_2, x_3)?;

        // Hash together all the openings provided by the prover into a new
        // transcript on the scalar field.
        let mut transcript_scalar = HScalar::init(C::Scalar::one());

        for eval in self
            .advice_evals
            .iter()
            .chain(self.aux_evals.iter())
            .chain(self.fixed_evals.iter())
            .chain(self.h_evals.iter())
            .chain(self.permutation_product_evals.iter())
            .chain(self.permutation_product_inv_evals.iter())
            .chain(self.permutation_evals.iter().flat_map(|evals| evals.iter()))
        {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let mut queries: Vec<VerifierQuery<'a, C>> = Vec::new();

        for (query_index, &(column, at)) in vk.cs.advice_queries.iter().enumerate() {
            let point = vk.domain.rotate_omega(x_3, at);
            queries.push(VerifierQuery {
                point,
                commitment: &self.advice_commitments[column.index],
                eval: self.advice_evals[query_index],
            });
        }

        for (query_index, &(column, at)) in vk.cs.aux_queries.iter().enumerate() {
            let point = vk.domain.rotate_omega(x_3, at);
            queries.push(VerifierQuery {
                point,
                commitment: &aux_commitments[column.index],
                eval: self.aux_evals[query_index],
            });
        }

        for (query_index, &(column, at)) in vk.cs.fixed_queries.iter().enumerate() {
            let point = vk.domain.rotate_omega(x_3, at);
            queries.push(VerifierQuery {
                point,
                commitment: &vk.fixed_commitments[column.index],
                eval: self.fixed_evals[query_index],
            });
        }

        for ((idx, _), &eval) in self
            .h_commitments
            .iter()
            .enumerate()
            .zip(self.h_evals.iter())
        {
            let commitment = &self.h_commitments[idx];
            queries.push(VerifierQuery {
                point: x_3,
                commitment,
                eval,
            });
        }

        // Handle permutation arguments, if any exist
        if !vk.cs.permutations.is_empty() {
            // Open permutation product commitments at x_3
            for ((idx, _), &eval) in self
                .permutation_product_commitments
                .iter()
                .enumerate()
                .zip(self.permutation_product_evals.iter())
            {
                let commitment = &self.permutation_product_commitments[idx];
                queries.push(VerifierQuery {
                    point: x_3,
                    commitment,
                    eval,
                });
            }
            // Open permutation commitments for each permutation argument at x_3
            for outer_idx in 0..vk.permutation_commitments.len() {
                let inner_len = vk.permutation_commitments[outer_idx].len();
                for inner_idx in 0..inner_len {
                    let commitment = &vk.permutation_commitments[outer_idx][inner_idx];
                    let eval = self.permutation_evals[outer_idx][inner_idx];
                    queries.push(VerifierQuery {
                        point: x_3,
                        commitment,
                        eval,
                    });
                }
            }

            // Open permutation product commitments at \omega^{-1} x_3
            let x_3_inv = vk.domain.rotate_omega(x_3, Rotation(-1));
            for ((idx, _), &eval) in self
                .permutation_product_commitments
                .iter()
                .enumerate()
                .zip(self.permutation_product_inv_evals.iter())
            {
                let commitment = &self.permutation_product_commitments[idx];
                queries.push(VerifierQuery {
                    point: x_3_inv,
                    commitment,
                    eval,
                });
            }
        }

        // We are now convinced the circuit is satisfied so long as the
        // polynomial commitments open to the correct values.
        self.multiopening
            .verify(
                params,
                &mut transcript,
                &mut transcript_scalar,
                queries,
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

        if self.permutation_evals.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

        for (permutation_evals, permutation) in
            self.permutation_evals.iter().zip(vk.cs.permutations.iter())
        {
            if permutation_evals.len() != permutation.len() {
                return Err(Error::IncompatibleParams);
            }
        }

        if self.permutation_product_inv_evals.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

        if self.permutation_product_evals.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

        if self.permutation_product_commitments.len() != vk.cs.permutations.len() {
            return Err(Error::IncompatibleParams);
        }

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
            // l_0(X) * (1 - z(X)) = 0
            .chain(
                self.permutation_product_evals
                    .iter()
                    .map(|product_eval| l_0 * &(C::Scalar::one() - &product_eval)),
            )
            // z(X) \prod (p(X) + \beta s_i(X) + \gamma)
            // - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
            .chain(
                vk.cs
                    .permutations
                    .iter()
                    .zip(self.permutation_evals.iter())
                    .zip(self.permutation_product_evals.iter())
                    .zip(self.permutation_product_inv_evals.iter())
                    .map(
                        |(((columns, permutation_evals), product_eval), product_inv_eval)| {
                            let mut left = *product_eval;
                            for (advice_eval, permutation_eval) in columns
                                .iter()
                                .map(|&column| {
                                    self.advice_evals[vk.cs.get_advice_query_index(column, 0)]
                                })
                                .zip(permutation_evals.iter())
                            {
                                left *= &(advice_eval + &(x_0 * permutation_eval) + &x_1);
                            }

                            let mut right = *product_inv_eval;
                            let mut current_delta = x_0 * &x_3;
                            for advice_eval in columns.iter().map(|&column| {
                                self.advice_evals[vk.cs.get_advice_query_index(column, 0)]
                            }) {
                                right *= &(advice_eval + &current_delta + &x_1);
                                current_delta *= &C::Scalar::DELTA;
                            }

                            left - &right
                        },
                    ),
            )
            .fold(C::Scalar::zero(), |h_eval, v| h_eval * &x_2 + &v);

        // Compute h(x_3) from the prover
        let (_, h_eval) = self
            .h_evals
            .iter()
            .fold((C::Scalar::one(), C::Scalar::zero()), |(cur, acc), eval| {
                (cur * &x_3n, acc + &(cur * eval))
            });

        // Did the prover commit to the correct polynomial?
        if expected_h_eval != (h_eval * &(x_3n - &C::Scalar::one())) {
            return Err(Error::ConstraintSystemFailure);
        }

        Ok(())
    }
}
