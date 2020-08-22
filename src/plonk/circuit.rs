use super::Error;

use crate::arithmetic::Field;

/// This represents a PLONK wire, which could be a fixed (selector) wire or an
/// advice wire.
#[derive(Debug)]
pub enum Wire {
    /// A wires
    A(usize),
    /// B wires
    B(usize),
    /// C wires
    C(usize),
    /// D wires
    D(usize),
}

/// This trait allows a [`Circuit`] to direct some backend to assign a witness
/// for a constraint system.
pub trait ConstraintSystem<F: Field> {
    /// Creates a gate.
    fn create_gate(
        &mut self,
        sa: F,
        sb: F,
        sc: F,
        sd: F,
        sm: F,
        f: impl Fn() -> Result<(F, F, F, F), Error>,
    ) -> Result<(Wire, Wire, Wire, Wire), Error>;

    /// a * b - c = 0
    fn multiply(
        &mut self,
        f: impl Fn() -> Result<(F, F, F), Error>,
    ) -> Result<(Wire, Wire, Wire, Wire), Error> {
        self.create_gate(F::zero(), F::zero(), F::one(), F::zero(), F::one(), || {
            let (a, b, c) = f()?;
            Ok((a, b, c, F::zero()))
        })
    }

    /// a + b - c = 0
    fn add(
        &mut self,
        f: impl Fn() -> Result<(F, F, F), Error>,
    ) -> Result<(Wire, Wire, Wire, Wire), Error> {
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
    /// Given the provided `cs`, synthesize the circuit. The concrete type of
    /// the caller will be different depending on the context, and they may or
    /// may not expect to have a witness present.
    fn synthesize(&self, cs: &mut impl ConstraintSystem<F>) -> Result<(), Error>;
}
