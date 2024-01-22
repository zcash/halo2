use halo2_middleware::circuit::{Any, Column};

/// A permutation argument.
#[derive(Debug, Clone)]
pub struct Argument {
    /// A sequence of columns involved in the argument.
    pub(super) columns: Vec<Column<Any>>,
}
