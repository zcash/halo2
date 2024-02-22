//! Traits and structs for implementing circuit components.

use halo2_common::plonk::{
    circuit::{Challenge, Column},
    permutation,
    sealed::{self, SealedPhase},
    Assigned, Assignment, Circuit, ConstraintSystem, Error, FirstPhase, FloorPlanner, SecondPhase,
    Selector, ThirdPhase,
};
use halo2_middleware::circuit::{Advice, Any, CompiledCircuitV2, Fixed, Instance, PreprocessingV2};
use halo2_middleware::ff::{BatchInvert, Field};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::RangeTo;

pub mod floor_planner;
mod table_layouter;

// Re-exports from common
pub use halo2_common::circuit::floor_planner::single_pass::SimpleFloorPlanner;
pub use halo2_common::circuit::{layouter, Layouter, Value};

/// Compile a circuit.  Runs configure and synthesize on the circuit in order to materialize the
/// circuit into its columns and the column configuration; as well as doing the fixed column and
/// copy constraints assignments.  The output of this function can then be used for the key
/// generation, and proof generation.
/// If `compress_selectors` is true, multiple selector columns may be multiplexed.
#[allow(clippy::type_complexity)]
pub fn compile_circuit<F: Field, ConcreteCircuit: Circuit<F>>(
    k: u32,
    circuit: &ConcreteCircuit,
    compress_selectors: bool,
) -> Result<
    (
        CompiledCircuitV2<F>,
        ConcreteCircuit::Config,
        ConstraintSystem<F>,
    ),
    Error,
> {
    let n = 2usize.pow(k);
    let mut cs = ConstraintSystem::default();
    #[cfg(feature = "circuit-params")]
    let config = ConcreteCircuit::configure_with_params(&mut cs, circuit.params());
    #[cfg(not(feature = "circuit-params"))]
    let config = ConcreteCircuit::configure(&mut cs);
    let cs = cs;

    if n < cs.minimum_rows() {
        return Err(Error::not_enough_rows_available(k));
    }

    let mut assembly = halo2_common::plonk::keygen::Assembly {
        k,
        fixed: vec![vec![F::ZERO.into(); n]; cs.num_fixed_columns],
        permutation: permutation::Assembly::new(n, &cs.permutation),
        selectors: vec![vec![false; n]; cs.num_selectors],
        usable_rows: 0..n - (cs.blinding_factors() + 1),
        _marker: std::marker::PhantomData,
    };

    // Synthesize the circuit to obtain URS
    ConcreteCircuit::FloorPlanner::synthesize(
        &mut assembly,
        circuit,
        config.clone(),
        cs.constants.clone(),
    )?;

    let mut fixed = batch_invert_assigned(assembly.fixed);
    let (cs, selector_polys) = if compress_selectors {
        cs.compress_selectors(assembly.selectors.clone())
    } else {
        // After this, the ConstraintSystem should not have any selectors: `verify` does not need them, and `keygen_pk` regenerates `cs` from scratch anyways.
        let selectors = std::mem::take(&mut assembly.selectors);
        cs.directly_convert_selectors_to_fixed(selectors)
    };
    fixed.extend(selector_polys);

    let preprocessing = PreprocessingV2 {
        permutation: halo2_middleware::permutation::AssemblyMid {
            copies: assembly.permutation.copies,
        },
        fixed,
    };

    Ok((
        CompiledCircuitV2 {
            cs: cs.clone().into(),
            preprocessing,
        },
        config,
        cs,
    ))
}

pub struct WitnessCollection<'a, F: Field> {
    pub k: u32,
    pub current_phase: sealed::Phase,
    pub advice: Vec<Vec<Assigned<F>>>,
    // pub unblinded_advice: HashSet<usize>,
    pub challenges: &'a HashMap<usize, F>,
    pub instances: &'a [&'a [F]],
    pub usable_rows: RangeTo<usize>,
    pub _marker: std::marker::PhantomData<F>,
}

impl<'a, F: Field> Assignment<F> for WitnessCollection<'a, F> {
    fn enter_region<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // Do nothing; we don't care about regions in this context.
    }

    fn exit_region(&mut self) {
        // Do nothing; we don't care about regions in this context.
    }

    fn enable_selector<A, AR>(&mut self, _: A, _: &Selector, _: usize) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // We only care about advice columns here

        Ok(())
    }

    fn annotate_column<A, AR>(&mut self, _annotation: A, _column: Column<Any>)
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // Do nothing
    }

    fn query_instance(&self, column: Column<Instance>, row: usize) -> Result<Value<F>, Error> {
        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        self.instances
            .get(column.index())
            .and_then(|column| column.get(row))
            .map(|v| Value::known(*v))
            .ok_or(Error::BoundsFailure)
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // Ignore assignment of advice column in different phase than current one.
        if self.current_phase.0 != column.column_type().phase {
            return Ok(());
        }

        if !self.usable_rows.contains(&row) {
            return Err(Error::not_enough_rows_available(self.k));
        }

        *self
            .advice
            .get_mut(column.index())
            .and_then(|v| v.get_mut(row))
            .ok_or(Error::BoundsFailure)? = to().into_field().assign()?;

        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _: A,
        _: Column<Fixed>,
        _: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // We only care about advice columns here

        Ok(())
    }

    fn copy(&mut self, _: Column<Any>, _: usize, _: Column<Any>, _: usize) -> Result<(), Error> {
        // We only care about advice columns here

        Ok(())
    }

    fn fill_from_row(
        &mut self,
        _: Column<Fixed>,
        _: usize,
        _: Value<Assigned<F>>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn get_challenge(&self, challenge: Challenge) -> Value<F> {
        self.challenges
            .get(&challenge.index())
            .cloned()
            .map(Value::known)
            .unwrap_or_else(Value::unknown)
    }

    fn push_namespace<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // Do nothing; we don't care about namespaces in this context.
    }

    fn pop_namespace(&mut self, _: Option<String>) {
        // Do nothing; we don't care about namespaces in this context.
    }
}

/// Witness calculator.  Frontend function
#[derive(Debug)]
pub struct WitnessCalculator<'a, F: Field, ConcreteCircuit: Circuit<F>> {
    k: u32,
    n: usize,
    unusable_rows_start: usize,
    circuit: &'a ConcreteCircuit,
    config: &'a ConcreteCircuit::Config,
    cs: &'a ConstraintSystem<F>,
    instances: &'a [&'a [F]],
    next_phase: u8,
}

impl<'a, F: Field, ConcreteCircuit: Circuit<F>> WitnessCalculator<'a, F, ConcreteCircuit> {
    /// Create a new WitnessCalculator
    pub fn new(
        k: u32,
        circuit: &'a ConcreteCircuit,
        config: &'a ConcreteCircuit::Config,
        cs: &'a ConstraintSystem<F>,
        instances: &'a [&'a [F]],
    ) -> Self {
        let n = 2usize.pow(k);
        let unusable_rows_start = n - (cs.blinding_factors() + 1);
        Self {
            k,
            n,
            unusable_rows_start,
            circuit,
            config,
            cs,
            instances,
            next_phase: 0,
        }
    }

    /// Calculate witness at phase
    pub fn calc(
        &mut self,
        phase: u8,
        challenges: &HashMap<usize, F>,
    ) -> Result<Vec<Option<Vec<F>>>, Error> {
        if phase != self.next_phase {
            return Err(Error::Other(format!(
                "Expected phase {}, got {}",
                self.next_phase, phase
            )));
        }
        let current_phase = match phase {
            0 => FirstPhase.to_sealed(),
            1 => SecondPhase.to_sealed(),
            2 => ThirdPhase.to_sealed(),
            _ => unreachable!("only phase [0,2] supported"),
        };

        let mut witness = WitnessCollection {
            k: self.k,
            current_phase,
            advice: vec![vec![Assigned::Zero; self.n]; self.cs.num_advice_columns],
            instances: self.instances,
            challenges,
            // The prover will not be allowed to assign values to advice
            // cells that exist within inactive rows, which include some
            // number of blinding factors and an extra row for use in the
            // permutation argument.
            usable_rows: ..self.unusable_rows_start,
            _marker: std::marker::PhantomData,
        };

        // Synthesize the circuit to obtain the witness and other information.
        ConcreteCircuit::FloorPlanner::synthesize(
            &mut witness,
            self.circuit,
            self.config.clone(),
            self.cs.constants.clone(),
        )
        .expect("todo");

        let column_indices = self
            .cs
            .advice_column_phase
            .iter()
            .enumerate()
            .filter_map(|(column_index, phase)| {
                if current_phase == *phase {
                    Some(column_index)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();

        self.next_phase += 1;
        let advice_values = batch_invert_assigned(witness.advice);
        Ok(advice_values
            .into_iter()
            .enumerate()
            .map(|(column_index, advice)| {
                if column_indices.contains(&column_index) {
                    Some(advice)
                } else {
                    None
                }
            })
            .collect())
    }
}

// Turn vectors of `Assigned<F>` into vectors of `F` by evaluation the divisions in `Assigned<F>`
// using batched inversions.
fn batch_invert_assigned<F: Field>(assigned: Vec<Vec<Assigned<F>>>) -> Vec<Vec<F>> {
    let mut assigned_denominators: Vec<_> = assigned
        .iter()
        .map(|f| {
            f.iter()
                .map(|value| value.denominator())
                .collect::<Vec<_>>()
        })
        .collect();

    assigned_denominators
        .iter_mut()
        .flat_map(|f| {
            f.iter_mut()
                // If the denominator is trivial, we can skip it, reducing the
                // size of the batch inversion.
                .filter_map(|d| d.as_mut())
        })
        .batch_invert();

    assigned
        .iter()
        .zip(assigned_denominators)
        .map(|(poly, inv_denoms)| {
            poly_invert(poly, inv_denoms.into_iter().map(|d| d.unwrap_or(F::ONE)))
        })
        .collect()
}

// Turn a slice of `Assigned<F>` into a vector of F by multiplying each numerator with the elements
// from `inv_denoms`, assuming that `inv_denoms` are the inverted denominators of the
// `Assigned<F>`.
fn poly_invert<F: Field>(
    poly: &[Assigned<F>],
    inv_denoms: impl Iterator<Item = F> + ExactSizeIterator,
) -> Vec<F> {
    assert_eq!(inv_denoms.len(), poly.len());
    poly.iter()
        .zip(inv_denoms)
        .map(|(a, inv_den)| a.numerator() * inv_den)
        .collect()
}
