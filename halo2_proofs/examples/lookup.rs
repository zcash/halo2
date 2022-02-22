// In this example, we showcase usage of lookups. We have two private variables and a lookup table of two columns and two rows, and we prove that the private
// variables are a key-value pair in the table
//
// Imperative pseudocode for this program would be:
// ```
//
// table TABLE {
//    42 -> 33,
//    43 -> 32,
// }
//
// def main(field k, field v):
//    assert(TABLE[k] == v)
//    return
// ```
//
// The full circuit looks like this
//
//        <-- private --> <---  table  --->
// |-----|-------|-------|--------|--------|
// | row |   k   |   v   |   42   |   33   |
// |-----|-------|-------|--------|--------|
// |  0  |   k   |   v   |   43   |   32   |
// |-----|-------|-------|--------|--------|
// |  1  |       |       |   (0)  |   (0)  |
// |-----|-------|-------|--------|--------|
//
// * One limitation is that the lookup argument is valid for the entire column, so as is it is not satisfied
// for rows 1 and above, as default values for these rows for private variables are 0.
// In order to make this work, we added `(0, 0)` to the lookup table. Is this the only way to activate a lookup
// argument only for certain rows?

use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, SimpleFloorPlanner},
    dev::MockProver,
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, TableColumn},
    poly::Rotation,
};

type TableConfig<F> = Vec<(F, F)>;

#[derive(Debug)]
struct TableCheckChip<F> {
    config: TableCheckChipConfig,
    marker: PhantomData<F>,
}

#[derive(Clone, Debug)]
struct TableCheckChipConfig {
    k: Column<Advice>,
    v: Column<Advice>,

    c_0: TableColumn,
    c_1: TableColumn,
}

impl<F: FieldExt> Chip<F> for TableCheckChip<F> {
    type Config = TableCheckChipConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: FieldExt> TableCheckChip<F> {
    fn new(config: <Self as Chip<F>>::Config) -> Self {
        TableCheckChip {
            config,
            marker: PhantomData,
        }
    }

    // Creates the columns and gates (constraint polynomials) required by this chip and stores
    // references to the columns in the chip config structure.
    fn configure(cs: &mut ConstraintSystem<F>) -> <Self as Chip<F>>::Config {
        let k = cs.advice_column();
        let v = cs.advice_column();

        let c_0 = cs.lookup_table_column();
        let c_1 = cs.lookup_table_column();

        cs.lookup(|meta| {
            let k = meta.query_advice(k, Rotation::cur());
            let v = meta.query_advice(v, Rotation::cur());
            vec![(k, c_0), (v, c_1)]
        });

        TableCheckChipConfig { k, v, c_0, c_1 }
    }

    fn assign_private_inputs(
        &self,
        layouter: &mut impl Layouter<F>,
        k: Option<F>,
        v: Option<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "assign private",
            |mut meta| {
                meta.assign_advice(|| "k", self.config.k, 0, || k.ok_or(Error::Synthesis))?;
                meta.assign_advice(|| "v", self.config.v, 0, || v.ok_or(Error::Synthesis))?;
                Ok(())
            },
        )
    }

    // Check the assigned cells are a key-value pair in the table
    fn assign_table(
        &self,
        layouter: &mut impl Layouter<F>,
        table_config: &TableConfig<F>,
    ) -> Result<(), Error> {
        layouter.assign_table(
            || "table",
            |mut table| {
                for (i, (k, v)) in table_config.iter().enumerate() {
                    table.assign_cell(
                        || format!("table row {} column 0", i),
                        self.config.c_0,
                        i,
                        || Ok(k),
                    )?;
                    table.assign_cell(
                        || format!("table row {} column 1", i),
                        self.config.c_1,
                        i,
                        || Ok(v),
                    )?;
                }
                Ok(())
            },
        )
    }
}

#[derive(Clone)]
struct MyCircuit<F> {
    table: TableConfig<F>,
    k: Option<F>,
    v: Option<F>,
}

impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
    // Our circuit uses one chip, thus we can reuse the chip's config as the circuit's config.
    type Config = TableCheckChipConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            table: TableConfig::default(),
            k: None,
            v: None,
        }
    }

    fn configure(cs: &mut ConstraintSystem<F>) -> Self::Config {
        TableCheckChip::configure(cs)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = TableCheckChip::new(config);
        chip.assign_table(&mut layouter, &self.table)?;
        chip.assign_private_inputs(&mut layouter, self.k, self.v)?;
        Ok(())
    }
}

fn main() {
    let k = 4;

    // The prover creates a circuit containing the values of the private inputs.
    let circuit = MyCircuit {
        table: vec![(42, 33), (43, 32), (0, 0)]
            .into_iter()
            .map(|(x, y)| (Fp::from(x), Fp::from(y)))
            .collect(),
        k: Some(Fp::from(42)),
        v: Some(Fp::from(33)),
    };

    // Create the area you want to draw on.
    // Use SVGBackend if you want to render to .svg instead.
    use plotters::prelude::*;
    let root = BitMapBackend::new("lookup.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Simple lookup example", ("sans-serif", 60))
        .unwrap();

    halo2_proofs::dev::CircuitLayout::default()
        .render::<Fp, _, _>(k, &circuit, &root)
        .unwrap();

    // Assert that the constraint system is satisfied.
    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    assert!(prover.verify().is_ok());

    let bad_circuit = MyCircuit {
        table: vec![(42, 33), (43, 32)]
            .into_iter()
            .map(|(x, y)| (Fp::from(x), Fp::from(y)))
            .collect(),
        k: Some(Fp::from(123)),
        v: Some(Fp::from(321)),
    };

    // Assert that changing the private inputs to values not contained in the table results in the constraint system becoming unsatisfied.
    let prover = MockProver::run(k, &bad_circuit, vec![]).unwrap();
    assert!(prover.verify().is_err());
}
