use super::{
    circuit::{Circuit, ConstraintSystem, Wire},
    hash_point, Error, Proof, SRS,
};
use crate::arithmetic::{
    eval_polynomial, get_challenge_scalar, Challenge, Curve, CurveAffine, Field,
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
            a: Vec<F>,
            b: Vec<F>,
            c: Vec<F>,
            d: Vec<F>,
            sa: Vec<F>,
            sb: Vec<F>,
            sc: Vec<F>,
            sd: Vec<F>,
            sm: Vec<F>,
        }

        impl<F: Field> ConstraintSystem<F> for WitnessCollection<F> {
            fn create_gate(
                &mut self,
                sa: F,
                sb: F,
                sc: F,
                sd: F,
                sm: F,
                f: impl Fn() -> Result<(F, F, F, F), Error>,
            ) -> Result<(Wire, Wire, Wire, Wire), Error> {
                let (a, b, c, d) = f()?;
                let tmp = Ok((
                    Wire::A(self.a.len()),
                    Wire::B(self.a.len()),
                    Wire::C(self.a.len()),
                    Wire::D(self.a.len()),
                ));
                self.a.push(a);
                self.b.push(b);
                self.c.push(c);
                self.d.push(d);
                self.sa.push(sa);
                self.sb.push(sb);
                self.sc.push(sc);
                self.sd.push(sd);
                self.sm.push(sm);
                tmp
            }
            // fn copy(&mut self, left: Wire, right: Wire) {
            //     unimplemented!()
            // }
        }

        let mut witness = WitnessCollection {
            a: vec![],
            b: vec![],
            c: vec![],
            d: vec![],
            sa: vec![],
            sb: vec![],
            sc: vec![],
            sd: vec![],
            sm: vec![],
        };

        // Synthesize the circuit to obtain the witness and other information.
        circuit.synthesize(&mut witness)?;

        // Create a transcript for obtaining Fiat-Shamir challenges.
        let mut transcript = HBase::init(C::Base::one());

        if witness.a.len() > params.n as usize {
            // The polynomial commitment does not support a high enough degree
            // polynomial to commit to our wires because this circuit has too
            // many gates.
            return Err(Error::IncompatibleParams);
        }

        witness.a.resize(params.n as usize, C::Scalar::zero());
        witness.b.resize(params.n as usize, C::Scalar::zero());
        witness.c.resize(params.n as usize, C::Scalar::zero());
        witness.d.resize(params.n as usize, C::Scalar::zero());
        witness.sa.resize(params.n as usize, C::Scalar::zero());
        witness.sb.resize(params.n as usize, C::Scalar::zero());
        witness.sc.resize(params.n as usize, C::Scalar::zero());
        witness.sd.resize(params.n as usize, C::Scalar::zero());
        witness.sm.resize(params.n as usize, C::Scalar::zero());

        // Compute commitments to the various wire values
        let a_blind = C::Scalar::one(); // TODO: not random
        let b_blind = C::Scalar::one(); // TODO: not random
        let c_blind = C::Scalar::one(); // TODO: not random
        let d_blind = C::Scalar::one(); // TODO: not random
        let a_commitment = params.commit_lagrange(&witness.a, a_blind).to_affine();
        let b_commitment = params.commit_lagrange(&witness.b, b_blind).to_affine();
        let c_commitment = params.commit_lagrange(&witness.c, c_blind).to_affine();
        let d_commitment = params.commit_lagrange(&witness.d, d_blind).to_affine();

        hash_point(&mut transcript, &a_commitment)?;
        hash_point(&mut transcript, &b_commitment)?;
        hash_point(&mut transcript, &c_commitment)?;
        hash_point(&mut transcript, &d_commitment)?;

        let domain = &srs.domain;

        let (a_coset, a_poly) = domain.obtain_coset(witness.a);
        let (b_coset, b_poly) = domain.obtain_coset(witness.b);
        let (c_coset, c_poly) = domain.obtain_coset(witness.c);
        let (d_coset, d_poly) = domain.obtain_coset(witness.d);

        // (a * sa) + (b * sb) + (a * sm * b) + (d * sd) - (c * sc)
        let mut h_poly = Vec::with_capacity(a_coset.len());
        for ((((((((a, b), c), d), sa), sb), sc), sd), sm) in a_coset
            .iter()
            .zip(b_coset.iter())
            .zip(c_coset.iter())
            .zip(d_coset.iter())
            .zip(srs.sa.0.iter())
            .zip(srs.sb.0.iter())
            .zip(srs.sc.0.iter())
            .zip(srs.sd.0.iter())
            .zip(srs.sm.0.iter())
        {
            h_poly.push((*a) * sa + &((*b) * sb) + &((*a) * sm * b) + &((*d) * sd) - &((*c) * sc));
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
        let h_blinds = vec![C::Scalar::one(); h_pieces.len()]; // TODO: not random

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

        let x: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        // Evaluate polynomials at x
        let a_eval_x = eval_polynomial(&a_poly, x);
        let b_eval_x = eval_polynomial(&b_poly, x);
        let c_eval_x = eval_polynomial(&c_poly, x);
        let d_eval_x = eval_polynomial(&d_poly, x);
        let sa_eval_x = eval_polynomial(&srs.sa.1, x);
        let sb_eval_x = eval_polynomial(&srs.sb.1, x);
        let sc_eval_x = eval_polynomial(&srs.sc.1, x);
        let sd_eval_x = eval_polynomial(&srs.sd.1, x);
        let sm_eval_x = eval_polynomial(&srs.sm.1, x);

        let h_evals_x: Vec<_> = h_pieces
            .iter()
            .map(|poly| eval_polynomial(poly, x))
            .collect();

        // We set up a second transcript on the scalar field to hash in openings of
        // our polynomial commitments.
        let mut transcript_scalar = HScalar::init(C::Scalar::one());
        transcript_scalar.absorb(a_eval_x);
        transcript_scalar.absorb(b_eval_x);
        transcript_scalar.absorb(c_eval_x);
        transcript_scalar.absorb(d_eval_x);
        transcript_scalar.absorb(sa_eval_x);
        transcript_scalar.absorb(sb_eval_x);
        transcript_scalar.absorb(sc_eval_x);
        transcript_scalar.absorb(sd_eval_x);
        transcript_scalar.absorb(sm_eval_x);

        // Hash each h(x) piece
        for eval in h_evals_x.iter() {
            transcript_scalar.absorb(*eval);
        }

        let transcript_scalar_point =
            C::Base::from_bytes(&(transcript_scalar.squeeze()).to_bytes()).unwrap();
        transcript.absorb(transcript_scalar_point);

        let y: C::Scalar = get_challenge_scalar(Challenge(transcript.squeeze().get_lower_128()));

        let mut q_commitment = h_commitments[0].clone().to_projective();
        let mut q_poly = h_pieces[0].clone();
        let mut q_blind = h_blinds[0];
        {
            let mut accumulate = |poly: &[_], blind: &C::Scalar, commitment: C| {
                for (a, q) in poly.iter().zip(q_poly.iter_mut()) {
                    *q = (*q * &y) + a;
                }
                q_commitment = (q_commitment * y) + &commitment.to_projective();
                q_blind = (q_blind * &y) + blind;
            };

            for ((poly, blind), commitment) in h_pieces
                .iter()
                .zip(h_blinds.iter())
                .zip(h_commitments.iter())
                .skip(1)
            {
                accumulate(&poly, blind, *commitment);
            }

            accumulate(&a_poly, &a_blind, a_commitment);
            accumulate(&b_poly, &b_blind, b_commitment);
            accumulate(&c_poly, &c_blind, c_commitment);
            accumulate(&d_poly, &d_blind, d_commitment);
            accumulate(&srs.sa.1, &Field::one(), srs.sa_commitment);
            accumulate(&srs.sb.1, &Field::one(), srs.sb_commitment);
            accumulate(&srs.sc.1, &Field::one(), srs.sc_commitment);
            accumulate(&srs.sd.1, &Field::one(), srs.sd_commitment);
            accumulate(&srs.sm.1, &Field::one(), srs.sm_commitment);
        }

        // Let's prove that the q_commitment opens at x to the expected value.
        let opening = params
            .create_proof(&mut transcript, &q_poly, q_blind, x)
            .map_err(|_| Error::ConstraintSystemFailure)?;

        Ok(Proof {
            a_commitment,
            b_commitment,
            c_commitment,
            d_commitment,
            h_commitments,
            a_eval_x,
            b_eval_x,
            c_eval_x,
            d_eval_x,
            sa_eval_x,
            sb_eval_x,
            sc_eval_x,
            sd_eval_x,
            sm_eval_x,
            h_evals_x,
            opening,
        })
    }
}
