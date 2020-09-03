use super::{
    circuit::{AdviceWire, Circuit, ConstraintSystem, FixedWire, MetaCircuit},
    domain::Rotation,
    hash_point, Error, Proof, SRS,
};
use crate::arithmetic::{
    eval_polynomial, get_challenge_scalar, kate_division, parallelize, Challenge, Curve,
    CurveAffine, Field,
};
use crate::polycommit::Params;
use crate::transcript::Hasher;

impl<C: CurveAffine> Proof<C> {
    /// This creates a proof for the provided `circuit` when given the public
    /// parameters `params` and the structured reference string `srs` that was
    /// previously computed for the same circuit.
    pub fn create<
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
        ConcreteCircuit: Circuit<C::Scalar>,
    >(
        params: &Params<C>,
        srs: &SRS<C>,
        circuit: &ConcreteCircuit,
    ) -> Result<Self, Error> {
        struct WitnessCollection<F: Field> {
            advice: Vec<Vec<F>>,
        }

        impl<F: Field> ConstraintSystem<F> for WitnessCollection<F> {
            fn assign_advice(
                &mut self,
                wire: AdviceWire,
                row: usize,
                to: impl FnOnce() -> Result<F, Error>,
            ) -> Result<(), Error> {
                *self
                    .advice
                    .get_mut(wire.0)
                    .and_then(|v| v.get_mut(row))
                    .ok_or(Error::BoundsFailure)? = to()?;

                Ok(())
            }

            fn assign_fixed(
                &mut self,
                _: FixedWire,
                _: usize,
                _: impl FnOnce() -> Result<F, Error>,
            ) -> Result<(), Error> {
                // We only care about advice wires here

                Ok(())
            }

            fn copy(
                &mut self,
                _: usize,
                _: usize,
                _: usize,
                _: usize,
                _: usize,
            ) -> Result<(), Error> {
                // We only care about advice wires here

                Ok(())
            }
        }

        let mut meta = MetaCircuit::default();
        let config = ConcreteCircuit::configure(&mut meta);

        // Get the largest permutation argument length in terms of the number of
        // advice wires involved.
        let mut largest_permutation_length = 0;
        for permutation in &meta.permutations {
            largest_permutation_length =
                std::cmp::max(permutation.len(), largest_permutation_length);
        }

        let mut witness = WitnessCollection {
            advice: vec![vec![C::Scalar::zero(); params.n as usize]; meta.num_advice_wires],
        };

        // Synthesize the circuit to obtain the witness and other information.
        circuit.synthesize(&mut witness, config)?;

        // Create a transcript for obtaining Fiat-Shamir challenges.
        let mut transcript = HBase::init(C::Base::one());

        // Compute commitments to advice wire polynomials
        let advice_blinds: Vec<_> = witness.advice.iter().map(|_| C::Scalar::random()).collect();
        let advice_commitments = witness
            .advice
            .iter()
            .zip(advice_blinds.iter())
            .map(|(poly, blind)| params.commit_lagrange(poly, *blind).to_affine())
            .collect();

        for commitment in &advice_commitments {
            hash_point(&mut transcript, commitment)?;
        }

        let domain = &srs.domain;

        let advice_polys: Vec<_> = witness
            .advice
            .clone()
            .into_iter()
            .map(|poly| domain.obtain_poly(poly))
            .collect();

        let advice_cosets: Vec<_> = meta
            .advice_queries
            .iter()
            .map(|&(wire, at)| {
                let poly = advice_polys[wire.0].clone();
                domain.obtain_coset(poly, at)
            })
            .collect();

        // Sample x_0 challenge
        let x_0: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Sample x_1 challenge
        let x_1: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // TODO: maybe put this in SRS?
        // Compute [omega^0, omega^1, ..., omega^{params.n - 1}]
        let mut omega_powers = Vec::with_capacity(params.n as usize);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..params.n {
                omega_powers.push(cur);
                cur *= &srs.domain.get_omega();
            }
        }

        // Compute [omega_powers * \delta^0, omega_powers * \delta^1, ..., omega_powers * \delta^m]
        let mut deltaomega = Vec::with_capacity(largest_permutation_length);
        {
            let mut cur = C::Scalar::one();
            for _ in 0..largest_permutation_length {
                let mut omega_powers = omega_powers.clone();
                for o in &mut omega_powers {
                    *o *= &cur;
                }

                deltaomega.push(omega_powers);

                cur *= &C::Scalar::DELTA;
            }
        }

        // Compute permutation product polynomial commitment
        let mut permutation_product_commitments = vec![];
        let mut permutation_product_blinds = vec![];

        // Iterate over each permutation
        for (wires, permutations) in srs.meta.permutations.iter().zip(srs.permutations.iter()) {
            // Goal is to compute the fraction
            //
            // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
            // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
            //
            // where p_j(X) is the jth advice wire in this permutation,
            // and i is the ith row of the wire.
            let mut modified_advice = Vec::with_capacity(wires.len());

            // Iterate over each wire of the permutation
            for (wire, permutation) in wires.iter().zip(permutations.iter()) {
                // Grab the advice wire's values from the witness
                let mut tmp = witness.advice[wire.0].clone();

                // For each row i, compute
                // p_j(\omega^i) + \beta s_j(\omega^i) + \gamma
                // where p_j(omega^i) = tmp[i]
                for (tmp, permutation) in tmp.iter_mut().zip(permutation.iter()) {
                    *tmp += &(x_0 * permutation);
                    *tmp += &x_1;
                }
                modified_advice.push(tmp);
            }

            // Batch invert to obtain the denominators for the permutation product
            // polynomial
            for v in &mut modified_advice {
                C::Scalar::batch_invert(v);
            }

            // Iterate over each wire again, this time finishing the computation
            // of the entire fraction by computing the numerators
            for ((wire, modified_advice), deltaomega) in wires
                .iter()
                .zip(modified_advice.iter_mut())
                .zip(deltaomega.iter())
            {
                // For each row i, we compute
                // p_j(\omega^i) + \delta^j \omega^i \beta + \gamma
                // for the jth wire of the permutation
                for ((wire, modified_advice), deltaomega) in witness.advice[wire.0]
                    .iter_mut()
                    .zip(modified_advice.iter_mut())
                    .zip(deltaomega.iter())
                {
                    let mut tmp = *deltaomega; // \delta^j \omega^i
                    tmp *= &x_0; // \delta^j \omega^i \beta
                    tmp += &x_1; // \delta^j \omega^i \beta + \gamma
                    tmp += wire; // p_j(\omega^i) + \delta^j \omega^i \beta + \gamma
                    *modified_advice *= &tmp;
                }
            }

            // The modified_advice vector is a vector of vectors of fractions of
            // the form
            //
            // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
            // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
            //
            // where j is the index into modified_advice, and i is the index
            // into modified_advice[j], for the jth wire in the permutation

            // Compute the evaluations of the permutation product polynomial
            // over our domain, starting with z[0] = 1
            let mut z = vec![C::Scalar::one()];
            for i in 1..(params.n as usize) {
                let mut tmp = z[i - 1];

                // Iterate over each wire's modified advice, where for the jth
                // wire we obtain the fraction
                //
                // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
                // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
                //
                // where i is the row of the permutation product polynomial
                // evaluation vector that we are currently evaluating.
                for modified_advice in modified_advice.iter() {
                    tmp *= &modified_advice[i];
                }
                z.push(tmp);
            }

            let blind = C::Scalar::random();

            permutation_product_commitments.push(params.commit_lagrange(&z, blind).to_affine());
            permutation_product_blinds.push(blind);
        }

        // Hash each permutation product commitment
        for c in &permutation_product_commitments {
            hash_point(&mut transcript, c)?;
        }

        // Obtain challenge for keeping all separate gates linearly independent
        let x_2: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Evaluate the circuit using the custom gates provided
        let mut h_poly = vec![C::Scalar::zero(); domain.coset_len()];
        for (i, poly) in meta.gates.iter().enumerate() {
            if i != 0 {
                for h in h_poly.iter_mut() {
                    *h *= &x_2;
                }
            }

            let evaluation: Vec<C::Scalar> = poly.evaluate(
                &|index| srs.fixed_cosets[index].clone(),
                &|index| advice_cosets[index].clone(),
                &|mut a, b| {
                    parallelize(&mut a, |a, start| {
                        for (a, b) in a.iter_mut().zip(b[start..].iter()) {
                            *a += b;
                        }
                    });
                    a
                },
                &|mut a, b| {
                    parallelize(&mut a, |a, start| {
                        for (a, b) in a.iter_mut().zip(b[start..].iter()) {
                            *a *= b;
                        }
                    });
                    a
                },
                &|mut a, scalar| {
                    parallelize(&mut a, |a, _| {
                        for a in a {
                            *a *= &scalar;
                        }
                    });
                    a
                },
            );

            assert_eq!(h_poly.len(), evaluation.len());

            if i == 0 {
                h_poly = evaluation;
            } else {
                for (h, e) in h_poly.iter_mut().zip(evaluation.into_iter()) {
                    *h += &e;
                }
            }
        }

        // Divide by t(X) = X^{params.n} - 1.
        let h_poly = domain.divide_by_vanishing_poly(h_poly);

        // Obtain final h(X) polynomial
        let h_poly = domain.from_coset(h_poly);

        // Split h(X) up into pieces
        let h_pieces = h_poly
            .chunks_exact(params.n as usize)
            .map(|v| v.to_vec())
            .collect::<Vec<_>>();
        drop(h_poly);
        let h_blinds: Vec<_> = h_pieces.iter().map(|_| C::Scalar::random()).collect();

        // Compute commitments to each h(X) piece
        let h_commitments: Vec<_> = h_pieces
            .iter()
            .zip(h_blinds.iter())
            .map(|(h_piece, blind)| params.commit(&h_piece, *blind).to_affine())
            .collect();

        // Hash each h(X) piece
        for c in h_commitments.iter() {
            hash_point(&mut transcript, c)?;
        }

        let x_3: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Evaluate polynomials at omega^i x_3
        let advice_evals: Vec<_> = meta
            .advice_queries
            .iter()
            .map(|&(wire, at)| eval_polynomial(&advice_polys[wire.0], domain.rotate_omega(x_3, at)))
            .collect();

        let fixed_evals: Vec<_> = meta
            .fixed_queries
            .iter()
            .map(|&(wire, at)| {
                eval_polynomial(&srs.fixed_polys[wire.0], domain.rotate_omega(x_3, at))
            })
            .collect();

        let mut permutation_evals: Vec<Vec<C::Scalar>> =
            Vec::with_capacity(meta.permutation_queries.len());
        for (permutation_idx, queries) in meta.permutation_queries.iter().enumerate() {
            let query_evals: Vec<C::Scalar> = queries
                .iter()
                .map(|&query_index| {
                    eval_polynomial(&srs.permutation_polys[permutation_idx][query_index], x_3)
                })
                .collect();
            permutation_evals.push(query_evals);
        }

        let h_evals: Vec<_> = h_pieces
            .iter()
            .map(|poly| eval_polynomial(poly, x_3))
            .collect();

        // We set up a second transcript on the scalar field to hash in openings of
        // our polynomial commitments.
        let mut transcript_scalar = HScalar::init(C::Scalar::one());

        // Hash each advice evaluation
        for eval in advice_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        // Hash each fixed evaluation
        for eval in fixed_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        // Hash each permutation evaluation
        for permutation in permutation_evals.iter() {
            for eval in permutation.iter() {
                transcript_scalar.absorb(*eval);
            }
        }

        // Hash each h(x) piece evaluation
        for eval in h_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Collapse openings at same points together into single openings using
        // x_4 challenge.
        let mut q_polys: Vec<Option<Vec<_>>> = vec![None; meta.rotations.len()];
        let mut q_blinds = vec![C::Scalar::zero(); meta.rotations.len()];
        let mut q_evals: Vec<_> = vec![C::Scalar::zero(); meta.rotations.len()];
        {
            let mut accumulate = |point_index: usize, new_poly: &Vec<_>, blind, eval| {
                q_polys[point_index]
                    .as_mut()
                    .map(|poly| {
                        parallelize(poly, |q, start| {
                            for (q, a) in q.iter_mut().zip(new_poly[start..].iter()) {
                                *q *= &x_4;
                                *q += a;
                            }
                        });
                    })
                    .or_else(|| {
                        q_polys[point_index] = Some(new_poly.clone());
                        Some(())
                    });
                q_blinds[point_index] *= &x_4;
                q_blinds[point_index] += &blind;
                q_evals[point_index] *= &x_4;
                q_evals[point_index] += &eval;
            };

            for (query_index, &(wire, ref at)) in meta.advice_queries.iter().enumerate() {
                let point_index = (*meta.rotations.get(at).unwrap()).0;

                accumulate(
                    point_index,
                    &advice_polys[wire.0],
                    advice_blinds[wire.0],
                    advice_evals[query_index],
                );
            }

            for (query_index, &(wire, ref at)) in meta.fixed_queries.iter().enumerate() {
                let point_index = (*meta.rotations.get(at).unwrap()).0;

                accumulate(
                    point_index,
                    &srs.fixed_polys[wire.0],
                    C::Scalar::one(),
                    fixed_evals[query_index],
                );
            }

            // We query the h(X) polynomial at x_3
            let current_index = (*meta.rotations.get(&Rotation::default()).unwrap()).0;
            for ((h_poly, h_blind), h_eval) in h_pieces
                .into_iter()
                .zip(h_blinds.iter())
                .zip(h_evals.iter())
            {
                accumulate(current_index, &h_poly, *h_blind, *h_eval);
            }
        }

        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut f_poly: Option<Vec<C::Scalar>> = None;
        for (&row, &point_index) in meta.rotations.iter() {
            let mut poly = q_polys[point_index.0].as_ref().unwrap().clone();
            let point = domain.rotate_omega(x_3, row);
            poly[0] -= &q_evals[point_index.0];
            let mut poly = kate_division(&poly, point);
            poly.push(C::Scalar::zero());

            f_poly = f_poly
                .map(|mut f_poly| {
                    parallelize(&mut f_poly, |q, start| {
                        for (q, a) in q.iter_mut().zip(poly[start..].iter()) {
                            *q *= &x_5;
                            *q += a;
                        }
                    });
                    f_poly
                })
                .or_else(|| Some(poly));
        }
        let mut f_poly = f_poly.unwrap();
        let mut f_blind = C::Scalar::random();

        let f_commitment = params.commit(&f_poly, f_blind).to_affine();

        hash_point(&mut transcript, &f_commitment)?;

        let x_6: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut q_evals = vec![];

        for (_, &point_index) in meta.rotations.iter() {
            q_evals.push(eval_polynomial(
                &q_polys[point_index.0].as_ref().unwrap(),
                x_6,
            ));
        }

        for eval in q_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let x_7: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        for (_, &point_index) in meta.rotations.iter() {
            f_blind *= &x_7;
            f_blind += &q_blinds[point_index.0];

            parallelize(&mut f_poly, |f, start| {
                for (f, a) in f
                    .iter_mut()
                    .zip(q_polys[point_index.0].as_ref().unwrap()[start..].iter())
                {
                    *f *= &x_7;
                    *f += a;
                }
            });
        }

        // Let's prove that the q_commitment opens at x to the expected value.
        let opening = params
            .create_proof(&mut transcript, &f_poly, f_blind, x_6)
            .map_err(|_| Error::ConstraintSystemFailure)?;

        Ok(Proof {
            advice_commitments,
            h_commitments,
            permutation_product_commitments,
            permutation_product_evals: vec![C::Scalar::one(); params.n as usize],
            permutation_product_inv_evals: vec![C::Scalar::one(); params.n as usize],
            permutation_evals,
            advice_evals,
            fixed_evals,
            h_evals,
            f_commitment,
            q_evals,
            opening,
        })
    }
}
