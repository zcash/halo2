use super::compress_selectors;
use super::expression::sealed;
use crate::plonk::{
    lookup, permutation, shuffle, Advice, AdviceQuery, Challenge, Column, Expression, FirstPhase,
    Fixed, FixedQuery, Instance, InstanceQuery, Phase, Selector, TableColumn,
};
use core::cmp::max;
use halo2_middleware::circuit::{Any, ColumnMid, ConstraintSystemMid, GateMid};
use halo2_middleware::ff::Field;
use halo2_middleware::poly::Rotation;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Debug;

/// Represents an index into a vector where each entry corresponds to a distinct
/// point that polynomials are queried at.
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct PointIndex(pub usize);

/// A "virtual cell" is a PLONK cell that has been queried at a particular relative offset
/// within a custom gate.
#[derive(Clone, Debug)]
pub struct VirtualCell {
    pub column: Column<Any>,
    pub rotation: Rotation,
}

impl<Col: Into<Column<Any>>> From<(Col, Rotation)> for VirtualCell {
    fn from((column, rotation): (Col, Rotation)) -> Self {
        VirtualCell {
            column: column.into(),
            rotation,
        }
    }
}

/// An individual polynomial constraint.
///
/// These are returned by the closures passed to `ConstraintSystem::create_gate`.
#[derive(Debug)]
pub struct Constraint<F: Field> {
    name: String,
    poly: Expression<F>,
}

impl<F: Field> From<Expression<F>> for Constraint<F> {
    fn from(poly: Expression<F>) -> Self {
        Constraint {
            name: "".to_string(),
            poly,
        }
    }
}

impl<F: Field, S: AsRef<str>> From<(S, Expression<F>)> for Constraint<F> {
    fn from((name, poly): (S, Expression<F>)) -> Self {
        Constraint {
            name: name.as_ref().to_string(),
            poly,
        }
    }
}

impl<F: Field> From<Expression<F>> for Vec<Constraint<F>> {
    fn from(poly: Expression<F>) -> Self {
        vec![Constraint {
            name: "".to_string(),
            poly,
        }]
    }
}

/// A set of polynomial constraints with a common selector.
///
/// ```
/// use halo2_middleware::poly::Rotation;
/// use halo2curves::pasta::Fp;
/// # use halo2_frontend::plonk::{Constraints, Expression, ConstraintSystem};
///
/// # let mut meta = ConstraintSystem::<Fp>::default();
/// let a = meta.advice_column();
/// let b = meta.advice_column();
/// let c = meta.advice_column();
/// let s = meta.selector();
///
/// meta.create_gate("foo", |meta| {
///     let next = meta.query_advice(a, Rotation::next());
///     let a = meta.query_advice(a, Rotation::cur());
///     let b = meta.query_advice(b, Rotation::cur());
///     let c = meta.query_advice(c, Rotation::cur());
///     let s_ternary = meta.query_selector(s);
///
///     let one_minus_a = Expression::Constant(Fp::one()) - a.clone();
///
///     Constraints::with_selector(
///         s_ternary,
///         std::array::IntoIter::new([
///             ("a is boolean", a.clone() * one_minus_a.clone()),
///             ("next == a ? b : c", next - (a * b + one_minus_a * c)),
///         ]),
///     )
/// });
/// ```
///
/// Note that the use of `std::array::IntoIter::new` is only necessary if you need to
/// support Rust 1.51 or 1.52. If your minimum supported Rust version is 1.53 or greater,
/// you can pass an array directly.
#[derive(Debug)]
pub struct Constraints<F: Field, C: Into<Constraint<F>>, Iter: IntoIterator<Item = C>> {
    selector: Expression<F>,
    constraints: Iter,
}

impl<F: Field, C: Into<Constraint<F>>, Iter: IntoIterator<Item = C>> Constraints<F, C, Iter> {
    /// Constructs a set of constraints that are controlled by the given selector.
    ///
    /// Each constraint `c` in `iterator` will be converted into the constraint
    /// `selector * c`.
    pub fn with_selector(selector: Expression<F>, constraints: Iter) -> Self {
        Constraints {
            selector,
            constraints,
        }
    }
}

fn apply_selector_to_constraint<F: Field, C: Into<Constraint<F>>>(
    (selector, c): (Expression<F>, C),
) -> Constraint<F> {
    let constraint: Constraint<F> = c.into();
    Constraint {
        name: constraint.name,
        poly: selector * constraint.poly,
    }
}

type ApplySelectorToConstraint<F, C> = fn((Expression<F>, C)) -> Constraint<F>;
type ConstraintsIterator<F, C, I> = std::iter::Map<
    std::iter::Zip<std::iter::Repeat<Expression<F>>, I>,
    ApplySelectorToConstraint<F, C>,
>;

impl<F: Field, C: Into<Constraint<F>>, Iter: IntoIterator<Item = C>> IntoIterator
    for Constraints<F, C, Iter>
{
    type Item = Constraint<F>;
    type IntoIter = ConstraintsIterator<F, C, Iter::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::repeat(self.selector)
            .zip(self.constraints)
            .map(apply_selector_to_constraint)
    }
}

/// Gate
#[derive(Clone, Debug)]
pub struct Gate<F: Field> {
    pub(crate) name: String,
    pub(crate) constraint_names: Vec<String>,
    pub(crate) polys: Vec<Expression<F>>,
    /// We track queried selectors separately from other cells, so that we can use them to
    /// trigger debug checks on gates.
    pub(crate) queried_selectors: Vec<Selector>,
    pub(crate) queried_cells: Vec<VirtualCell>,
}

impl<F: Field> Gate<F> {
    /// Returns the gate name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the name of the constraint at index `constraint_index`.
    pub fn constraint_name(&self, constraint_index: usize) -> &str {
        self.constraint_names[constraint_index].as_str()
    }

    /// Returns constraints of this gate
    pub fn polynomials(&self) -> &[Expression<F>] {
        &self.polys
    }

    pub fn queried_selectors(&self) -> &[Selector] {
        &self.queried_selectors
    }

    pub fn queried_cells(&self) -> &[VirtualCell] {
        &self.queried_cells
    }
}

impl<F: Field> From<ConstraintSystem<F>> for ConstraintSystemMid<F> {
    fn from(cs: ConstraintSystem<F>) -> Self {
        ConstraintSystemMid {
            num_fixed_columns: cs.num_fixed_columns,
            num_advice_columns: cs.num_advice_columns,
            num_instance_columns: cs.num_instance_columns,
            num_challenges: cs.num_challenges,
            unblinded_advice_columns: cs.unblinded_advice_columns,
            advice_column_phase: cs.advice_column_phase.iter().map(|p| p.0).collect(),
            challenge_phase: cs.challenge_phase.iter().map(|p| p.0).collect(),
            gates: cs
                .gates
                .into_iter()
                .flat_map(|mut g| {
                    let constraint_names = std::mem::take(&mut g.constraint_names);
                    let gate_name = g.name.clone();
                    g.polys.into_iter().enumerate().map(move |(i, e)| {
                        let name = match constraint_names[i].as_str() {
                            "" => gate_name.clone(),
                            constraint_name => format!("{gate_name}:{constraint_name}"),
                        };
                        GateMid {
                            name,
                            poly: e.into(),
                        }
                    })
                })
                .collect(),
            permutation: halo2_middleware::permutation::ArgumentMid {
                columns: cs
                    .permutation
                    .columns
                    .into_iter()
                    .map(|c| c.into())
                    .collect(),
            },
            lookups: cs
                .lookups
                .into_iter()
                .map(|l| halo2_middleware::lookup::ArgumentMid {
                    name: l.name,
                    input_expressions: l.input_expressions.into_iter().map(|e| e.into()).collect(),
                    table_expressions: l.table_expressions.into_iter().map(|e| e.into()).collect(),
                })
                .collect(),
            shuffles: cs
                .shuffles
                .into_iter()
                .map(|s| halo2_middleware::shuffle::ArgumentMid {
                    name: s.name.clone(),
                    input_expressions: s.input_expressions.into_iter().map(|e| e.into()).collect(),
                    shuffle_expressions: s
                        .shuffle_expressions
                        .into_iter()
                        .map(|e| e.into())
                        .collect(),
                })
                .collect(),
            general_column_annotations: cs.general_column_annotations,
            minimum_degree: cs.minimum_degree,
        }
    }
}

/// This is a description of the circuit environment, such as the gate, column and
/// permutation arrangements.
#[derive(Debug, Clone)]
pub struct ConstraintSystem<F: Field> {
    pub(crate) num_fixed_columns: usize,
    pub(crate) num_advice_columns: usize,
    pub(crate) num_instance_columns: usize,
    pub(crate) num_selectors: usize,
    pub(crate) num_challenges: usize,

    /// Contains the index of each advice column that is left unblinded.
    pub(crate) unblinded_advice_columns: Vec<usize>,

    /// Contains the phase for each advice column. Should have same length as num_advice_columns.
    pub(crate) advice_column_phase: Vec<sealed::Phase>,
    /// Contains the phase for each challenge. Should have same length as num_challenges.
    pub(crate) challenge_phase: Vec<sealed::Phase>,

    /// This is a cached vector that maps virtual selectors to the concrete
    /// fixed column that they were compressed into. This is just used by dev
    /// tooling right now.
    pub(crate) selector_map: Vec<Column<Fixed>>,

    pub(crate) gates: Vec<Gate<F>>,
    pub(crate) advice_queries: Vec<(Column<Advice>, Rotation)>,
    // Contains an integer for each advice column
    // identifying how many distinct queries it has
    // so far; should be same length as num_advice_columns.
    pub(crate) num_advice_queries: Vec<usize>,
    pub(crate) instance_queries: Vec<(Column<Instance>, Rotation)>,
    pub(crate) fixed_queries: Vec<(Column<Fixed>, Rotation)>,

    // Permutation argument for performing equality constraints
    pub(crate) permutation: permutation::Argument,

    // Vector of lookup arguments, where each corresponds to a sequence of
    // input expressions and a sequence of table expressions involved in the lookup.
    pub(crate) lookups: Vec<lookup::Argument<F>>,

    // Vector of shuffle arguments, where each corresponds to a sequence of
    // input expressions and a sequence of shuffle expressions involved in the shuffle.
    pub(crate) shuffles: Vec<shuffle::Argument<F>>,

    // List of indexes of Fixed columns which are associated to a circuit-general Column tied to their annotation.
    pub(crate) general_column_annotations: HashMap<ColumnMid, String>,

    // Vector of fixed columns, which can be used to store constant values
    // that are copied into advice columns.
    pub(crate) constants: Vec<Column<Fixed>>,

    pub(crate) minimum_degree: Option<usize>,
}

impl<F: Field> Default for ConstraintSystem<F> {
    fn default() -> ConstraintSystem<F> {
        ConstraintSystem {
            num_fixed_columns: 0,
            num_advice_columns: 0,
            num_instance_columns: 0,
            num_selectors: 0,
            num_challenges: 0,
            unblinded_advice_columns: Vec::new(),
            advice_column_phase: Vec::new(),
            challenge_phase: Vec::new(),
            selector_map: vec![],
            gates: vec![],
            fixed_queries: Vec::new(),
            advice_queries: Vec::new(),
            num_advice_queries: Vec::new(),
            instance_queries: Vec::new(),
            permutation: permutation::Argument::default(),
            lookups: Vec::new(),
            shuffles: Vec::new(),
            general_column_annotations: HashMap::new(),
            constants: vec![],
            minimum_degree: None,
        }
    }
}

impl<F: Field> ConstraintSystem<F> {
    /// Enables this fixed column to be used for global constant assignments.
    ///
    /// # Side-effects
    ///
    /// The column will be equality-enabled.
    pub fn enable_constant(&mut self, column: Column<Fixed>) {
        if !self.constants.contains(&column) {
            self.constants.push(column);
            self.enable_equality(column);
        }
    }

    /// Enable the ability to enforce equality over cells in this column
    pub fn enable_equality<C: Into<Column<Any>>>(&mut self, column: C) {
        let column = column.into();
        self.query_any_index(column, Rotation::cur());
        self.permutation.add_column(column);
    }

    /// Add a lookup argument for some input expressions and table columns.
    ///
    /// `table_map` returns a map between input expressions and the table columns
    /// they need to match.
    pub fn lookup<S: AsRef<str>>(
        &mut self,
        name: S,
        table_map: impl FnOnce(&mut VirtualCells<'_, F>) -> Vec<(Expression<F>, TableColumn)>,
    ) -> usize {
        let mut cells = VirtualCells::new(self);
        let table_map = table_map(&mut cells)
            .into_iter()
            .map(|(mut input, table)| {
                if input.contains_simple_selector() {
                    panic!("expression containing simple selector supplied to lookup argument");
                }
                let mut table = cells.query_fixed(table.inner(), Rotation::cur());
                input.query_cells(&mut cells);
                table.query_cells(&mut cells);
                (input, table)
            })
            .collect();
        let index = self.lookups.len();

        self.lookups
            .push(lookup::Argument::new(name.as_ref(), table_map));

        index
    }

    /// Add a lookup argument for some input expressions and table expressions.
    ///
    /// `table_map` returns a map between input expressions and the table expressions
    /// they need to match.
    pub fn lookup_any<S: AsRef<str>>(
        &mut self,
        name: S,
        table_map: impl FnOnce(&mut VirtualCells<'_, F>) -> Vec<(Expression<F>, Expression<F>)>,
    ) -> usize {
        let mut cells = VirtualCells::new(self);
        let table_map = table_map(&mut cells)
            .into_iter()
            .map(|(mut input, mut table)| {
                if input.contains_simple_selector() {
                    panic!("expression containing simple selector supplied to lookup argument");
                }
                if table.contains_simple_selector() {
                    panic!("expression containing simple selector supplied to lookup argument");
                }
                input.query_cells(&mut cells);
                table.query_cells(&mut cells);
                (input, table)
            })
            .collect();
        let index = self.lookups.len();

        self.lookups
            .push(lookup::Argument::new(name.as_ref(), table_map));

        index
    }

    /// Add a shuffle argument for some input expressions and table expressions.
    pub fn shuffle<S: AsRef<str>>(
        &mut self,
        name: S,
        shuffle_map: impl FnOnce(&mut VirtualCells<'_, F>) -> Vec<(Expression<F>, Expression<F>)>,
    ) -> usize {
        let mut cells = VirtualCells::new(self);
        let shuffle_map = shuffle_map(&mut cells)
            .into_iter()
            .map(|(mut input, mut table)| {
                input.query_cells(&mut cells);
                table.query_cells(&mut cells);
                (input, table)
            })
            .collect();
        let index = self.shuffles.len();

        self.shuffles
            .push(shuffle::Argument::new(name.as_ref(), shuffle_map));

        index
    }

    pub(super) fn query_fixed_index(&mut self, column: Column<Fixed>, at: Rotation) -> usize {
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
        self.num_advice_queries[column.index] += 1;

        index
    }

    pub(super) fn query_instance_index(&mut self, column: Column<Instance>, at: Rotation) -> usize {
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

    fn query_any_index(&mut self, column: Column<Any>, at: Rotation) -> usize {
        match column.column_type() {
            Any::Advice => self.query_advice_index(Column::<Advice>::try_from(column).unwrap(), at),
            Any::Fixed => self.query_fixed_index(Column::<Fixed>::try_from(column).unwrap(), at),
            Any::Instance => {
                self.query_instance_index(Column::<Instance>::try_from(column).unwrap(), at)
            }
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

    pub fn get_any_query_index(&self, column: Column<Any>, at: Rotation) -> usize {
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

    /// Sets the minimum degree required by the circuit, which can be set to a
    /// larger amount than actually needed. This can be used, for example, to
    /// force the permutation argument to involve more columns in the same set.
    pub fn set_minimum_degree(&mut self, degree: usize) {
        self.minimum_degree = Some(degree);
    }

    /// Creates a new gate.
    ///
    /// # Panics
    ///
    /// A gate is required to contain polynomial constraints. This method will panic if
    /// `constraints` returns an empty iterator.
    pub fn create_gate<C: Into<Constraint<F>>, Iter: IntoIterator<Item = C>, S: AsRef<str>>(
        &mut self,
        name: S,
        constraints: impl FnOnce(&mut VirtualCells<'_, F>) -> Iter,
    ) {
        let mut cells = VirtualCells::new(self);
        let constraints = constraints(&mut cells);
        let (constraint_names, polys): (_, Vec<_>) = constraints
            .into_iter()
            .map(|c| c.into())
            .map(|mut c: Constraint<F>| {
                c.poly.query_cells(&mut cells);
                (c.name, c.poly)
            })
            .unzip();

        let queried_selectors = cells.queried_selectors;
        let queried_cells = cells.queried_cells;

        assert!(
            !polys.is_empty(),
            "Gates must contain at least one constraint."
        );

        self.gates.push(Gate {
            name: name.as_ref().to_string(),
            constraint_names,
            polys,
            queried_selectors,
            queried_cells,
        });
    }

    /// This will compress selectors together depending on their provided
    /// assignments. This `ConstraintSystem` will then be modified to add new
    /// fixed columns (representing the actual selectors) and will return the
    /// polynomials for those columns. Finally, an internal map is updated to
    /// find which fixed column corresponds with a given `Selector`.
    ///
    /// Do not call this twice. Yes, this should be a builder pattern instead.
    pub fn compress_selectors(mut self, selectors: Vec<Vec<bool>>) -> (Self, Vec<Vec<F>>) {
        // The number of provided selector assignments must be the number we
        // counted for this constraint system.
        assert_eq!(selectors.len(), self.num_selectors);

        // Compute the maximal degree of every selector. We only consider the
        // expressions in gates, as lookup arguments cannot support simple
        // selectors. Selectors that are complex or do not appear in any gates
        // will have degree zero.
        let mut degrees = vec![0; selectors.len()];
        for expr in self.gates.iter().flat_map(|gate| gate.polys.iter()) {
            if let Some(selector) = expr.extract_simple_selector() {
                degrees[selector.0] = max(degrees[selector.0], expr.degree());
            }
        }

        // We will not increase the degree of the constraint system, so we limit
        // ourselves to the largest existing degree constraint.
        let max_degree = self.degree();

        let mut new_columns = vec![];
        let (polys, selector_assignment) = compress_selectors::process(
            selectors
                .into_iter()
                .zip(degrees)
                .enumerate()
                .map(
                    |(i, (activations, max_degree))| compress_selectors::SelectorDescription {
                        selector: i,
                        activations,
                        max_degree,
                    },
                )
                .collect(),
            max_degree,
            || {
                let column = self.fixed_column();
                new_columns.push(column);
                Expression::Fixed(FixedQuery {
                    index: Some(self.query_fixed_index(column, Rotation::cur())),
                    column_index: column.index,
                    rotation: Rotation::cur(),
                })
            },
        );

        let mut selector_map = vec![None; selector_assignment.len()];
        let mut selector_replacements = vec![None; selector_assignment.len()];
        for assignment in selector_assignment {
            selector_replacements[assignment.selector] = Some(assignment.expression);
            selector_map[assignment.selector] = Some(new_columns[assignment.combination_index]);
        }

        self.selector_map = selector_map
            .into_iter()
            .map(|a| a.unwrap())
            .collect::<Vec<_>>();
        let selector_replacements = selector_replacements
            .into_iter()
            .map(|a| a.unwrap())
            .collect::<Vec<_>>();
        self.replace_selectors_with_fixed(&selector_replacements);

        (self, polys)
    }

    /// Does not combine selectors and directly replaces them everywhere with fixed columns.
    pub fn directly_convert_selectors_to_fixed(
        mut self,
        selectors: Vec<Vec<bool>>,
    ) -> (Self, Vec<Vec<F>>) {
        // The number of provided selector assignments must be the number we
        // counted for this constraint system.
        assert_eq!(selectors.len(), self.num_selectors);

        let (polys, selector_replacements): (Vec<_>, Vec<_>) = selectors
            .into_iter()
            .map(|selector| {
                let poly = selector
                    .iter()
                    .map(|b| if *b { F::ONE } else { F::ZERO })
                    .collect::<Vec<_>>();
                let column = self.fixed_column();
                let rotation = Rotation::cur();
                let expr = Expression::Fixed(FixedQuery {
                    index: Some(self.query_fixed_index(column, rotation)),
                    column_index: column.index,
                    rotation,
                });
                (poly, expr)
            })
            .unzip();

        self.replace_selectors_with_fixed(&selector_replacements);
        self.num_selectors = 0;

        (self, polys)
    }

    fn replace_selectors_with_fixed(&mut self, selector_replacements: &[Expression<F>]) {
        fn replace_selectors<F: Field>(
            expr: &mut Expression<F>,
            selector_replacements: &[Expression<F>],
            must_be_nonsimple: bool,
        ) {
            *expr = expr.evaluate(
                &|constant| Expression::Constant(constant),
                &|selector| {
                    if must_be_nonsimple {
                        // Simple selectors are prohibited from appearing in
                        // expressions in the lookup argument by
                        // `ConstraintSystem`.
                        assert!(!selector.is_simple());
                    }

                    selector_replacements[selector.0].clone()
                },
                &|query| Expression::Fixed(query),
                &|query| Expression::Advice(query),
                &|query| Expression::Instance(query),
                &|challenge| Expression::Challenge(challenge),
                &|a| -a,
                &|a, b| a + b,
                &|a, b| a * b,
                &|a, f| a * f,
            );
        }

        // Substitute selectors for the real fixed columns in all gates
        for expr in self.gates.iter_mut().flat_map(|gate| gate.polys.iter_mut()) {
            replace_selectors(expr, selector_replacements, false);
        }

        // Substitute non-simple selectors for the real fixed columns in all
        // lookup expressions
        for expr in self.lookups.iter_mut().flat_map(|lookup| {
            lookup
                .input_expressions
                .iter_mut()
                .chain(lookup.table_expressions.iter_mut())
        }) {
            replace_selectors(expr, selector_replacements, true);
        }

        for expr in self.shuffles.iter_mut().flat_map(|shuffle| {
            shuffle
                .input_expressions
                .iter_mut()
                .chain(shuffle.shuffle_expressions.iter_mut())
        }) {
            replace_selectors(expr, selector_replacements, true);
        }
    }

    /// Allocate a new (simple) selector. Simple selectors cannot be added to
    /// expressions nor multiplied by other expressions containing simple
    /// selectors. Also, simple selectors may not appear in lookup argument
    /// inputs.
    pub fn selector(&mut self) -> Selector {
        let index = self.num_selectors;
        self.num_selectors += 1;
        Selector(index, true)
    }

    /// Allocate a new complex selector that can appear anywhere
    /// within expressions.
    pub fn complex_selector(&mut self) -> Selector {
        let index = self.num_selectors;
        self.num_selectors += 1;
        Selector(index, false)
    }

    /// Allocates a new fixed column that can be used in a lookup table.
    pub fn lookup_table_column(&mut self) -> TableColumn {
        TableColumn {
            inner: self.fixed_column(),
        }
    }

    /// Annotate a Lookup column.
    pub fn annotate_lookup_column<A, AR>(&mut self, column: TableColumn, annotation: A)
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        // We don't care if the table has already an annotation. If it's the case we keep the new one.
        self.general_column_annotations.insert(
            ColumnMid {
                index: column.inner().index,
                column_type: halo2_middleware::circuit::Any::Fixed,
            },
            annotation().into(),
        );
    }

    /// Annotate an Instance column.
    pub fn annotate_lookup_any_column<A, AR, T>(&mut self, column: T, annotation: A)
    where
        A: Fn() -> AR,
        AR: Into<String>,
        T: Into<Column<Any>>,
    {
        let col_any = column.into();
        // We don't care if the table has already an annotation. If it's the case we keep the new one.
        self.general_column_annotations.insert(
            ColumnMid {
                column_type: col_any.column_type,
                index: col_any.index,
            },
            annotation().into(),
        );
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

    /// Allocate a new unblinded advice column at `FirstPhase`
    pub fn unblinded_advice_column(&mut self) -> Column<Advice> {
        self.unblinded_advice_column_in(FirstPhase)
    }

    /// Allocate a new advice column at `FirstPhase`
    pub fn advice_column(&mut self) -> Column<Advice> {
        self.advice_column_in(FirstPhase)
    }

    /// Allocate a new unblinded advice column in given phase. This allows for the generation of deterministic commitments to advice columns
    /// which can be used to split large circuits into smaller ones, whose proofs can then be "joined" together by their common witness commitments.
    pub fn unblinded_advice_column_in<P: Phase>(&mut self, phase: P) -> Column<Advice> {
        let phase = phase.to_sealed();
        if let Some(previous_phase) = phase.prev() {
            self.assert_phase_exists(
                previous_phase,
                format!("Column<Advice> in later phase {phase:?}").as_str(),
            );
        }

        let tmp = Column {
            index: self.num_advice_columns,
            column_type: Advice,
        };
        self.unblinded_advice_columns.push(tmp.index);
        self.num_advice_columns += 1;
        self.num_advice_queries.push(0);
        self.advice_column_phase.push(phase);
        tmp
    }

    /// Allocate a new advice column in given phase
    ///
    /// # Panics
    ///
    /// It panics if previous phase before the given one doesn't have advice column allocated.
    pub fn advice_column_in<P: Phase>(&mut self, phase: P) -> Column<Advice> {
        let phase = phase.to_sealed();
        if let Some(previous_phase) = phase.prev() {
            self.assert_phase_exists(
                previous_phase,
                format!("Column<Advice> in later phase {phase:?}").as_str(),
            );
        }

        let tmp = Column {
            index: self.num_advice_columns,
            column_type: Advice,
        };
        self.num_advice_columns += 1;
        self.num_advice_queries.push(0);
        self.advice_column_phase.push(phase);
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

    /// Requests a challenge that is usable after the given phase.
    ///
    /// # Panics
    ///
    /// It panics if the given phase doesn't have advice column allocated.
    pub fn challenge_usable_after<P: Phase>(&mut self, phase: P) -> Challenge {
        let phase = phase.to_sealed();
        self.assert_phase_exists(
            phase,
            format!("Challenge usable after phase {phase:?}").as_str(),
        );

        let tmp = Challenge {
            index: self.num_challenges,
            phase: phase.0,
        };
        self.num_challenges += 1;
        self.challenge_phase.push(phase);
        tmp
    }

    /// Helper funciotn to assert phase exists, to make sure phase-aware resources
    /// are allocated in order, and to avoid any phase to be skipped accidentally
    /// to cause unexpected issue in the future.
    fn assert_phase_exists(&self, phase: sealed::Phase, resource: &str) {
        self.advice_column_phase
            .iter()
            .find(|advice_column_phase| **advice_column_phase == phase)
            .unwrap_or_else(|| {
                panic!(
                    "No Column<Advice> is used in phase {phase:?} while allocating a new {resource:?}"
                )
            });
    }

    /// Returns the list of phases
    pub fn phases(&self) -> impl Iterator<Item = sealed::Phase> {
        let max_phase = self
            .advice_column_phase
            .iter()
            .max()
            .map(|phase| phase.0)
            .unwrap_or_default();
        (0..=max_phase).map(sealed::Phase)
    }

    /// Compute the degree of the constraint system (the maximum degree of all
    /// constraints).
    pub fn degree(&self) -> usize {
        // The permutation argument will serve alongside the gates, so must be
        // accounted for.
        let mut degree = self.permutation.required_degree();

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

        // The lookup argument also serves alongside the gates and must be accounted
        // for.
        degree = std::cmp::max(
            degree,
            self.shuffles
                .iter()
                .map(|l| l.required_degree())
                .max()
                .unwrap_or(1),
        );

        // Account for each gate to ensure our quotient polynomial is the
        // correct degree and that our extended domain is the right size.
        degree = std::cmp::max(
            degree,
            self.gates
                .iter()
                .flat_map(|gate| gate.polynomials().iter().map(|poly| poly.degree()))
                .max()
                .unwrap_or(0),
        );

        std::cmp::max(degree, self.minimum_degree.unwrap_or(1))
    }

    /// Compute the number of blinding factors necessary to perfectly blind
    /// each of the prover's witness polynomials.
    pub fn blinding_factors(&self) -> usize {
        // All of the prover's advice columns are evaluated at no more than
        let factors = *self.num_advice_queries.iter().max().unwrap_or(&1);
        // distinct points during gate checks.

        // - The permutation argument witness polynomials are evaluated at most 3 times.
        // - Each lookup argument has independent witness polynomials, and they are
        //   evaluated at most 2 times.
        let factors = std::cmp::max(3, factors);

        // Each polynomial is evaluated at most an additional time during
        // multiopen (at x_3 to produce q_evals):
        let factors = factors + 1;

        // h(x) is derived by the other evaluations so it does not reveal
        // anything; in fact it does not even appear in the proof.

        // h(x_3) is also not revealed; the verifier only learns a single
        // evaluation of a polynomial in x_1 which has h(x_3) and another random
        // polynomial evaluated at x_3 as coefficients -- this random polynomial
        // is "random_poly" in the vanishing argument.

        // Add an additional blinding factor as a slight defense against
        // off-by-one errors.
        factors + 1
    }

    /// Returns the minimum necessary rows that need to exist in order to
    /// account for e.g. blinding factors.
    pub fn minimum_rows(&self) -> usize {
        self.blinding_factors() // m blinding factors
            + 1 // for l_{-(m + 1)} (l_last)
            + 1 // for l_0 (just for extra breathing room for the permutation
                // argument, to essentially force a separation in the
                // permutation polynomial between the roles of l_last, l_0
                // and the interstitial values.)
            + 1 // for at least one row
    }

    /// Returns number of fixed columns
    pub fn num_fixed_columns(&self) -> usize {
        self.num_fixed_columns
    }

    /// Returns number of advice columns
    pub fn num_advice_columns(&self) -> usize {
        self.num_advice_columns
    }

    /// Returns number of instance columns
    pub fn num_instance_columns(&self) -> usize {
        self.num_instance_columns
    }

    /// Returns number of selectors
    pub fn num_selectors(&self) -> usize {
        self.num_selectors
    }

    /// Returns number of challenges
    pub fn num_challenges(&self) -> usize {
        self.num_challenges
    }

    /// Returns phase of advice columns
    pub fn advice_column_phase(&self) -> Vec<u8> {
        self.advice_column_phase
            .iter()
            .map(|phase| phase.0)
            .collect()
    }

    /// Returns phase of challenges
    pub fn challenge_phase(&self) -> Vec<u8> {
        self.challenge_phase.iter().map(|phase| phase.0).collect()
    }

    /// Returns gates
    pub fn gates(&self) -> &Vec<Gate<F>> {
        &self.gates
    }

    /// Returns general column annotations
    pub fn general_column_annotations(&self) -> &HashMap<ColumnMid, String> {
        &self.general_column_annotations
    }

    /// Returns advice queries
    pub fn advice_queries(&self) -> &Vec<(Column<Advice>, Rotation)> {
        &self.advice_queries
    }

    /// Returns instance queries
    pub fn instance_queries(&self) -> &Vec<(Column<Instance>, Rotation)> {
        &self.instance_queries
    }

    /// Returns fixed queries
    pub fn fixed_queries(&self) -> &Vec<(Column<Fixed>, Rotation)> {
        &self.fixed_queries
    }

    /// Returns permutation argument
    pub fn permutation(&self) -> &permutation::Argument {
        &self.permutation
    }

    /// Returns lookup arguments
    pub fn lookups(&self) -> &Vec<lookup::Argument<F>> {
        &self.lookups
    }

    /// Returns shuffle arguments
    pub fn shuffles(&self) -> &Vec<shuffle::Argument<F>> {
        &self.shuffles
    }

    /// Returns constants
    pub fn constants(&self) -> &Vec<Column<Fixed>> {
        &self.constants
    }
}

/// Exposes the "virtual cells" that can be queried while creating a custom gate or lookup
/// table.
#[derive(Debug)]
pub struct VirtualCells<'a, F: Field> {
    pub(super) meta: &'a mut ConstraintSystem<F>,
    pub(super) queried_selectors: Vec<Selector>,
    pub(super) queried_cells: Vec<VirtualCell>,
}

impl<'a, F: Field> VirtualCells<'a, F> {
    fn new(meta: &'a mut ConstraintSystem<F>) -> Self {
        VirtualCells {
            meta,
            queried_selectors: vec![],
            queried_cells: vec![],
        }
    }

    /// Query a selector at the current position.
    pub fn query_selector(&mut self, selector: Selector) -> Expression<F> {
        self.queried_selectors.push(selector);
        Expression::Selector(selector)
    }

    /// Query a fixed column at a relative position
    pub fn query_fixed(&mut self, column: Column<Fixed>, at: Rotation) -> Expression<F> {
        self.queried_cells.push((column, at).into());
        Expression::Fixed(FixedQuery {
            index: Some(self.meta.query_fixed_index(column, at)),
            column_index: column.index,
            rotation: at,
        })
    }

    /// Query an advice column at a relative position
    pub fn query_advice(&mut self, column: Column<Advice>, at: Rotation) -> Expression<F> {
        self.queried_cells.push((column, at).into());
        Expression::Advice(AdviceQuery {
            index: Some(self.meta.query_advice_index(column, at)),
            column_index: column.index,
            rotation: at,
        })
    }

    /// Query an instance column at a relative position
    pub fn query_instance(&mut self, column: Column<Instance>, at: Rotation) -> Expression<F> {
        self.queried_cells.push((column, at).into());
        Expression::Instance(InstanceQuery {
            index: Some(self.meta.query_instance_index(column, at)),
            column_index: column.index,
            rotation: at,
        })
    }

    /// Query an Any column at a relative position
    pub fn query_any<C: Into<Column<Any>>>(&mut self, column: C, at: Rotation) -> Expression<F> {
        let column = column.into();
        match column.column_type() {
            Any::Advice => self.query_advice(Column::<Advice>::try_from(column).unwrap(), at),
            Any::Fixed => self.query_fixed(Column::<Fixed>::try_from(column).unwrap(), at),
            Any::Instance => self.query_instance(Column::<Instance>::try_from(column).unwrap(), at),
        }
    }

    /// Query a challenge
    pub fn query_challenge(&mut self, challenge: Challenge) -> Expression<F> {
        Expression::Challenge(challenge)
    }
}
