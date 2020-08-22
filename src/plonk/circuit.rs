use core::cmp::max;
use core::ops::{Add, Mul};

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
    /// Fixed wires
    Fixed(usize),
    /// Advice wires
    Advice(usize),
}

/// Represents a pointer to a value in the constraint system.
#[derive(Clone, Debug)]
pub struct Variable(pub Wire, pub usize);

/// This trait allows a [`Circuit`] to direct some backend to assign a witness
/// for a constraint system.
pub trait ConstraintSystem<F: Field> {
    /// Assign a wire value
    fn assign(&mut self, var: Variable, to: impl FnOnce() -> Result<F, Error>)
        -> Result<(), Error>;

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
    fn configure(meta: &mut MetaCircuit) -> Self::Config;

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
    /// This is a wire queried at a certain relative location
    Wire(Wire, isize),
    /// This is the sum of two polynomials
    Sum(Box<Polynomial<F>>, Box<Polynomial<F>>),
    /// This is the product of two polynomials
    Product(Box<Polynomial<F>>, Box<Polynomial<F>>),
    /// This is a scaled polynomial
    Scaled(Box<Polynomial<F>>, F),
}

impl<F: Field> Polynomial<F> {
    fn degree(&self) -> usize {
        match self {
            Polynomial::Wire(_, _) => 1,
            Polynomial::Sum(ref a, ref b) => max(a.degree(), b.degree()),
            Polynomial::Product(ref a, ref b) => a.degree() + b.degree(),
            Polynomial::Scaled(ref poly, _) => poly.degree(),
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
pub struct MetaCircuit {
    pub(crate) num_fixed_wires: usize,
    pub(crate) num_advice_wires: usize,
    // permutations: Vec<Vec<Wire>>,
    // gates: Vec<Polynomial>,
    // queries: HashSet<(Wire, usize)>,
    // num_queries: usize,
}

impl Default for MetaCircuit {
    fn default() -> MetaCircuit {
        MetaCircuit {
            num_fixed_wires: 0,
            num_advice_wires: 0,
        }
    }
}

impl MetaCircuit {
    /// Allocate a new fixed wire
    pub fn fixed_wire(&mut self) -> Wire {
        let tmp = Wire::Fixed(self.num_fixed_wires);
        self.num_fixed_wires += 1;
        tmp
    }
    /// Allocate a new advice wire
    pub fn advice_wire(&mut self) -> Wire {
        let tmp = Wire::Advice(self.num_advice_wires);
        self.num_advice_wires += 1;
        tmp
    }
}
