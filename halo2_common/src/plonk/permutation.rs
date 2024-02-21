//! Implementation of permutation argument.

use crate::plonk::{Column, Error};
use halo2_middleware::circuit::{Any, Cell};
use halo2_middleware::permutation::ArgumentV2;

/// A permutation argument.
#[derive(Default, Debug, Clone)]
pub struct Argument {
    /// A sequence of columns involved in the argument.
    pub columns: Vec<Column<Any>>,
}

impl From<ArgumentV2> for Argument {
    fn from(arg: ArgumentV2) -> Self {
        Self {
            columns: arg.columns.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl Argument {
    /// Returns the minimum circuit degree required by the permutation argument.
    /// The argument may use larger degree gates depending on the actual
    /// circuit's degree and how many columns are involved in the permutation.
    pub(crate) fn required_degree(&self) -> usize {
        // degree 2:
        // l_0(X) * (1 - z(X)) = 0
        //
        // We will fit as many polynomials p_i(X) as possible
        // into the required degree of the circuit, so the
        // following will not affect the required degree of
        // this middleware.
        //
        // (1 - (l_last(X) + l_blind(X))) * (
        //   z(\omega X) \prod (p(X) + \beta s_i(X) + \gamma)
        // - z(X) \prod (p(X) + \delta^i \beta X + \gamma)
        // )
        //
        // On the first sets of columns, except the first
        // set, we will do
        //
        // l_0(X) * (z(X) - z'(\omega^(last) X)) = 0
        //
        // where z'(X) is the permutation for the previous set
        // of columns.
        //
        // On the final set of columns, we will do
        //
        // degree 3:
        // l_last(X) * (z'(X)^2 - z'(X)) = 0
        //
        // which will allow the last value to be zero to
        // ensure the argument is perfectly complete.

        // There are constraints of degree 3 regardless of the
        // number of columns involved.
        3
    }

    pub(crate) fn add_column(&mut self, column: Column<Any>) {
        if !self.columns.contains(&column) {
            self.columns.push(column);
        }
    }

    /// Returns columns that participate on the permutation argument.
    pub fn get_columns(&self) -> Vec<Column<Any>> {
        self.columns.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Assembly {
    pub n: usize,
    pub columns: Vec<Column<Any>>,
    pub copies: Vec<(Cell, Cell)>,
}

impl Assembly {
    pub fn new(n: usize, p: &Argument) -> Self {
        Self {
            n,
            columns: p.columns.clone(),
            copies: Vec::new(),
        }
    }

    pub fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), Error> {
        if !self.columns.contains(&left_column) {
            return Err(Error::ColumnNotInPermutation(left_column));
        }
        if !self.columns.contains(&right_column) {
            return Err(Error::ColumnNotInPermutation(right_column));
        }
        // Check bounds
        if left_row >= self.n || right_row >= self.n {
            return Err(Error::BoundsFailure);
        }
        self.copies.push((
            Cell {
                column: left_column.into(),
                row: left_row,
            },
            Cell {
                column: right_column.into(),
                row: right_row,
            },
        ));
        Ok(())
    }
}
