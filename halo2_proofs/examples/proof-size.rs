use ff::Field;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
};
use halo2curves::pasta::Fp;

use halo2_proofs::dev::cost_model::{from_circuit_to_model_circuit, CommitmentScheme};
use halo2_proofs::plonk::{Expression, Selector, TableColumn};
use halo2_proofs::poly::Rotation;

// We use a lookup example
#[derive(Clone, Copy)]
struct TestCircuit {}

#[derive(Debug, Clone)]
struct MyConfig {
    selector: Selector,
    table: TableColumn,
    advice: Column<Advice>,
}

impl Circuit<Fp> for TestCircuit {
    type Config = MyConfig;
    type FloorPlanner = SimpleFloorPlanner;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self {}
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> MyConfig {
        let config = MyConfig {
            selector: meta.complex_selector(),
            table: meta.lookup_table_column(),
            advice: meta.advice_column(),
        };

        meta.lookup("lookup", |meta| {
            let selector = meta.query_selector(config.selector);
            let not_selector = Expression::Constant(Fp::ONE) - selector.clone();
            let advice = meta.query_advice(config.advice, Rotation::cur());
            vec![(selector * advice + not_selector, config.table)]
        });

        config
    }

    fn synthesize(&self, config: MyConfig, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_table(
            || "8-bit table",
            |mut table| {
                for row in 0u64..(1 << 8) {
                    table.assign_cell(
                        || format!("row {row}"),
                        config.table,
                        row as usize,
                        || Value::known(Fp::from(row + 1)),
                    )?;
                }

                Ok(())
            },
        )?;

        layouter.assign_region(
            || "assign values",
            |mut region| {
                for offset in 0u64..(1 << 10) {
                    config.selector.enable(&mut region, offset as usize)?;
                    region.assign_advice(
                        || format!("offset {offset}"),
                        config.advice,
                        offset as usize,
                        || Value::known(Fp::from((offset % 256) + 1)),
                    )?;
                }

                Ok(())
            },
        )
    }
}

const K: u32 = 11;

fn main() {
    let circuit = TestCircuit {};

    let model = from_circuit_to_model_circuit::<_, _, 56, 56>(
        K,
        &circuit,
        vec![],
        CommitmentScheme::KZGGWC,
    );
    println!(
        "Cost of circuit with 8 bit lookup table: \n{}",
        serde_json::to_string_pretty(&model).unwrap()
    );
}
