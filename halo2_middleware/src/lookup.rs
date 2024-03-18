use super::circuit::VarMid;
use super::expression::{Expression, Variable};
use ff::Field;

/// Expressions involved in a lookup argument, with a name as metadata.
#[derive(Clone, Debug)]
pub struct Argument<F: Field, V: Variable> {
    pub name: String,
    pub input_expressions: Vec<Expression<F, V>>,
    pub table_expressions: Vec<Expression<F, V>>,
}

pub type ArgumentMid<F> = Argument<F, VarMid>;
