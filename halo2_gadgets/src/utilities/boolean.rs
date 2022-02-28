//! Boolean utility.
use halo2_proofs::plonk::Assigned;
use pasta_curves::arithmetic::FieldExt;

/// A bit.
#[derive(Clone, Copy, Debug)]
pub struct Bit(bool);

impl<F: FieldExt> From<&Bit> for Assigned<F> {
    fn from(bit: &Bit) -> Self {
        Assigned::Trivial(F::from(bit.0))
    }
}

impl From<bool> for Bit {
    fn from(bit: bool) -> Self {
        Self(bit)
    }
}
