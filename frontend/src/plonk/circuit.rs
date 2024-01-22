use super::{lookup, permutation, shuffle};
use crate::circuit::layouter::SyncDeps;
use crate::circuit::{Layouter, Region, Value};
use crate::error::Error;
use core::cmp::max;
use core::ops::{Add, Mul};
use halo2_middleware::circuit::{Advice, Any, Challenge, Column, Fixed, Instance};
use halo2_middleware::ff::Field;
use halo2_middleware::metadata;
use halo2_middleware::plonk::Assigned;
use halo2_middleware::poly::Rotation;
use sealed::SealedPhase;
use std::collections::HashMap;
use std::iter::{Product, Sum};
use std::{
    convert::TryFrom,
    ops::{Neg, Sub},
};

mod compress_selectors;

pub(crate) mod sealed {
    /// Phase of advice column
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Phase(pub(crate) u8);

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

impl SealedPhase for super::FirstPhase {
    fn to_sealed(self) -> sealed::Phase {
        sealed::Phase(0)
    }
}

/// Second phase
#[derive(Debug)]
pub struct SecondPhase;

impl SealedPhase for super::SecondPhase {
    fn to_sealed(self) -> sealed::Phase {
        sealed::Phase(1)
    }
}

/// Third phase
#[derive(Debug)]
pub struct ThirdPhase;

impl SealedPhase for super::ThirdPhase {
    fn to_sealed(self) -> sealed::Phase {
        sealed::Phase(2)
    }
}

/// A selector, representing a fixed boolean value per row of the circuit.
///
/// Selectors can be used to conditionally enable (portions of) gates:
/// ```
/// use halo2_proofs::poly::Rotation;
/// # use halo2curves::pasta::Fp;
/// # use halo2_proofs::plonk::ConstraintSystem;
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
/// use halo2_proofs::{
///     circuit::{Chip, Layouter, Value},
///     plonk::{Advice, Column, Error, Selector},
/// };
/// use ff::Field;
/// # use halo2_proofs::plonk::Fixed;
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
pub struct Selector(pub(crate) usize, bool);

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

/// A fixed column of a lookup table.
///
/// A lookup table can be loaded into this column via [`Layouter::assign_table`]. Columns
/// can currently only contain a single table, but they may be used in multiple lookup
/// arguments via [`ConstraintSystem::lookup`].
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
    inner: Column<Fixed>,
}

impl TableColumn {
    /// Returns inner column
    pub fn inner(&self) -> Column<Fixed> {
        self.inner
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

    /// Allows the developer to include an annotation for an specific column within a `Region`.
    ///
    /// This is usually useful for debugging circuit failures.
    fn annotate_column<A, AR>(&mut self, annotation: A, column: Column<Any>)
    where
        A: FnOnce() -> AR,
        AR: Into<String>;

    /// Exits the current region.
    ///
    /// Panics if we are not currently in a region (if `enter_region` was not called).
    ///
    /// Not intended for downstream consumption; use [`Layouter::assign_region`] instead.
    ///
    /// [`Layouter::assign_region`]: crate::circuit::Layouter#method.assign_region
    fn exit_region(&mut self);

    /// Enables a selector at the given row.
    fn enable_selector<A, AR>(
        &mut self,
        annotation: A,
        selector: &Selector,
        row: usize,
    ) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>;

    /// Queries the cell of an instance column at a particular absolute row.
    ///
    /// Returns the cell's value, if known.
    fn query_instance(&self, column: Column<Instance>, row: usize) -> Result<Value<F>, Error>;

    /// Assign an advice column value (witness)
    fn assign_advice<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>;

    /// Assign a fixed value
    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Fixed>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>;

    /// Assign two cells to have the same value
    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), Error>;

    /// Fills a fixed `column` starting from the given `row` with value `to`.
    fn fill_from_row(
        &mut self,
        column: Column<Fixed>,
        row: usize,
        to: Value<Assigned<F>>,
    ) -> Result<(), Error>;

    /// Queries the value of the given challenge.
    ///
    /// Returns `Value::unknown()` if the current synthesis phase is before the challenge can be queried.
    fn get_challenge(&self, challenge: Challenge) -> Value<F>;

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

/// A floor planning strategy for a circuit.
///
/// The floor planner is chip-agnostic and applies its strategy to the circuit it is used
/// within.
pub trait FloorPlanner {
    /// Given the provided `cs`, synthesize the given circuit.
    ///
    /// `constants` is the list of fixed columns that the layouter may use to assign
    /// global constant values. These columns will all have been equality-enabled.
    ///
    /// Internally, a floor planner will perform the following operations:
    /// - Instantiate a [`Layouter`] for this floor planner.
    /// - Perform any necessary setup or measurement tasks, which may involve one or more
    ///   calls to `Circuit::default().synthesize(config, &mut layouter)`.
    /// - Call `circuit.synthesize(config, &mut layouter)` exactly once.
    fn synthesize<F: Field, CS: Assignment<F> + SyncDeps, C: Circuit<F>>(
        cs: &mut CS,
        circuit: &C,
        config: C::Config,
        constants: Vec<Column<Fixed>>,
    ) -> Result<(), Error>;
}

/// This is a trait that circuits provide implementations for so that the
/// backend prover can ask the circuit to synthesize using some given
/// [`ConstraintSystem`] implementation.
pub trait Circuit<F: Field> {
    /// This is a configuration object that stores things like columns.
    type Config: Clone;
    /// The floor planner used for this circuit. This is an associated type of the
    /// `Circuit` trait because its behaviour is circuit-critical.
    type FloorPlanner: FloorPlanner;
    /// Optional circuit configuration parameters. Requires the `circuit-params` feature.
    #[cfg(feature = "circuit-params")]
    type Params: Default;

    /// Returns a copy of this circuit with no witness values (i.e. all witnesses set to
    /// `None`). For most circuits, this will be equal to `Self::default()`.
    fn without_witnesses(&self) -> Self;

    /// Returns a reference to the parameters that should be used to configure the circuit.
    /// Requires the `circuit-params` feature.
    #[cfg(feature = "circuit-params")]
    fn params(&self) -> Self::Params {
        Self::Params::default()
    }

    /// The circuit is given an opportunity to describe the exact gate
    /// arrangement, column arrangement, etc.  Takes a runtime parameter.  The default
    /// implementation calls `configure` ignoring the `_params` argument in order to easily support
    /// circuits that don't use configuration parameters.
    #[cfg(feature = "circuit-params")]
    fn configure_with_params(
        meta: &mut ConstraintSystem<F>,
        _params: Self::Params,
    ) -> Self::Config {
        Self::configure(meta)
    }

    /// The circuit is given an opportunity to describe the exact gate
    /// arrangement, column arrangement, etc.
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config;

    /// Given the provided `cs`, synthesize the circuit. The concrete type of
    /// the caller will be different depending on the context, and they may or
    /// may not expect to have a witness present.
    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error>;
}

/// Query of fixed column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FixedQuery {
    /// Query index
    pub(crate) index: Option<usize>,
    /// Column index
    pub(crate) column_index: usize,
    /// Rotation of this query
    pub(crate) rotation: Rotation,
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
    pub(crate) index: Option<usize>,
    /// Column index
    pub(crate) column_index: usize,
    /// Rotation of this query
    pub(crate) rotation: Rotation,
    /// Phase of this advice column
    pub(crate) phase: sealed::Phase,
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

    /// Phase of this advice column
    pub fn phase(&self) -> u8 {
        self.phase.0
    }
}

/// Query of instance column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InstanceQuery {
    /// Query index
    pub(crate) index: Option<usize>,
    /// Column index
    pub(crate) column_index: usize,
    /// Rotation of this query
    pub(crate) rotation: Rotation,
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
                        column_type: Advice { phase: query.phase },
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
    fn contains_simple_selector(&self) -> bool {
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

    /// Extracts a simple selector from this gate, if present
    fn extract_simple_selector(&self) -> Option<Selector> {
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
                // Only show advice's phase if it's not in first phase.
                if query.phase != FirstPhase.to_sealed() {
                    debug_struct.field("phase", &query.phase);
                }
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
/// use halo2_proofs::{plonk::{Constraints, Expression}, poly::Rotation};
/// use halo2curves::pasta::Fp;
/// # use halo2_proofs::plonk::ConstraintSystem;
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
    name: String,
    constraint_names: Vec<String>,
    polys: Vec<Expression<F>>,
    /// We track queried selectors separately from other cells, so that we can use them to
    /// trigger debug checks on gates.
    queried_selectors: Vec<Selector>,
    queried_cells: Vec<VirtualCell>,
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

    pub(crate) fn queried_selectors(&self) -> &[Selector] {
        &self.queried_selectors
    }

    pub(crate) fn queried_cells(&self) -> &[VirtualCell] {
        &self.queried_cells
    }
}

/// Exposes the "virtual cells" that can be queried while creating a custom gate or lookup
/// table.
#[derive(Debug)]
pub struct VirtualCells<'a, F: Field> {
    meta: &'a mut ConstraintSystem<F>,
    queried_selectors: Vec<Selector>,
    queried_cells: Vec<VirtualCell>,
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
            phase: column.column_type().phase,
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
            Any::Advice(_) => self.query_advice(Column::<Advice>::try_from(column).unwrap(), at),
            Any::Fixed => self.query_fixed(Column::<Fixed>::try_from(column).unwrap(), at),
            Any::Instance => self.query_instance(Column::<Instance>::try_from(column).unwrap(), at),
        }
    }

    /// Query a challenge
    pub fn query_challenge(&mut self, challenge: Challenge) -> Expression<F> {
        Expression::Challenge(challenge)
    }
}

/// A "virtual cell" is a PLONK cell that has been queried at a particular relative offset
/// within a custom gate.
#[derive(Clone, Debug)]
pub struct VirtualCell {
    pub(crate) column: Column<Any>,
    pub(crate) rotation: Rotation,
}

impl<Col: Into<Column<Any>>> From<(Col, Rotation)> for VirtualCell {
    fn from((column, rotation): (Col, Rotation)) -> Self {
        VirtualCell {
            column: column.into(),
            rotation,
        }
    }
}
