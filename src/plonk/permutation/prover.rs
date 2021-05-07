use ff::Field;
use group::Curve;
use std::iter;

use super::super::{circuit::Any, ChallengeBeta, ChallengeGamma, ChallengeX};
use super::{Argument, ProvingKey};
use crate::{
    arithmetic::{eval_polynomial, parallelize, BatchInvert, CurveAffine, FieldExt},
    plonk::{self, Error},
    poly::{
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{EncodedChallenge, TranscriptWrite},
};

pub(crate) struct Committed<C: CurveAffine> {
    permutation_product_poly: Polynomial<C::Scalar, Coeff>,
    permutation_product_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permutation_product_coset_inv: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permutation_product_blind: Blind<C::Scalar>,
}

pub(crate) struct Constructed<C: CurveAffine> {
    permutation_product_poly: Polynomial<C::Scalar, Coeff>,
    permutation_product_blind: Blind<C::Scalar>,
}

pub(crate) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
}

impl Argument {
    pub(in crate::plonk) fn commit<
        C: CurveAffine,
        E: EncodedChallenge<C>,
        T: TranscriptWrite<C, E>,
    >(
        &self,
        params: &Params<C>,
        pk: &plonk::ProvingKey<C>,
        pkey: &ProvingKey<C>,
        advice: &[Polynomial<C::Scalar, LagrangeCoeff>],
        fixed: &[Polynomial<C::Scalar, LagrangeCoeff>],
        instance: &[Polynomial<C::Scalar, LagrangeCoeff>],
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
        transcript: &mut T,
    ) -> Result<Committed<C>, Error> {
        let domain = &pk.vk.domain;

        // Goal is to compute the products of fractions
        //
        // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
        // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
        //
        // where p_j(X) is the jth column in this permutation,
        // and i is the ith row of the column.

        let mut modified_values = vec![C::Scalar::one(); params.n as usize];

        // Iterate over each column of the permutation
        for (&column, permuted_column_values) in self.columns.iter().zip(pkey.permutations.iter()) {
            let values = match column.column_type() {
                Any::Advice => advice,
                Any::Fixed => fixed,
                Any::Instance => instance,
            };
            parallelize(&mut modified_values, |modified_values, start| {
                for ((modified_values, value), permuted_value) in modified_values
                    .iter_mut()
                    .zip(values[column.index()][start..].iter())
                    .zip(permuted_column_values[start..].iter())
                {
                    *modified_values *= &(*beta * permuted_value + &*gamma + value);
                }
            });
        }

        // Invert to obtain the denominator for the permutation product polynomial
        modified_values.batch_invert();

        // Iterate over each column again, this time finishing the computation
        // of the entire fraction by computing the numerators
        let mut deltaomega = C::Scalar::one();
        for &column in self.columns.iter() {
            let omega = domain.get_omega();
            let values = match column.column_type() {
                Any::Advice => advice,
                Any::Fixed => fixed,
                Any::Instance => instance,
            };
            parallelize(&mut modified_values, |modified_values, start| {
                let mut deltaomega = deltaomega * &omega.pow_vartime(&[start as u64, 0, 0, 0]);
                for (modified_values, value) in modified_values
                    .iter_mut()
                    .zip(values[column.index()][start..].iter())
                {
                    // Multiply by p_j(\omega^i) + \delta^j \omega^i \beta
                    *modified_values *= &(deltaomega * &*beta + &*gamma + value);
                    deltaomega *= &omega;
                }
            });
            deltaomega *= &C::Scalar::DELTA;
        }

        // The modified_values vector is a vector of products of fractions
        // of the form
        //
        // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
        // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
        //
        // where i is the index into modified_values, for the jth column in
        // the permutation

        // Compute the evaluations of the permutation product polynomial
        // over our domain, starting with z[0] = 1
        let mut z = vec![C::Scalar::one()];
        for row in 1..(params.n as usize) {
            let mut tmp = z[row - 1];

            tmp *= &modified_values[row];
            z.push(tmp);
        }
        let z = domain.lagrange_from_vec(z);

        let blind = Blind(C::Scalar::rand());

        let permutation_product_commitment_projective = params.commit_lagrange(&z, blind);
        let permutation_product_blind = blind;
        let z = domain.lagrange_to_coeff(z);
        let permutation_product_poly = z.clone();
        let permutation_product_coset = domain.coeff_to_extended(z.clone(), Rotation::cur());
        let permutation_product_coset_inv = domain.coeff_to_extended(z, Rotation::prev());

        let permutation_product_commitment = permutation_product_commitment_projective.to_affine();

        // Hash the permutation product commitment
        transcript
            .write_point(permutation_product_commitment)
            .map_err(|_| Error::TranscriptError)?;

        Ok(Committed {
            permutation_product_poly,
            permutation_product_coset,
            permutation_product_coset_inv,
            permutation_product_blind,
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
        fixed_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        instance_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
    ) -> (
        Constructed<C>,
        impl Iterator<Item = Polynomial<C::Scalar, ExtendedLagrangeCoeff>> + 'a,
    ) {
        let domain = &pk.vk.domain;
        let expressions = iter::empty()
            // l_0(X) * (1 - z(X)) = 0
            .chain(Some(
                Polynomial::one_minus(self.permutation_product_coset.clone()) * &pk.l0,
            ))
            // z(X) \prod (p(X) + \beta s_i(X) + \gamma) - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
            .chain(Some({
                let mut left = self.permutation_product_coset.clone();
                for (values, permutation) in p
                    .columns
                    .iter()
                    .map(|&column| match column.column_type() {
                        Any::Advice => {
                            &advice_cosets[pk.vk.cs.get_any_query_index(column, Rotation::cur())]
                        }
                        Any::Fixed => {
                            &fixed_cosets[pk.vk.cs.get_any_query_index(column, Rotation::cur())]
                        }
                        Any::Instance => {
                            &instance_cosets[pk.vk.cs.get_any_query_index(column, Rotation::cur())]
                        }
                    })
                    .zip(pkey.cosets.iter())
                {
                    parallelize(&mut left, |left, start| {
                        for ((left, value), permutation) in left
                            .iter_mut()
                            .zip(values[start..].iter())
                            .zip(permutation[start..].iter())
                        {
                            *left *= &(*value + &(*beta * permutation) + &*gamma);
                        }
                    });
                }

                let mut right = self.permutation_product_coset_inv.clone();
                let mut current_delta = *beta * &C::Scalar::ZETA;
                let step = domain.get_extended_omega();
                for values in p.columns.iter().map(|&column| match column.column_type() {
                    Any::Advice => {
                        &advice_cosets[pk.vk.cs.get_any_query_index(column, Rotation::cur())]
                    }
                    Any::Fixed => {
                        &fixed_cosets[pk.vk.cs.get_any_query_index(column, Rotation::cur())]
                    }
                    Any::Instance => {
                        &instance_cosets[pk.vk.cs.get_any_query_index(column, Rotation::cur())]
                    }
                }) {
                    parallelize(&mut right, move |right, start| {
                        let mut beta_term =
                            current_delta * &step.pow_vartime(&[start as u64, 0, 0, 0]);
                        for (right, value) in right.iter_mut().zip(values[start..].iter()) {
                            *right *= &(*value + &beta_term + &*gamma);
                            beta_term *= &step;
                        }
                    });
                    current_delta *= &C::Scalar::DELTA;
                }

                left - &right
            }));

        (
            Constructed {
                permutation_product_poly: self.permutation_product_poly,
                permutation_product_blind: self.permutation_product_blind,
            },
            expressions,
        )
    }
}

impl<C: CurveAffine> super::ProvingKey<C> {
    fn evaluate(&self, x: ChallengeX<C>) -> Vec<C::Scalar> {
        self.polys
            .iter()
            .map(|poly| eval_polynomial(poly, *x))
            .collect()
    }

    fn open(&self, x: ChallengeX<C>) -> impl Iterator<Item = ProverQuery<'_, C>> + Clone {
        self.polys.iter().map(move |poly| ProverQuery {
            point: *x,
            poly,
            blind: Blind::default(),
        })
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<E: EncodedChallenge<C>, T: TranscriptWrite<C, E>>(
        self,
        pk: &plonk::ProvingKey<C>,
        pkey: &ProvingKey<C>,
        x: ChallengeX<C>,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let domain = &pk.vk.domain;

        let permutation_product_eval = eval_polynomial(&self.permutation_product_poly, *x);

        let permutation_product_inv_eval = eval_polynomial(
            &self.permutation_product_poly,
            domain.rotate_omega(*x, Rotation(-1)),
        );

        let permutation_evals = pkey.evaluate(x);

        // Hash permutation product evals
        for eval in iter::empty()
            .chain(Some(&permutation_product_eval))
            .chain(Some(&permutation_product_inv_eval))
            .chain(permutation_evals.iter())
        {
            transcript
                .write_scalar(*eval)
                .map_err(|_| Error::TranscriptError)?;
        }

        Ok(Evaluated { constructed: self })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open<'a>(
        &'a self,
        pk: &'a plonk::ProvingKey<C>,
        pkey: &'a ProvingKey<C>,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        let x_inv = pk.vk.domain.rotate_omega(*x, Rotation(-1));

        iter::empty()
            // Open permutation product commitments at x and \omega^{-1} x
            .chain(Some(ProverQuery {
                point: *x,
                poly: &self.constructed.permutation_product_poly,
                blind: self.constructed.permutation_product_blind,
            }))
            .chain(Some(ProverQuery {
                point: x_inv,
                poly: &self.constructed.permutation_product_poly,
                blind: self.constructed.permutation_product_blind,
            }))
            // Open permutation polynomial commitments at x
            .chain(pkey.open(x))
    }
}
