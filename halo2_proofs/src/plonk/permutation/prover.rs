use group::ff::{Field, PrimeField};
use maybe_rayon::prelude::*;
use pasta_curves::arithmetic::CurveExt;
use std::iter;

use super::super::{circuit::Any, prover::Draws, ChallengeBeta, ChallengeGamma, ChallengeX};
use super::{Argument, ProvingKey};
use crate::{
    arithmetic::{
        chained_grand_products, evaluate_batch, parallel_batch_invert, parallelize, CurveAffine,
    },
    plonk::{self, Error},
    poly::{
        self,
        commitment::{Blind, Params},
        multiopen::ProverQuery,
        Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{EncodedChallenge, TranscriptWrite},
};

pub struct CommittedSet<C: CurveAffine, Ev> {
    permutation_product_poly: Polynomial<C::Scalar, Coeff>,
    permutation_product_coset: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
    permutation_product_blind: Blind<C::Scalar>,
}

pub(crate) struct Committed<C: CurveAffine, Ev> {
    sets: Vec<CommittedSet<C, Ev>>,
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
    #[allow(clippy::too_many_arguments)]
    pub(in crate::plonk) fn commit<C: CurveAffine>(
        &self,
        params: &Params<C>,
        pk: &plonk::ProvingKey<C>,
        pkey: &ProvingKey<C>,
        advice: &[Polynomial<C::Scalar, LagrangeCoeff>],
        fixed: &[Polynomial<C::Scalar, LagrangeCoeff>],
        instance: &[Polynomial<C::Scalar, LagrangeCoeff>],
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
        mut draws: Draws<C::Scalar>,
    ) -> CommittedPending<C> {
        let domain = &pk.vk.domain;

        // How many columns can be included in a single permutation polynomial?
        // We need to multiply by z(X) and (1 - (l_last(X) + l_blind(X))). This
        // will never underflow because of the requirement of at least a degree
        // 3 circuit for the permutation argument.
        assert!(pk.vk.cs_degree >= 3);
        let chunk_len = pk.vk.cs_degree - 2;
        let blinding_factors = pk.vk.cs.blinding_factors();

        let n = params.n as usize;
        // The rows after this one hold blinding factors. Its value is the
        // product carried into the next set.
        let last_active_row = n - (blinding_factors + 1);

        // Column `i` of the whole permutation uses the coset `delta^i <omega>`.
        let delta_powers: Vec<C::Scalar> =
            iter::successors(Some(C::Scalar::ONE), |d| Some(*d * C::Scalar::DELTA))
                .take(self.columns.len())
                .collect();

        // The fractions of each set are independent of the other sets, and
        // `chained_grand_products` computes the chained grand products with
        // every set in parallel.
        let set_fractions: Vec<Vec<C::Scalar>> = self
            .columns
            .chunks(chunk_len)
            .zip(pkey.permutations.chunks(chunk_len))
            .enumerate()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(set_index, (columns, permutations))| {
                // Goal is to compute the products of fractions
                //
                // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
                // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
                //
                // where p_j(X) is the jth column in this permutation,
                // and i is the ith row of the column.
                let mut modified_values = vec![C::Scalar::ONE; n];

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
                parallel_batch_invert(&mut modified_values);

                // Iterate over each column again, this time finishing the computation
                // of the entire fraction by computing the numerators
                let omega = domain.get_omega();
                for (offset, &column) in columns.iter().enumerate() {
                    let delta = delta_powers[set_index * chunk_len + offset];
                    let values = match column.column_type() {
                        Any::Advice => advice,
                        Any::Fixed => fixed,
                        Any::Instance => instance,
                    };
                    parallelize(&mut modified_values, |modified_values, start| {
                        let mut deltaomega = delta * &omega.pow_vartime([start as u64]);
                        for (modified_values, value) in modified_values
                            .iter_mut()
                            .zip(values[column.index()][start..].iter())
                        {
                            // Multiply by p_j(\omega^i) + \delta^j \omega^i \beta
                            *modified_values *= &(deltaomega * &*beta + &*gamma + value);
                            deltaomega *= &omega;
                        }
                    });
                }

                // The modified_values vector is a vector of products of fractions
                // of the form
                //
                // (p_j(\omega^i) + \delta^j \omega^i \beta + \gamma) /
                // (p_j(\omega^i) + \beta s_j(\omega^i) + \gamma)
                //
                // where i is the index into modified_values, for the jth column in
                // the permutation.
                modified_values
            })
            .collect();

        // Append the blinding factors. Randomness is drawn set by set in the
        // same order as a sequential prover.
        let blinded: Vec<_> = chained_grand_products(set_fractions, last_active_row + 1)
            .into_iter()
            .map(|mut z| {
                z.extend((0..blinding_factors).map(|_| draws.next()));
                let blind = Blind(draws.next());
                (domain.lagrange_from_vec(z), blind)
            })
            .collect();

        // Commit to and transform every set at once.
        let committed: Vec<_> = blinded
            .into_par_iter()
            .map(|(z, blind)| {
                let commitment = params.commit_lagrange(&z, blind).to_affine_vartime();
                let poly = domain.lagrange_to_coeff(z);
                let coset = domain.coeff_to_extended(poly.clone());
                (commitment, poly, coset, blind)
            })
            .collect();

        CommittedPending { sets: committed }
    }

    /// The number of field elements [`Argument::commit`] draws: for each set,
    /// its blinding rows and then its blind.
    pub(in crate::plonk) fn commit_draws<C: CurveAffine>(
        &self,
        pk: &plonk::ProvingKey<C>,
    ) -> usize {
        let chunk_len = pk.vk.cs_degree - 2;
        self.columns.len().div_ceil(chunk_len) * (pk.vk.cs.blinding_factors() + 1)
    }
}

/// The permutation product sets of one circuit, committed but not yet
/// hashed into the transcript, with their cosets not yet registered.
/// Computing this needs no shared state, so circuits are committed in
/// parallel; [`CommittedPending::finish`] then runs in transcript order.
pub(in crate::plonk) struct CommittedPending<C: CurveAffine> {
    #[allow(clippy::type_complexity)]
    sets: Vec<(
        C,
        Polynomial<C::Scalar, Coeff>,
        Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
        Blind<C::Scalar>,
    )>,
}

impl<C: CurveAffine> CommittedPending<C> {
    /// Hashes each set's commitment and registers its coset.
    pub(in crate::plonk) fn finish<
        E: EncodedChallenge<C>,
        Ev: Copy + Send + Sync,
        T: TranscriptWrite<C, E>,
    >(
        self,
        evaluator: &mut poly::Evaluator<Ev, C::Scalar, ExtendedLagrangeCoeff>,
        transcript: &mut T,
    ) -> Result<Committed<C, Ev>, Error> {
        let mut sets = Vec::with_capacity(self.sets.len());
        for (commitment, permutation_product_poly, coset, permutation_product_blind) in self.sets {
            // Hash the permutation product commitment
            transcript.write_point(commitment)?;

            sets.push(CommittedSet {
                permutation_product_poly,
                permutation_product_coset: evaluator.register_poly(coset),
                permutation_product_blind,
            });
        }

        Ok(Committed { sets })
    }
}

impl<C: CurveAffine, Ev: Copy + Send + Sync> Committed<C, Ev> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::plonk) fn construct<'a>(
        self,
        pk: &'a plonk::ProvingKey<C>,
        p: &'a Argument,
        advice_cosets: &'a [poly::AstLeaf<Ev, ExtendedLagrangeCoeff>],
        fixed_cosets: &'a [poly::AstLeaf<Ev, ExtendedLagrangeCoeff>],
        instance_cosets: &'a [poly::AstLeaf<Ev, ExtendedLagrangeCoeff>],
        permutation_cosets: &'a [poly::AstLeaf<Ev, ExtendedLagrangeCoeff>],
        l0: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
        l_blind: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
        l_last: poly::AstLeaf<Ev, ExtendedLagrangeCoeff>,
        beta: ChallengeBeta<C>,
        gamma: ChallengeGamma<C>,
    ) -> (
        Constructed<C>,
        impl Iterator<Item = poly::Ast<Ev, C::Scalar, ExtendedLagrangeCoeff>> + 'a,
    ) {
        let chunk_len = pk.vk.cs_degree - 2;
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
            .chain(
                self.sets
                    .first()
                    .map(|first_set| (poly::Ast::one() - first_set.permutation_product_coset) * l0),
            )
            // Enforce only for the last set.
            // l_last(X) * (z_l(X)^2 - z_l(X)) = 0
            .chain(self.sets.last().map(|last_set| {
                ((poly::Ast::from(last_set.permutation_product_coset)
                    * last_set.permutation_product_coset)
                    - last_set.permutation_product_coset)
                    * l_last
            }))
            // Except for the first set, enforce.
            // l_0(X) * (z_i(X) - z_{i-1}(\omega^(last) X)) = 0
            .chain(
                self.sets
                    .iter()
                    .skip(1)
                    .zip(self.sets.iter())
                    .map(|(set, last_set)| {
                        (poly::Ast::from(set.permutation_product_coset)
                            - last_set
                                .permutation_product_coset
                                .with_rotation(last_rotation))
                            * l0
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
                    .zip(permutation_cosets.chunks(chunk_len))
                    .enumerate()
                    .map(move |(chunk_index, ((set, columns), cosets))| {
                        let mut left = poly::Ast::<_, C::Scalar, _>::from(
                            set.permutation_product_coset
                                .with_rotation(Rotation::next()),
                        );
                        for (values, permutation) in columns
                            .iter()
                            .map(|&column| match column.column_type() {
                                Any::Advice => &advice_cosets[column.index()],
                                Any::Fixed => &fixed_cosets[column.index()],
                                Any::Instance => &instance_cosets[column.index()],
                            })
                            .zip(cosets.iter())
                        {
                            left *= poly::Ast::<_, C::Scalar, _>::from(*values)
                                + (poly::Ast::ConstantTerm(*beta) * poly::Ast::from(*permutation))
                                + poly::Ast::ConstantTerm(*gamma);
                        }

                        let mut right = poly::Ast::from(set.permutation_product_coset);
                        let mut current_delta = *beta
                            * &(C::Scalar::DELTA.pow_vartime([(chunk_index * chunk_len) as u64]));
                        for values in columns.iter().map(|&column| match column.column_type() {
                            Any::Advice => &advice_cosets[column.index()],
                            Any::Fixed => &fixed_cosets[column.index()],
                            Any::Instance => &instance_cosets[column.index()],
                        }) {
                            right *= poly::Ast::from(*values)
                                + poly::Ast::LinearTerm(current_delta)
                                + poly::Ast::ConstantTerm(*gamma);
                            current_delta *= &C::Scalar::DELTA;
                        }

                        (left - right) * (poly::Ast::one() - (poly::Ast::from(l_last) + l_blind))
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
        let queries: Vec<(&[C::Scalar], C::Scalar)> =
            self.polys.iter().map(|poly| (&poly[..], *x)).collect();
        for eval in evaluate_batch(&queries) {
            transcript.write_scalar(eval)?;
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

        let x_next = domain.rotate_omega(*x, Rotation::next());
        let x_last = domain.rotate_omega(*x, Rotation(-((blinding_factors + 1) as i32)));

        // Each set is evaluated at x and omega x. Every set but the last is
        // also evaluated at omega^u x, so the verifier can constrain the last
        // value of its running product to equal the first value of the next
        // set's running product, chaining them together.
        let last_set = self.sets.len().saturating_sub(1);
        let queries: Vec<(&[C::Scalar], C::Scalar)> = self
            .sets
            .iter()
            .enumerate()
            .flat_map(|(index, set)| {
                let poly = &set.permutation_product_poly[..];
                iter::once((poly, *x))
                    .chain(iter::once((poly, x_next)))
                    .chain((index < last_set).then_some((poly, x_last)))
            })
            .collect();

        // Hash permutation product evals
        for eval in evaluate_batch(&queries) {
            transcript.write_scalar(eval)?;
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
