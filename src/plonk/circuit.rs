use core::cmp::max;
use core::ops::{Add, Mul};
use ff::Field;
use std::{
    convert::TryFrom,
    ops::{Neg, Sub},
};

use super::{lookup, permutation, Error};
use crate::poly::Rotation;

/// A column type
pub trait ColumnType: 'static + Sized + std::fmt::Debug {}

/// A column with an index and type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Advice;

/// A fixed column
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Fixed;

/// An instance column
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Instance;

/// An enum over the Advice, Fixed, Instance structs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Any {
    /// An Advice variant
    Advice,
    /// A Fixed variant
    Fixed,
    /// An Instance variant
    Instance,
}

impl ColumnType for Advice {}
impl ColumnType for Fixed {}
impl ColumnType for Instance {}
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

impl From<Column<Instance>> for Column<Any> {
    fn from(advice: Column<Instance>) -> Column<Any> {
        Column {
            index: advice.index(),
            column_type: Any::Instance,
        }
    }
}

impl TryFrom<Column<Any>> for Column<Advice> {
    type Error = &'static str;

    fn try_from(any: Column<Any>) -> Result<Self, Self::Error> {
        match any.column_type() {
            Any::Advice => Ok(Column {
                index: any.index(),
                column_type: Advice,
            }),
            _ => Err("Cannot convert into Column<Advice>"),
        }
    }
}

impl TryFrom<Column<Any>> for Column<Fixed> {
    type Error = &'static str;

    fn try_from(any: Column<Any>) -> Result<Self, Self::Error> {
        match any.column_type() {
            Any::Fixed => Ok(Column {
                index: any.index(),
                column_type: Fixed,
            }),
            _ => Err("Cannot convert into Column<Fixed>"),
        }
    }
}

impl TryFrom<Column<Any>> for Column<Instance> {
    type Error = &'static str;

    fn try_from(any: Column<Any>) -> Result<Self, Self::Error> {
        match any.column_type() {
            Any::Instance => Ok(Column {
                index: any.index(),
                column_type: Instance,
            }),
            _ => Err("Cannot convert into Column<Instance>"),
        }
    }
}

/// This trait allows a [`Circuit`] to direct some backend to assign a witness
/// for a constraint system.
pub trait Assignment<F: Field> {
    /// Creates a new region and enters into it.
    ///
    /// Panics if we are currently in a region (if `exit_region` was not called).
    ///
    /// Not intended for downstream consumption; use [`Layouter::assign_region`] instead.
    ///
    /// [`Layouter::assign_region`]: crate::circuit::Layouter#method.assign_region
    fn enter_region<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR;

    /// Exits the current region.
    ///
    /// Panics if we are not currently in a region (if `enter_region` was not called).
    ///
    /// Not intended for downstream consumption; use [`Layouter::assign_region`] instead.
    ///
    /// [`Layouter::assign_region`]: crate::circuit::Layouter#method.assign_region
    fn exit_region(&mut self);

    /// Assign an advice column value (witness)
    fn assign_advice<V, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<F, Error>,
        A: FnOnce() -> AR,
        AR: Into<String>;

    /// Assign a fixed value
    fn assign_fixed<V, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Fixed>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<F, Error>,
        A: FnOnce() -> AR,
        AR: Into<String>;

    /// Assign two advice columns to have the same value
    fn copy(
        &mut self,
        permutation: usize,
        left_column: usize,
        left_row: usize,
        right_column: usize,
        right_row: usize,
    ) -> Result<(), Error>;

    /// Creates a new (sub)namespace and enters into it.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    ///
    /// [`Layouter::namespace`]: crate::circuit::Layouter#method.namespace
    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR;

    /// Exits out of the existing namespace.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    ///
    /// [`Layouter::namespace`]: crate::circuit::Layouter#method.namespace
    fn pop_namespace(&mut self, gadget_name: Option<String>);
}

/// This is a trait that circuits provide implementations for so that the
/// backend prover can ask the circuit to synthesize using some given
/// [`ConstraintSystem`] implementation.
pub trait Circuit<F: Field> {
    /// This is a configuration object that stores things like columns.
    type Config: Clone;

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
    /// This is a constant polynomial
    Constant(F),
    /// This is a fixed column queried at a certain relative location
    Fixed(usize),
    /// This is an advice (witness) column queried at a certain relative location
    Advice(usize),
    /// This is an instance (external) column queried at a certain relative location
    Instance(usize),
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
        constant: &impl Fn(F) -> T,
        fixed_column: &impl Fn(usize) -> T,
        advice_column: &impl Fn(usize) -> T,
        instance_column: &impl Fn(usize) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
        scaled: &impl Fn(T, F) -> T,
    ) -> T {
        match self {
            Expression::Constant(scalar) => constant(*scalar),
            Expression::Fixed(index) => fixed_column(*index),
            Expression::Advice(index) => advice_column(*index),
            Expression::Instance(index) => instance_column(*index),
            Expression::Sum(a, b) => {
                let a = a.evaluate(
                    constant,
                    fixed_column,
                    advice_column,
                    instance_column,
                    sum,
                    product,
                    scaled,
                );
                let b = b.evaluate(
                    constant,
                    fixed_column,
                    advice_column,
                    instance_column,
                    sum,
                    product,
                    scaled,
                );
                sum(a, b)
            }
            Expression::Product(a, b) => {
                let a = a.evaluate(
                    constant,
                    fixed_column,
                    advice_column,
                    instance_column,
                    sum,
                    product,
                    scaled,
                );
                let b = b.evaluate(
                    constant,
                    fixed_column,
                    advice_column,
                    instance_column,
                    sum,
                    product,
                    scaled,
                );
                product(a, b)
            }
            Expression::Scaled(a, f) => {
                let a = a.evaluate(
                    constant,
                    fixed_column,
                    advice_column,
                    instance_column,
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
            Expression::Constant(_) => 0,
            Expression::Fixed(_) => 1,
            Expression::Advice(_) => 1,
            Expression::Instance(_) => 1,
            Expression::Sum(a, b) => max(a.degree(), b.degree()),
            Expression::Product(a, b) => a.degree() + b.degree(),
            Expression::Scaled(poly, _) => poly.degree(),
        }
    }
}

impl<F: Field> Neg for Expression<F> {
    type Output = Expression<F>;
    fn neg(self) -> Self::Output {
        Expression::Scaled(Box::new(self), -F::one())
    }
}

impl<F> Add for Expression<F> {
    type Output = Expression<F>;
    fn add(self, rhs: Expression<F>) -> Expression<F> {
        Expression::Sum(Box::new(self), Box::new(rhs))
    }
}

impl<F: Field> Sub for Expression<F> {
    type Output = Expression<F>;
    fn sub(self, rhs: Expression<F>) -> Expression<F> {
        Expression::Sum(Box::new(self), Box::new(-rhs))
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
pub struct ConstraintSystem<F: Field> {
    pub(crate) num_fixed_columns: usize,
    pub(crate) num_advice_columns: usize,
    pub(crate) num_instance_columns: usize,
    pub(crate) gates: Vec<(&'static str, Expression<F>)>,
    pub(crate) advice_queries: Vec<(Column<Advice>, Rotation)>,
    pub(crate) instance_queries: Vec<(Column<Instance>, Rotation)>,
    pub(crate) fixed_queries: Vec<(Column<Fixed>, Rotation)>,

    // Vector of permutation arguments, where each corresponds to a sequence of columns
    // that are involved in a permutation argument.
    pub(crate) permutations: Vec<permutation::Argument>,

    // Vector of lookup arguments, where each corresponds to a sequence of
    // input expressions and a sequence of table expressions involved in the lookup.
    pub(crate) lookups: Vec<lookup::Argument<F>>,
}

/// Represents the minimal parameters that determine a `ConstraintSystem`.
#[derive(Debug)]
pub struct PinnedConstraintSystem<'a, F: Field> {
    num_fixed_columns: &'a usize,
    num_advice_columns: &'a usize,
    num_instance_columns: &'a usize,
    gates: PinnedGates<'a, F>,
    advice_queries: &'a Vec<(Column<Advice>, Rotation)>,
    instance_queries: &'a Vec<(Column<Instance>, Rotation)>,
    fixed_queries: &'a Vec<(Column<Fixed>, Rotation)>,
    permutations: &'a Vec<permutation::Argument>,
    lookups: &'a Vec<lookup::Argument<F>>,
}

struct PinnedGates<'a, F: Field>(&'a Vec<(&'static str, Expression<F>)>);

impl<'a, F: Field> std::fmt::Debug for PinnedGates<'a, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_list()
            .entries(self.0.iter().map(|(_, expr)| expr))
            .finish()
    }
}

impl<F: Field> Default for ConstraintSystem<F> {
    fn default() -> ConstraintSystem<F> {
        ConstraintSystem {
            num_fixed_columns: 0,
            num_advice_columns: 0,
            num_instance_columns: 0,
            gates: vec![],
            fixed_queries: Vec::new(),
            advice_queries: Vec::new(),
            instance_queries: Vec::new(),
            permutations: Vec::new(),
            lookups: Vec::new(),
        }
    }
}

impl<F: Field> ConstraintSystem<F> {
    /// Obtain a pinned version of this constraint system; a structure with the
    /// minimal parameters needed to determine the rest of the constraint
    /// system.
    pub fn pinned(&self) -> PinnedConstraintSystem<'_, F> {
        PinnedConstraintSystem {
            num_fixed_columns: &self.num_fixed_columns,
            num_advice_columns: &self.num_advice_columns,
            num_instance_columns: &self.num_instance_columns,
            gates: PinnedGates(&self.gates),
            fixed_queries: &self.fixed_queries,
            advice_queries: &self.advice_queries,
            instance_queries: &self.instance_queries,
            permutations: &self.permutations,
            lookups: &self.lookups,
        }
    }

    /// Add a permutation argument for some advice columns
    pub fn permutation(&mut self, columns: &[Column<Any>]) -> usize {
        let index = self.permutations.len();

        for column in columns {
            self.query_any_index(*column, Rotation::cur());
        }
        self.permutations
            .push(permutation::Argument::new(columns.to_vec()));

        index
    }

    /// Add a lookup argument for some input expressions and table expressions.
    /// The function will panic if the number of input expressions and table
    /// expressions are not the same.
    pub fn lookup(
        &mut self,
        input_expressions: &[Expression<F>],
        table_expressions: &[Expression<F>],
    ) -> usize {
        assert_eq!(input_expressions.len(), table_expressions.len());

        let index = self.lookups.len();

        self.lookups
            .push(lookup::Argument::new(input_expressions, table_expressions));

        index
    }

    fn query_fixed_index(&mut self, column: Column<Fixed>, at: Rotation) -> usize {
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
    pub fn query_fixed(&mut self, column: Column<Fixed>, at: Rotation) -> Expression<F> {
        Expression::Fixed(self.query_fixed_index(column, at))
    }

    pub(crate) fn query_advice_index(&mut self, column: Column<Advice>, at: Rotation) -> usize {
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
    pub fn query_advice(&mut self, column: Column<Advice>, at: Rotation) -> Expression<F> {
        Expression::Advice(self.query_advice_index(column, at))
    }

    fn query_instance_index(&mut self, column: Column<Instance>, at: Rotation) -> usize {
        // Return existing query, if it exists
        for (index, instance_query) in self.instance_queries.iter().enumerate() {
            if instance_query == &(column, at) {
                return index;
            }
        }

        // Make a new query
        let index = self.instance_queries.len();
        self.instance_queries.push((column, at));

        index
    }

    /// Query an instance column at a relative position
    pub fn query_instance(&mut self, column: Column<Instance>, at: Rotation) -> Expression<F> {
        Expression::Instance(self.query_instance_index(column, at))
    }

    fn query_any_index(&mut self, column: Column<Any>, at: Rotation) -> usize {
        match column.column_type() {
            Any::Advice => self.query_advice_index(Column::<Advice>::try_from(column).unwrap(), at),
            Any::Fixed => self.query_fixed_index(Column::<Fixed>::try_from(column).unwrap(), at),
            Any::Instance => {
                self.query_instance_index(Column::<Instance>::try_from(column).unwrap(), at)
            }
        }
    }

    /// Query an Any column at a relative position
    pub fn query_any(&mut self, column: Column<Any>, at: Rotation) -> Expression<F> {
        match column.column_type() {
            Any::Advice => Expression::Advice(
                self.query_advice_index(Column::<Advice>::try_from(column).unwrap(), at),
            ),
            Any::Fixed => Expression::Fixed(
                self.query_fixed_index(Column::<Fixed>::try_from(column).unwrap(), at),
            ),
            Any::Instance => Expression::Instance(
                self.query_instance_index(Column::<Instance>::try_from(column).unwrap(), at),
            ),
        }
    }

    pub(crate) fn get_advice_query_index(&self, column: Column<Advice>, at: Rotation) -> usize {
        for (index, advice_query) in self.advice_queries.iter().enumerate() {
            if advice_query == &(column, at) {
                return index;
            }
        }

        panic!("get_advice_query_index called for non-existent query");
    }

    pub(crate) fn get_fixed_query_index(&self, column: Column<Fixed>, at: Rotation) -> usize {
        for (index, fixed_query) in self.fixed_queries.iter().enumerate() {
            if fixed_query == &(column, at) {
                return index;
            }
        }

        panic!("get_fixed_query_index called for non-existent query");
    }

    pub(crate) fn get_instance_query_index(&self, column: Column<Instance>, at: Rotation) -> usize {
        for (index, instance_query) in self.instance_queries.iter().enumerate() {
            if instance_query == &(column, at) {
                return index;
            }
        }

        panic!("get_instance_query_index called for non-existent query");
    }

    pub(crate) fn get_any_query_index(&self, column: Column<Any>, at: Rotation) -> usize {
        match column.column_type() {
            Any::Advice => {
                self.get_advice_query_index(Column::<Advice>::try_from(column).unwrap(), at)
            }
            Any::Fixed => {
                self.get_fixed_query_index(Column::<Fixed>::try_from(column).unwrap(), at)
            }
            Any::Instance => {
                self.get_instance_query_index(Column::<Instance>::try_from(column).unwrap(), at)
            }
        }
    }

    /// Create a new gate
    pub fn create_gate(&mut self, name: &'static str, f: impl FnOnce(&mut Self) -> Expression<F>) {
        let poly = f(self);
        self.gates.push((name, poly));
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

    /// Allocate a new instance column
    pub fn instance_column(&mut self) -> Column<Instance> {
        let tmp = Column {
            index: self.num_instance_columns,
            column_type: Instance,
        };
        self.num_instance_columns += 1;
        tmp
    }
}
