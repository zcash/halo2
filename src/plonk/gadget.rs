use super::{Advice, Column};
use crate::{arithmetic::FieldExt, plonk::Error};

mod boolean;

/// This represents a specific cell: an advice column at a certain row in the
/// ConstraintSystem.
#[derive(Copy, Clone, Debug)]
pub struct Variable(Column<Advice>, usize);

trait MinimalCs<FF: FieldExt> {
    fn constrain_equal(&mut self, a: Variable, b: Variable) -> Result<(), Error>;
    fn public_input<F>(&mut self, f: F) -> Result<Variable, Error>
    where
        F: FnOnce() -> Result<FF, Error>;
}
