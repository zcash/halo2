use super::{
    circuit::{AdviceWire, Assignment, Circuit, ConstraintSystem, FixedWire, Wire},
    hash_point, Error, Proof, SRS,
};
use crate::arithmetic::{
    eval_polynomial, get_challenge_scalar, kate_division, parallelize, BatchInvert, Challenge,
    Curve, CurveAffine, Field,
};
use crate::poly::{
    commitment::{Blind, OpeningProof, Params},
    Coeff, LagrangeCoeff, Polynomial, Rotation,
};
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
        aux_lagrange_polys: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    ) -> Result<Self, Error> {
        struct WitnessCollection<F: Field> {
            advice: Vec<Polynomial<F, LagrangeCoeff>>,
            _marker: std::marker::PhantomData<F>,
        }

        impl<F: Field> Assignment<F> for WitnessCollection<F> {
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

        let domain = &srs.domain;
        let mut meta = ConstraintSystem::default();
        let config = ConcreteCircuit::configure(&mut meta);

        let mut witness = WitnessCollection {
            advice: vec![domain.empty_lagrange(); meta.num_advice_wires],
            _marker: std::marker::PhantomData,
        };

        // Synthesize the circuit to obtain the witness and other information.
        circuit.synthesize(&mut witness, config)?;

        let witness = witness;

        // Create a transcript for obtaining Fiat-Shamir challenges.
        let mut transcript = HBase::init(C::Base::one());

        // Compute commitments to advice wire polynomials
        let advice_blinds: Vec<_> = witness
            .advice
            .iter()
            .map(|_| Blind(C::Scalar::random()))
            .collect();
        let advice_commitments_projective: Vec<_> = witness
            .advice
            .iter()
            .zip(advice_blinds.iter())
            .map(|(poly, blind)| params.commit_lagrange(poly, *blind))
            .collect();
        let mut advice_commitments = vec![C::zero(); advice_commitments_projective.len()];
        C::Projective::batch_to_affine(&advice_commitments_projective, &mut advice_commitments);
        let advice_commitments = advice_commitments;
        drop(advice_commitments_projective);

        for commitment in &advice_commitments {
            hash_point(&mut transcript, commitment)?;
        }

        let advice_polys: Vec<_> = witness
            .advice
            .clone()
            .into_iter()
            .map(|poly| domain.lagrange_to_coeff(poly))
            .collect();

        let advice_cosets: Vec<_> = meta
            .advice_queries
            .iter()
            .map(|&(wire, at)| {
                let poly = advice_polys[wire.0].clone();
                domain.coeff_to_extended(poly, at)
            })
            .collect();

        // Compute commitments to auxiliary wire polynomials
        let aux_commitments_projective: Vec<_> = aux_lagrange_polys
            .iter()
            .map(|poly| params.commit_lagrange(poly, Blind::default()))
            .collect();
        let mut aux_commitments = vec![C::zero(); aux_commitments_projective.len()];
        C::Projective::batch_to_affine(&aux_commitments_projective, &mut aux_commitments);
        let aux_commitments = aux_commitments;
        drop(aux_commitments_projective);

        for commitment in &aux_commitments {
            hash_point(&mut transcript, commitment)?;
        }

        let aux_polys: Vec<_> = aux_lagrange_polys
            .clone()
            .into_iter()
            .map(|poly| domain.lagrange_to_coeff(poly))
            .collect();

        let aux_cosets: Vec<_> = meta
            .aux_queries
            .iter()
            .map(|&(wire, at)| {
                let poly = aux_polys[wire.0].clone();
                domain.coeff_to_extended(poly, at)
            })
            .collect();

        // Sample x_0 challenge
        let x_0: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Sample x_1 challenge
        let x_1: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Compute permutation product polynomial commitment
        let mut permutation_product_polys = vec![];
        let mut permutation_product_cosets = vec![];
        let mut permutation_product_cosets_inv = vec![];
        let mut permutation_product_commitments_projective = vec![];
        let mut permutation_product_blinds = vec![];

        // Iterate over each permutation
        let mut permutation_modified_advice = vec![];
        for (wires, permuted_values) in srs.cs.permutations.iter().zip(srs.permutations.iter()) {
            // Goal is to compute the products of fractions
            //
            // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
            // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
            //
            // where p_j(X) is the jth advice wire in this permutation,
            // and i is the ith row of the wire.
            let mut modified_advice = vec![C::Scalar::one(); params.n as usize];

            // Iterate over each wire of the permutation
            for (&(wire, _), permuted_wire_values) in wires.iter().zip(permuted_values.iter()) {
                match wire {
                    Wire::Advice(wire) => {
                        parallelize(&mut modified_advice, |modified_advice, start| {
                            for ((modified_advice, advice_value), permuted_advice_value) in
                                modified_advice
                                    .iter_mut()
                                    .zip(witness.advice[wire.0][start..].iter())
                                    .zip(permuted_wire_values[start..].iter())
                            {
                                *modified_advice *=
                                    &(x_0 * permuted_advice_value + &x_1 + advice_value);
                            }
                        });
                    }
                    Wire::Aux(wire) => {
                        parallelize(&mut modified_advice, |modified_aux, start| {
                            for ((modified_aux, aux_value), permuted_aux_value) in modified_aux
                                .iter_mut()
                                .zip(aux_lagrange_polys[wire.0][start..].iter())
                                .zip(permuted_wire_values[start..].iter())
                            {
                                *modified_aux *= &(x_0 * permuted_aux_value + &x_1 + aux_value);
                            }
                        });
                    }
                    // TODO: implement for fixed wires
                    _ => unreachable!(),
                }
            }

            permutation_modified_advice.push(modified_advice);
        }

        // Batch invert to obtain the denominators for the permutation product
        // polynomials
        permutation_modified_advice
            .iter_mut()
            .flat_map(|v| v.iter_mut())
            .batch_invert();

        for (wires, mut modified_advice) in srs
            .cs
            .permutations
            .iter()
            .zip(permutation_modified_advice.into_iter())
        {
            // Iterate over each wire again, this time finishing the computation
            // of the entire fraction by computing the numerators
            let mut deltaomega = C::Scalar::one();
            for &(wire, _) in wires.iter() {
                let omega = domain.get_omega();
                match wire {
                    Wire::Advice(wire) => {
                        parallelize(&mut modified_advice, |modified_advice, start| {
                            let mut deltaomega =
                                deltaomega * &omega.pow_vartime(&[start as u64, 0, 0, 0]);
                            for (modified_advice, advice_value) in modified_advice
                                .iter_mut()
                                .zip(witness.advice[wire.0][start..].iter())
                            {
                                // Multiply by p_j(\omega^i) + \delta^j \omega^i \beta
                                *modified_advice *= &(deltaomega * &x_0 + &x_1 + advice_value);
                                deltaomega *= &omega;
                            }
                        });
                    }
                    Wire::Aux(wire) => {
                        parallelize(&mut modified_advice, |modified_advice, start| {
                            let mut deltaomega =
                                deltaomega * &omega.pow_vartime(&[start as u64, 0, 0, 0]);
                            for (modified_advice, advice_value) in modified_advice
                                .iter_mut()
                                .zip(aux_lagrange_polys[wire.0][start..].iter())
                            {
                                // Multiply by p_j(\omega^i) + \delta^j \omega^i \beta
                                *modified_advice *= &(deltaomega * &x_0 + &x_1 + advice_value);
                                deltaomega *= &omega;
                            }
                        });
                    }
                    // TODO: implement for fixed wires
                    _ => unreachable!(),
                }
                deltaomega *= &C::Scalar::DELTA;
            }

            // The modified_advice vector is a vector of products of fractions
            // of the form
            //
            // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
            // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
            //
            // where i is the index into modified_advice, for the jth wire in
            // the permutation

            // Compute the evaluations of the permutation product polynomial
            // over our domain, starting with z[0] = 1
            let mut z = vec![C::Scalar::one()];
            for row in 1..(params.n as usize) {
                let mut tmp = z[row - 1];

                tmp *= &modified_advice[row];
                z.push(tmp);
            }
            let z = domain.lagrange_from_vec(z);

            let blind = Blind(C::Scalar::random());

            permutation_product_commitments_projective.push(params.commit_lagrange(&z, blind));
            permutation_product_blinds.push(blind);
            let z = domain.lagrange_to_coeff(z);
            permutation_product_polys.push(z.clone());
            permutation_product_cosets
                .push(domain.coeff_to_extended(z.clone(), Rotation::default()));
            permutation_product_cosets_inv.push(domain.coeff_to_extended(z, Rotation(-1)));
        }
        let mut permutation_product_commitments =
            vec![C::zero(); permutation_product_commitments_projective.len()];
        C::Projective::batch_to_affine(
            &permutation_product_commitments_projective,
            &mut permutation_product_commitments,
        );
        let permutation_product_commitments = permutation_product_commitments;
        drop(permutation_product_commitments_projective);

        // Hash each permutation product commitment
        for c in &permutation_product_commitments {
            hash_point(&mut transcript, c)?;
        }

        // Obtain challenge for keeping all separate gates linearly independent
        let x_2: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Evaluate the circuit using the custom gates provided
        let mut h_poly = domain.empty_extended();
        for poly in meta.gates.iter() {
            h_poly = h_poly * x_2;

            let evaluation = poly.evaluate(
                &|index| srs.fixed_cosets[index].clone(),
                &|index| advice_cosets[index].clone(),
                &|index| aux_cosets[index].clone(),
                &|a, b| a + &b,
                &|a, b| a * &b,
                &|a, scalar| a * scalar,
            );

            h_poly = h_poly + &evaluation;
        }

        // l_0(X) * (1 - z(X)) = 0
        for coset in permutation_product_cosets.iter() {
            parallelize(&mut h_poly, |h, start| {
                for ((h, c), l0) in h
                    .iter_mut()
                    .zip(coset[start..].iter())
                    .zip(srs.l0[start..].iter())
                {
                    *h *= &x_2;
                    *h += &(*l0 * &(C::Scalar::one() - c));
                }
            });
        }

        // z(X) \prod (p(X) + \beta s_i(X) + \gamma) - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
        for (permutation_index, wires) in srs.cs.permutations.iter().enumerate() {
            h_poly = h_poly * x_2;

            let mut left = permutation_product_cosets[permutation_index].clone();
            for (advice, permutation) in wires
                .iter()
                .map(|&(_, index)| &advice_cosets[index])
                .zip(srs.permutation_cosets[permutation_index].iter())
            {
                parallelize(&mut left, |left, start| {
                    for ((left, advice), permutation) in left
                        .iter_mut()
                        .zip(advice[start..].iter())
                        .zip(permutation[start..].iter())
                    {
                        *left *= &(*advice + &(x_0 * permutation) + &x_1);
                    }
                });
            }

            let mut right = permutation_product_cosets_inv[permutation_index].clone();
            let mut current_delta = x_0 * &C::Scalar::ZETA;
            let step = domain.get_extended_omega();
            for advice in wires.iter().map(|&(_, index)| &advice_cosets[index]) {
                parallelize(&mut right, move |right, start| {
                    let mut beta_term = current_delta * &step.pow_vartime(&[start as u64, 0, 0, 0]);
                    for (right, advice) in right.iter_mut().zip(advice[start..].iter()) {
                        *right *= &(*advice + &beta_term + &x_1);
                        beta_term *= &step;
                    }
                });
                current_delta *= &C::Scalar::DELTA;
            }

            h_poly = h_poly + &left - &right;
        }

        // Divide by t(X) = X^{params.n} - 1.
        let h_poly = domain.divide_by_vanishing_poly(h_poly);

        // Obtain final h(X) polynomial
        let h_poly = domain.extended_to_coeff(h_poly);

        // Split h(X) up into pieces
        let h_pieces = h_poly
            .chunks_exact(params.n as usize)
            .map(|v| domain.coeff_from_vec(v.to_vec()))
            .collect::<Vec<_>>();
        drop(h_poly);
        let h_blinds: Vec<_> = h_pieces
            .iter()
            .map(|_| Blind(C::Scalar::random()))
            .collect();

        // Compute commitments to each h(X) piece
        let h_commitments_projective: Vec<_> = h_pieces
            .iter()
            .zip(h_blinds.iter())
            .map(|(h_piece, blind)| params.commit(&h_piece, *blind))
            .collect();
        let mut h_commitments = vec![C::zero(); h_commitments_projective.len()];
        C::Projective::batch_to_affine(&h_commitments_projective, &mut h_commitments);
        let h_commitments = h_commitments;
        drop(h_commitments_projective);

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

        let aux_evals: Vec<_> = meta
            .aux_queries
            .iter()
            .map(|&(wire, at)| eval_polynomial(&aux_polys[wire.0], domain.rotate_omega(x_3, at)))
            .collect();

        let fixed_evals: Vec<_> = meta
            .fixed_queries
            .iter()
            .map(|&(wire, at)| {
                eval_polynomial(&srs.fixed_polys[wire.0], domain.rotate_omega(x_3, at))
            })
            .collect();

        let permutation_product_evals: Vec<C::Scalar> = permutation_product_polys
            .iter()
            .map(|poly| eval_polynomial(poly, x_3))
            .collect();

        let permutation_product_inv_evals: Vec<C::Scalar> = permutation_product_polys
            .iter()
            .map(|poly| eval_polynomial(poly, domain.rotate_omega(x_3, Rotation(-1))))
            .collect();

        let permutation_evals: Vec<Vec<C::Scalar>> = srs
            .permutation_polys
            .iter()
            .map(|polys| {
                polys
                    .iter()
                    .map(|poly| eval_polynomial(poly, x_3))
                    .collect()
            })
            .collect();

        let h_evals: Vec<_> = h_pieces
            .iter()
            .map(|poly| eval_polynomial(poly, x_3))
            .collect();

        // We set up a second transcript on the scalar field to hash in openings of
        // our polynomial commitments.
        let mut transcript_scalar = HScalar::init(C::Scalar::one());

        // Hash each advice evaluation
        for eval in advice_evals
            .iter()
            .chain(aux_evals.iter())
            .chain(fixed_evals.iter())
            .chain(h_evals.iter())
            .chain(permutation_product_evals.iter())
            .chain(permutation_product_inv_evals.iter())
            .chain(permutation_evals.iter().flat_map(|evals| evals.iter()))
        {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Collapse openings at same points together into single openings using
        // x_4 challenge.
        let mut q_polys: Vec<Option<Polynomial<C::Scalar, Coeff>>> =
            vec![None; meta.rotations.len()];
        let mut q_blinds = vec![Blind(C::Scalar::zero()); meta.rotations.len()];
        let mut q_evals: Vec<_> = vec![C::Scalar::zero(); meta.rotations.len()];
        {
            let mut accumulate =
                |point_index: usize, new_poly: &Polynomial<_, Coeff>, blind, eval| {
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
                    q_blinds[point_index] *= x_4;
                    q_blinds[point_index] += blind;
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

            for (query_index, &(wire, ref at)) in meta.aux_queries.iter().enumerate() {
                let point_index = (*meta.rotations.get(at).unwrap()).0;

                accumulate(
                    point_index,
                    &aux_polys[wire.0],
                    Blind::default(),
                    aux_evals[query_index],
                );
            }

            for (query_index, &(wire, ref at)) in meta.fixed_queries.iter().enumerate() {
                let point_index = (*meta.rotations.get(at).unwrap()).0;

                accumulate(
                    point_index,
                    &srs.fixed_polys[wire.0],
                    Blind::default(),
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

            // Handle permutation arguments, if any exist
            if !srs.cs.permutations.is_empty() {
                // Open permutation product commitments at x_3
                for ((poly, blind), eval) in permutation_product_polys
                    .iter()
                    .zip(permutation_product_blinds.iter())
                    .zip(permutation_product_evals.iter())
                {
                    accumulate(current_index, poly, *blind, *eval);
                }

                // Open permutation polynomial commitments at x_3
                for (poly, eval) in srs
                    .permutation_polys
                    .iter()
                    .zip(permutation_evals.iter())
                    .flat_map(|(polys, evals)| polys.iter().zip(evals.iter()))
                {
                    accumulate(current_index, poly, Blind::default(), *eval);
                }

                let current_index = (*srs.cs.rotations.get(&Rotation(-1)).unwrap()).0;
                // Open permutation product commitments at \omega^{-1} x_3
                for ((poly, blind), eval) in permutation_product_polys
                    .iter()
                    .zip(permutation_product_blinds.iter())
                    .zip(permutation_product_inv_evals.iter())
                {
                    accumulate(current_index, poly, *blind, *eval);
                }
            }
        }

        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut f_poly: Option<Polynomial<C::Scalar, Coeff>> = None;
        for (&row, &point_index) in meta.rotations.iter() {
            let mut poly = q_polys[point_index.0].as_ref().unwrap().clone();
            let point = domain.rotate_omega(x_3, row);
            poly[0] -= &q_evals[point_index.0];
            // TODO: change kate_division interface?
            let mut poly = kate_division(&poly[..], point);
            poly.push(C::Scalar::zero());
            let poly = domain.coeff_from_vec(poly);

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

        let f_poly = f_poly.unwrap();
        let mut f_blind = Blind(C::Scalar::random());
        let mut f_commitment = params.commit(&f_poly, f_blind).to_affine();

        let (opening, q_evals) = loop {
            let mut transcript = transcript.clone();
            let mut transcript_scalar = transcript_scalar.clone();
            hash_point(&mut transcript, &f_commitment)?;

            let x_6: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let mut q_evals = vec![C::Scalar::zero(); meta.rotations.len()];

            for (_, &point_index) in meta.rotations.iter() {
                q_evals[point_index.0] =
                    eval_polynomial(&q_polys[point_index.0].as_ref().unwrap(), x_6);
            }

            for eval in q_evals.iter() {
                transcript_scalar.absorb(*eval);
            }

            let transcript_scalar_point =
                C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
            transcript.absorb(transcript_scalar_point);

            let x_7: C::Scalar =
                get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

            let mut f_blind_dup = f_blind.clone();
            let mut f_poly = f_poly.clone();
            for (_, &point_index) in meta.rotations.iter() {
                f_blind_dup *= x_7;
                f_blind_dup += q_blinds[point_index.0];

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
            let opening = OpeningProof::create(&params, &mut transcript, &f_poly, f_blind_dup, x_6);

            if opening.is_ok() {
                break (opening.unwrap(), q_evals);
            } else {
                f_blind += C::Scalar::one();
                f_commitment = (f_commitment + params.h).to_affine();
            }
        };

        Ok(Proof {
            advice_commitments,
            h_commitments,
            permutation_product_commitments,
            permutation_product_evals,
            permutation_product_inv_evals,
            permutation_evals,
            advice_evals,
            fixed_evals,
            aux_evals,
            h_evals,
            f_commitment,
            q_evals,
            opening,
        })
    }
}
