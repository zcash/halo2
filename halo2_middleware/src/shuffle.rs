use super::circuit::VarMid;
use super::expression::{Expression, Variable};
use ff::Field;

/// Expressions involved in a shuffle argument, with a name as metadata.
#[derive(Clone, Debug)]
pub struct Argument<F: Field, V: Variable> {
    pub name: String,
    pub input_expressions: Vec<Expression<F, V>>,
    pub shuffle_expressions: Vec<Expression<F, V>>,
}

pub type ArgumentMid<F> = Argument<F, VarMid>;
