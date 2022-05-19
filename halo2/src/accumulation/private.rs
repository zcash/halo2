use std::io;

use ff::{BatchInvert, Field};
use group::{Curve, Group};
use halo2_proofs::{
    arithmetic::{eval_polynomial, kate_division},
    poly::{
        commitment::{Blind, Params},
        Coeff, EvaluationDomain, Polynomial,
    },
    transcript::{EncodedChallenge, TranscriptRead, TranscriptWrite},
};
use pasta_curves::arithmetic::CurveAffine;
use rand_core::RngCore;

use super::AccumulationScheme;

/// This provides an implementation of "private accumulation" in which the
/// witness is a large polynomial that the prover keeps to itself; in exchange,
/// the update procedure is efficient.
#[derive(Debug, Clone)]
pub struct PrivateAccumulation<'a, C: CurveAffine> {
    params: &'a Params<C>,
    domain: &'a EvaluationDomain<C::Scalar>,
}

impl<'a, C: CurveAffine> PrivateAccumulation<'a, C> {
    /// Initializes this private accumulation context given parameters and an
    /// evaluation domain.
    pub fn new(params: &'a Params<C>, domain: &'a EvaluationDomain<C::Scalar>) -> Self {
        PrivateAccumulation { params, domain }
    }

    /// Creates a blank accumulator/witness pair given a polynomial `poly` that
    /// has a root at `root`.
    pub fn from_poly(
        &self,
        poly: Polynomial<C::Scalar, Coeff>,
        root: C::Scalar,
    ) -> (PrivateAccumulator<C>, PrivateAccumulatorWitness<C>) {
        let blind = Blind::default();
        let commitment = self.params.commit(&poly, blind).to_affine();

        (
            PrivateAccumulator { commitment, root },
            PrivateAccumulatorWitness { poly, blind },
        )
    }
}

/// A private accumulator that consists of a commitment to a polynomial and a
/// claimed root of that polynomial.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateAccumulator<C: CurveAffine> {
    commitment: C,
    root: C::Scalar,
}

/// The witness for a private accumulator which contains the committed
/// polynomial and the blinding factor for the commitment.
#[derive(Debug, Clone)]
pub struct PrivateAccumulatorWitness<C: CurveAffine> {
    poly: Polynomial<C::Scalar, Coeff>,
    blind: Blind<C::Scalar>,
}

impl<'a, C: CurveAffine> AccumulationScheme<C> for PrivateAccumulation<'a, C> {
    type InputAccumulator = PrivateAccumulator<C>;
    type OutputAccumulator = PrivateAccumulator<C>;
    type InputWitness = PrivateAccumulatorWitness<C>;
    type OutputWitness = PrivateAccumulatorWitness<C>;

    fn blank(&self) -> (Self::OutputAccumulator, Self::OutputWitness) {
        let poly = self.domain.empty_coeff();
        let blind = Blind::default();
        let commitment = self.params.commit(&poly, blind).to_affine();
        let root = C::Scalar::zero();

        (
            PrivateAccumulator { commitment, root },
            PrivateAccumulatorWitness { poly, blind },
        )
    }

    /// It is assumed that the input accumulators are already in the transcript.
    fn verify_update<E, T>(
        &self,
        transcript: &mut T,
        accumulators: &[Self::InputAccumulator],
    ) -> io::Result<Self::OutputAccumulator>
    where
        E: EncodedChallenge<C>,
        T: TranscriptRead<C, E>,
    {
        // Sample a random challenge \mu
        let mu = transcript.squeeze_challenge().get_scalar();

        // Ask the prover to supply a commitment to
        // p(X) = \sum\limits_{i = 0}^{\ell} \frac{a_i(X)}{X - r_i} \mu^i
        let p = transcript.read_point()?;

        // Sample a random challenge x
        let x = transcript.squeeze_challenge().get_scalar();

        // Ask the prover for a commitment to a random polynomial that
        // has a root at x.
        let r = transcript.read_point()?;

        // Ask the prover for a_i(x) for all i
        let evals_at_x = (0..accumulators.len())
            .map(|_| transcript.read_scalar())
            .collect::<Result<Vec<_>, _>>()?;

        // Sample a random challenge y
        let y = transcript.squeeze_challenge().get_scalar();

        // New accumulator commitment
        let mut commitment = C::CurveExt::identity();

        // Expected value that the accumulator commitment will evaluate to at x
        let mut expected_at_x = C::Scalar::zero();

        // Expected value that p(X) should evaluate to given the values a_i(x)
        // sent previously by the prover
        let mut expected_p_at_x = C::Scalar::zero();

        // Denominators in the expected p(X) polynomial evaluated at x
        let mut denominators = accumulators
            .iter()
            .map(|accumulator| x - accumulator.root)
            .collect::<Vec<_>>();
        denominators.iter_mut().batch_invert();

        // Based on the prover's provided evaluations, compute a new commitment
        // and its expected opening at x
        for ((eval, denominator), accumulator) in evals_at_x
            .into_iter()
            .zip(denominators.into_iter())
            .zip(accumulators.iter())
        {
            // Add a_i(X) to the multiopen
            expected_at_x *= y;
            expected_at_x += eval;
            commitment = commitment * y;
            commitment = commitment + accumulator.commitment;

            // Add a_i(X) / (x - root_i) to the expected evaluation
            // of p(X) at x
            expected_p_at_x *= mu;
            expected_p_at_x += eval * denominator;
        }
        // Add p(X) to the multiopen
        expected_at_x *= y;
        expected_at_x += expected_p_at_x;
        commitment = commitment * y;
        commitment = commitment + p;

        // Force x to be a root if its expected opening at x is correct
        commitment = commitment - (self.params.get_g()[0] * expected_at_x);

        // Add r(X) to the multiopen
        commitment = commitment * y;
        commitment = commitment + r;

        Ok(PrivateAccumulator {
            commitment: commitment.to_affine(),
            root: x,
        })
    }

    fn prove_update<E, T>(
        &self,
        transcript: &mut T,
        accumulators: &[(Self::InputAccumulator, Self::InputWitness)],
        mut rng: impl RngCore,
    ) -> io::Result<(Self::OutputAccumulator, Self::OutputWitness)>
    where
        E: EncodedChallenge<C>,
        T: TranscriptWrite<C, E>,
    {
        let mu = transcript.squeeze_challenge().get_scalar();

        let p_poly = accumulators.iter().fold(
            self.domain.empty_coeff(),
            |result, &(ref accumulator, ref witness)| {
                let mut quotient = self.domain.empty_coeff();
                let quotient_degree = quotient.len();
                quotient[..][0..quotient_degree - 1]
                    .copy_from_slice(&kate_division(&witness.poly[..], accumulator.root));
                result * mu + &quotient
            },
        );

        let p_blind = Blind(C::Scalar::random(&mut rng));
        let p = self.params.commit(&p_poly, p_blind).to_affine();
        transcript.write_point(p)?;

        let x = transcript.squeeze_challenge().get_scalar();

        let mut r_poly = self.domain.empty_coeff();
        for coeff in &mut r_poly[..] {
            *coeff = C::Scalar::random(&mut rng);
        }
        let r_eval_at_x = eval_polynomial(&r_poly, x);
        r_poly[0] -= r_eval_at_x;

        let r_blind = Blind(C::Scalar::random(&mut rng));
        let r = self.params.commit(&r_poly, r_blind).to_affine();
        transcript.write_point(r)?;

        for &(_, ref witness) in accumulators {
            transcript.write_scalar(eval_polynomial(&witness.poly[..], x))?;
        }

        let y = transcript.squeeze_challenge().get_scalar();
        let mut new_commitment = C::CurveExt::identity();
        let mut new_poly = self.domain.empty_coeff();
        let mut new_blind = Blind(C::Scalar::zero());
        for &(ref accumulator, ref witness) in accumulators.iter() {
            new_commitment = new_commitment * y;
            new_commitment = new_commitment + accumulator.commitment;
            new_blind = new_blind * Blind(y);
            new_blind = new_blind + witness.blind;
            new_poly = new_poly * y;
            new_poly = new_poly + &witness.poly;
        }
        new_blind = new_blind * Blind(y);
        new_blind = new_blind + p_blind;
        new_blind = new_blind * Blind(y);
        new_blind = new_blind + r_blind;
        new_poly = new_poly * y;
        new_poly = new_poly + &p_poly;
        drop(p_poly);
        let eval_at_x = eval_polynomial(&new_poly, x);
        new_commitment = new_commitment * y;
        new_commitment = new_commitment + p;
        new_commitment = new_commitment - (self.params.get_g()[0] * eval_at_x);
        new_poly[0] -= eval_at_x;
        new_commitment = new_commitment * y;
        new_commitment = new_commitment + r;
        new_poly = new_poly * y;
        new_poly = new_poly + &r_poly;

        Ok((
            PrivateAccumulator {
                commitment: new_commitment.to_affine(),
                root: x,
            },
            PrivateAccumulatorWitness {
                poly: new_poly,
                blind: new_blind,
            },
        ))
    }

    fn decide(&self, accumulator: &Self::OutputAccumulator, witness: &Self::OutputWitness) -> bool {
        // This simply checks that the commitment is consistent with the witness
        // polynomial, and that the witness polynomial indeed has the witness
        // root.
        (self.params.commit(&witness.poly, witness.blind)
            == C::CurveExt::from(accumulator.commitment))
            && eval_polynomial(&witness.poly, accumulator.root)
                .is_zero()
                .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
    use pasta_curves::pallas;
    use rand_core::OsRng;

    #[test]
    fn test_private_accumulator() {
        const K: u32 = 4;
        let params = Params::<pallas::Affine>::new(K);
        let domain = EvaluationDomain::<pallas::Scalar>::new(1, K);
        let accumulation = PrivateAccumulation::new(&params, &domain);

        let random_polyroot = || {
            let mut random_poly = domain.empty_coeff();
            for coeff in &mut random_poly[..] {
                *coeff = pallas::Scalar::random(OsRng);
            }
            let random_root = pallas::Scalar::random(OsRng);
            let eval = eval_polynomial(&random_poly, random_root);
            random_poly[0] -= eval;
            (random_poly, random_root)
        };

        let (a, x) = random_polyroot();
        let (b, y) = random_polyroot();

        let (acc1, witness1) = accumulation.from_poly(a.clone(), x);
        let (acc2, witness2) = accumulation.from_poly(b.clone(), y);
        let (acc3, witness3) = accumulation.blank();

        assert!(accumulation.decide(&acc1, &witness1));
        assert!(accumulation.decide(&acc2, &witness2));
        assert!(accumulation.decide(&acc3, &witness3));

        {
            assert!(!accumulation.decide(&acc1, &witness2));
            assert!(!accumulation.decide(&acc2, &witness1));
            assert!(!accumulation.decide(&acc3, &witness1));
        }

        let mut proof: Vec<u8> = vec![];
        let (new_acc, new_witness1) = accumulation
            .prove_update(
                &mut Blake2bWrite::<_, pallas::Affine, Challenge255<pallas::Affine>>::init(
                    &mut proof,
                ),
                &[
                    (acc1.clone(), witness1.clone()),
                    (acc2.clone(), witness2.clone()),
                    (acc3.clone(), witness3.clone()),
                ],
                OsRng,
            )
            .expect("should prove");

        assert!(accumulation.decide(&new_acc, &new_witness1));

        let verifier_acc1 = accumulation
            .verify_update(
                &mut Blake2bRead::<_, pallas::Affine, Challenge255<pallas::Affine>>::init(
                    &proof[..],
                ),
                &[acc1.clone(), acc2.clone(), acc3.clone()],
            )
            .expect("should verify");

        assert!(accumulation.decide(&verifier_acc1, &new_witness1));
        assert_eq!(new_acc, verifier_acc1);

        let (c, z) = random_polyroot();
        let (acc4, witness4) = accumulation.blank();
        let (acc5, witness5) = accumulation.from_poly(c.clone(), z);

        let mut proof: Vec<u8> = vec![];
        let (new_acc2, new_witness2) = accumulation
            .prove_update(
                &mut Blake2bWrite::<_, pallas::Affine, Challenge255<pallas::Affine>>::init(
                    &mut proof,
                ),
                &[
                    (new_acc.clone(), new_witness1.clone()),
                    (acc4.clone(), witness4.clone()),
                    (acc5.clone(), witness5.clone()),
                ],
                OsRng,
            )
            .expect("should prove");

        assert!(accumulation.decide(&new_acc2, &new_witness2));

        let verifier_acc2 = accumulation
            .verify_update(
                &mut Blake2bRead::<_, pallas::Affine, Challenge255<pallas::Affine>>::init(
                    &proof[..],
                ),
                &[new_acc.clone(), acc4.clone(), acc5.clone()],
            )
            .expect("should verify");

        assert!(accumulation.decide(&verifier_acc2, &new_witness2));
        assert!(!accumulation.decide(&verifier_acc2, &new_witness1));
        assert_eq!(new_acc2, verifier_acc2);
    }
}
