use crate::poly::Rotation;
use crate::{lookup, metadata, permutation, shuffle};
use core::cmp::max;
use ff::Field;
use std::collections::HashMap;

/// Query of fixed column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FixedQueryMid {
    /// Column index
    pub column_index: usize,
    /// Rotation of this query
    pub rotation: Rotation,
}

/// Query of advice column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AdviceQueryMid {
    /// Column index
    pub column_index: usize,
    /// Rotation of this query
    pub rotation: Rotation,
    /// Phase of this advice column
    pub phase: u8,
}

/// Query of instance column at a certain relative location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InstanceQueryMid {
    /// Column index
    pub column_index: usize,
    /// Rotation of this query
    pub rotation: Rotation,
}

/// A challenge squeezed from transcript after advice columns at the phase have been committed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Challenge {
    pub index: usize,
    pub phase: u8,
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
}

/// Low-degree expression representing an identity that must hold over the committed columns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionMid<F> {
    /// This is a constant polynomial
    Constant(F),
    /// This is a fixed column queried at a certain relative location
    Fixed(FixedQueryMid),
    /// This is an advice (witness) column queried at a certain relative location
    Advice(AdviceQueryMid),
    /// This is an instance (external) column queried at a certain relative location
    Instance(InstanceQueryMid),
    /// This is a challenge
    Challenge(Challenge),
    /// This is a negated polynomial
    Negated(Box<ExpressionMid<F>>),
    /// This is the sum of two polynomials
    Sum(Box<ExpressionMid<F>>, Box<ExpressionMid<F>>),
    /// This is the product of two polynomials
    Product(Box<ExpressionMid<F>>, Box<ExpressionMid<F>>),
    /// This is a scaled polynomial
    Scaled(Box<ExpressionMid<F>>, F),
}

impl<F: Field> ExpressionMid<F> {
    /// Compute the degree of this polynomial
    pub fn degree(&self) -> usize {
        use ExpressionMid::*;
        match self {
            Constant(_) => 0,
            Fixed(_) => 1,
            Advice(_) => 1,
            Instance(_) => 1,
            Challenge(_) => 0,
            Negated(poly) => poly.degree(),
            Sum(a, b) => max(a.degree(), b.degree()),
            Product(a, b) => a.degree() + b.degree(),
            Scaled(poly, _) => poly.degree(),
        }
    }
}

/// A Gate contains a single polynomial identity with a name as metadata.
#[derive(Clone, Debug)]
pub struct GateV2Backend<F: Field> {
    pub name: String,
    pub poly: ExpressionMid<F>,
}

impl<F: Field> GateV2Backend<F> {
    /// Returns the gate name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the polynomial identity of this gate
    pub fn polynomial(&self) -> &ExpressionMid<F> {
        &self.poly
    }
}

/// This is a description of the circuit environment, such as the gate, column and
/// permutation arrangements.
#[derive(Debug, Clone)]
pub struct ConstraintSystemV2Backend<F: Field> {
    pub num_fixed_columns: usize,
    pub num_advice_columns: usize,
    pub num_instance_columns: usize,
    pub num_challenges: usize,

    /// Contains the index of each advice column that is left unblinded.
    pub unblinded_advice_columns: Vec<usize>,

    /// Contains the phase for each advice column. Should have same length as num_advice_columns.
    pub advice_column_phase: Vec<u8>,
    /// Contains the phase for each challenge. Should have same length as num_challenges.
    pub challenge_phase: Vec<u8>,

    pub gates: Vec<GateV2Backend<F>>,

    // Permutation argument for performing equality constraints
    pub permutation: permutation::ArgumentV2,

    // Vector of lookup arguments, where each corresponds to a sequence of
    // input expressions and a sequence of table expressions involved in the lookup.
    pub lookups: Vec<lookup::ArgumentV2<F>>,

    // Vector of shuffle arguments, where each corresponds to a sequence of
    // input expressions and a sequence of shuffle expressions involved in the shuffle.
    pub shuffles: Vec<shuffle::ArgumentV2<F>>,

    // List of indexes of Fixed columns which are associated to a circuit-general Column tied to their annotation.
    pub general_column_annotations: HashMap<metadata::Column, String>,
}

/// Data that needs to be preprocessed from a circuit
#[derive(Debug, Clone)]
pub struct PreprocessingV2<F: Field> {
    pub permutation: permutation::AssemblyMid,
    pub fixed: Vec<Vec<F>>,
}

/// This is a description of a low level Plonkish compiled circuit. Contains the Constraint System
/// as well as the fixed columns and copy constraints information.
#[derive(Debug, Clone)]
pub struct CompiledCircuitV2<F: Field> {
    pub preprocessing: PreprocessingV2<F>,
    pub cs: ConstraintSystemV2Backend<F>,
}

/// A column type
pub trait ColumnType:
    'static + Sized + Copy + std::fmt::Debug + PartialEq + Eq + Into<Any>
{
    /// Return expression from cell
    fn query_cell<F: Field>(&self, index: usize, at: Rotation) -> ExpressionMid<F>;
}

/// A column with an index and type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Column<C: ColumnType> {
    pub index: usize,
    pub column_type: C,
}

// TODO: Remove all these methods, and directly access the fields?
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
    pub fn query_cell<F: Field>(&self, at: Rotation) -> ExpressionMid<F> {
        self.column_type.query_cell(self.index, at)
    }

    /// Return expression from column at the current row
    pub fn cur<F: Field>(&self) -> ExpressionMid<F> {
        self.query_cell(Rotation::cur())
    }

    /// Return expression from column at the next row
    pub fn next<F: Field>(&self) -> ExpressionMid<F> {
        self.query_cell(Rotation::next())
    }

    /// Return expression from column at the previous row
    pub fn prev<F: Field>(&self) -> ExpressionMid<F> {
        self.query_cell(Rotation::prev())
    }

    /// Return expression from column at the specified rotation
    pub fn rot<F: Field>(&self, rotation: i32) -> ExpressionMid<F> {
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

/// An advice column
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Advice {
    pub phase: u8,
}

impl Default for Advice {
    fn default() -> Advice {
        Advice { phase: 0 }
    }
}

impl Advice {
    /// Returns `Advice` in given `Phase`
    pub fn new(phase: u8) -> Advice {
        Advice { phase }
    }

    /// Phase of this column
    pub fn phase(&self) -> u8 {
        self.phase
    }
}

impl std::fmt::Debug for Advice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Advice");
        // Only show advice's phase if it's not in first phase.
        if self.phase != 0 {
            debug_struct.field("phase", &self.phase);
        }
        debug_struct.finish()
    }
}

/// A fixed column
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Fixed;

/// An instance column
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Instance;

/// An enum over the Advice, Fixed, Instance structs
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Any {
    /// An Advice variant
    Advice(Advice),
    /// A Fixed variant
    Fixed,
    /// An Instance variant
    Instance,
}

impl Any {
    /// Returns Advice variant in `FirstPhase`
    pub fn advice() -> Any {
        Any::Advice(Advice::default())
    }

    /// Returns Advice variant in given `Phase`
    pub fn advice_in(phase: u8) -> Any {
        Any::Advice(Advice::new(phase))
    }
}

impl std::fmt::Debug for Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Any::Advice(advice) => {
                let mut debug_struct = f.debug_struct("Advice");
                // Only show advice's phase if it's not in first phase.
                if advice.phase != 0 {
                    debug_struct.field("phase", &advice.phase);
                }
                debug_struct.finish()
            }
            Any::Fixed => f.debug_struct("Fixed").finish(),
            Any::Instance => f.debug_struct("Instance").finish(),
        }
    }
}

impl Ord for Any {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // This ordering is consensus-critical! The layouters rely on deterministic column
        // orderings.
        match (self, other) {
            (Any::Instance, Any::Instance) | (Any::Fixed, Any::Fixed) => std::cmp::Ordering::Equal,
            (Any::Advice(lhs), Any::Advice(rhs)) => lhs.phase.cmp(&rhs.phase),
            // Across column types, sort Instance < Advice < Fixed.
            (Any::Instance, Any::Advice(_))
            | (Any::Advice(_), Any::Fixed)
            | (Any::Instance, Any::Fixed) => std::cmp::Ordering::Less,
            (Any::Fixed, Any::Instance)
            | (Any::Fixed, Any::Advice(_))
            | (Any::Advice(_), Any::Instance) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Any {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ColumnType for Advice {
    fn query_cell<F: Field>(&self, index: usize, at: Rotation) -> ExpressionMid<F> {
        ExpressionMid::Advice(AdviceQueryMid {
            column_index: index,
            rotation: at,
            phase: self.phase,
        })
    }
}
impl ColumnType for Fixed {
    fn query_cell<F: Field>(&self, index: usize, at: Rotation) -> ExpressionMid<F> {
        ExpressionMid::Fixed(FixedQueryMid {
            column_index: index,
            rotation: at,
        })
    }
}
impl ColumnType for Instance {
    fn query_cell<F: Field>(&self, index: usize, at: Rotation) -> ExpressionMid<F> {
        ExpressionMid::Instance(InstanceQueryMid {
            column_index: index,
            rotation: at,
        })
    }
}
impl ColumnType for Any {
    fn query_cell<F: Field>(&self, index: usize, at: Rotation) -> ExpressionMid<F> {
        match self {
            Any::Advice(Advice { phase }) => ExpressionMid::Advice(AdviceQueryMid {
                column_index: index,
                rotation: at,
                phase: *phase,
            }),
            Any::Fixed => ExpressionMid::Fixed(FixedQueryMid {
                column_index: index,
                rotation: at,
            }),
            Any::Instance => ExpressionMid::Instance(InstanceQueryMid {
                column_index: index,
                rotation: at,
            }),
        }
    }
}

impl From<Advice> for Any {
    fn from(advice: Advice) -> Any {
        Any::Advice(advice)
    }
}

impl From<Fixed> for Any {
    fn from(_: Fixed) -> Any {
        Any::Fixed
    }
}

impl From<Instance> for Any {
    fn from(_: Instance) -> Any {
        Any::Instance
    }
}

impl From<Column<Advice>> for Column<Any> {
    fn from(advice: Column<Advice>) -> Column<Any> {
        Column {
            index: advice.index(),
            column_type: Any::Advice(advice.column_type),
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
            Any::Advice(advice) => Ok(Column {
                index: any.index(),
                column_type: *advice,
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
