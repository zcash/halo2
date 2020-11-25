use ff::Field;
use std::iter;

use super::Proof;
use crate::{
    arithmetic::{eval_polynomial, parallelize, BatchInvert, Curve, CurveAffine, FieldExt},
    plonk::{Error, ProvingKey},
    poly::{
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{Hasher, Transcript},
};

#[derive(Clone)]
pub(crate) struct Committed<C: CurveAffine> {
    permutation_product_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    permutation_product_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutation_product_cosets_inv: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutation_product_blinds: Vec<Blind<C::Scalar>>,
    permutation_product_commitments: Vec<C>,
}

pub(crate) struct Constructed<C: CurveAffine> {
    permutation_product_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    permutation_product_blinds: Vec<Blind<C::Scalar>>,
    permutation_product_commitments: Vec<C>,
}

pub(crate) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
    permutation_product_evals: Vec<C::Scalar>,
    permutation_product_inv_evals: Vec<C::Scalar>,
    permutation_evals: Vec<Vec<C::Scalar>>,
}

impl<C: CurveAffine> Proof<C> {
    pub(crate) fn commit<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        params: &Params<C>,
        pk: &ProvingKey<C>,
        advice: &[Polynomial<C::Scalar, LagrangeCoeff>],
        x_0: C::Scalar,
        x_1: C::Scalar,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<Committed<C>, Error> {
        let domain = &pk.vk.domain;

        // Compute permutation product polynomial commitment
        let mut permutation_product_polys = vec![];
        let mut permutation_product_cosets = vec![];
        let mut permutation_product_cosets_inv = vec![];
        let mut permutation_product_commitments_projective = vec![];
        let mut permutation_product_blinds = vec![];

        // Iterate over each permutation
        let mut permutation_modified_advice = pk
            .vk
            .cs
            .permutations
            .iter()
            .zip(pk.permutations.iter())
            // Goal is to compute the products of fractions
            //
            // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
            // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
            //
            // where p_j(X) is the jth advice column in this permutation,
            // and i is the ith row of the column.
            .map(|(columns, permuted_values)| {
                let mut modified_advice = vec![C::Scalar::one(); params.n as usize];

                // Iterate over each column of the permutation
                for (&column, permuted_column_values) in columns.iter().zip(permuted_values.iter())
                {
                    parallelize(&mut modified_advice, |modified_advice, start| {
                        for ((modified_advice, advice_value), permuted_advice_value) in
                            modified_advice
                                .iter_mut()
                                .zip(advice[column.index()][start..].iter())
                                .zip(permuted_column_values[start..].iter())
                        {
                            *modified_advice *=
                                &(x_0 * permuted_advice_value + &x_1 + advice_value);
                        }
                    });
                }

                modified_advice
            })
            .collect::<Vec<_>>();

        // Batch invert to obtain the denominators for the permutation product
        // polynomials
        permutation_modified_advice
            .iter_mut()
            .flat_map(|v| v.iter_mut())
            .batch_invert();

        for (columns, mut modified_advice) in pk
            .vk
            .cs
            .permutations
            .iter()
            .zip(permutation_modified_advice.into_iter())
        {
            // Iterate over each column again, this time finishing the computation
            // of the entire fraction by computing the numerators
            let mut deltaomega = C::Scalar::one();
            for &column in columns.iter() {
                let omega = domain.get_omega();
                parallelize(&mut modified_advice, |modified_advice, start| {
                    let mut deltaomega = deltaomega * &omega.pow_vartime(&[start as u64, 0, 0, 0]);
                    for (modified_advice, advice_value) in modified_advice
                        .iter_mut()
                        .zip(advice[column.index()][start..].iter())
                    {
                        // Multiply by p_j(\omega^i) + \delta^j \omega^i \beta
                        *modified_advice *= &(deltaomega * &x_0 + &x_1 + advice_value);
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
            transcript
                .absorb_point(c)
                .map_err(|_| Error::TranscriptError)?;
        }

        Ok(Committed {
            permutation_product_polys,
            permutation_product_cosets,
            permutation_product_cosets_inv,
            permutation_product_blinds,
            permutation_product_commitments,
        })
    }
}

impl<C: CurveAffine> Committed<C> {
    pub(crate) fn construct<'a>(
        self,
        pk: &'a ProvingKey<C>,
        advice_cosets: &'a [Polynomial<C::Scalar, ExtendedLagrangeCoeff>],
        x_0: C::Scalar,
        x_1: C::Scalar,
    ) -> Result<
        (
            Constructed<C>,
            impl Iterator<Item = Polynomial<C::Scalar, ExtendedLagrangeCoeff>> + 'a,
        ),
        Error,
    > {
        let domain = &pk.vk.domain;
        let permutation_product_cosets_owned = self.permutation_product_cosets.clone();
        let permutation_product_cosets = self.permutation_product_cosets;
        let permutation_product_cosets_inv = self.permutation_product_cosets_inv;

        let expressions = iter::empty()
            // l_0(X) * (1 - z(X)) = 0
            .chain(
                permutation_product_cosets_owned
                    .into_iter()
                    .map(move |coset| Polynomial::one_minus(coset) * &pk.l0),
            )
            // z(X) \prod (p(X) + \beta s_i(X) + \gamma) - z(omega^{-1} X) \prod (p(X) + \delta^i \beta X + \gamma)
            .chain(pk.vk.cs.permutations.iter().enumerate().map(
                move |(permutation_index, columns)| {
                    let mut left = permutation_product_cosets[permutation_index].clone();
                    for (advice, permutation) in columns
                        .iter()
                        .map(|&column| &advice_cosets[pk.vk.cs.get_advice_query_index(column, 0)])
                        .zip(pk.permutation_cosets[permutation_index].iter())
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
                    for advice in columns
                        .iter()
                        .map(|&column| &advice_cosets[pk.vk.cs.get_advice_query_index(column, 0)])
                    {
                        parallelize(&mut right, move |right, start| {
                            let mut beta_term =
                                current_delta * &step.pow_vartime(&[start as u64, 0, 0, 0]);
                            for (right, advice) in right.iter_mut().zip(advice[start..].iter()) {
                                *right *= &(*advice + &beta_term + &x_1);
                                beta_term *= &step;
                            }
                        });
                        current_delta *= &C::Scalar::DELTA;
                    }

                    left - &right
                },
            ));

        Ok((
            Constructed {
                permutation_product_polys: self.permutation_product_polys,
                permutation_product_blinds: self.permutation_product_blinds,
                permutation_product_commitments: self.permutation_product_commitments,
            },
            expressions,
        ))
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(crate) fn evaluate<HBase: Hasher<C::Base>, HScalar: Hasher<C::Scalar>>(
        self,
        pk: &ProvingKey<C>,
        x_3: C::Scalar,
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Evaluated<C> {
        let domain = &pk.vk.domain;

        let permutation_product_evals: Vec<C::Scalar> = self
            .permutation_product_polys
            .iter()
            .map(|poly| eval_polynomial(poly, x_3))
            .collect();

        let permutation_product_inv_evals: Vec<C::Scalar> = self
            .permutation_product_polys
            .iter()
            .map(|poly| eval_polynomial(poly, domain.rotate_omega(x_3, Rotation(-1))))
            .collect();

        let permutation_evals: Vec<Vec<C::Scalar>> = pk
            .permutation_polys
            .iter()
            .map(|polys| {
                polys
                    .iter()
                    .map(|poly| eval_polynomial(poly, x_3))
                    .collect()
            })
            .collect();

        // Hash each advice evaluation
        for eval in permutation_product_evals
            .iter()
            .chain(permutation_product_inv_evals.iter())
            .chain(permutation_evals.iter().flat_map(|evals| evals.iter()))
        {
            transcript.absorb_scalar(*eval);
        }

        Evaluated {
            constructed: self,
            permutation_product_evals,
            permutation_product_inv_evals,
            permutation_evals,
        }
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub fn open<'a>(
        &'a self,
        pk: &'a ProvingKey<C>,
        x_3: C::Scalar,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        let x_3_inv = pk.vk.domain.rotate_omega(x_3, Rotation(-1));

        iter::empty()
            // Open permutation product commitments at x_3
            .chain(
                self.constructed
                    .permutation_product_polys
                    .iter()
                    .zip(self.constructed.permutation_product_blinds.iter())
                    .zip(self.permutation_product_evals.iter())
                    .map(move |((poly, blind), eval)| ProverQuery {
                        point: x_3,
                        poly,
                        blind: *blind,
                        eval: *eval,
                    }),
            )
            // Open permutation polynomial commitments at x_3
            .chain(
                pk.permutation_polys
                    .iter()
                    .zip(self.permutation_evals.iter())
                    .flat_map(|(polys, evals)| polys.iter().zip(evals.iter()))
                    .map(move |(poly, eval)| ProverQuery {
                        point: x_3,
                        poly,
                        blind: Blind::default(),
                        eval: *eval,
                    }),
            )
            // Open permutation product commitments at \omega^{-1} x_3
            .chain(
                self.constructed
                    .permutation_product_polys
                    .iter()
                    .zip(self.constructed.permutation_product_blinds.iter())
                    .zip(self.permutation_product_inv_evals.iter())
                    .map(move |((poly, blind), eval)| ProverQuery {
                        point: x_3_inv,
                        poly,
                        blind: *blind,
                        eval: *eval,
                    }),
            )
    }

    pub(crate) fn build(self) -> Proof<C> {
        Proof {
            permutation_product_commitments: self.constructed.permutation_product_commitments,
            permutation_product_evals: self.permutation_product_evals,
            permutation_product_inv_evals: self.permutation_product_inv_evals,
            permutation_evals: self.permutation_evals,
        }
    }
}
