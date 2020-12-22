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

    pub(crate) fn required_degree(&self) -> usize {
        assert_eq!(self.input_columns.len(), self.table_columns.len());

        // degree 2:
        // l_0(X) * (1 - z'(X)) = 0
        //
        // degree 3:
        // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
        // - z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
        //
        // degree 2:
        // l_0(X) * (a'(X) - s'(X)) = 0
        //
        // degree 2:
        // (a′(X)−s′(X))⋅(a′(X)−a′(\omega{-1} X)) = 0
        3
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
