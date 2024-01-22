use super::circuit::Expression;
use halo2_middleware::ff::Field;

/// Expressions involved in a shuffle argument, with a name as metadata.
#[derive(Clone)]
pub struct Argument<F: Field> {
    pub(crate) name: String,
    pub(crate) input_expressions: Vec<Expression<F>>,
    pub(crate) shuffle_expressions: Vec<Expression<F>>,
}
