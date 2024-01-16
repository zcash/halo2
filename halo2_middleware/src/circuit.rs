use crate::poly::Rotation;
use crate::{lookup, permutation, shuffle};
use core::cmp::max;
use ff::Field;

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
    index: usize,
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
    name: String,
    poly: ExpressionMid<F>,
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
    pub(crate) num_fixed_columns: usize,
    pub(crate) num_advice_columns: usize,
    pub(crate) num_instance_columns: usize,
    pub(crate) num_challenges: usize,

    /// Contains the index of each advice column that is left unblinded.
    pub(crate) unblinded_advice_columns: Vec<usize>,

    /// Contains the phase for each advice column. Should have same length as num_advice_columns.
    pub(crate) advice_column_phase: Vec<u8>,
    /// Contains the phase for each challenge. Should have same length as num_challenges.
    pub(crate) challenge_phase: Vec<u8>,

    pub(crate) gates: Vec<GateV2Backend<F>>,

    // Permutation argument for performing equality constraints
    pub(crate) permutation: permutation::Argument,

    // Vector of lookup arguments, where each corresponds to a sequence of
    // input expressions and a sequence of table expressions involved in the lookup.
    pub(crate) lookups: Vec<lookup::ArgumentV2<F>>,

    // Vector of shuffle arguments, where each corresponds to a sequence of
    // input expressions and a sequence of shuffle expressions involved in the shuffle.
    pub(crate) shuffles: Vec<shuffle::ArgumentV2<F>>,

    // List of indexes of Fixed columns which are associated to a circuit-general Column tied to their annotation.
    pub(crate) general_column_annotations: HashMap<metadata::Column, String>,
}

/// Data that needs to be preprocessed from a circuit
#[derive(Debug, Clone)]
pub struct PreprocessingV2<F: Field> {
    // TODO(Edu): Can we replace this by a simpler structure?
    pub(crate) permutation: permutation::keygen::Assembly,
    pub(crate) fixed: Vec<Vec<F>>,
}

/// This is a description of a low level Plonkish compiled circuit. Contains the Constraint System
/// as well as the fixed columns and copy constraints information.
#[derive(Debug, Clone)]
pub struct CompiledCircuitV2<F: Field> {
    pub(crate) preprocessing: PreprocessingV2<F>,
    pub(crate) cs: ConstraintSystemV2Backend<F>,
}
