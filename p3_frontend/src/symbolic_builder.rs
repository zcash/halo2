//! `SymbolicAirBuilder` copied from plonky3 and adapted for the Air to Plonkish usecase, at commit
//! `7b5b8a69f633bc61c530f3722701e5f701b11963`.

// The MIT License (MIT)
//
// Copyright (c) 2022 The Plonky3 Authors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use alloc::vec;
use alloc::vec::Vec;

use p3_air::AirBuilder;
use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;

use crate::air::AirBuilderWithPublicValues;
use crate::symbolic_expression::{Location, SymbolicExpression};
use crate::symbolic_variable::SymbolicVariable;

/// An `AirBuilder` for evaluating constraints symbolically, and recording them for later use.
pub struct SymbolicAirBuilder<F: Field> {
    pub(crate) main: RowMajorMatrix<SymbolicVariable<F>>,
    pub(crate) public_values: Vec<SymbolicVariable<F>>,
    pub(crate) constraints: Vec<SymbolicExpression<F>>,
}

impl<F: Field> SymbolicAirBuilder<F> {
    pub(crate) fn new(width: usize, num_public_values: usize) -> Self {
        let values = [false, true]
            .into_iter()
            .flat_map(|is_next| {
                (0..width).map(move |column| SymbolicVariable::new_query(is_next, column))
            })
            .collect();
        Self {
            main: RowMajorMatrix::new(values, width),
            public_values: (0..num_public_values)
                .map(|i| SymbolicVariable::new_public(i))
                .collect(),
            constraints: vec![],
        }
    }
}

impl<F: Field> AirBuilder for SymbolicAirBuilder<F> {
    type F = F;
    type Expr = SymbolicExpression<F>;
    type Var = SymbolicVariable<F>;
    type M = RowMajorMatrix<Self::Var>;

    fn main(&self) -> Self::M {
        self.main.clone()
    }

    fn is_first_row(&self) -> Self::Expr {
        SymbolicExpression::Location(Location::FirstRow)
    }

    fn is_last_row(&self) -> Self::Expr {
        SymbolicExpression::Location(Location::LastRow)
    }

    // TODO: Figure out what's a window size > 2.
    fn is_transition_window(&self, size: usize) -> Self::Expr {
        if size == 2 {
            SymbolicExpression::Location(Location::Transition)
        } else {
            panic!("uni-stark only supports a window size of 2")
        }
    }

    fn assert_zero<I: Into<Self::Expr>>(&mut self, x: I) {
        self.constraints.push(x.into());
    }
}

impl<F: Field> AirBuilderWithPublicValues for SymbolicAirBuilder<F> {
    fn public_values(&self) -> &[Self::Var] {
        self.public_values.as_slice()
    }
}
