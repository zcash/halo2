use super::circuit::Expression;
use ff::Field;

pub(crate) mod prover;
pub(crate) mod verifier;

#[derive(Clone, Debug)]
pub(crate) struct Argument<F: Field> {
    pub input_expressions: Vec<Expression<F>>,
    pub table_expressions: Vec<Expression<F>>,
}

impl<F: Field> Argument<F> {
    pub fn new(input_expressions: &[Expression<F>], table_expressions: &[Expression<F>]) -> Self {
        assert_eq!(input_expressions.len(), table_expressions.len());
        Argument {
            input_expressions: input_expressions.to_vec(),
            table_expressions: table_expressions.to_vec(),
        }
    }

    pub(crate) fn required_degree(&self) -> usize {
        assert_eq!(self.input_expressions.len(), self.table_expressions.len());

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
        for expr in self.input_expressions.iter() {
            input_degree = std::cmp::max(input_degree, expr.degree());
        }
        let mut table_degree = 1;
        for expr in self.table_expressions.iter() {
            table_degree = std::cmp::max(table_degree, expr.degree());
        }

        1 + input_degree + table_degree
    }
}
