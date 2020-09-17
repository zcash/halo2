use core::cmp::max;
use core::ops::{Add, Mul};
use std::collections::BTreeMap;

use super::Error;
use crate::arithmetic::Field;

use crate::poly::Rotation;
/// This represents a wire which has a fixed (permanent) value
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FixedWire(pub usize);

/// This represents a wire which has a witness-specific value
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AdviceWire(pub usize);

/// This represents a wire which has an externally assigned value
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AuxWire(pub usize);

/// This trait allows a [`Circuit`] to direct some backend to assign a witness
/// for a constraint system.
pub trait Assignment<F: Field> {
    /// Assign an advice wire value (witness)
    fn assign_advice(
        &mut self,
        wire: AdviceWire,
        row: usize,
        to: impl FnOnce() -> Result<F, Error>,
    ) -> Result<(), Error>;

    /// Assign a fixed value
    fn assign_fixed(
        &mut self,
        wire: FixedWire,
        row: usize,
        to: impl FnOnce() -> Result<F, Error>,
    ) -> Result<(), Error>;

    /// Assign two advice wires to have the same value
    fn copy(
        &mut self,
        permutation: usize,
        left_wire: usize,
        left_row: usize,
        right_wire: usize,
        right_row: usize,
    ) -> Result<(), Error>;
}

/// This is a trait that circuits provide implementations for so that the
/// backend prover can ask the circuit to synthesize using some given
/// [`ConstraintSystem`] implementation.
pub trait Circuit<F: Field> {
    /// This is a configuration object that stores things like wires.
    type Config;

    /// The circuit is given an opportunity to describe the exact gate
    /// arrangement, wire arrangement, etc.
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config;

    /// Given the provided `cs`, synthesize the circuit. The concrete type of
    /// the caller will be different depending on the context, and they may or
    /// may not expect to have a witness present.
    fn synthesize(&self, cs: &mut impl Assignment<F>, config: Self::Config) -> Result<(), Error>;
}

/// Low-degree expression representing an identity that must hold over the committed wires.
#[derive(Clone, Debug)]
pub enum Expression<F> {
    /// This is a fixed wire queried at a certain relative location
    Fixed(usize),
    /// This is an advice (witness) wire queried at a certain relative location
    Advice(usize),
    /// This is an auxiliary (external) wire queried at a certain relative location
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
        fixed_wire: &impl Fn(usize) -> T,
        advice_wire: &impl Fn(usize) -> T,
        aux_wire: &impl Fn(usize) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
        scaled: &impl Fn(T, F) -> T,
    ) -> T {
        match self {
            Expression::Fixed(index) => fixed_wire(*index),
            Expression::Advice(index) => advice_wire(*index),
            Expression::Aux(index) => aux_wire(*index),
            Expression::Sum(a, b) => {
                let a = a.evaluate(fixed_wire, advice_wire, aux_wire, sum, product, scaled);
                let b = b.evaluate(fixed_wire, advice_wire, aux_wire, sum, product, scaled);
                sum(a, b)
            }
            Expression::Product(a, b) => {
                let a = a.evaluate(fixed_wire, advice_wire, aux_wire, sum, product, scaled);
                let b = b.evaluate(fixed_wire, advice_wire, aux_wire, sum, product, scaled);
                product(a, b)
            }
            Expression::Scaled(a, f) => {
                let a = a.evaluate(fixed_wire, advice_wire, aux_wire, sum, product, scaled);
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

/// This is a description of the circuit environment, such as the gate, wire and
/// permutation arrangements.
#[derive(Debug, Clone)]
pub struct ConstraintSystem<F> {
    pub(crate) num_fixed_wires: usize,
    pub(crate) num_advice_wires: usize,
    pub(crate) num_aux_wires: usize,
    pub(crate) gates: Vec<Expression<F>>,
    pub(crate) advice_queries: Vec<(AdviceWire, Rotation)>,
    pub(crate) aux_queries: Vec<(AuxWire, Rotation)>,
    pub(crate) fixed_queries: Vec<(FixedWire, Rotation)>,

    // Mapping from a witness vector rotation to the index in the point vector.
    pub(crate) rotations: BTreeMap<Rotation, PointIndex>,

    // Vector of permutation arguments, where each corresponds to a set of wires
    // that are involved in a permutation argument, as well as the corresponding
    // query index for each wire. As an example, we could have a permutation
    // argument between wires (A, B, C) which allows copy constraints to be
    // enforced between advice wire values in A, B and C, and another
    // permutation between wires (B, C, D) which allows the same with D instead
    // of A.
    pub(crate) permutations: Vec<Vec<(AdviceWire, usize)>>,
}

impl<F: Field> Default for ConstraintSystem<F> {
    fn default() -> ConstraintSystem<F> {
        let mut rotations = BTreeMap::new();
        rotations.insert(Rotation::default(), PointIndex(0));

        ConstraintSystem {
            num_fixed_wires: 0,
            num_advice_wires: 0,
            num_aux_wires: 0,
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
    /// Add a permutation argument for some advice wires
    pub fn permutation(&mut self, wires: &[AdviceWire]) -> usize {
        let index = self.permutations.len();
        if index == 0 {
            let at = Rotation(-1);
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }
        let wires = wires
            .iter()
            .map(|&wire| (wire, self.query_advice_index(wire, 0)))
            .collect();
        self.permutations.push(wires);

        index
    }

    fn query_fixed_index(&mut self, wire: FixedWire, at: i32) -> usize {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // Return existing query, if it exists
        for (index, fixed_query) in self.fixed_queries.iter().enumerate() {
            if fixed_query == &(wire, at) {
                return index;
            }
        }

        // Make a new query
        let index = self.fixed_queries.len();
        self.fixed_queries.push((wire, at));

        index
    }

    /// Query a fixed wire at a relative position
    pub fn query_fixed(&mut self, wire: FixedWire, at: i32) -> Expression<F> {
        Expression::Fixed(self.query_fixed_index(wire, at))
    }

    fn query_advice_index(&mut self, wire: AdviceWire, at: i32) -> usize {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // Return existing query, if it exists
        for (index, advice_query) in self.advice_queries.iter().enumerate() {
            if advice_query == &(wire, at) {
                return index;
            }
        }

        // Make a new query
        let index = self.advice_queries.len();
        self.advice_queries.push((wire, at));

        index
    }

    /// Query an advice wire at a relative position
    pub fn query_advice(&mut self, wire: AdviceWire, at: i32) -> Expression<F> {
        Expression::Advice(self.query_advice_index(wire, at))
    }

    fn query_aux_index(&mut self, wire: AuxWire, at: i32) -> usize {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // Return existing query, if it exists
        for (index, aux_query) in self.aux_queries.iter().enumerate() {
            if aux_query == &(wire, at) {
                return index;
            }
        }

        // Make a new query
        let index = self.aux_queries.len();
        self.aux_queries.push((wire, at));

        index
    }

    /// Query an auxiliary wire at a relative position
    pub fn query_aux(&mut self, wire: AuxWire, at: i32) -> Expression<F> {
        Expression::Aux(self.query_aux_index(wire, at))
    }

    /// Create a new gate
    pub fn create_gate(&mut self, f: impl FnOnce(&mut Self) -> Expression<F>) {
        let poly = f(self);
        self.gates.push(poly);
    }

    /// Allocate a new fixed wire
    pub fn fixed_wire(&mut self) -> FixedWire {
        let tmp = FixedWire(self.num_fixed_wires);
        self.num_fixed_wires += 1;
        tmp
    }

    /// Allocate a new advice wire
    pub fn advice_wire(&mut self) -> AdviceWire {
        let tmp = AdviceWire(self.num_advice_wires);
        self.num_advice_wires += 1;
        tmp
    }

    /// Allocate a new auxiliary wire
    pub fn aux_wire(&mut self) -> AuxWire {
        let tmp = AuxWire(self.num_aux_wires);
        self.num_aux_wires += 1;
        tmp
    }
}
