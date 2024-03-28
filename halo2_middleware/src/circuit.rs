use crate::expression::{Expression, Variable};
use crate::poly::Rotation;
use crate::{lookup, permutation, shuffle};
use ff::Field;
use std::collections::HashMap;
use std::fmt;

/// A challenge squeezed from transcript after advice columns at the phase have been committed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ChallengeMid {
    pub index: usize,
    pub phase: u8,
}

impl ChallengeMid {
    /// Index of this challenge.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Phase of this challenge.
    pub fn phase(&self) -> u8 {
        self.phase
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryMid {
    /// Column index
    pub column_index: usize,
    /// The type of the column.
    pub column_type: Any,
    /// Rotation of this query
    pub rotation: Rotation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VarMid {
    /// This is a generic column query
    Query(QueryMid),
    /// This is a challenge
    Challenge(ChallengeMid),
}

impl Variable for VarMid {
    fn degree(&self) -> usize {
        match self {
            VarMid::Query(_) => 1,
            VarMid::Challenge(_) => 0,
        }
    }

    fn complexity(&self) -> usize {
        match self {
            VarMid::Query(_) => 1,
            VarMid::Challenge(_) => 0,
        }
    }

    fn write_identifier<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            VarMid::Query(query) => {
                match query.column_type {
                    Any::Fixed => write!(writer, "fixed")?,
                    Any::Advice => write!(writer, "advice")?,
                    Any::Instance => write!(writer, "instance")?,
                };
                write!(writer, "[{}][{}]", query.column_index, query.rotation.0)
            }
            VarMid::Challenge(challenge) => {
                write!(writer, "challenge[{}]", challenge.index())
            }
        }
    }
}

pub type ExpressionMid<F> = Expression<F, VarMid>;

/// A Gate contains a single polynomial identity with a name as metadata.
#[derive(Clone, Debug)]
pub struct Gate<F: Field, V: Variable> {
    pub name: String,
    pub poly: Expression<F, V>,
}

impl<F: Field, V: Variable> Gate<F, V> {
    /// Returns the gate name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the polynomial identity of this gate
    pub fn polynomial(&self) -> &Expression<F, V> {
        &self.poly
    }
}

pub type GateMid<F> = Gate<F, VarMid>;

/// This is a description of the circuit environment, such as the gate, column and
/// permutation arrangements.
#[derive(Debug, Clone)]
pub struct ConstraintSystemMid<F: Field> {
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

    pub gates: Vec<GateMid<F>>,

    // Permutation argument for performing equality constraints
    pub permutation: permutation::ArgumentMid,

    // Vector of lookup arguments, where each corresponds to a sequence of
    // input expressions and a sequence of table expressions involved in the lookup.
    pub lookups: Vec<lookup::ArgumentMid<F>>,

    // Vector of shuffle arguments, where each corresponds to a sequence of
    // input expressions and a sequence of shuffle expressions involved in the shuffle.
    pub shuffles: Vec<shuffle::ArgumentMid<F>>,

    // List of indexes of Fixed columns which are associated to a circuit-general Column tied to their annotation.
    pub general_column_annotations: HashMap<ColumnMid, String>,

    // The minimum degree required by the circuit, which can be set to a
    // larger amount than actually needed. This can be used, for example, to
    // force the permutation argument to involve more columns in the same set.
    pub minimum_degree: Option<usize>,
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
    pub cs: ConstraintSystemMid<F>,
}

/// An enum over the Advice, Fixed, Instance structs
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum Any {
    /// An Advice variant
    Advice,
    /// A Fixed variant
    Fixed,
    /// An Instance variant
    Instance,
}

impl std::fmt::Debug for Any {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Any::Advice => f.debug_struct("Advice").finish(),
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
            (Any::Instance, Any::Instance)
            | (Any::Fixed, Any::Fixed)
            | (Any::Advice, Any::Advice) => std::cmp::Ordering::Equal,
            // Across column types, sort Instance < Advice < Fixed.
            (Any::Instance, Any::Advice)
            | (Any::Advice, Any::Fixed)
            | (Any::Instance, Any::Fixed) => std::cmp::Ordering::Less,
            (Any::Fixed, Any::Instance)
            | (Any::Fixed, Any::Advice)
            | (Any::Advice, Any::Instance) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Any {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A column with an index and type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ColumnMid {
    /// The type of the column.
    pub column_type: Any,
    /// The index of the column.
    pub index: usize,
}

impl ColumnMid {
    pub fn new(column_type: Any, index: usize) -> Self {
        ColumnMid { column_type, index }
    }
}

impl fmt::Display for ColumnMid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let column_type = match self.column_type {
            Any::Advice => "a",
            Any::Fixed => "f",
            Any::Instance => "i",
        };
        write!(f, "{}{}", column_type, self.index)
    }
}

/// A cell identifies a position in the plonkish matrix identified by a column and a row offset.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Cell {
    pub column: ColumnMid,
    pub row: usize,
}
