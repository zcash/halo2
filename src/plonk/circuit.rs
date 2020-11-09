use core::cmp::max;
use core::ops::{Add, Mul};
use std::collections::BTreeMap;
use std::convert::TryFrom;

use super::Error;
use crate::arithmetic::Field;
use crate::poly::Rotation;

/// A column type
pub trait ColumnType: 'static + Sized {}

/// A column with an index and type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Column<C: ColumnType> {
    index: usize,
    column_type: C,
}

impl<C: ColumnType> Column<C> {
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn column_type(&self) -> &C {
        &self.column_type
    }
}

/// An advice column
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Advice;

/// A fixed column
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fixed;

/// An auxiliary column
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Aux;

/// An enum over the Advice, Fixed, Aux structs
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Any {
    /// An Advice variant
    Advice,
    /// A Fixed variant
    Fixed,
    /// An Auxiliary variant
    Aux,
}

impl ColumnType for Advice {}
impl ColumnType for Fixed {}
impl ColumnType for Aux {}
impl ColumnType for Any {}

impl From<Column<Advice>> for Column<Any> {
    fn from(advice: Column<Advice>) -> Column<Any> {
        Column {
            index: advice.index(),
            column_type: Any::Advice,
        }
    }
}

impl From<Column<Fixed>> for Column<Any> {
    fn from(advice: Column<Fixed>) -> Column<Any> {
        Column {
            index: advice.index(),
            column_type: Any::Fixed,
        }
    }
}

impl From<Column<Aux>> for Column<Any> {
    fn from(advice: Column<Aux>) -> Column<Any> {
        Column {
            index: advice.index(),
            column_type: Any::Aux,
        }
    }
}

impl TryFrom<Column<Any>> for Column<Advice> {
    type Error = &'static str;

    fn try_from(any: Column<Any>) -> Result<Self, Self::Error> {
        match any.column_type() {
            Any::Advice => {
                return Ok(Column {
                    index: any.index(),
                    column_type: Advice,
                })
            }
            _ => Err("Cannot convert into Column<Advice>"),
        }
    }
}

impl TryFrom<Column<Any>> for Column<Fixed> {
    type Error = &'static str;

    fn try_from(any: Column<Any>) -> Result<Self, Self::Error> {
        match any.column_type() {
            Any::Fixed => {
                return Ok(Column {
                    index: any.index(),
                    column_type: Fixed,
                })
            }
            _ => Err("Cannot convert into Column<Fixed>"),
        }
    }
}

impl TryFrom<Column<Any>> for Column<Aux> {
    type Error = &'static str;

    fn try_from(any: Column<Any>) -> Result<Self, Self::Error> {
        match any.column_type() {
            Any::Aux => {
                return Ok(Column {
                    index: any.index(),
                    column_type: Aux,
                })
            }
            _ => Err("Cannot convert into Column<Aux>"),
        }
    }
}

/// This trait allows a [`Circuit`] to direct some backend to assign a witness
/// for a constraint system.
pub trait Assignment<F: Field> {
    /// Assign an advice column value (witness)
    fn assign_advice(
        &mut self,
        column: Column<Advice>,
        row: usize,
        to: impl FnOnce() -> Result<F, Error>,
    ) -> Result<(), Error>;

    /// Assign a fixed value
    fn assign_fixed(
        &mut self,
        column: Column<Fixed>,
        row: usize,
        to: impl FnOnce() -> Result<F, Error>,
    ) -> Result<(), Error>;

    /// Assign two advice columns to have the same value
    fn copy(
        &mut self,
        permutation: usize,
        left_column: usize,
        left_row: usize,
        right_column: usize,
        right_row: usize,
    ) -> Result<(), Error>;
}

/// This is a trait that circuits provide implementations for so that the
/// backend prover can ask the circuit to synthesize using some given
/// [`ConstraintSystem`] implementation.
pub trait Circuit<F: Field> {
    /// This is a configuration object that stores things like columns.
    type Config;

    /// The circuit is given an opportunity to describe the exact gate
    /// arrangement, column arrangement, etc.
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config;

    /// Given the provided `cs`, synthesize the circuit. The concrete type of
    /// the caller will be different depending on the context, and they may or
    /// may not expect to have a witness present.
    fn synthesize(&self, cs: &mut impl Assignment<F>, config: Self::Config) -> Result<(), Error>;
}

/// Low-degree expression representing an identity that must hold over the committed columns.
#[derive(Clone, Debug)]
pub enum Expression<F> {
    /// This is a fixed column queried at a certain relative location
    Fixed(usize),
    /// This is an advice (witness) column queried at a certain relative location
    Advice(usize),
    /// This is an auxiliary (external) column queried at a certain relative location
    Aux(usize),
    /// This is the sum of two polynomials
    Sum(Box<Expression<F>>, Box<Expression<F>>),
    /// This is the product of two polynomials
    Product(Box<Expression<F>>, Box<Expression<F>>),
    /// This is a scaled polynomial
    Scaled(Box<Expression<F>>, F),
}

impl<F: Field> Expression<F> {
    /// Evaluate the polynomial using the provided closures to perform the
    /// operations.
    pub fn evaluate<T>(
        &self,
        fixed_column: &impl Fn(usize) -> T,
        advice_column: &impl Fn(usize) -> T,
        aux_column: &impl Fn(usize) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
        scaled: &impl Fn(T, F) -> T,
    ) -> T {
        match self {
            Expression::Fixed(index) => fixed_column(*index),
            Expression::Advice(index) => advice_column(*index),
            Expression::Aux(index) => aux_column(*index),
            Expression::Sum(a, b) => {
                let a = a.evaluate(
                    fixed_column,
                    advice_column,
                    aux_column,
                    sum,
                    product,
                    scaled,
                );
                let b = b.evaluate(
                    fixed_column,
                    advice_column,
                    aux_column,
                    sum,
                    product,
                    scaled,
                );
                sum(a, b)
            }
            Expression::Product(a, b) => {
                let a = a.evaluate(
                    fixed_column,
                    advice_column,
                    aux_column,
                    sum,
                    product,
                    scaled,
                );
                let b = b.evaluate(
                    fixed_column,
                    advice_column,
                    aux_column,
                    sum,
                    product,
                    scaled,
                );
                product(a, b)
            }
            Expression::Scaled(a, f) => {
                let a = a.evaluate(
                    fixed_column,
                    advice_column,
                    aux_column,
                    sum,
                    product,
                    scaled,
                );
                scaled(a, *f)
            }
        }
    }

    /// Compute the degree of this polynomial
    pub fn degree(&self) -> usize {
        match self {
            Expression::Fixed(_) => 1,
            Expression::Advice(_) => 1,
            Expression::Aux(_) => 1,
            Expression::Sum(a, b) => max(a.degree(), b.degree()),
            Expression::Product(a, b) => a.degree() + b.degree(),
            Expression::Scaled(poly, _) => poly.degree(),
        }
    }
}

impl<F> Add for Expression<F> {
    type Output = Expression<F>;
    fn add(self, rhs: Expression<F>) -> Expression<F> {
        Expression::Sum(Box::new(self), Box::new(rhs))
    }
}

impl<F> Mul for Expression<F> {
    type Output = Expression<F>;
    fn mul(self, rhs: Expression<F>) -> Expression<F> {
        Expression::Product(Box::new(self), Box::new(rhs))
    }
}

impl<F> Mul<F> for Expression<F> {
    type Output = Expression<F>;
    fn mul(self, rhs: F) -> Expression<F> {
        Expression::Scaled(Box::new(self), rhs)
    }
}

/// Represents an index into a vector where each entry corresponds to a distinct
/// point that polynomials are queried at.
#[derive(Copy, Clone, Debug)]
pub(crate) struct PointIndex(pub usize);

/// This is a description of the circuit environment, such as the gate, column and
/// permutation arrangements.
#[derive(Debug, Clone)]
pub struct ConstraintSystem<F> {
    pub(crate) num_fixed_columns: usize,
    pub(crate) num_advice_columns: usize,
    pub(crate) num_aux_columns: usize,
    pub(crate) gates: Vec<Expression<F>>,
    pub(crate) advice_queries: Vec<(Column<Advice>, Rotation)>,
    pub(crate) aux_queries: Vec<(Column<Aux>, Rotation)>,
    pub(crate) fixed_queries: Vec<(Column<Fixed>, Rotation)>,

    // Mapping from a witness vector rotation to the index in the point vector.
    pub(crate) rotations: BTreeMap<Rotation, PointIndex>,

    // Vector of permutation arguments, where each corresponds to a sequence of columns
    // that are involved in a permutation argument.
    pub(crate) permutations: Vec<Vec<Column<Advice>>>,
}

impl<F: Field> Default for ConstraintSystem<F> {
    fn default() -> ConstraintSystem<F> {
        let mut rotations = BTreeMap::new();
        rotations.insert(Rotation::default(), PointIndex(0));

        ConstraintSystem {
            num_fixed_columns: 0,
            num_advice_columns: 0,
            num_aux_columns: 0,
            gates: vec![],
            fixed_queries: Vec::new(),
            advice_queries: Vec::new(),
            aux_queries: Vec::new(),
            rotations,
            permutations: Vec::new(),
        }
    }
}

impl<F: Field> ConstraintSystem<F> {
    /// Add a permutation argument for some advice columns
    pub fn permutation(&mut self, columns: &[Column<Advice>]) -> usize {
        let index = self.permutations.len();
        if self.permutations.is_empty() {
            let at = Rotation(-1);
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        for column in columns {
            self.query_advice_index(*column, 0);
        }
        self.permutations.push(columns.to_vec());

        index
    }

    fn query_fixed_index(&mut self, column: Column<Fixed>, at: i32) -> usize {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // Return existing query, if it exists
        for (index, fixed_query) in self.fixed_queries.iter().enumerate() {
            if fixed_query == &(column, at) {
                return index;
            }
        }

        // Make a new query
        let index = self.fixed_queries.len();
        self.fixed_queries.push((column, at));

        index
    }

    /// Query a fixed column at a relative position
    pub fn query_fixed(&mut self, column: Column<Fixed>, at: i32) -> Expression<F> {
        Expression::Fixed(self.query_fixed_index(column, at))
    }

    pub(crate) fn query_advice_index(&mut self, column: Column<Advice>, at: i32) -> usize {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // Return existing query, if it exists
        for (index, advice_query) in self.advice_queries.iter().enumerate() {
            if advice_query == &(column, at) {
                return index;
            }
        }

        // Make a new query
        let index = self.advice_queries.len();
        self.advice_queries.push((column, at));

        index
    }

    /// Query an advice column at a relative position
    pub fn query_advice(&mut self, column: Column<Advice>, at: i32) -> Expression<F> {
        Expression::Advice(self.query_advice_index(column, at))
    }

    fn query_aux_index(&mut self, column: Column<Aux>, at: i32) -> usize {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // Return existing query, if it exists
        for (index, aux_query) in self.aux_queries.iter().enumerate() {
            if aux_query == &(column, at) {
                return index;
            }
        }

        // Make a new query
        let index = self.aux_queries.len();
        self.aux_queries.push((column, at));

        index
    }

    /// Query an auxiliary column at a relative position
    pub fn query_aux(&mut self, column: Column<Aux>, at: i32) -> Expression<F> {
        Expression::Aux(self.query_aux_index(column, at))
    }

    fn query_any_index(&mut self, column: Column<Any>, at: i32) -> usize {
        let index = match column.column_type() {
            Any::Advice => self.query_advice_index(Column::<Advice>::try_from(column).unwrap(), at),
            Any::Fixed => self.query_fixed_index(Column::<Fixed>::try_from(column).unwrap(), at),
            Any::Aux => self.query_aux_index(Column::<Aux>::try_from(column).unwrap(), at),
        };

        index
    }

    /// Query an Any column at a relative position
    pub fn query_any(&mut self, column: Column<Any>, at: i32) -> Expression<F> {
        match column.column_type() {
            Any::Advice => Expression::Advice(
                self.query_advice_index(Column::<Advice>::try_from(column).unwrap(), at),
            ),
            Any::Fixed => Expression::Fixed(
                self.query_fixed_index(Column::<Fixed>::try_from(column).unwrap(), at),
            ),
            Any::Aux => {
                Expression::Aux(self.query_aux_index(Column::<Aux>::try_from(column).unwrap(), at))
            }
        }
    }

    pub(crate) fn get_advice_query_index(&self, column: Column<Advice>, at: i32) -> usize {
        let at = Rotation(at);
        for (index, advice_query) in self.advice_queries.iter().enumerate() {
            if advice_query == &(column, at) {
                return index;
            }
        }

        panic!("get_advice_query_index called for non-existent query");
    }

    pub(crate) fn get_fixed_query_index(&self, column: Column<Fixed>, at: i32) -> usize {
        let at = Rotation(at);
        for (index, fixed_query) in self.fixed_queries.iter().enumerate() {
            if fixed_query == &(column, at) {
                return index;
            }
        }

        panic!("get_fixed_query_index called for non-existent query");
    }

    pub(crate) fn get_aux_query_index(&self, column: Column<Aux>, at: i32) -> usize {
        let at = Rotation(at);
        for (index, aux_query) in self.aux_queries.iter().enumerate() {
            if aux_query == &(column, at) {
                return index;
            }
        }

        panic!("get_aux_query_index called for non-existent query");
    }

    pub(crate) fn get_any_query_index(&self, column: Column<Any>, at: i32) -> usize {
        let index = match column.column_type() {
            Any::Advice => {
                self.get_advice_query_index(Column::<Advice>::try_from(column).unwrap(), at)
            }
            Any::Fixed => {
                self.get_fixed_query_index(Column::<Fixed>::try_from(column).unwrap(), at)
            }
            Any::Aux => self.get_aux_query_index(Column::<Aux>::try_from(column).unwrap(), at),
        };

        index
    }

    /// Create a new gate
    pub fn create_gate(&mut self, f: impl FnOnce(&mut Self) -> Expression<F>) {
        let poly = f(self);
        self.gates.push(poly);
    }

    /// Allocate a new fixed column
    pub fn fixed_column(&mut self) -> Column<Fixed> {
        let tmp = Column {
            index: self.num_fixed_columns,
            column_type: Fixed,
        };
        self.num_fixed_columns += 1;
        tmp
    }

    /// Allocate a new advice column
    pub fn advice_column(&mut self) -> Column<Advice> {
        let tmp = Column {
            index: self.num_advice_columns,
            column_type: Advice,
        };
        self.num_advice_columns += 1;
        tmp
    }

    /// Allocate a new auxiliary column
    pub fn aux_column(&mut self) -> Column<Aux> {
        let tmp = Column {
            index: self.num_aux_columns,
            column_type: Aux,
        };
        self.num_aux_columns += 1;
        tmp
    }
}
