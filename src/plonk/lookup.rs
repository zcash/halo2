use super::circuit::{Any, Column};
use crate::arithmetic::CurveAffine;

mod prover;
mod verifier;

#[derive(Clone, Debug)]
pub(crate) struct Argument {
    pub input_columns: Vec<Column<Any>>,
    pub table_columns: Vec<Column<Any>>,
}

impl Argument {
    pub fn new(input_columns: &[Column<Any>], table_columns: &[Column<Any>]) -> Self {
        assert_eq!(input_columns.len(), table_columns.len());
        Argument {
            input_columns: input_columns.to_vec(),
            table_columns: table_columns.to_vec(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Proof<C: CurveAffine> {
    product_commitment: C,
    product_eval: C::Scalar,
    product_inv_eval: C::Scalar,
    permuted_input_commitment: C,
    permuted_table_commitment: C,
    permuted_input_eval: C::Scalar,
    permuted_input_inv_eval: C::Scalar,
    permuted_table_eval: C::Scalar,
}
