#![allow(clippy::many_single_char_names)]
#![allow(clippy::op_ref)]

use assert_matches::assert_matches;
use ff::{FromUniformBytes, WithSmallOrderMulGroup};
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Cell, Layouter, Region, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::plonk::{
    create_proof as create_plonk_proof, keygen_pk, keygen_vk, verify_proof as verify_plonk_proof,
    Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Instance,
    ProvingKey, Selector, TableColumn, VerifyingKey,
};
use halo2_proofs::poly::commitment::{CommitmentScheme, ParamsProver, Prover, Verifier};
use halo2_proofs::poly::Rotation;
use halo2_proofs::poly::VerificationStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, TranscriptReadBuffer,
    TranscriptWriterBuffer,
};
use rand_core::{OsRng, RngCore};
use std::marker::PhantomData;

#[derive(Clone)]
struct MyCircuitConfig {
    // A gate that uses selector, fixed, advice, has addition, multiplication and rotation
    // s_gate[0] * (a[0] + b[0] * c[0] * d[0] - a[1])
    s_gate: Selector,
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    d: Column<Fixed>,

    // Copy constraints between columns (a, b) and (a, d)

    // A dynamic lookup: s_lookup * [1, a[0], b[0]] in s_ltable * [1, d[0], c[0]]
    s_lookup: Column<Fixed>,
    s_ltable: Column<Fixed>,

    // A shuffle: s_shufle * [1, a[0]] shuffle_of s_stable * [1, b[0]]
    s_shuffle: Column<Fixed>,
    s_stable: Column<Fixed>,

    // Instance
    instance: Column<Instance>,
}

#[derive(Clone)]
struct MyCircuit<F: Field> {
    _marker: std::marker::PhantomData<F>,
}

impl<F: Field + From<u64>> Circuit<F> for MyCircuit<F> {
    type Config = MyCircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self {
            _marker: std::marker::PhantomData {},
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> MyCircuitConfig {
        let s_gate = meta.selector();
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let d = meta.fixed_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(d);

        let s_lookup = meta.fixed_column();
        let s_ltable = meta.fixed_column();

        let s_shuffle = meta.fixed_column();
        let s_stable = meta.fixed_column();

        let instance = meta.instance_column();
        meta.enable_equality(instance);

        let one = Expression::Constant(F::ONE);

        meta.create_gate("gate_a", |meta| {
            let s_gate = meta.query_selector(s_gate);
            let a1 = meta.query_advice(a, Rotation::next());
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            let d = meta.query_fixed(d, Rotation::cur());

            vec![s_gate * (a + b * c * d - a1)]
        });

        meta.lookup_any("lookup", |meta| {
            let s_lookup = meta.query_fixed(s_lookup, Rotation::cur());
            let s_ltable = meta.query_fixed(s_ltable, Rotation::cur());
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_advice(c, Rotation::cur());
            let d = meta.query_fixed(d, Rotation::cur());
            let lhs = [one.clone(), a, b].map(|c| c * s_lookup.clone());
            let rhs = [one.clone(), d, c].map(|c| c * s_ltable.clone());
            lhs.into_iter().zip(rhs.into_iter()).collect()
        });

        meta.shuffle("shuffle", |meta| {
            let s_shuffle = meta.query_fixed(s_shuffle, Rotation::cur());
            let s_stable = meta.query_fixed(s_stable, Rotation::cur());
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let lhs = [one.clone(), a].map(|c| c * s_shuffle.clone());
            let rhs = [one.clone(), b].map(|c| c * s_stable.clone());
            lhs.into_iter().zip(rhs.into_iter()).collect()
        });

        MyCircuitConfig {
            s_gate,
            a,
            b,
            c,
            d,
            s_lookup,
            s_ltable,
            s_shuffle,
            s_stable,
            instance,
        }
    }

    fn synthesize(
        &self,
        config: MyCircuitConfig,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let assign_gate = |region: &mut Region<'_, F>,
                           offset: &mut usize,
                           a_assigned: Option<AssignedCell<F, F>>,
                           abcd: [u64; 4]|
         -> Result<(AssignedCell<F, F>, [AssignedCell<F, F>; 4]), Error> {
            let [a, b, c, d] = abcd;
            config.s_gate.enable(region, *offset);
            let a_assigned = if let Some(a_assigned) = a_assigned {
                a_assigned
            } else {
                region.assign_advice(|| "", config.a, *offset, || Value::known(F::from(a)))?
            };
            let a = a_assigned.value();
            let [b, c, d] = [b, c, d].map(|v| Value::known(F::from(b)));
            let b_assigned = region.assign_advice(|| "", config.b, *offset, || b)?;
            let c_assigned = region.assign_advice(|| "", config.c, *offset, || c)?;
            let d_assigned = region.assign_fixed(|| "", config.d, *offset, || d)?;
            *offset += 1;
            let res = a
                .zip(b.zip(c.zip(d)))
                .map(|(a, (b, (c, d)))| *a + b * c * d);
            // let res = a + b * c * d;
            let res_assigned = region.assign_advice(|| "", config.a, *offset, || res)?;
            Ok((
                res_assigned,
                [a_assigned, b_assigned, c_assigned, d_assigned],
            ))
        };

        let instances = layouter.assign_region(
            || "single",
            |mut region| {
                let mut offset = 0;
                let mut instances = Vec::new();
                // Enable the gate on a few consecutive rows with rotations
                let (res, _) = assign_gate(&mut region, &mut offset, None, [2, 3, 4, 1])?;
                instances.push(res.clone());
                let (res, _) = assign_gate(&mut region, &mut offset, Some(res), [0, 6, 7, 1])?;
                instances.push(res.clone());
                let (res, _) = assign_gate(&mut region, &mut offset, Some(res), [0, 8, 9, 1])?;
                instances.push(res.clone());
                let (res, _) = assign_gate(
                    &mut region,
                    &mut offset,
                    Some(res),
                    [0, 0xffffffff, 0xdeadbeef, 1],
                )?;
                let _ = assign_gate(
                    &mut region,
                    &mut offset,
                    Some(res),
                    [0, 0xabad1d3a, 0x12345678, 0x42424242],
                )?;
                offset += 1;

                // Enable the gate on non-consecutive rows with advice-advice copy constraints enabled
                let (_, abcd1) = assign_gate(&mut region, &mut offset, None, [5, 2, 1, 1])?;
                offset += 1;
                let (_, abcd2) = assign_gate(&mut region, &mut offset, None, [2, 3, 1, 1])?;
                offset += 1;
                let (_, abcd3) = assign_gate(&mut region, &mut offset, None, [4, 2, 1, 1])?;
                offset += 1;
                region.constrain_equal(abcd1[1].cell(), abcd2[0].cell())?;
                region.constrain_equal(abcd2[0].cell(), abcd3[1].cell())?;
                instances.push(abcd1[1].clone());
                instances.push(abcd2[0].clone());

                // Enable the gate on non-consecutive rows with advice-fixed copy constraints enabled
                let (_, abcd1) = assign_gate(&mut region, &mut offset, None, [5, 9, 1, 9])?;
                offset += 1;
                let (_, abcd2) = assign_gate(&mut region, &mut offset, None, [2, 9, 1, 1])?;
                offset += 1;
                let (_, abcd3) = assign_gate(&mut region, &mut offset, None, [9, 2, 1, 1])?;
                offset += 1;
                region.constrain_equal(abcd1[1].cell(), abcd1[3].cell())?;
                region.constrain_equal(abcd2[1].cell(), abcd1[3].cell())?;
                region.constrain_equal(abcd3[0].cell(), abcd1[3].cell())?;

                // Enable a dynamic lookup (powers of two)
                let table: Vec<_> = (0u64..=10).map(|exp| (exp, 2u64.pow(exp as u32))).collect();
                let lookups = [(2, 4), (2, 4), (10, 1024), (0, 1), (2, 4)];
                for (table_row, lookup_row) in table
                    .iter()
                    .zip(lookups.iter().chain(std::iter::repeat(&(0, 1))))
                {
                    region.assign_fixed(|| "", config.s_lookup, offset, || Value::known(F::ONE))?;
                    region.assign_fixed(|| "", config.s_ltable, offset, || Value::known(F::ONE))?;
                    let lookup_row0 = Value::known(F::from(lookup_row.0));
                    let lookup_row1 = Value::known(F::from(lookup_row.1));
                    region.assign_advice(|| "", config.a, offset, || lookup_row0)?;
                    region.assign_advice(|| "", config.b, offset, || lookup_row1)?;
                    let table_row0 = Value::known(F::from(table_row.0));
                    let table_row1 = Value::known(F::from(table_row.1));
                    region.assign_fixed(|| "", config.d, offset, || table_row0)?;
                    region.assign_advice(|| "", config.c, offset, || table_row1)?;
                    offset += 1;
                }

                // Enable a dynamic shuffle (sequence from 0 to 15)
                let table: Vec<_> = (0u64..16).collect();
                let shuffle = [0u64, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15];
                assert_eq!(table.len(), shuffle.len());

                for (table_row, shuffle_row) in table.iter().zip(shuffle.iter()) {
                    region.assign_fixed(
                        || "",
                        config.s_shuffle,
                        offset,
                        || Value::known(F::ONE),
                    )?;
                    region.assign_fixed(|| "", config.s_stable, offset, || Value::known(F::ONE))?;
                    let shuffle_row0 = Value::known(F::from(*shuffle_row));
                    region.assign_advice(|| "", config.a, offset, || shuffle_row0)?;
                    let table_row0 = Value::known(F::from(*table_row));
                    region.assign_advice(|| "", config.b, offset, || table_row0)?;
                    offset += 1;
                }

                Ok(instances)
            },
        )?;

        println!("DBG instances: {:?}", instances);
        for (i, instance) in instances.iter().enumerate() {
            layouter.constrain_instance(instance.cell(), config.instance, i)?;
        }

        Ok(())
    }
}

use halo2curves::bn256::Fr;

#[test]
fn test_mycircuit() {
    let k = 8;
    let circuit: MyCircuit<Fr> = MyCircuit {
        _marker: std::marker::PhantomData {},
    };
    let instance = vec![
        Fr::from(0x1d),
        Fr::from(0xf5),
        Fr::from(0x2f5),
        Fr::from(0x2),
        Fr::from(0x2),
    ];
    let prover = MockProver::run(k, &circuit, vec![instance]).unwrap();
    prover.assert_satisfied();
}
