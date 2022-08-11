use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    pasta::Fp,
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column,
        ConstraintSystem, DynamicTable, DynamicTableMap, Error, Selector, SingleVerifier,
    },
    poly::{commitment::Params, Rotation},
    transcript::{Blake2bRead, Blake2bWrite},
};
use pasta_curves::{vesta, EqAffine};
use rand_core::OsRng;

#[derive(Clone)]
struct EvenOddCircuitConfig {
    is_even: Selector,
    is_odd: Selector,
    a: Column<Advice>,
    // starts at zero to use as default
    table_vals: Column<Advice>,
    even: DynamicTable,
    odd: DynamicTable,
}

struct DynLookupCircuit {}
impl Circuit<Fp> for DynLookupCircuit {
    type Config = EvenOddCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let a = meta.advice_column();
        let table_vals = meta.advice_column();
        let is_even = meta.complex_selector();
        let is_odd = meta.complex_selector();
        let even = meta.create_dynamic_table("even", &[], &[table_vals]);
        let odd = meta.create_dynamic_table("odd", &[], &[table_vals]);

        meta.lookup_dynamic(&even, |cells| {
            let a = cells.query_advice(a, Rotation::cur());
            let is_even = cells.query_selector(is_even);

            DynamicTableMap {
                selector: is_even,
                table_map: vec![(a.clone(), table_vals.into())],
            }
        });

        meta.lookup_dynamic(&odd, |cells| {
            let a = cells.query_advice(a, Rotation::cur());
            let is_odd = cells.query_selector(is_odd);

            DynamicTableMap {
                selector: is_odd,
                table_map: vec![(a.clone(), table_vals.into())],
            }
        });

        EvenOddCircuitConfig {
            a,
            table_vals,
            is_even,
            is_odd,
            even,
            odd,
        }
    }

    fn without_witnesses(&self) -> Self {
        Self {}
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        for i in 0..=5 {
            layouter.assign_region(
                || format!("lookup: {}", i),
                |mut region| {
                    // Enable the lookup on rows
                    if i % 2 == 0 {
                        config.is_even.enable(&mut region, 0)?;
                    } else {
                        config.is_odd.enable(&mut region, 0)?;
                    };

                    region.assign_advice(|| "", config.a, 0, || Value::known(Fp::from(i as u64)))
                },
            )?;
        }
        layouter.assign_region(
            || "table",
            |mut region| {
                for i in 0..=5 {
                    region.assign_advice(
                        || "",
                        config.table_vals,
                        i,
                        || Value::known(Fp::from(i as u64)),
                    )?;

                    let table = if i % 2 == 0 {
                        &config.even
                    } else {
                        &config.odd
                    };
                    table.add_row(&mut region, i)?;
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}

fn main() {
    let k = 5;

    MockProver::run(k, &DynLookupCircuit {}, vec![])
        .unwrap()
        .verify()
        .unwrap();

    let params: Params<EqAffine> = Params::new(k);
    let verifier = SingleVerifier::new(&params);
    let vk = keygen_vk(&params, &DynLookupCircuit {}).unwrap();
    let pk = keygen_pk(&params, vk.clone(), &DynLookupCircuit {}).unwrap();
    let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);
    create_proof(
        &params,
        &pk,
        &[DynLookupCircuit {}],
        &[],
        &mut OsRng,
        &mut transcript,
    )
    .expect("Failed to create proof");

    let proof: Vec<u8> = transcript.finalize();

    let mut transcript = Blake2bRead::init(&proof[..]);
    verify_proof(&params, pk.get_vk(), verifier, &[], &mut transcript)
        .expect("could not verify_proof");
}
