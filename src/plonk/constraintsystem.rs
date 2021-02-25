use ff::Field;
use std::convert::TryFrom;

use super::{
    lookup, permutation, Advice, Any, Chip, Column, Error, Expression, Fixed, Instance, Region,
};
use crate::{arithmetic::FieldExt, poly::Rotation};

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

    /// Add a permutation argument for some columns
    pub fn permutation(&mut self, columns: &[Column<Any>]) -> Permutation {
        let index = self.permutations.len();

        for column in columns {
            self.query_any_index(*column, Rotation::cur());
        }
        self.permutations
            .push(permutation::Argument::new(columns.to_vec()));

        Permutation {
            index,
            mapping: columns.to_vec(),
        }
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

    /// Query a selector at a relative position.
    pub fn query_selector(&mut self, selector: Selector, at: Rotation) -> Expression<F> {
        Expression::Fixed(self.query_fixed_index(selector.0, at))
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

    /// Allocate a new selector.
    pub fn selector(&mut self) -> Selector {
        // TODO: Track selectors separately, and combine selectors where possible.
        // https://github.com/zcash/halo2/issues/116
        Selector(self.fixed_column())
    }

    /// Allocate a new fixed column
    pub fn fixed_column(&mut self) -> Column<Fixed> {
        let tmp = Column::new(self.num_fixed_columns, Fixed);
        self.num_fixed_columns += 1;
        tmp
    }

    /// Allocate a new advice column
    pub fn advice_column(&mut self) -> Column<Advice> {
        let tmp = Column::new(self.num_advice_columns, Advice);
        self.num_advice_columns += 1;
        tmp
    }

    /// Allocate a new instance column
    pub fn instance_column(&mut self) -> Column<Instance> {
        let tmp = Column::new(self.num_instance_columns, Instance);
        self.num_instance_columns += 1;
        tmp
    }

    /// Compute the degree of the constraint system (the maximum degree of all
    /// constraints).
    pub fn degree(&self) -> usize {
        // The permutation argument will serve alongside the gates, so must be
        // accounted for.
        let mut degree = self
            .permutations
            .iter()
            .map(|p| p.required_degree())
            .max()
            .unwrap_or(1);

        // The lookup argument also serves alongside the gates and must be accounted
        // for.
        degree = std::cmp::max(
            degree,
            self.lookups
                .iter()
                .map(|l| l.required_degree())
                .max()
                .unwrap_or(1),
        );

        // Account for each gate to ensure our quotient polynomial is the
        // correct degree and that our extended domain is the right size.
        for (_, poly) in self.gates.iter() {
            degree = std::cmp::max(degree, poly.degree());
        }

        degree
    }
}

/// A selector, representing a fixed boolean value per row of the circuit.
///
/// Selectors can be used to conditionally enable (portions of) gates:
/// ```
/// use halo2::poly::Rotation;
/// # use halo2::pasta::Fp;
/// # use halo2::plonk::ConstraintSystem;
///
/// # let mut meta = ConstraintSystem::<Fp>::default();
/// let a = meta.advice_column();
/// let b = meta.advice_column();
/// let s = meta.selector();
///
/// meta.create_gate("foo", |meta| {
///     let a = meta.query_advice(a, Rotation::prev());
///     let b = meta.query_advice(b, Rotation::cur());
///     let s = meta.query_selector(s, Rotation::cur());
///
///     // On rows where the selector is enabled, a is constrained to equal b.
///     // On rows where the selector is disabled, a and b can take any value.
///     s * (a - b)
/// });
/// ```
///
/// Selectors are disabled on all rows by default, and must be explicitly enabled on each
/// row when required:
/// ```
/// use halo2::plonk::{Chip, Layouter, Advice, Column, Error, Selector};
/// # use ff::Field;
/// # use halo2::plonk::Fixed;
///
/// struct Config {
///     a: Column<Advice>,
///     b: Column<Advice>,
///     s: Selector,
/// }
///
/// fn circuit_logic<C: Chip>(mut layouter: impl Layouter<C>) -> Result<(), Error> {
///     let config = layouter.config().clone();
///     # let config: Config = todo!();
///     layouter.assign_region(|| "bar", |mut region| {
///         region.assign_advice(|| "a", config.a, 0, || Ok(C::Field::one()))?;
///         region.assign_advice(|| "a", config.b, 1, || Ok(C::Field::one()))?;
///         config.s.enable(&mut region, 1)
///     })?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Selector(Column<Fixed>);

impl Selector {
    /// Enable this selector at the given offset within the given region.
    pub fn enable<C: Chip>(&self, region: &mut Region<C>, offset: usize) -> Result<(), Error> {
        // TODO: Ensure that the default for a selector's cells is always zero, if we
        // alter the proving system to change the global default.
        // TODO: Add Region::enable_selector method to allow the layouter to control the
        // selector's assignment.
        // https://github.com/zcash/halo2/issues/116
        region
            .assign_fixed(|| "", self.0, offset, || Ok(C::Field::one()))
            .map(|_| ())
    }
}

/// A permutation.
#[derive(Clone, Debug)]
pub struct Permutation {
    /// The index of this permutation.
    index: usize,
    /// The mapping between columns involved in this permutation.
    mapping: Vec<Column<Any>>,
}

impl Permutation {
    /// Configures a new permutation for the given columns.
    pub fn new<F: FieldExt>(meta: &mut ConstraintSystem<F>, columns: &[Column<Any>]) -> Self {
        meta.permutation(columns)
    }

    /// Returns index of permutation
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns mapping of permutation
    pub fn mapping(&self) -> &[Column<Any>] {
        &self.mapping
    }
}

/// Represents an index into a vector where each entry corresponds to a distinct
/// point that polynomials are queried at.
#[derive(Copy, Clone, Debug)]
pub(crate) struct PointIndex(pub usize);
