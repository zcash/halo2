use std::convert::TryInto;
use std::marker::PhantomData;

use group::ff::PrimeField;
use halo2_gadgets::poseidon::{
    primitives::{self as poseidon, ConstantLength, P128Pow5T3 as OrchardNullifier, Spec},
    Hash, Pow5Chip, Pow5Config,
};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
};
use pasta_curves::pallas;

type Fp = pallas::Base;

// ANCHOR: circuit
/// A simple circuit that hashes two private inputs using Poseidon and exposes
/// the digest as a public input.
///
/// This example demonstrates how to build a circuit by composing an existing
/// gadget (the Poseidon hash chip) rather than writing constraints from scratch.
struct HashCircuit<
    S: Spec<Fp, WIDTH, RATE>,
    const WIDTH: usize,
    const RATE: usize,
> {
    left: Value<Fp>,
    right: Value<Fp>,
    _spec: PhantomData<S>,
}

impl<S: Spec<Fp, WIDTH, RATE>, const WIDTH: usize, const RATE: usize>
    HashCircuit<S, WIDTH, RATE>
{
    fn new(left: Value<Fp>, right: Value<Fp>) -> Self {
        Self {
            left,
            right,
            _spec: PhantomData,
        }
    }
}
// ANCHOR_END: circuit

// ANCHOR: config
/// The circuit configuration bundles the Poseidon chip config together with
/// the columns our circuit needs for loading inputs and exposing outputs.
#[derive(Clone, Debug)]
struct HashCircuitConfig<Fp: PrimeField, const WIDTH: usize, const RATE: usize> {
    /// The first advice column, which we also use to load message words.
    advices: [Column<Advice>; WIDTH],
    /// Instance column for the public hash output.
    instance: Column<Instance>,
    /// The Poseidon chip configuration.
    poseidon_config: Pow5Config<Fp, WIDTH, RATE>,
}
// ANCHOR_END: config

// ANCHOR: circuit-impl
impl<S: Spec<Fp, WIDTH, RATE>, const WIDTH: usize, const RATE: usize> Circuit<Fp>
    for HashCircuit<S, WIDTH, RATE>
{
    type Config = HashCircuitConfig<Fp, WIDTH, RATE>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            left: Value::unknown(),
            right: Value::unknown(),
            _spec: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        // Allocate columns for the Poseidon chip.  The chip needs WIDTH advice
        // columns for state, one extra advice column for the partial S-box, and
        // two sets of WIDTH fixed columns for round constants.
        let advices: [Column<Advice>; WIDTH] = (0..WIDTH)
            .map(|_| meta.advice_column())
            .collect::<Vec<_>>()
            .try_into()
            .expect("correct number of advice columns");
        let partial_sbox = meta.advice_column();

        let rc_a: [Column<Fixed>; WIDTH] = (0..WIDTH)
            .map(|_| meta.fixed_column())
            .collect::<Vec<_>>()
            .try_into()
            .expect("correct number of rc_a columns");
        let rc_b: [Column<Fixed>; WIDTH] = (0..WIDTH)
            .map(|_| meta.fixed_column())
            .collect::<Vec<_>>()
            .try_into()
            .expect("correct number of rc_b columns");

        // We need a constant column so that the chip can initialise its state.
        meta.enable_constant(rc_b[0]);

        // Instance column for the public output.
        let instance = meta.instance_column();
        meta.enable_equality(instance);

        let poseidon_config =
            Pow5Chip::configure::<S>(meta, advices, partial_sbox, rc_a, rc_b);

        HashCircuitConfig {
            advices,
            instance,
            poseidon_config,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        // Load our two private inputs into the circuit.
        let message = layouter.assign_region(
            || "load message",
            |mut region| {
                let left = region.assign_advice(
                    || "left",
                    config.advices[0],
                    0,
                    || self.left,
                )?;
                let right = region.assign_advice(
                    || "right",
                    config.advices[1],
                    0,
                    || self.right,
                )?;
                Ok([left, right])
            },
        )?;

        // Construct the Poseidon chip and use the Hash gadget to hash the message.
        let chip = Pow5Chip::construct(config.poseidon_config);
        let hasher = Hash::<_, _, S, ConstantLength<2>, WIDTH, RATE>::init(
            chip,
            layouter.namespace(|| "hasher"),
        )?;
        let digest = hasher.hash(layouter.namespace(|| "hash"), message)?;

        // Expose the hash digest as a public input.
        layouter.constrain_instance(digest.cell(), config.instance, 0)
    }
}
// ANCHOR_END: circuit-impl

// ANCHOR: main
fn main() {
    use halo2_proofs::dev::MockProver;

    // The Poseidon permutation requires a few rounds, so we need enough rows.
    let k = 7;

    // Choose two private inputs.
    let left = Fp::from(42);
    let right = Fp::from(99);

    // Compute the expected hash outside the circuit using the Poseidon primitives.
    let expected = poseidon::Hash::<_, OrchardNullifier, ConstantLength<2>, 3, 2>::init()
        .hash([left, right]);

    // Instantiate the circuit with private inputs.
    let circuit = HashCircuit::<OrchardNullifier, 3, 2>::new(
        Value::known(left),
        Value::known(right),
    );

    // The public input is the expected hash digest.
    let public_inputs = vec![expected];

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()])
        .expect("MockProver::run should succeed");
    assert_eq!(prover.verify(), Ok(()));

    // If we try a wrong public input, the proof will fail.
    let wrong_inputs = vec![expected + Fp::one()];
    let prover = MockProver::run(k, &circuit, vec![wrong_inputs])
        .expect("MockProver::run should succeed");
    assert!(prover.verify().is_err());
}
// ANCHOR_END: main
