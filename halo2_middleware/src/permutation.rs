use crate::circuit::ColumnMid;

// TODO: Dedup with other Cell definition, or move this to a higher level
#[derive(Clone, Debug)]
pub struct Cell {
    pub column: ColumnMid,
    pub row: usize,
}

#[derive(Clone, Debug)]
pub struct AssemblyMid {
    pub copies: Vec<(Cell, Cell)>,
}

/// A permutation argument.
#[derive(Debug, Clone)]
pub struct ArgumentV2 {
    /// A sequence of columns involved in the argument.
    pub columns: Vec<ColumnMid>,
}
