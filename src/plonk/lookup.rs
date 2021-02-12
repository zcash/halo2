use super::circuit::Expression;
use ff::Field;

pub(crate) mod prover;
pub(crate) mod verifier;

#[derive(Clone, Debug)]
pub(crate) struct Argument<F: Field> {
    pub input_columns: Vec<Expression<F>>,
    pub table_columns: Vec<Expression<F>>,
}

impl<F: Field> Argument<F> {
    pub fn new(input_columns: &[Expression<F>], table_columns: &[Expression<F>]) -> Self {
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
        // degree (1 + input_degree + table_degree):
        // z'(X) (a'(X) + \beta) (s'(X) + \gamma)
        // - z'(\omega^{-1} X) (\theta^{m-1} a_0(X) + ... + a_{m-1}(X) + \beta) (\theta^{m-1} s_0(X) + ... + s_{m-1}(X) + \gamma)
        //
        // degree 2:
        // l_0(X) * (a'(X) - s'(X)) = 0
        //
        // degree 2:
        // (a′(X)−s′(X))⋅(a′(X)−a′(\omega{-1} X)) = 0
        let mut input_degree = 1;
        for expr in self.input_columns.iter() {
            input_degree = std::cmp::max(input_degree, expr.degree());
        }
        let mut table_degree = 1;
        for expr in self.table_columns.iter() {
            table_degree = std::cmp::max(table_degree, expr.degree());
        }

        1 + input_degree + table_degree
    }
}
