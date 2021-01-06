use ff::Field;
use std::iter;

use super::{Argument, Proof, ProvingKey};
use crate::{
    arithmetic::{eval_polynomial, parallelize, BatchInvert, Curve, CurveAffine, FieldExt},
    plonk::{self, ChallengeBeta, ChallengeGamma, ChallengeX, Error},
    poly::{
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{Hasher, Transcript},
};

pub(crate) struct Committed<C: CurveAffine> {
    permutation_product_poly: Polynomial<C::Scalar, Coeff>,
    permutation_product_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permutation_product_coset_inv: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permutation_product_blind: Blind<C::Scalar>,
    permutation_product_commitment: C,
}

pub(crate) struct Constructed<C: CurveAffine> {
    permutation_product_poly: Polynomial<C::Scalar, Coeff>,
    permutation_product_blind: Blind<C::Scalar>,
    permutation_product_commitment: C,
}

pub(crate) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
    permutation_product_eval: C::Scalar,
    permutation_product_inv_eval: C::Scalar,
    permutation_evals: Vec<C::Scalar>,
}

impl Argument {
    pub(in crate::plonk) fn commit<
        C: CurveAffine,
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    >(
        &self,
        params: &Params<C>,
        pk: &plonk::ProvingKey<C>,
        pkey: &ProvingKey<C>,
        advice: &[Polynomial<C::Scalar, LagrangeCoeff>],
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<Committed<C>, Error> {
        let domain = &pk.vk.domain;

        // Goal is to compute the products of fractions
        //
        // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
        // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
        //
        // where p_j(X) is the jth advice column in this permutation,
        // and i is the ith row of the column.

        let mut modified_advice = vec![C::Scalar::one(); params.n as usize];

        // Iterate over each column of the permutation
        for (&column, permuted_column_values) in self.columns.iter().zip(pkey.permutations.iter()) {
            parallelize(&mut modified_advice, |modified_advice, start| {
                for ((modified_advice, advice_value), permuted_advice_value) in modified_advice
                    .iter_mut()
                    .zip(advice[column.index()][start..].iter())
                    .zip(permuted_column_values[start..].iter())
                {
                    *modified_advice *= &(*beta * permuted_advice_value + &*gamma + advice_value);
                }
            });
        }

        // Invert to obtain the denominator for the permutation product polynomial
        modified_advice.batch_invert();

        // Iterate over each column again, this time finishing the computation
        // of the entire fraction by computing the numerators
        let mut deltaomega = C::Scalar::one();
        for &column in self.columns.iter() {
            let omega = domain.get_omega();
            parallelize(&mut modified_advice, |modified_advice, start| {
                let mut deltaomega = deltaomega * &omega.pow_vartime(&[start as u64, 0, 0, 0]);
                for (modified_advice, advice_value) in modified_advice
                    .iter_mut()
                    .zip(advice[column.index()][start..].iter())
                {
                    // Multiply by p_j(\omega^i) + \delta^j \omega^i \beta
                    *modified_advice *= &(deltaomega * &*beta + &*gamma + advice_value);
                    deltaomega *= &omega;
                }
            });
            deltaomega *= &C::Scalar::DELTA;
        }

        // The modified_advice vector is a vector of products of fractions
        // of the form
        //
        // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
        // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
        //
        // where i is the index into modified_advice, for the jth column in
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

        let blind = Blind(C::Scalar::rand());

        let permutation_product_commitment_projective = params.commit_lagrange(&z, blind);
        let permutation_product_blind = blind;
        let z = domain.lagrange_to_coeff(z);
        let permutation_product_poly = z.clone();
        let permutation_product_coset = domain.coeff_to_extended(z.clone(), Rotation::default());
        let permutation_product_coset_inv = domain.coeff_to_extended(z, Rotation(-1));

        let permutation_product_commitment = permutation_product_commitment_projective.to_affine();

        // Hash the permutation product commitment
        transcript
            .absorb_point(&permutation_product_commitment)
            .map_err(|_| Error::TranscriptError)?;

        Ok(Committed {
            permutation_product_poly,
            permutation_product_coset,
            permutation_product_coset_inv,
            permutation_product_blind,
            permutation_product_commitment,
        })
    }
}

impl<C: CurveAffine> Committed<C> {
    pub(in crate::plonk) fn construct<'a>(
        self,
        pk: &'a plonk::ProvingKey<C>,
        p: &'a Argument,
        pkey: &'a ProvingKey<C>,
        advice_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        beta: ChallengeBeta<C::Scalar>,
        gamma: ChallengeGamma<C::Scalar>,
    ) -> Result<
        (
            Constructed<C>,
            impl Iterator<Item = Polynomial<C::Scalar, ExtendedLagrangeCoeff>> + 'a,
        ),
        Error,
    > {
        let domain = &pk.vk.domain;

        let expressions = iter::empty()
            // l_0(X) * (1 - z(X)) = 0
            .chain(Some(
                Polynomial::one_minus(self.permutation_product_coset.clone()) * &pk.l0,
            ))
            // z(X) \prod (p(X) + \beta s_i(X) + \gamma) - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
            .chain(Some({
                let mut left = self.permutation_product_coset.clone();
                for (advice, permutation) in p
                    .columns
                    .iter()
                    .map(|&column| &advice_cosets[pk.vk.cs.get_advice_query_index(column, 0)])
                    .zip(pkey.cosets.iter())
                {
                    parallelize(&mut left, |left, start| {
                        for ((left, advice), permutation) in left
                            .iter_mut()
                            .zip(advice[start..].iter())
                            .zip(permutation[start..].iter())
                        {
                            *left *= &(*advice + &(*beta * permutation) + &*gamma);
                        }
                    });
                }

                let mut right = self.permutation_product_coset_inv.clone();
                let mut current_delta = *beta * &C::Scalar::ZETA;
                let step = domain.get_extended_omega();
                for advice in p
                    .columns
                    .iter()
                    .map(|&column| &advice_cosets[pk.vk.cs.get_advice_query_index(column, 0)])
                {
                    parallelize(&mut right, move |right, start| {
                        let mut beta_term =
                            current_delta * &step.pow_vartime(&[start as u64, 0, 0, 0]);
                        for (right, advice) in right.iter_mut().zip(advice[start..].iter()) {
                            *right *= &(*advice + &beta_term + &*gamma);
                            beta_term *= &step;
                        }
                    });
                    current_delta *= &C::Scalar::DELTA;
                }

                left - &right
            }));

        Ok((
            Constructed {
                permutation_product_poly: self.permutation_product_poly,
                permutation_product_blind: self.permutation_product_blind,
                permutation_product_commitment: self.permutation_product_commitment,
            },
            expressions,
        ))
    }
}

impl<C: CurveAffine> super::ProvingKey<C> {
    fn evaluate(&self, x: ChallengeX<C::Scalar>) -> Vec<C::Scalar> {
        self.polys
            .iter()
            .map(|poly| eval_polynomial(poly, *x))
            .collect()
    }

    fn open<'a>(
        &'a self,
        evals: &'a [C::Scalar],
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        self.polys
            .iter()
            .zip(evals.iter())
            .map(move |(poly, eval)| ProverQuery {
                point: *x,
                poly,
                blind: Blind::default(),
                eval: *eval,
            })
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        self,
        pk: &plonk::ProvingKey<C>,
        pkey: &ProvingKey<C>,
        x: ChallengeX<C::Scalar>,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Evaluated<C> {
        let domain = &pk.vk.domain;

        let permutation_product_eval = eval_polynomial(&self.permutation_product_poly, *x);

        let permutation_product_inv_eval = eval_polynomial(
            &self.permutation_product_poly,
            domain.rotate_omega(*x, Rotation(-1)),
        );

        let permutation_evals = pkey.evaluate(x);

        // Hash each advice evaluation
        for eval in iter::empty()
            .chain(Some(&permutation_product_eval))
            .chain(Some(&permutation_product_inv_eval))
            .chain(permutation_evals.iter())
        {
            transcript.absorb_scalar(*eval);
        }

        Evaluated {
            constructed: self,
            permutation_product_eval,
            permutation_product_inv_eval,
            permutation_evals,
        }
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open<'a>(
        &'a self,
        pk: &'a plonk::ProvingKey<C>,
        pkey: &'a ProvingKey<C>,
        x: ChallengeX<C::Scalar>,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        let x_inv = pk.vk.domain.rotate_omega(*x, Rotation(-1));

        iter::empty()
            // Open permutation product commitments at x and \omega^{-1} x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.permutation_product_poly,
                blind: self.constructed.permutation_product_blind,
                eval: self.permutation_product_eval,
            }))
            .chain(Some(ProverQuery {
                point: x_inv,
                poly: &self.constructed.permutation_product_poly,
                blind: self.constructed.permutation_product_blind,
                eval: self.permutation_product_inv_eval,
            }))
            // Open permutation polynomial commitments at x
            .chain(pkey.open(&self.permutation_evals, x))
    }

    pub(crate) fn build(self) -> Proof<C> {
        Proof {
            permutation_product_commitment: self.constructed.permutation_product_commitment,
            permutation_product_eval: self.permutation_product_eval,
            permutation_product_inv_eval: self.permutation_product_inv_eval,
            permutation_evals: self.permutation_evals,
        }
    }
}
