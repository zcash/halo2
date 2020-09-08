//! Field element variables
use super::{Error, StandardCS, Variable};
use crate::arithmetic::Field;

/// Represents a number allocated in the circuit.
#[derive(Debug)]
pub struct AllocatedNum<F: Field> {
    variable: Variable,
    value: Option<F>,
}

impl<F: Field> AllocatedNum<F> {
    /// Allocate a number as part of the witness
    pub fn alloc(
        mut cs: impl StandardCS<F>,
        f: impl FnOnce() -> Result<F, Error>,
    ) -> Result<Self, Error> {
        let mut value = None;
        let variable = cs.alloc(|| {
            value = Some(f()?);
            Ok(value.unwrap())
        })?;

        Ok(AllocatedNum { variable, value })
    }

    /// Adds this allocated number to another
    pub fn add(&self, mut cs: impl StandardCS<F>, other: &Self) -> Result<Self, Error> {
        let mut value = None;
        let (a, b, c) = cs.raw_add(|| {
            let left = self.value.ok_or(Error::SynthesisError)?;
            let right = other.value.ok_or(Error::SynthesisError)?;

            value = Some(left + right);

            Ok((left, right, value.unwrap()))
        })?;

        cs.copy(self.variable, a)?;
        cs.copy(other.variable, b)?;

        Ok(AllocatedNum { value, variable: c })
    }

    /// Multiplies this allocated number by another
    pub fn mul(&self, mut cs: impl StandardCS<F>, other: &Self) -> Result<Self, Error> {
        let mut value = None;
        let (a, b, c) = cs.raw_multiply(|| {
            let left = self.value.ok_or(Error::SynthesisError)?;
            let right = other.value.ok_or(Error::SynthesisError)?;

            value = Some(left * right);

            Ok((left, right, value.unwrap()))
        })?;

        cs.copy(self.variable, a)?;
        cs.copy(other.variable, b)?;

        Ok(AllocatedNum { value, variable: c })
    }
}
