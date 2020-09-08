//! This module contains helpful gadgets and infrastructure for building
//! circuits.
use crate::{
    arithmetic::Field,
    plonk::{AdviceWire, ConstraintSystem, Error, FixedWire, MetaCircuit},
};

pub mod num;

/// Represents an advice wire somewhere
#[derive(Copy, Clone, Debug)]
pub struct Variable(AdviceWire, usize);

/// This is a backend for circuit synthesis which supports copy constraint
/// enforcement and raw addition / multiplication operations.
pub trait StandardCS<FF: Field> {
    /// stub
    fn raw_multiply<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
    where
        F: FnOnce() -> Result<(FF, FF, FF), Error>;
    /// stub
    fn raw_add<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
    where
        F: FnOnce() -> Result<(FF, FF, FF), Error>;
    /// stub
    fn copy(&mut self, a: Variable, b: Variable) -> Result<(), Error>;
    /// stub
    fn alloc<F>(&mut self, value: F) -> Result<Variable, Error>
    where
        F: FnOnce() -> Result<FF, Error>;
}

impl<'a, FF: Field, T: StandardCS<FF>> StandardCS<FF> for &'a mut T {
    fn raw_multiply<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
    where
        F: FnOnce() -> Result<(FF, FF, FF), Error>,
    {
        T::raw_multiply(self, f)
    }
    fn raw_add<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
    where
        F: FnOnce() -> Result<(FF, FF, FF), Error>,
    {
        T::raw_add(self, f)
    }
    fn copy(&mut self, a: Variable, b: Variable) -> Result<(), Error> {
        T::copy(self, a, b)
    }
    fn alloc<F>(&mut self, value: F) -> Result<Variable, Error>
    where
        F: FnOnce() -> Result<FF, Error>,
    {
        T::alloc(self, value)
    }
}

/// This is a standard circuit configuration with A, B, C wires and
/// addition/multiplication support.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct StandardConfig {
    pub a: AdviceWire,
    pub b: AdviceWire,
    pub c: AdviceWire,

    pub sa: FixedWire,
    pub sb: FixedWire,
    pub sc: FixedWire,
    pub sm: FixedWire,

    pub perm: usize,
}

impl StandardConfig {
    /// Initialize this circuit configuration
    pub fn new<F: Field>(meta: &mut MetaCircuit<F>) -> Self {
        let a = meta.advice_wire();
        let b = meta.advice_wire();
        let c = meta.advice_wire();

        let perm = meta.permutation(&[a, b, c]);

        let sm = meta.fixed_wire();
        let sa = meta.fixed_wire();
        let sb = meta.fixed_wire();
        let sc = meta.fixed_wire();

        meta.create_gate(|meta| {
            let a = meta.query_advice(a, 0);
            let b = meta.query_advice(b, 0);
            let c = meta.query_advice(c, 0);

            let sa = meta.query_fixed(sa, 0);
            let sb = meta.query_fixed(sb, 0);
            let sc = meta.query_fixed(sc, 0);
            let sm = meta.query_fixed(sm, 0);

            a.clone() * sa + b.clone() * sb + a * b * sm + (c * sc * (-F::one()))
        });

        StandardConfig {
            a,
            b,
            c,
            sa,
            sb,
            sc,
            sm,
            perm,
        }
    }
}

/// Standard constraint system synthesizer
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Standard<'a, F: Field, CS: ConstraintSystem<F>> {
    pub cs: &'a mut CS,
    pub config: StandardConfig,
    current_gate: usize,
    alloc_gate: Option<(AdviceWire, usize)>,
    _marker: std::marker::PhantomData<F>,
}

impl<'a, F: Field, CS: ConstraintSystem<F>> Standard<'a, F, CS> {
    /// Create a new synthesis backend for the standard configuration
    pub fn new(cs: &'a mut CS, config: StandardConfig) -> Self {
        Standard {
            cs,
            config,
            current_gate: 0,
            alloc_gate: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, F: Field, CS: ConstraintSystem<F>> StandardCS<F> for Standard<'a, F, CS> {
    fn raw_multiply<FF>(&mut self, f: FF) -> Result<(Variable, Variable, Variable), Error>
    where
        FF: FnOnce() -> Result<(F, F, F), Error>,
    {
        let index = self.current_gate;
        self.current_gate += 1;
        let mut value = None;
        self.cs.assign_advice(self.config.a, index, || {
            value = Some(f()?);
            Ok(value.ok_or(Error::SynthesisError)?.0)
        })?;
        self.cs.assign_advice(self.config.b, index, || {
            Ok(value.ok_or(Error::SynthesisError)?.1)
        })?;
        self.cs.assign_advice(self.config.c, index, || {
            Ok(value.ok_or(Error::SynthesisError)?.2)
        })?;

        self.cs
            .assign_fixed(self.config.sa, index, || Ok(F::zero()))?;
        self.cs
            .assign_fixed(self.config.sb, index, || Ok(F::zero()))?;
        self.cs
            .assign_fixed(self.config.sc, index, || Ok(F::one()))?;
        self.cs
            .assign_fixed(self.config.sm, index, || Ok(F::one()))?;
        Ok((
            Variable(self.config.a, index),
            Variable(self.config.b, index),
            Variable(self.config.c, index),
        ))
    }

    fn raw_add<FF>(&mut self, f: FF) -> Result<(Variable, Variable, Variable), Error>
    where
        FF: FnOnce() -> Result<(F, F, F), Error>,
    {
        let index = self.current_gate;
        self.current_gate += 1;
        let mut value = None;
        self.cs.assign_advice(self.config.a, index, || {
            value = Some(f()?);
            Ok(value.ok_or(Error::SynthesisError)?.0)
        })?;
        self.cs.assign_advice(self.config.b, index, || {
            Ok(value.ok_or(Error::SynthesisError)?.1)
        })?;
        self.cs.assign_advice(self.config.c, index, || {
            Ok(value.ok_or(Error::SynthesisError)?.2)
        })?;

        self.cs
            .assign_fixed(self.config.sa, index, || Ok(F::one()))?;
        self.cs
            .assign_fixed(self.config.sb, index, || Ok(F::one()))?;
        self.cs
            .assign_fixed(self.config.sc, index, || Ok(F::one()))?;
        self.cs
            .assign_fixed(self.config.sm, index, || Ok(F::zero()))?;
        Ok((
            Variable(self.config.a, index),
            Variable(self.config.b, index),
            Variable(self.config.c, index),
        ))
    }
    fn copy(&mut self, left: Variable, right: Variable) -> Result<(), Error> {
        let left_wire = match left.0 {
            x if x == self.config.a => 0,
            x if x == self.config.b => 1,
            x if x == self.config.c => 2,
            _ => unreachable!(),
        };
        let right_wire = match right.0 {
            x if x == self.config.a => 0,
            x if x == self.config.b => 1,
            x if x == self.config.c => 2,
            _ => unreachable!(),
        };

        self.cs
            .copy(self.config.perm, left_wire, left.1, right_wire, right.1)
    }
    fn alloc<FF>(&mut self, f: FF) -> Result<Variable, Error>
    where
        FF: FnOnce() -> Result<F, Error>,
    {
        let ret;
        let newval = match &self.alloc_gate {
            &None => {
                let row = self.current_gate;
                self.current_gate += 1;
                self.cs.assign_advice(self.config.a, row, f)?;
                ret = Variable(AdviceWire(0), row);
                (AdviceWire(1), row)
            }
            &Some((wire, row)) if wire.0 == 0 => {
                self.cs.assign_advice(self.config.a, row, f)?;
                ret = Variable(wire, row);
                (AdviceWire(1), row)
            }
            &Some((wire, row)) if wire.0 == 1 => {
                self.cs.assign_advice(self.config.b, row, f)?;
                ret = Variable(wire, row);
                (AdviceWire(2), row)
            }
            &Some((wire, row)) if wire.0 == 2 => {
                self.cs.assign_advice(self.config.c, row, f)?;
                ret = Variable(wire, row);
                let row = self.current_gate;
                self.current_gate += 1;
                (AdviceWire(0), row)
            }
            _ => panic!("unexpected wire"),
        };

        self.alloc_gate = Some(newval);

        Ok(ret)
    }
}
