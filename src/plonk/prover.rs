use super::{
    circuit::{AdviceWire, Circuit, ConstraintSystem, FixedWire, MetaCircuit},
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
        }

        let mut meta = MetaCircuit::default();
        let config = ConcreteCircuit::configure(&mut meta);

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
                        for (a, b) in a.into_iter().zip(b[start..].iter()) {
                            *a += b;
                        }
                    });
                    a
                },
                &|mut a, b| {
                    parallelize(&mut a, |a, start| {
                        for (a, b) in a.into_iter().zip(b[start..].iter()) {
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
        let advice_evals_x: Vec<_> = meta
            .advice_queries
            .iter()
            .map(|&(wire, at)| {
                let mut point = x_3;
                if at >= 0 {
                    point *= &domain.get_omega().pow(&[at as u64, 0, 0, 0]);
                } else {
                    point *= &domain.get_omega_inv().pow(&[at.abs() as u64, 0, 0, 0]);
                }

                eval_polynomial(&advice_polys[wire.0], point)
            })
            .collect();

        let fixed_evals_x: Vec<_> = meta
            .fixed_queries
            .iter()
            .map(|&(wire, at)| {
                let mut point = x_3;
                if at >= 0 {
                    point *= &domain.get_omega().pow(&[at as u64, 0, 0, 0]);
                } else {
                    point *= &domain.get_omega_inv().pow(&[at.abs() as u64, 0, 0, 0]);
                }

                eval_polynomial(&srs.fixed_polys[wire.0], point)
            })
            .collect();

        let h_evals_x: Vec<_> = h_pieces
            .iter()
            .map(|poly| eval_polynomial(poly, x_3))
            .collect();

        // We set up a second transcript on the scalar field to hash in openings of
        // our polynomial commitments.
        let mut transcript_scalar = HScalar::init(C::Scalar::one());

        // Hash each advice evaluation
        for eval in advice_evals_x.iter() {
            transcript_scalar.absorb(*eval);
        }

        // Hash each fixed evaluation
        for eval in fixed_evals_x.iter() {
            transcript_scalar.absorb(*eval);
        }

        // Hash each h(x) piece evaluation
        for eval in h_evals_x.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let x_4: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Collapse openings at same points together into single openings using
        // x_4 challenge.
        let mut q_polys: Vec<Option<Vec<_>>> = vec![None; meta.query_rows.len()];
        let mut q_blinds = vec![C::Scalar::zero(); meta.query_rows.len()];
        let mut q_evals: Vec<_> = vec![C::Scalar::zero(); meta.query_rows.len()];
        {
            for (i, &(wire, ref at)) in meta.advice_queries.iter().enumerate() {
                let query_row = *meta.query_rows.get(at).unwrap();

                if q_polys[query_row].is_none() {
                    q_polys[query_row] = Some(advice_polys[wire.0].clone());
                    q_blinds[query_row] = advice_blinds[wire.0];
                    q_evals[query_row] = advice_evals_x[i];
                } else {
                    parallelize(q_polys[query_row].as_mut().unwrap(), |q, start| {
                        for (q, a) in q.iter_mut().zip(advice_polys[wire.0][start..].iter()) {
                            *q *= &x_4;
                            *q += a;
                        }
                    });
                    q_blinds[query_row] *= &x_4;
                    q_blinds[query_row] += &advice_blinds[wire.0];
                    q_evals[query_row] *= &x_4;
                    q_evals[query_row] += &advice_evals_x[i];
                }
            }

            for (i, &(wire, ref at)) in meta.fixed_queries.iter().enumerate() {
                let query_row = *meta.query_rows.get(at).unwrap();

                if q_polys[query_row].is_none() {
                    q_polys[query_row] = Some(srs.fixed_polys[wire.0].clone());
                    q_blinds[query_row] = C::Scalar::one();
                    q_evals[query_row] = fixed_evals_x[i];
                } else {
                    parallelize(q_polys[query_row].as_mut().unwrap(), |q, start| {
                        for (q, a) in q.iter_mut().zip(srs.fixed_polys[wire.0][start..].iter()) {
                            *q *= &x_4;
                            *q += a;
                        }
                    });
                    q_blinds[query_row] *= &x_4;
                    q_blinds[query_row] += &C::Scalar::one();
                    q_evals[query_row] *= &x_4;
                    q_evals[query_row] += &fixed_evals_x[i];
                }
            }

            for ((h_poly, h_blind), h_eval) in h_pieces
                .into_iter()
                .zip(h_blinds.iter())
                .zip(h_evals_x.iter())
            {
                // We query the h(X) polynomial at x_3
                let cur_row = *meta.query_rows.get(&0).unwrap();

                if q_polys[cur_row].is_none() {
                    q_polys[cur_row] = Some(h_poly);
                    q_blinds[cur_row] = *h_blind;
                    q_evals[cur_row] = *h_eval;
                } else {
                    parallelize(q_polys[cur_row].as_mut().unwrap(), |q, start| {
                        for (q, a) in q.iter_mut().zip(h_poly[start..].iter()) {
                            *q *= &x_4;
                            *q += a;
                        }
                    });
                    q_blinds[cur_row] *= &x_4;
                    q_blinds[cur_row] += h_blind;
                    q_evals[cur_row] *= &x_4;
                    q_evals[cur_row] += h_eval;
                }
            }
        }

        let x_5: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut f_poly = None;
        for (&row, &col) in meta.query_rows.iter() {
            let mut poly = q_polys[col].as_ref().unwrap().clone();
            let mut point = x_3;
            if row >= 0 {
                point *= &domain.get_omega().pow_vartime(&[row as u64, 0, 0, 0]);
            } else {
                point *= &domain
                    .get_omega_inv()
                    .pow_vartime(&[row.abs() as u64, 0, 0, 0]);
            }
            poly[0] -= &q_evals[col];
            let mut poly = kate_division(&poly, point);
            poly.push(C::Scalar::zero());

            if f_poly.is_none() {
                f_poly = Some(poly);
            } else {
                parallelize(f_poly.as_mut().unwrap(), |q, start| {
                    for (q, a) in q.iter_mut().zip(poly[start..].iter()) {
                        *q *= &x_5;
                        *q += a;
                    }
                });
            }
        }
        let mut f_poly = f_poly.unwrap();
        let mut f_blind = C::Scalar::random();

        let f_commitment = params.commit(&f_poly, f_blind).to_affine();

        hash_point(&mut transcript, &f_commitment)?;

        let x_6: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut q_evals = vec![];

        for (_, &col) in meta.query_rows.iter() {
            q_evals.push(eval_polynomial(&q_polys[col].as_ref().unwrap(), x_6));
        }

        for eval in q_evals.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let x_7: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        for (_, &col) in meta.query_rows.iter() {
            f_blind *= &x_7;
            f_blind += &q_blinds[col];

            parallelize(&mut f_poly, |f, start| {
                for (f, a) in f
                    .iter_mut()
                    .zip(q_polys[col].as_ref().unwrap()[start..].iter())
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
            advice_evals_x,
            fixed_evals_x,
            h_evals_x,
            f_commitment,
            q_evals,
            opening,
        })
    }
}
