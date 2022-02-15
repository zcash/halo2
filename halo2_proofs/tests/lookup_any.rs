use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner},
    dev::MockProver,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};
use pairing::bn256::Fr as Fp;

#[test]
fn lookup_any() {
    #[derive(Clone, Debug)]
    struct MyConfig<F: FieldExt> {
        input: Column<Advice>,
        // Selector to enable lookups on even numbers.
        q_even: Selector,
        // Use an advice column as the lookup table column for even numbers.
        table_even: Column<Advice>,
        // Selector to enable lookups on odd numbers.
        q_odd: Selector,
        // Use an instance column as the lookup table column for odd numbers.
        table_odd: Column<Instance>,
        _marker: PhantomData<F>,
    }

    impl<F: FieldExt> MyConfig<F> {
        fn configure(meta: &mut ConstraintSystem<F>) -> Self {
            let config = Self {
                input: meta.advice_column(),
                q_even: meta.complex_selector(),
                table_even: meta.advice_column(),
                q_odd: meta.complex_selector(),
                table_odd: meta.instance_column(),
                _marker: PhantomData,
            };

            // Lookup on even numbers
            meta.lookup_any("even number", |meta| {
                let input = meta.query_advice(config.input, Rotation::cur());

                let q_even = meta.query_selector(config.q_even);
                let table_even = meta.query_advice(config.table_even, Rotation::cur());

                vec![(q_even * input, table_even)]
            });

            // Lookup on odd numbers
            meta.lookup_any("odd number", |meta| {
                let input = meta.query_advice(config.input, Rotation::cur());

                let q_odd = meta.query_selector(config.q_odd);
                let table_odd = meta.query_instance(config.table_odd, Rotation::cur());

                vec![(q_odd * input, table_odd)]
            });

            config
        }

        fn witness_even(
            &self,
            mut layouter: impl Layouter<F>,
            value: Option<F>,
        ) -> Result<(), Error> {
            layouter.assign_region(
                || "witness even number",
                |mut region| {
                    // Enable the even lookup.
                    self.q_even.enable(&mut region, 0)?;

                    region.assign_advice(
                        || "even input",
                        self.input,
                        0,
                        || value.ok_or(Error::Synthesis),
                    )?;
                    Ok(())
                },
            )
        }

        fn witness_odd(
            &self,
            mut layouter: impl Layouter<F>,
            value: Option<F>,
        ) -> Result<(), Error> {
            layouter.assign_region(
                || "witness odd number",
                |mut region| {
                    // Enable the odd lookup.
                    self.q_odd.enable(&mut region, 0)?;

                    region.assign_advice(
                        || "odd input",
                        self.input,
                        0,
                        || value.ok_or(Error::Synthesis),
                    )?;
                    Ok(())
                },
            )
        }

        fn load_even_lookup(
            &self,
            mut layouter: impl Layouter<F>,
            values: &[F],
        ) -> Result<(), Error> {
            layouter.assign_region(
                || "load values for even lookup table",
                |mut region| {
                    for (offset, value) in values.iter().enumerate() {
                        region.assign_advice(
                            || "even table value",
                            self.table_even,
                            offset,
                            || Ok(*value),
                        )?;
                    }

                    Ok(())
                },
            )
        }
    }

    #[derive(Default)]
    struct MyCircuit<F: FieldExt> {
        even_lookup: Vec<F>,
        even_witnesses: Vec<Option<F>>,
        odd_witnesses: Vec<Option<F>>,
    }

    impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
        // Since we are using a single chip for everything, we can just reuse its config.
        type Config = MyConfig<F>;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            Self::Config::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            // Load allowed values for even lookup table
            config.load_even_lookup(
                layouter.namespace(|| "witness even numbers"),
                &self.even_lookup,
            )?;

            // Witness even numbers
            for even in self.even_witnesses.iter() {
                config.witness_even(layouter.namespace(|| "witness even numbers"), *even)?;
            }

            // Witness odd numbers
            for odd in self.odd_witnesses.iter() {
                config.witness_odd(layouter.namespace(|| "witness odd numbers"), *odd)?;
            }

            Ok(())
        }
    }

    // Run MockProver.
    let k = 4;

    // Prepare the private and public inputs to the circuit.
    let even_lookup = vec![
        Fp::from(0),
        Fp::from(2),
        Fp::from(4),
        Fp::from(6),
        Fp::from(8),
    ];
    let odd_lookup = vec![
        Fp::from(1),
        Fp::from(3),
        Fp::from(5),
        Fp::from(7),
        Fp::from(9),
    ];
    let even_witnesses = vec![Some(Fp::from(0)), Some(Fp::from(2)), Some(Fp::from(4))];
    let odd_witnesses = vec![Some(Fp::from(1)), Some(Fp::from(3)), Some(Fp::from(5))];

    // Instantiate the circuit with the private inputs.
    let circuit = MyCircuit {
        even_lookup: even_lookup.clone(),
        even_witnesses,
        odd_witnesses,
    };

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![odd_lookup]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we pass in a public input containing only even numbers,
    // the odd number lookup will fail.
    let prover = MockProver::run(k, &circuit, vec![even_lookup]).unwrap();
    assert!(prover.verify().is_err())
}
