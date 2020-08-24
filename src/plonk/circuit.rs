use core::cmp::max;
use core::ops::{Add, Mul};
use std::collections::HashMap;

use super::Error;
use crate::arithmetic::Field;

/// This represents a PLONK wire, which could be a fixed (selector) wire or an
/// advice wire.
#[derive(Copy, Clone, Debug)]
pub enum Wire {
    /// A wires
    A,
    /// B wires
    B,
    /// C wires
    C,
    /// D wires
    D,
}

/// This represents a wire which has a fixed (permanent) value
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FixedWire(pub usize);

/// This represents a wire which has a witness-specific value
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AdviceWire(pub usize);

/// Represents a pointer to a value in the constraint system.
#[derive(Clone, Debug)]
pub struct Variable(pub Wire, pub usize);

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

    /// Creates a gate.
    fn create_gate(
        &mut self,
        sa: F,
        sb: F,
        sc: F,
        sd: F,
        sm: F,
        f: impl Fn() -> Result<(F, F, F, F), Error>,
    ) -> Result<(Variable, Variable, Variable, Variable), Error>;

    /// a * b - c = 0
    fn multiply(
        &mut self,
        f: impl Fn() -> Result<(F, F, F), Error>,
    ) -> Result<(Variable, Variable, Variable, Variable), Error> {
        self.create_gate(F::zero(), F::zero(), F::one(), F::zero(), F::one(), || {
            let (a, b, c) = f()?;
            Ok((a, b, c, F::zero()))
        })
    }

    /// a + b - c = 0
    fn add(
        &mut self,
        f: impl Fn() -> Result<(F, F, F), Error>,
    ) -> Result<(Variable, Variable, Variable, Variable), Error> {
        self.create_gate(F::one(), F::one(), F::one(), F::zero(), F::zero(), || {
            let (a, b, c) = f()?;
            Ok((a, b, c, F::zero()))
        })
    }

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
    Fixed(FixedWire, isize),
    /// This is an advice (witness) wire queried at a certain relative location
    Advice(AdviceWire, isize),
    /// This is the sum of two polynomials
    Sum(Box<Polynomial<F>>, Box<Polynomial<F>>),
    /// This is the product of two polynomials
    Product(Box<Polynomial<F>>, Box<Polynomial<F>>),
    /// This is a scaled polynomial
    Scaled(Box<Polynomial<F>>, F),
}

impl<F: Field> Polynomial<F> {
    fn evaluate<T>(
        &self,
        fixed_wire: &impl Fn(FixedWire, isize) -> T,
        advice_wire: &impl Fn(AdviceWire, isize) -> T,
        sum: &impl Fn(T, T) -> T,
        product: &impl Fn(T, T) -> T,
        scaled: &impl Fn(T, F) -> T,
    ) -> T {
        match self {
            Polynomial::Fixed(a, location) => fixed_wire(*a, *location),
            Polynomial::Advice(a, location) => advice_wire(*a, *location),
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
}

impl<F: Field> Polynomial<F> {
    fn degree(&self) -> usize {
        match self {
            Polynomial::Fixed(_, _) => 1,
            Polynomial::Advice(_, _) => 1,
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

/// This is a description of the circuit environment, such as the gate, wire and
/// permutation arrangements.
#[derive(Debug, Clone)]
pub struct MetaCircuit<F> {
    pub(crate) num_fixed_wires: usize,
    pub(crate) num_advice_wires: usize,
    // permutations: Vec<Vec<Wire>>,
    gates: Vec<Polynomial<F>>,
    advice_queries: HashMap<(AdviceWire, isize), usize>,
    fixed_queries: HashMap<(FixedWire, isize), usize>,
    // num_queries: usize,
}

impl<F: Field> Default for MetaCircuit<F> {
    fn default() -> MetaCircuit<F> {
        MetaCircuit {
            num_fixed_wires: 0,
            num_advice_wires: 0,
            gates: vec![],
            fixed_queries: HashMap::new(),
            advice_queries: HashMap::new(),
        }
    }
}

impl<F: Field> MetaCircuit<F> {
    /// Query a fixed wire at a relative position
    pub fn query_fixed(&mut self, wire: FixedWire, at: isize) -> Polynomial<F> {
        let len = self.fixed_queries.len();
        self.fixed_queries.entry((wire, at)).or_insert_with(|| len);

        Polynomial::Fixed(wire, at)
    }

    /// Query an advice wire at a relative position
    pub fn query_advice(&mut self, wire: AdviceWire, at: isize) -> Polynomial<F> {
        let len = self.advice_queries.len();
        self.advice_queries.entry((wire, at)).or_insert_with(|| len);

        Polynomial::Advice(wire, at)
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
