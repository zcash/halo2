//! Traits and structs for implementing circuit components.

use core::cmp::max;
use core::ops::{Add, Mul};
use halo2_common::circuit::layouter::SyncDeps;
use halo2_common::plonk::sealed::SealedPhase;
use halo2_common::plonk::FloorPlanner;
use halo2_common::plonk::{lookup, permutation, shuffle, Error, Queries};
use halo2_common::plonk::{Circuit, ConstraintSystem};
use halo2_common::{
    circuit::{Layouter, Region, Value},
    poly::{batch_invert_assigned, Polynomial},
};
use halo2_middleware::circuit::{
    Advice, AdviceQueryMid, Any, Challenge, Column, CompiledCircuitV2, ConstraintSystemV2Backend,
    ExpressionMid, Fixed, FixedQueryMid, GateV2Backend, Instance, InstanceQueryMid,
    PreprocessingV2,
};
use halo2_middleware::ff::Field;
use halo2_middleware::metadata;
use halo2_middleware::plonk::Assigned;
use halo2_middleware::poly::Rotation;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt::Debug;
use std::iter::{Product, Sum};
use std::ops::RangeTo;
use std::{
    convert::TryFrom,
    ops::{Neg, Sub},
};

/// Compile a circuit.  Runs configure and synthesize on the circuit in order to materialize the
/// circuit into its columns and the column configuration; as well as doing the fixed column and
/// copy constraints assignments.  The output of this function can then be used for the key
/// generation, and proof generation.
/// If `compress_selectors` is true, multiple selector columns may be multiplexed.
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
        fixed: vec![Polynomial::new_empty(n, F::ZERO.into()); cs.num_fixed_columns],
        permutation: permutation::AssemblyFront::new(n, &cs.permutation),
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

    let fixed = batch_invert_assigned(assembly.fixed);
    let (cs, selector_polys) = if compress_selectors {
        cs.compress_selectors(assembly.selectors.clone())
    } else {
        // After this, the ConstraintSystem should not have any selectors: `verify` does not need them, and `keygen_pk` regenerates `cs` from scratch anyways.
        let selectors = std::mem::take(&mut assembly.selectors);
        cs.directly_convert_selectors_to_fixed(selectors)
    };
    let mut fixed: Vec<_> = fixed.into_iter().map(|p| p.values).collect();
    fixed.extend(selector_polys.into_iter());

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
