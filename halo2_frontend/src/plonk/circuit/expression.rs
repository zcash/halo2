use crate::circuit::Region;
use crate::plonk::circuit::{Advice, ColumnType, Fixed, Instance, VirtualCells};
use crate::plonk::Error;
use core::cmp::max;
use core::ops::{Add, Mul};
use halo2_middleware::circuit::{Any, ChallengeMid, ColumnMid, ExpressionMid, QueryMid, VarMid};
use halo2_middleware::ff::Field;
use halo2_middleware::poly::Rotation;
use sealed::SealedPhase;
use std::fmt::Debug;
use std::iter::{Product, Sum};
use std::{
    convert::TryFrom,
    ops::{Neg, Sub},
};

/// A column with an index and type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Column<C: ColumnType> {
    pub index: usize,
    pub column_type: C,
}

impl From<Column<Any>> for ColumnMid {
    fn from(val: Column<Any>) -> Self {
        ColumnMid {
            index: val.index(),
            column_type: (*val.column_type()),
        }
    }
}

impl<C: ColumnType> Column<C> {
    pub fn new(index: usize, column_type: C) -> Self {
        Column { index, column_type }
    }

    /// Index of this column.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Type of this column.
    pub fn column_type(&self) -> &C {
        &self.column_type
    }

    /// Return expression from column at a relative position
    pub fn query_cell<F: Field>(&self, at: Rotation) -> Expression<F> {
        self.column_type.query_cell(self.index, at)
    }

    /// Return expression from column at the current row
    pub fn cur<F: Field>(&self) -> Expression<F> {
        self.query_cell(Rotation::cur())
    }

    /// Return expression from column at the next row
    pub fn next<F: Field>(&self) -> Expression<F> {
        self.query_cell(Rotation::next())
    }

    /// Return expression from column at the previous row
    pub fn prev<F: Field>(&self) -> Expression<F> {
        self.query_cell(Rotation::prev())
    }

    /// Return expression from column at the specified rotation
    pub fn rot<F: Field>(&self, rotation: i32) -> Expression<F> {
        self.query_cell(Rotation(rotation))
    }
}

impl<C: ColumnType> Ord for Column<C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // This ordering is consensus-critical! The layouters rely on deterministic column
        // orderings.
        match self.column_type.into().cmp(&other.column_type.into()) {
            // Indices are assigned within column types.
            std::cmp::Ordering::Equal => self.index.cmp(&other.index),
            order => order,
        }
    }
}

impl<C: ColumnType> PartialOrd for Column<C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<ColumnMid> for Column<Any> {
    fn from(column: ColumnMid) -> Column<Any> {
        Column {
            index: column.index,
            column_type: column.column_type,
        }
    }
}

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

pub mod sealed {
    /// Phase of advice column
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Phase(pub u8);

    impl Phase {
        pub fn prev(&self) -> Option<Phase> {
            self.0.checked_sub(1).map(Phase)
        }
    }

    impl SealedPhase for Phase {
        fn to_sealed(self) -> Phase {
            self
        }
    }

    /// Sealed trait to help keep `Phase` private.
    pub trait SealedPhase {
        fn to_sealed(self) -> Phase;
    }
}

/// Phase of advice column
pub trait Phase: SealedPhase {}

impl<P: SealedPhase> Phase for P {}

/// First phase
#[derive(Debug)]
pub struct FirstPhase;

impl SealedPhase for FirstPhase {
    fn to_sealed(self) -> sealed::Phase {
        sealed::Phase(0)
    }
}

/// Second phase
#[derive(Debug)]
pub struct SecondPhase;

impl SealedPhase for SecondPhase {
    fn to_sealed(self) -> sealed::Phase {
        sealed::Phase(1)
    }
}

/// Third phase
#[derive(Debug)]
pub struct ThirdPhase;

impl SealedPhase for ThirdPhase {
    fn to_sealed(self) -> sealed::Phase {
        sealed::Phase(2)
    }
}

/// A selector, representing a fixed boolean value per row of the circuit.
///
/// Selectors can be used to conditionally enable (portions of) gates:
/// ```
/// use halo2_middleware::poly::Rotation;
/// # use halo2curves::pasta::Fp;
/// # use halo2_frontend::plonk::ConstraintSystem;
///
/// # let mut meta = ConstraintSystem::<Fp>::default();
/// let a = meta.advice_column();
/// let b = meta.advice_column();
/// let s = meta.selector();
///
/// meta.create_gate("foo", |meta| {
///     let a = meta.query_advice(a, Rotation::prev());
///     let b = meta.query_advice(b, Rotation::cur());
///     let s = meta.query_selector(s);
///
///     // On rows where the selector is enabled, a is constrained to equal b.
///     // On rows where the selector is disabled, a and b can take any value.
///     vec![s * (a - b)]
/// });
/// ```
///
/// Selectors are disabled on all rows by default, and must be explicitly enabled on each
/// row when required:
/// ```
/// use halo2_frontend::circuit::{Chip, Layouter, Value};
/// use halo2_frontend::plonk::{Advice, Fixed, Error, Column, Selector};
/// use halo2_middleware::ff::Field;
///
/// struct Config {
///     a: Column<Advice>,
///     b: Column<Advice>,
///     s: Selector,
/// }
///
/// fn circuit_logic<F: Field, C: Chip<F>>(chip: C, mut layouter: impl Layouter<F>) -> Result<(), Error> {
///     let config = chip.config();
///     # let config: Config = todo!();
///     layouter.assign_region(|| "bar", |mut region| {
///         region.assign_advice(|| "a", config.a, 0, || Value::known(F::ONE))?;
///         region.assign_advice(|| "a", config.b, 1, || Value::known(F::ONE))?;
///         config.s.enable(&mut region, 1)
///     })?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Selector(pub usize, pub(crate) bool);

impl Selector {
    /// Enable this selector at the given offset within the given region.
    pub fn enable<F: Field>(&self, region: &mut Region<F>, offset: usize) -> Result<(), Error> {
        region.enable_selector(|| "", self, offset)
    }

    /// Is this selector "simple"? Simple selectors can only be multiplied
    /// by expressions that contain no other simple selectors.
    pub fn is_simple(&self) -> bool {
        self.1
    }

    /// Returns index of this selector
    pub fn index(&self) -> usize {
        self.0
    }

    /// Return expression from selector
    pub fn expr<F: Field>(&self) -> Expression<F> {
        Expression::Selector(*self)
    }
}

/// Query of fixed column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FixedQuery {
    /// Query index
    pub index: Option<usize>,
    /// Column index
    pub column_index: usize,
    /// Rotation of this query
    pub rotation: Rotation,
}

impl FixedQuery {
    /// Column index
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    /// Rotation of this query
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }
}

/// Query of advice column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AdviceQuery {
    /// Query index
    pub index: Option<usize>,
    /// Column index
    pub column_index: usize,
    /// Rotation of this query
    pub rotation: Rotation,
}

impl AdviceQuery {
    /// Column index
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    /// Rotation of this query
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }
}

/// Query of instance column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InstanceQuery {
    /// Query index
    pub index: Option<usize>,
    /// Column index
    pub column_index: usize,
    /// Rotation of this query
    pub rotation: Rotation,
}

impl InstanceQuery {
    /// Column index
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    /// Rotation of this query
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }
}

/// A fixed column of a lookup table.
///
/// A lookup table can be loaded into this column via [`Layouter::assign_table`]. Columns
/// can currently only contain a single table, but they may be used in multiple lookup
/// arguments via [`super::constraint_system::ConstraintSystem::lookup`].
///
/// Lookup table columns are always "encumbered" by the lookup arguments they are used in;
/// they cannot simultaneously be used as general fixed columns.
///
/// [`Layouter::assign_table`]: crate::circuit::Layouter::assign_table
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TableColumn {
    /// The fixed column that this table column is stored in.
    ///
    /// # Security
    ///
    /// This inner column MUST NOT be exposed in the public API, or else chip developers
    /// can load lookup tables into their circuits without default-value-filling the
    /// columns, which can cause soundness bugs.
    pub(super) inner: Column<Fixed>,
}

impl TableColumn {
    /// Returns inner column
    pub fn inner(&self) -> Column<Fixed> {
        self.inner
    }
}

/// A challenge squeezed from transcript after advice columns at the phase have been committed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Challenge {
    pub index: usize,
    pub(crate) phase: u8,
}

impl Challenge {
    /// Index of this challenge.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Phase of this challenge.
    pub fn phase(&self) -> u8 {
        self.phase
    }

    /// Return Expression
    pub fn expr<F: Field>(&self) -> Expression<F> {
        Expression::Challenge(*self)
    }
}

impl From<Challenge> for ChallengeMid {
    fn from(val: Challenge) -> Self {
        ChallengeMid {
            index: val.index,
            phase: val.phase,
        }
    }
}

impl From<ChallengeMid> for Challenge {
    fn from(c: ChallengeMid) -> Self {
        Self {
            index: c.index,
            phase: c.phase,
        }
    }
}

/// Low-degree expression representing an identity that must hold over the committed columns.
#[derive(Clone, PartialEq, Eq)]
pub enum Expression<F> {
    /// This is a constant polynomial
    Constant(F),
    /// This is a virtual selector
    Selector(Selector),
    /// This is a fixed column queried at a certain relative location
    Fixed(FixedQuery),
    /// This is an advice (witness) column queried at a certain relative location
    Advice(AdviceQuery),
    /// This is an instance (external) column queried at a certain relative location
    Instance(InstanceQuery),
    /// This is a challenge
    Challenge(Challenge),
    /// This is a negated polynomial
    Negated(Box<Expression<F>>),
    /// This is the sum of two polynomials
    Sum(Box<Expression<F>>, Box<Expression<F>>),
    /// This is the product of two polynomials
    Product(Box<Expression<F>>, Box<Expression<F>>),
    /// This is a scaled polynomial
    Scaled(Box<Expression<F>>, F),
}

impl<F> From<Expression<F>> for ExpressionMid<F> {
    fn from(val: Expression<F>) -> Self {
        match val {
            Expression::Constant(c) => ExpressionMid::Constant(c),
            Expression::Selector(_) => unreachable!(),
            Expression::Fixed(FixedQuery {
                column_index,
                rotation,
                ..
            }) => ExpressionMid::Var(VarMid::Query(QueryMid {
                column_index,
                column_type: Any::Fixed,
                rotation,
            })),
            Expression::Advice(AdviceQuery {
                column_index,
                rotation,
                ..
            }) => ExpressionMid::Var(VarMid::Query(QueryMid {
                column_index,
                column_type: Any::Advice,
                rotation,
            })),
            Expression::Instance(InstanceQuery {
                column_index,
                rotation,
                ..
            }) => ExpressionMid::Var(VarMid::Query(QueryMid {
                column_index,
                column_type: Any::Instance,
                rotation,
            })),
            Expression::Challenge(c) => ExpressionMid::Var(VarMid::Challenge(c.into())),
            Expression::Negated(e) => ExpressionMid::Negated(Box::new((*e).into())),
            Expression::Sum(lhs, rhs) => {
                ExpressionMid::Sum(Box::new((*lhs).into()), Box::new((*rhs).into()))
            }
            Expression::Product(lhs, rhs) => {
                ExpressionMid::Product(Box::new((*lhs).into()), Box::new((*rhs).into()))
            }
            Expression::Scaled(e, c) => ExpressionMid::Scaled(Box::new((*e).into()), c),
        }
    }
}

impl<F: Field> Expression<F> {
    /// Make side effects
    pub fn query_cells(&mut self, cells: &mut VirtualCells<'_, F>) {
        match self {
            Expression::Constant(_) => (),
            Expression::Selector(selector) => {
                if !cells.queried_selectors.contains(selector) {
                    cells.queried_selectors.push(*selector);
                }
            }
            Expression::Fixed(query) => {
                if query.index.is_none() {
                    let col = Column {
                        index: query.column_index,
                        column_type: Fixed,
                    };
                    cells.queried_cells.push((col, query.rotation).into());
                    query.index = Some(cells.meta.query_fixed_index(col, query.rotation));
                }
            }
            Expression::Advice(query) => {
                if query.index.is_none() {
                    let col = Column {
                        index: query.column_index,
                        column_type: Advice,
                    };
                    cells.queried_cells.push((col, query.rotation).into());
                    query.index = Some(cells.meta.query_advice_index(col, query.rotation));
                }
            }
            Expression::Instance(query) => {
                if query.index.is_none() {
                    let col = Column {
                        index: query.column_index,
                        column_type: Instance,
                    };
                    cells.queried_cells.push((col, query.rotation).into());
                    query.index = Some(cells.meta.query_instance_index(col, query.rotation));
                }
            }
            Expression::Challenge(_) => (),
            Expression::Negated(a) => a.query_cells(cells),
            Expression::Sum(a, b) => {
                a.query_cells(cells);
                b.query_cells(cells);
            }
            Expression::Product(a, b) => {
                a.query_cells(cells);
                b.query_cells(cells);
            }
            Expression::Scaled(a, _) => a.query_cells(cells),
        };
    }

    /// Evaluate the polynomial using the provided closures to perform the
    /// operations.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate<T>(
        &self,
        constant: &impl Fn(F) -> T,
        selector_column: &impl Fn(Selector) -> T,
        fixed_column: &impl Fn(FixedQuery) -> T,
        advice_column: &impl Fn(AdviceQuery) -> T,
        instance_column: &impl Fn(InstanceQuery) -> T,
        challenge: &impl Fn(Challenge) -> T,
        negated: &impl Fn(T) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
        scaled: &impl Fn(T, F) -> T,
    ) -> T {
        match self {
            Expression::Constant(scalar) => constant(*scalar),
            Expression::Selector(selector) => selector_column(*selector),
            Expression::Fixed(query) => fixed_column(*query),
            Expression::Advice(query) => advice_column(*query),
            Expression::Instance(query) => instance_column(*query),
            Expression::Challenge(value) => challenge(*value),
            Expression::Negated(a) => {
                let a = a.evaluate(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                );
                negated(a)
            }
            Expression::Sum(a, b) => {
                let a = a.evaluate(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                );
                let b = b.evaluate(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                );
                sum(a, b)
            }
            Expression::Product(a, b) => {
                let a = a.evaluate(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                );
                let b = b.evaluate(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                );
                product(a, b)
            }
            Expression::Scaled(a, f) => {
                let a = a.evaluate(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                );
                scaled(a, *f)
            }
        }
    }

    /// Evaluate the polynomial lazily using the provided closures to perform the
    /// operations.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_lazy<T: PartialEq>(
        &self,
        constant: &impl Fn(F) -> T,
        selector_column: &impl Fn(Selector) -> T,
        fixed_column: &impl Fn(FixedQuery) -> T,
        advice_column: &impl Fn(AdviceQuery) -> T,
        instance_column: &impl Fn(InstanceQuery) -> T,
        challenge: &impl Fn(Challenge) -> T,
        negated: &impl Fn(T) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
        scaled: &impl Fn(T, F) -> T,
        zero: &T,
    ) -> T {
        match self {
            Expression::Constant(scalar) => constant(*scalar),
            Expression::Selector(selector) => selector_column(*selector),
            Expression::Fixed(query) => fixed_column(*query),
            Expression::Advice(query) => advice_column(*query),
            Expression::Instance(query) => instance_column(*query),
            Expression::Challenge(value) => challenge(*value),
            Expression::Negated(a) => {
                let a = a.evaluate_lazy(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                    zero,
                );
                negated(a)
            }
            Expression::Sum(a, b) => {
                let a = a.evaluate_lazy(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                    zero,
                );
                let b = b.evaluate_lazy(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                    zero,
                );
                sum(a, b)
            }
            Expression::Product(a, b) => {
                let (a, b) = if a.complexity() <= b.complexity() {
                    (a, b)
                } else {
                    (b, a)
                };
                let a = a.evaluate_lazy(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                    zero,
                );

                if a == *zero {
                    a
                } else {
                    let b = b.evaluate_lazy(
                        constant,
                        selector_column,
                        fixed_column,
                        advice_column,
                        instance_column,
                        challenge,
                        negated,
                        sum,
                        product,
                        scaled,
                        zero,
                    );
                    product(a, b)
                }
            }
            Expression::Scaled(a, f) => {
                let a = a.evaluate_lazy(
                    constant,
                    selector_column,
                    fixed_column,
                    advice_column,
                    instance_column,
                    challenge,
                    negated,
                    sum,
                    product,
                    scaled,
                    zero,
                );
                scaled(a, *f)
            }
        }
    }

    fn write_identifier<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Expression::Constant(scalar) => write!(writer, "{scalar:?}"),
            Expression::Selector(selector) => write!(writer, "selector[{}]", selector.0),
            Expression::Fixed(query) => {
                write!(
                    writer,
                    "fixed[{}][{}]",
                    query.column_index, query.rotation.0
                )
            }
            Expression::Advice(query) => {
                write!(
                    writer,
                    "advice[{}][{}]",
                    query.column_index, query.rotation.0
                )
            }
            Expression::Instance(query) => {
                write!(
                    writer,
                    "instance[{}][{}]",
                    query.column_index, query.rotation.0
                )
            }
            Expression::Challenge(challenge) => {
                write!(writer, "challenge[{}]", challenge.index())
            }
            Expression::Negated(a) => {
                writer.write_all(b"(-")?;
                a.write_identifier(writer)?;
                writer.write_all(b")")
            }
            Expression::Sum(a, b) => {
                writer.write_all(b"(")?;
                a.write_identifier(writer)?;
                writer.write_all(b"+")?;
                b.write_identifier(writer)?;
                writer.write_all(b")")
            }
            Expression::Product(a, b) => {
                writer.write_all(b"(")?;
                a.write_identifier(writer)?;
                writer.write_all(b"*")?;
                b.write_identifier(writer)?;
                writer.write_all(b")")
            }
            Expression::Scaled(a, f) => {
                a.write_identifier(writer)?;
                write!(writer, "*{f:?}")
            }
        }
    }

    /// Identifier for this expression. Expressions with identical identifiers
    /// do the same calculation (but the expressions don't need to be exactly equal
    /// in how they are composed e.g. `1 + 2` and `2 + 1` can have the same identifier).
    pub fn identifier(&self) -> String {
        let mut cursor = std::io::Cursor::new(Vec::new());
        self.write_identifier(&mut cursor).unwrap();
        String::from_utf8(cursor.into_inner()).unwrap()
    }

    /// Compute the degree of this polynomial
    pub fn degree(&self) -> usize {
        match self {
            Expression::Constant(_) => 0,
            Expression::Selector(_) => 1,
            Expression::Fixed(_) => 1,
            Expression::Advice(_) => 1,
            Expression::Instance(_) => 1,
            Expression::Challenge(_) => 0,
            Expression::Negated(poly) => poly.degree(),
            Expression::Sum(a, b) => max(a.degree(), b.degree()),
            Expression::Product(a, b) => a.degree() + b.degree(),
            Expression::Scaled(poly, _) => poly.degree(),
        }
    }

    /// Approximate the computational complexity of this expression.
    pub fn complexity(&self) -> usize {
        match self {
            Expression::Constant(_) => 0,
            Expression::Selector(_) => 1,
            Expression::Fixed(_) => 1,
            Expression::Advice(_) => 1,
            Expression::Instance(_) => 1,
            Expression::Challenge(_) => 0,
            Expression::Negated(poly) => poly.complexity() + 5,
            Expression::Sum(a, b) => a.complexity() + b.complexity() + 15,
            Expression::Product(a, b) => a.complexity() + b.complexity() + 30,
            Expression::Scaled(poly, _) => poly.complexity() + 30,
        }
    }

    /// Square this expression.
    pub fn square(self) -> Self {
        self.clone() * self
    }

    /// Returns whether or not this expression contains a simple `Selector`.
    pub(super) fn contains_simple_selector(&self) -> bool {
        self.evaluate(
            &|_| false,
            &|selector| selector.is_simple(),
            &|_| false,
            &|_| false,
            &|_| false,
            &|_| false,
            &|a| a,
            &|a, b| a || b,
            &|a, b| a || b,
            &|a, _| a,
        )
    }

    // TODO: Where is this used?
    /// Extracts a simple selector from this gate, if present
    pub(super) fn extract_simple_selector(&self) -> Option<Selector> {
        let op = |a, b| match (a, b) {
            (Some(a), None) | (None, Some(a)) => Some(a),
            (Some(_), Some(_)) => panic!("two simple selectors cannot be in the same expression"),
            _ => None,
        };

        self.evaluate(
            &|_| None,
            &|selector| {
                if selector.is_simple() {
                    Some(selector)
                } else {
                    None
                }
            },
            &|_| None,
            &|_| None,
            &|_| None,
            &|_| None,
            &|a| a,
            &op,
            &op,
            &|a, _| a,
        )
    }
}

impl<F: std::fmt::Debug> std::fmt::Debug for Expression<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(scalar) => f.debug_tuple("Constant").field(scalar).finish(),
            Expression::Selector(selector) => f.debug_tuple("Selector").field(selector).finish(),
            // Skip enum variant and print query struct directly to maintain backwards compatibility.
            Expression::Fixed(query) => {
                let mut debug_struct = f.debug_struct("Fixed");
                match query.index {
                    None => debug_struct.field("query_index", &query.index),
                    Some(idx) => debug_struct.field("query_index", &idx),
                };
                debug_struct
                    .field("column_index", &query.column_index)
                    .field("rotation", &query.rotation)
                    .finish()
            }
            Expression::Advice(query) => {
                let mut debug_struct = f.debug_struct("Advice");
                match query.index {
                    None => debug_struct.field("query_index", &query.index),
                    Some(idx) => debug_struct.field("query_index", &idx),
                };
                debug_struct
                    .field("column_index", &query.column_index)
                    .field("rotation", &query.rotation);
                debug_struct.finish()
            }
            Expression::Instance(query) => {
                let mut debug_struct = f.debug_struct("Instance");
                match query.index {
                    None => debug_struct.field("query_index", &query.index),
                    Some(idx) => debug_struct.field("query_index", &idx),
                };
                debug_struct
                    .field("column_index", &query.column_index)
                    .field("rotation", &query.rotation)
                    .finish()
            }
            Expression::Challenge(challenge) => {
                f.debug_tuple("Challenge").field(challenge).finish()
            }
            Expression::Negated(poly) => f.debug_tuple("Negated").field(poly).finish(),
            Expression::Sum(a, b) => f.debug_tuple("Sum").field(a).field(b).finish(),
            Expression::Product(a, b) => f.debug_tuple("Product").field(a).field(b).finish(),
            Expression::Scaled(poly, scalar) => {
                f.debug_tuple("Scaled").field(poly).field(scalar).finish()
            }
        }
    }
}

impl<F: Field> Neg for Expression<F> {
    type Output = Expression<F>;
    fn neg(self) -> Self::Output {
        Expression::Negated(Box::new(self))
    }
}

impl<F: Field> Add for Expression<F> {
    type Output = Expression<F>;
    fn add(self, rhs: Expression<F>) -> Expression<F> {
        if self.contains_simple_selector() || rhs.contains_simple_selector() {
            panic!("attempted to use a simple selector in an addition");
        }
        Expression::Sum(Box::new(self), Box::new(rhs))
    }
}

impl<F: Field> Sub for Expression<F> {
    type Output = Expression<F>;
    fn sub(self, rhs: Expression<F>) -> Expression<F> {
        if self.contains_simple_selector() || rhs.contains_simple_selector() {
            panic!("attempted to use a simple selector in a subtraction");
        }
        Expression::Sum(Box::new(self), Box::new(-rhs))
    }
}

impl<F: Field> Mul for Expression<F> {
    type Output = Expression<F>;
    fn mul(self, rhs: Expression<F>) -> Expression<F> {
        if self.contains_simple_selector() && rhs.contains_simple_selector() {
            panic!("attempted to multiply two expressions containing simple selectors");
        }
        Expression::Product(Box::new(self), Box::new(rhs))
    }
}

impl<F: Field> Mul<F> for Expression<F> {
    type Output = Expression<F>;
    fn mul(self, rhs: F) -> Expression<F> {
        Expression::Scaled(Box::new(self), rhs)
    }
}

impl<F: Field> Sum<Self> for Expression<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x)
            .unwrap_or(Expression::Constant(F::ZERO))
    }
}

impl<F: Field> Product<Self> for Expression<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc * x)
            .unwrap_or(Expression::Constant(F::ONE))
    }
}

#[cfg(test)]
mod tests {
    use super::Expression;
    use halo2curves::bn256::Fr;

    #[test]
    fn iter_sum() {
        let exprs: Vec<Expression<Fr>> = vec![
            Expression::Constant(1.into()),
            Expression::Constant(2.into()),
            Expression::Constant(3.into()),
        ];
        let happened: Expression<Fr> = exprs.into_iter().sum();
        let expected: Expression<Fr> = Expression::Sum(
            Box::new(Expression::Sum(
                Box::new(Expression::Constant(1.into())),
                Box::new(Expression::Constant(2.into())),
            )),
            Box::new(Expression::Constant(3.into())),
        );

        assert_eq!(happened, expected);
    }

    #[test]
    fn iter_product() {
        let exprs: Vec<Expression<Fr>> = vec![
            Expression::Constant(1.into()),
            Expression::Constant(2.into()),
            Expression::Constant(3.into()),
        ];
        let happened: Expression<Fr> = exprs.into_iter().product();
        let expected: Expression<Fr> = Expression::Product(
            Box::new(Expression::Product(
                Box::new(Expression::Constant(1.into())),
                Box::new(Expression::Constant(2.into())),
            )),
            Box::new(Expression::Constant(3.into())),
        );

        assert_eq!(happened, expected);
    }
}
