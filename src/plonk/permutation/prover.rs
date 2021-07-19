use ff::Field;
use group::Curve;
use std::iter::{self, ExactSizeIterator};

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

pub struct CommittedSet<C: CurveAffine> {
    permutation_product_poly: Polynomial<C::Scalar, Coeff>,
    permutation_product_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permutation_product_blind: Blind<C::Scalar>,
}

pub(crate) struct Committed<C: CurveAffine> {
    sets: Vec<CommittedSet<C>>,
}

pub struct ConstructedSet<C: CurveAffine> {
    permutation_product_poly: Polynomial<C::Scalar, Coeff>,
    permutation_product_blind: Blind<C::Scalar>,
}

pub(crate) struct Constructed<C: CurveAffine> {
    sets: Vec<ConstructedSet<C>>,
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

        // How many columns can be included in a single permutation polynomial?
        // We need to multiply by z(X) and (1 - (l_last(X) + l_blind(X))). This
        // will never underflow because of the requirement of at least a degree
        // 3 circuit for the permutation argument.
        assert!(pk.vk.cs.degree() >= 3);
        let chunk_len = pk.vk.cs.degree() - 2;
        let blinding_factors = pk.vk.cs.blinding_factors();

        // Each column gets its own delta power.
        let mut deltaomega = C::Scalar::one();

        // Track the "last" value from the previous column set
        let mut last_z = C::Scalar::one();

        let mut sets = vec![];

        for (columns, permutations) in self
            .columns
            .chunks(chunk_len)
            .zip(pkey.permutations.chunks(chunk_len))
        {
            // Goal is to compute the products of fractions
            //
            // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
            // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
            //
            // where p_j(X) is the jth column in this permutation,
            // and i is the ith row of the column.

            let mut modified_values = vec![C::Scalar::one(); params.n as usize];

            // Iterate over each column of the permutation
            for (&column, permuted_column_values) in columns.iter().zip(permutations.iter()) {
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
            for &column in columns.iter() {
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
            let mut z = vec![last_z];
            for row in 1..(params.n as usize) {
                let mut tmp = z[row - 1];

                tmp *= &modified_values[row - 1];
                z.push(tmp);
            }
            let mut z = domain.lagrange_from_vec(z);
            // Set blinding factors
            for z in &mut z[params.n as usize - blinding_factors..] {
                *z = C::Scalar::rand();
            }
            // Set new last_z
            last_z = z[params.n as usize - (blinding_factors + 1)];

            let blind = Blind(C::Scalar::rand());

            let permutation_product_commitment_projective = params.commit_lagrange(&z, blind);
            let permutation_product_blind = blind;
            let z = domain.lagrange_to_coeff(z);
            let permutation_product_poly = z.clone();

            let permutation_product_coset = domain.coeff_to_extended(z.clone());

            let permutation_product_commitment =
                permutation_product_commitment_projective.to_affine();

            // Hash the permutation product commitment
            transcript
                .write_point(permutation_product_commitment)
                .map_err(|_| Error::TranscriptError)?;

            sets.push(CommittedSet {
                permutation_product_poly,
                permutation_product_coset,
                permutation_product_blind,
            });
        }

        Ok(Committed { sets })
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
        let chunk_len = pk.vk.cs.degree() - 2;
        let blinding_factors = pk.vk.cs.blinding_factors();
        let last_rotation = Rotation(-((blinding_factors + 1) as i32));

        let constructed = Constructed {
            sets: self
                .sets
                .iter()
                .map(|set| ConstructedSet {
                    permutation_product_poly: set.permutation_product_poly.clone(),
                    permutation_product_blind: set.permutation_product_blind,
                })
                .collect(),
        };

        let expressions = iter::empty()
            // Enforce only for the first set.
            // l_0(X) * (1 - z_0(X)) = 0
            .chain(self.sets.first().map(|first_set| {
                Polynomial::one_minus(first_set.permutation_product_coset.clone()) * &pk.l0
            }))
            // Enforce only for the last set.
            // l_last(X) * (z_l(X)^2 - z_l(X)) = 0
            .chain(self.sets.last().map(|last_set| {
                ((last_set.permutation_product_coset.clone() * &last_set.permutation_product_coset)
                    - &last_set.permutation_product_coset)
                    * &pk.l_last
            }))
            // Except for the first set, enforce.
            // l_0(X) * (z_i(X) - z_{i-1}(\omega^(last) X)) = 0
            .chain(
                self.sets
                    .iter()
                    .skip(1)
                    .zip(self.sets.iter())
                    .map(|(set, last_set)| {
                        domain.sub_extended(
                            set.permutation_product_coset.clone(),
                            &last_set.permutation_product_coset,
                            last_rotation,
                        ) * &pk.l0
                    })
                    .collect::<Vec<_>>(),
            )
            // And for all the sets we enforce:
            // (1 - (l_last(X) + l_blind(X))) * (
            //   z_i(\omega X) \prod_j (p(X) + \beta s_j(X) + \gamma)
            // - z_i(X) \prod_j (p(X) + \delta^j \beta X + \gamma)
            // )
            .chain(
                self.sets
                    .into_iter()
                    .zip(p.columns.chunks(chunk_len))
                    .zip(pkey.cosets.chunks(chunk_len))
                    .enumerate()
                    .map(move |(chunk_index, ((set, columns), cosets))| {
                        let mut left = domain
                            .rotate_extended(&set.permutation_product_coset, Rotation::next());
                        for (values, permutation) in columns
                            .iter()
                            .map(|&column| match column.column_type() {
                                Any::Advice => &advice_cosets[column.index()],
                                Any::Fixed => &fixed_cosets[column.index()],
                                Any::Instance => &instance_cosets[column.index()],
                            })
                            .zip(cosets.iter())
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

                        let mut right = set.permutation_product_coset;
                        let mut current_delta = *beta
                            * &C::Scalar::ZETA
                            * &(C::Scalar::DELTA.pow_vartime(&[(chunk_index * chunk_len) as u64]));
                        let step = domain.get_extended_omega();
                        for values in columns.iter().map(|&column| match column.column_type() {
                            Any::Advice => &advice_cosets[column.index()],
                            Any::Fixed => &fixed_cosets[column.index()],
                            Any::Instance => &instance_cosets[column.index()],
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

                        (left - &right) * &Polynomial::one_minus(pk.l_last.clone() + &pk.l_blind)
                    }),
            );

        (constructed, expressions)
    }
}

impl<C: CurveAffine> super::ProvingKey<C> {
    pub(in crate::plonk) fn open(
        &self,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = ProverQuery<'_, C>> + Clone {
        self.polys.iter().map(move |poly| ProverQuery {
            point: *x,
            poly,
            blind: Blind::default(),
        })
    }

    pub(in crate::plonk) fn evaluate<E: EncodedChallenge<C>, T: TranscriptWrite<C, E>>(
        &self,
        x: ChallengeX<C>,
        transcript: &mut T,
    ) -> Result<(), Error> {
        // Hash permutation evals
        for eval in self.polys.iter().map(|poly| eval_polynomial(poly, *x)) {
            transcript
                .write_scalar(eval)
                .map_err(|_| Error::TranscriptError)?;
        }

        Ok(())
    }
}

impl<C: CurveAffine> Constructed<C> {
    pub(in crate::plonk) fn evaluate<E: EncodedChallenge<C>, T: TranscriptWrite<C, E>>(
        self,
        pk: &plonk::ProvingKey<C>,
        x: ChallengeX<C>,
        transcript: &mut T,
    ) -> Result<Evaluated<C>, Error> {
        let domain = &pk.vk.domain;
        let blinding_factors = pk.vk.cs.blinding_factors();

        {
            let mut sets = self.sets.iter();

            while let Some(set) = sets.next() {
                let permutation_product_eval = eval_polynomial(&set.permutation_product_poly, *x);

                let permutation_product_next_eval = eval_polynomial(
                    &set.permutation_product_poly,
                    domain.rotate_omega(*x, Rotation::next()),
                );

                // Hash permutation product evals
                for eval in iter::empty()
                    .chain(Some(&permutation_product_eval))
                    .chain(Some(&permutation_product_next_eval))
                {
                    transcript
                        .write_scalar(*eval)
                        .map_err(|_| Error::TranscriptError)?;
                }

                // If we have any remaining sets to process, evaluate this set at omega^u
                // so we can constrain the last value of its running product to equal the
                // first value of the next set's running product, chaining them together.
                if sets.len() > 0 {
                    let permutation_product_last_eval = eval_polynomial(
                        &set.permutation_product_poly,
                        domain.rotate_omega(*x, Rotation(-((blinding_factors + 1) as i32))),
                    );

                    transcript
                        .write_scalar(permutation_product_last_eval)
                        .map_err(|_| Error::TranscriptError)?;
                }
            }
        }

        Ok(Evaluated { constructed: self })
    }
}

impl<C: CurveAffine> Evaluated<C> {
    pub(in crate::plonk) fn open<'a>(
        &'a self,
        pk: &'a plonk::ProvingKey<C>,
        x: ChallengeX<C>,
    ) -> impl Iterator<Item = ProverQuery<'a, C>> + Clone {
        let blinding_factors = pk.vk.cs.blinding_factors();
        let x_next = pk.vk.domain.rotate_omega(*x, Rotation::next());
        let x_last = pk
            .vk
            .domain
            .rotate_omega(*x, Rotation(-((blinding_factors + 1) as i32)));

        iter::empty()
            .chain(self.constructed.sets.iter().flat_map(move |set| {
                iter::empty()
                    // Open permutation product commitments at x and \omega x
                    .chain(Some(ProverQuery {
                        point: *x,
                        poly: &set.permutation_product_poly,
                        blind: set.permutation_product_blind,
                    }))
                    .chain(Some(ProverQuery {
                        point: x_next,
                        poly: &set.permutation_product_poly,
                        blind: set.permutation_product_blind,
                    }))
            }))
            // Open it at \omega^{last} x for all but the last set. This rotation is only
            // sensical for the first row, but we only use this rotation in a constraint
            // that is gated on l_0.
            .chain(
                self.constructed
                    .sets
                    .iter()
                    .rev()
                    .skip(1)
                    .flat_map(move |set| {
                        Some(ProverQuery {
                            point: x_last,
                            poly: &set.permutation_product_poly,
                            blind: set.permutation_product_blind,
                        })
                    }),
            )
    }
}
