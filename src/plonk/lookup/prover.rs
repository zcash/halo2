use super::super::{
    circuit::{Advice, Any, Aux, Column, Fixed},
    Error, ProvingKey,
};
use super::Argument;
use crate::{
    arithmetic::{eval_polynomial, parallelize, BatchInvert, Curve, CurveAffine, FieldExt},
    poly::{
        commitment::{Blind, Params},
        Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial, Rotation,
    },
    transcript::{Hasher, Transcript},
};
use ff::Field;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Permuted<C: CurveAffine> {
    permuted_input_value: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    permuted_input_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_input_inv_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_input_blind: Blind<C::Scalar>,
    permuted_input_commitment: C,
    permuted_table_value: Polynomial<C::Scalar, LagrangeCoeff>,
    permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    permuted_table_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    permuted_table_blind: Blind<C::Scalar>,
    permuted_table_commitment: C,
}

#[derive(Clone, Debug)]
pub(crate) struct Product<C: CurveAffine> {
    product_poly: Polynomial<C::Scalar, Coeff>,
    product_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    product_inv_coset: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    product_blind: Blind<C::Scalar>,
    product_commitment: C,
}

#[derive(Clone, Debug)]
pub(crate) struct Committed<C: CurveAffine> {
    permuted: Permuted<C>,
    product: Product<C>,
}

pub(crate) struct Constructed<C: CurveAffine> {
    permuted_input_poly: Polynomial<C::Scalar, Coeff>,
    permuted_input_blind: Blind<C::Scalar>,
    permuted_input_commitment: C,
    permuted_table_poly: Polynomial<C::Scalar, Coeff>,
    permuted_table_blind: Blind<C::Scalar>,
    permuted_table_commitment: C,
    product_poly: Polynomial<C::Scalar, Coeff>,
    product_blind: Blind<C::Scalar>,
    product_commitment: C,
}

pub(crate) struct Evaluated<C: CurveAffine> {
    constructed: Constructed<C>,
    pub product_eval: C::Scalar,
    pub product_inv_eval: C::Scalar,
    pub permuted_input_eval: C::Scalar,
    pub permuted_input_inv_eval: C::Scalar,
    pub permuted_table_eval: C::Scalar,
}

impl Argument {
    /// Given a Lookup with input columns [A_0, A_1, ..., A_m] and table columns
    /// [S_0, S_1, ..., S_m], this method
    /// - constructs A_compressed = A_0 + theta A_1 + theta^2 A_2 + ... and
    ///   S_compressed = S_0 + theta S_1 + theta^2 S_2 + ...,
    /// - permutes A_compressed and S_compressed using permute_column_pair() helper,
    ///   obtaining A' and S', and
    /// - constructs Permuted<C> struct using permuted_input_value = A', and
    ///   permuted_table_value = S'.
    /// The Permuted<C> struct is used to update the Lookup, and is then returned.
    pub(in crate::plonk) fn commit_permuted<
        C: CurveAffine,
        HBase: Hasher<C::Base>,
        HScalar: Hasher<C::Scalar>,
    >(
        &self,
        pk: &ProvingKey<C>,
        params: &Params<C>,
        domain: &EvaluationDomain<C::Scalar>,
        theta: C::Scalar,
        advice_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        fixed_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        aux_values: &[Polynomial<C::Scalar, LagrangeCoeff>],
        transcript: &mut Transcript<C, HBase, HScalar>,
    ) -> Result<Permuted<C>, Error> {
        // Values of input columns involved in the lookup
        let unpermuted_input_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .input_columns
            .iter()
            .map(|&input| match input.column_type() {
                Any::Advice => advice_values[input.index()].clone(),
                Any::Fixed => fixed_values[input.index()].clone(),
                Any::Aux => aux_values[input.index()].clone(),
            })
            .collect();

        // Compressed version of input columns
        let compressed_input_value = unpermuted_input_values
            .iter()
            .fold(domain.empty_lagrange(), |acc, input| acc * theta + input);

        // Values of table columns involved in the lookup
        let unpermuted_table_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>> = self
            .table_columns
            .iter()
            .map(|&table| match table.column_type() {
                Any::Advice => advice_values[table.index()].clone(),
                Any::Fixed => fixed_values[table.index()].clone(),
                Any::Aux => aux_values[table.index()].clone(),
            })
            .collect();

        // Compressed version of table columns
        let compressed_table_value = unpermuted_table_values
            .iter()
            .fold(domain.empty_lagrange(), |acc, table| acc * theta + table);

        // Permute compressed (InputColumn, TableColumn) pair
        let (permuted_input_value, permuted_table_value) =
            permute_column_pair::<C>(domain, &compressed_input_value, &compressed_table_value)?;

        // Construct Permuted struct
        let permuted_input_poly = pk.vk.domain.lagrange_to_coeff(permuted_input_value.clone());
        let permuted_input_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_input_poly.clone(), Rotation::default());
        let permuted_input_inv_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_input_poly.clone(), Rotation(-1));

        let permuted_input_blind = Blind(C::Scalar::rand());
        let permuted_input_commitment = params
            .commit_lagrange(&permuted_input_value, permuted_input_blind)
            .to_affine();

        let permuted_table_poly = pk.vk.domain.lagrange_to_coeff(permuted_table_value.clone());
        let permuted_table_coset = pk
            .vk
            .domain
            .coeff_to_extended(permuted_table_poly.clone(), Rotation::default());
        let permuted_table_blind = Blind(C::Scalar::rand());
        let permuted_table_commitment = params
            .commit_lagrange(&permuted_table_value, permuted_table_blind)
            .to_affine();

        // Hash each permuted input commitment
        transcript
            .absorb_point(&permuted_input_commitment)
            .map_err(|_| Error::TranscriptError)?;

        // Hash each permuted table commitment
        transcript
            .absorb_point(&permuted_table_commitment)
            .map_err(|_| Error::TranscriptError)?;

        Ok(Permuted {
            permuted_input_value,
            permuted_input_poly,
            permuted_input_coset,
            permuted_input_inv_coset,
            permuted_input_blind,
            permuted_input_commitment,
            permuted_table_value,
            permuted_table_poly,
            permuted_table_coset,
            permuted_table_blind,
            permuted_table_commitment,
        })
    }
}

/// Given a column of input values A and a column of table values S,
/// this method permutes A and S to produce A' and S', such that:
/// - like values in A' are vertically adjacent to each other; and
/// - the first row in a sequence of like values in A' is the row
///   that has the corresponding value in S'.
/// This method returns (A', S') if no errors are encountered.
fn permute_column_pair<C: CurveAffine>(
    domain: &EvaluationDomain<C::Scalar>,
    input_column: &Polynomial<C::Scalar, LagrangeCoeff>,
    table_column: &Polynomial<C::Scalar, LagrangeCoeff>,
) -> Result<
    (
        Polynomial<C::Scalar, LagrangeCoeff>,
        Polynomial<C::Scalar, LagrangeCoeff>,
    ),
    Error,
> {
    let mut permuted_input_column = input_column.clone();

    // Sort input lookup column values
    permuted_input_column.sort();

    // A BTreeMap of each unique element in the table column and its count
    let mut leftover_table_map: BTreeMap<C::Scalar, u32> =
        table_column.iter().fold(BTreeMap::new(), |mut acc, coeff| {
            *acc.entry(*coeff).or_insert(0) += 1;
            acc
        });
    let mut repeated_input_rows = vec![];
    let mut permuted_table_coeffs = vec![C::Scalar::zero(); table_column.len()];

    for row in 0..permuted_input_column.len() {
        let input_value = permuted_input_column[row];

        // If this is the first occurence of `input_value` in the input column
        if row == 0 || input_value != permuted_input_column[row - 1] {
            permuted_table_coeffs[row] = input_value;
            // Remove one instance of input_value from leftover_table_map
            if let Some(count) = leftover_table_map.get_mut(&input_value) {
                assert!(*count > 0);
                *count -= 1;
            } else {
                // Return error if input_value not found
                return Err(Error::ConstraintSystemFailure);
            }
        // If input value is repeated
        } else {
            repeated_input_rows.push(row);
        }
    }

    // Populate permuted table at unfilled rows with leftover table elements
    for (coeff, count) in leftover_table_map.iter() {
        for _ in 0..*count {
            permuted_table_coeffs[repeated_input_rows.pop().unwrap() as usize] = *coeff;
        }
    }
    assert!(repeated_input_rows.is_empty());

    let mut permuted_table_column = domain.empty_lagrange();
    parallelize(
        &mut permuted_table_column,
        |permuted_table_column, start| {
            for (permuted_table_value, permuted_table_coeff) in permuted_table_column
                .iter_mut()
                .zip(permuted_table_coeffs[start..].iter())
            {
                *permuted_table_value += permuted_table_coeff;
            }
        },
    );

    Ok((permuted_input_column, permuted_table_column))
}
