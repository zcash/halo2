use super::circuit::ExpressionMid;
use ff::Field;

/// Expressions involved in a lookup argument, with a name as metadata.
#[derive(Clone, Debug)]
pub struct ArgumentV2<F: Field> {
    pub(crate) name: String,
    pub(crate) input_expressions: Vec<ExpressionMid<F>>,
    pub(crate) table_expressions: Vec<ExpressionMid<F>>,
}
