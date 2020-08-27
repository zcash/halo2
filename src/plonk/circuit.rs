use core::cmp::max;
use core::ops::{Add, Mul};
use std::collections::HashMap;

use super::Error;
use crate::arithmetic::Field;

use super::domain::Rotation;
/// This represents a wire which has a fixed (permanent) value
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FixedWire(pub usize);

/// This represents a wire which has a witness-specific value
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AdviceWire(pub usize);

/// This trait allows a [`Circuit`] to direct some backend to assign a witness
/// for a constraint system.
pub trait ConstraintSystem<F: Field> {
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

    // fn copy(&mut self, left: Wire, right: Wire);
}

/// This is a trait that circuits provide implementations for so that the
/// backend prover can ask the circuit to synthesize using some given
/// [`ConstraintSystem`] implementation.
pub trait Circuit<F: Field> {
    /// This is a configuration object that stores things like wires.
    type Config;

    /// The circuit is given an opportunity to describe the exact gate
    /// arrangement, wire arrangement, etc.
    fn configure(meta: &mut MetaCircuit<F>) -> Self::Config;

    /// Given the provided `cs`, synthesize the circuit. The concrete type of
    /// the caller will be different depending on the context, and they may or
    /// may not expect to have a witness present.
    fn synthesize(
        &self,
        cs: &mut impl ConstraintSystem<F>,
        config: Self::Config,
    ) -> Result<(), Error>;
}

/// Low-degree polynomial representing an identity that must hold over the committed wires.
#[derive(Clone, Debug)]
pub enum Polynomial<F> {
    /// This is a fixed wire queried at a certain relative location
    Fixed(usize),
    /// This is an advice (witness) wire queried at a certain relative location
    Advice(usize),
    /// This is the sum of two polynomials
    Sum(Box<Polynomial<F>>, Box<Polynomial<F>>),
    /// This is the product of two polynomials
    Product(Box<Polynomial<F>>, Box<Polynomial<F>>),
    /// This is a scaled polynomial
    Scaled(Box<Polynomial<F>>, F),
}

impl<F: Field> Polynomial<F> {
    /// Evaluate the polynomial using the provided closures to perform the
    /// operations.
    pub fn evaluate<T>(
        &self,
        fixed_wire: &impl Fn(usize) -> T,
        advice_wire: &impl Fn(usize) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
        scaled: &impl Fn(T, F) -> T,
    ) -> T {
        match self {
            Polynomial::Fixed(index) => fixed_wire(*index),
            Polynomial::Advice(index) => advice_wire(*index),
            Polynomial::Sum(a, b) => {
                let a = a.evaluate(fixed_wire, advice_wire, sum, product, scaled);
                let b = b.evaluate(fixed_wire, advice_wire, sum, product, scaled);
                sum(a, b)
            }
            Polynomial::Product(a, b) => {
                let a = a.evaluate(fixed_wire, advice_wire, sum, product, scaled);
                let b = b.evaluate(fixed_wire, advice_wire, sum, product, scaled);
                product(a, b)
            }
            Polynomial::Scaled(a, f) => {
                let a = a.evaluate(fixed_wire, advice_wire, sum, product, scaled);
                scaled(a, *f)
            }
        }
    }

    /// Compute the degree of this polynomial
    pub fn degree(&self) -> usize {
        match self {
            Polynomial::Fixed(_) => 1,
            Polynomial::Advice(_) => 1,
            Polynomial::Sum(a, b) => max(a.degree(), b.degree()),
            Polynomial::Product(a, b) => a.degree() + b.degree(),
            Polynomial::Scaled(poly, _) => poly.degree(),
        }
    }
}

impl<F> Add for Polynomial<F> {
    type Output = Polynomial<F>;
    fn add(self, rhs: Polynomial<F>) -> Polynomial<F> {
        Polynomial::Sum(Box::new(self), Box::new(rhs))
    }
}

impl<F> Mul for Polynomial<F> {
    type Output = Polynomial<F>;
    fn mul(self, rhs: Polynomial<F>) -> Polynomial<F> {
        Polynomial::Product(Box::new(self), Box::new(rhs))
    }
}

impl<F> Mul<F> for Polynomial<F> {
    type Output = Polynomial<F>;
    fn mul(self, rhs: F) -> Polynomial<F> {
        Polynomial::Scaled(Box::new(self), rhs)
    }
}

/// Represents an index into a vector where each entry corresponds to a distinct
/// point that polynomials are queried at.
#[derive(Copy, Clone, Debug)]
pub struct PointIndex(pub usize);

/// This is a description of the circuit environment, such as the gate, wire and
/// permutation arrangements.
#[derive(Debug, Clone)]
pub struct MetaCircuit<F> {
    pub(crate) num_fixed_wires: usize,
    pub(crate) num_advice_wires: usize,
    // permutations: Vec<Vec<Wire>>,
    pub(crate) gates: Vec<Polynomial<F>>,
    pub(crate) advice_queries: Vec<(AdviceWire, Rotation)>,
    pub(crate) fixed_queries: Vec<(FixedWire, Rotation)>,

    // Mapping from a witness vector rotation to the index in the point vector.
    pub(crate) rotations: HashMap<Rotation, PointIndex>,
}

impl<F: Field> Default for MetaCircuit<F> {
    fn default() -> MetaCircuit<F> {
        let mut rotations = HashMap::new();
        rotations.insert(Rotation::default(), PointIndex(0));

        MetaCircuit {
            num_fixed_wires: 0,
            num_advice_wires: 0,
            gates: vec![],
            fixed_queries: Vec::new(),
            advice_queries: Vec::new(),
            rotations,
        }
    }
}

impl<F: Field> MetaCircuit<F> {
    /// Query a fixed wire at a relative position
    pub fn query_fixed(&mut self, wire: FixedWire, at: i32) -> Polynomial<F> {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // TODO: check for existing query so we don't make redundant queries
        let index = self.fixed_queries.len();
        self.fixed_queries.push((wire, at));

        Polynomial::Fixed(index)
    }

    /// Query an advice wire at a relative position
    pub fn query_advice(&mut self, wire: AdviceWire, at: i32) -> Polynomial<F> {
        let at = Rotation(at);
        {
            let len = self.rotations.len();
            self.rotations.entry(at).or_insert(PointIndex(len));
        }

        // TODO: check for existing query so we don't make redundant queries
        let index = self.advice_queries.len();
        self.advice_queries.push((wire, at));

        Polynomial::Advice(index)
    }

    /// Create a new gate
    pub fn create_gate(&mut self, f: impl FnOnce(&mut Self) -> Polynomial<F>) {
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
}
