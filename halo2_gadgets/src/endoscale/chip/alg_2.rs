use ff::PrimeFieldBits;
use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Assigned, Column, ConstraintSystem, Error, Instance, Selector, TableColumn},
};

use crate::{
    endoscale::util::compute_endoscalar_with_acc,
    utilities::{
        decompose_running_sum::{RunningSum, RunningSumConfig},
        i2lebsp, le_bits_to_field_elem,
    },
};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Bitstring<F: FieldExt + PrimeFieldBits, const K: usize> {
    running_sum: RunningSum<F, K>,
    pad_len: usize,
}

/// Configuration for endoscalar table.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TableConfig<F: FieldExt, const K: usize> {
    pub(crate) bits: TableColumn,
    pub(crate) endoscalar: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: FieldExt, const K: usize> TableConfig<F, K> {
    #[allow(dead_code)]
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        TableConfig {
            bits: meta.lookup_table_column(),
            endoscalar: meta.lookup_table_column(),
            _marker: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "endoscalar_map",
            |mut table| {
                for index in 0..(1 << K) {
                    table.assign_cell(
                        || "bits",
                        self.bits,
                        index,
                        || Value::known(F::from(index as u64)),
                    )?;
                    table.assign_cell(
                        || "endoscalar",
                        self.endoscalar,
                        index,
                        || {
                            Value::known(compute_endoscalar_with_acc(
                                Some(F::zero()),
                                &i2lebsp::<K>(index as u64),
                            ))
                        },
                    )?;
                }
                Ok(())
            },
        )
    }
}

/// Config used in Algorithm 2 (endoscaling in the field).
#[derive(Clone, Debug)]
pub(super) struct Alg2Config<C: CurveAffine, const K: usize, const MAX_BITSTRING_LENGTH: usize>
where
    C::Base: PrimeFieldBits,
{
    // Selector enabling a lookup in the (bitstring, endoscalar) table.
    q_lookup: Selector,
    // Selector for Alg 2 endoscaling.
    q_endoscale_scalar: Selector,
    // Public inputs are provided as endoscalars. Each endoscalar corresponds
    // to a K-bit chunk.
    endoscalars: Column<Instance>,
    // An additional advice column where endoscalar values are copied and used
    // in the lookup argument.
    endoscalars_copy: Column<Advice>,
    // Advice column where accumulator is witnessed.
    acc: Column<Advice>,
    // Configuration for running sum decomposition into K-bit chunks.
    pub(super) running_sum_chunks: RunningSumConfig<C::Base, K>,
    // Table mapping words to their corresponding endoscalars.
    table: TableConfig<C::Base, K>,
}

impl<C: CurveAffine, const K: usize, const MAX_BITSTRING_LENGTH: usize>
    Alg2Config<C, K, MAX_BITSTRING_LENGTH>
where
    C::Base: PrimeFieldBits,
{
    pub(super) fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        endoscalars: Column<Instance>,
        endoscalars_copy: Column<Advice>,
        acc: Column<Advice>,
        running_sum_chunks: RunningSumConfig<C::Base, K>,
    ) -> Self {
        meta.enable_equality(endoscalars);
        meta.enable_equality(endoscalars_copy);
        meta.enable_equality(acc);

        let table = TableConfig::configure(meta);

        Self {
            q_lookup: meta.complex_selector(),
            q_endoscale_scalar: meta.selector(),
            endoscalars,
            endoscalars_copy,
            acc,
            running_sum_chunks,
            table,
        }
    }

    pub(super) fn witness_bitstring(
        &self,
        mut layouter: impl Layouter<C::Base>,
        bits: &[Value<bool>],
    ) -> Result<Bitstring<C::Base, K>, Error> {
        let word_num_bits = bits.len();
        let pad_len = (K - (word_num_bits % K)) % K;

        // Right-pad bitstring to a multiple of K if needed
        let mut bits: Value<Vec<bool>> = bits.iter().copied().collect();
        if pad_len > 0 {
            bits = bits.map(|bits| {
                let padding = std::iter::repeat(false).take(pad_len);
                bits.iter().copied().chain(padding).collect()
            });
        }

        let alpha = bits.map(|b| le_bits_to_field_elem(&b));

        let running_sum = layouter.assign_region(
            || "witness bitstring",
            |mut region| {
                let offset = 0;

                let num_windows = (word_num_bits + pad_len) / K;
                self.running_sum_chunks.witness_decompose(
                    &mut region,
                    offset,
                    alpha,
                    true,
                    word_num_bits + pad_len,
                    num_windows,
                )
            },
        )?;

        Ok(Bitstring {
            running_sum,
            pad_len,
        })
    }

    pub(super) fn compute_endoscalar(
        &self,
        mut _layouter: &mut impl Layouter<C::Base>,
        _bitstring: &Bitstring<C::Base, K>,
    ) -> Result<AssignedCell<Assigned<C::Base>, C::Base>, Error> {
        todo!()
    }

    pub(super) fn constrain_bitstring(
        &self,
        mut _layouter: &mut impl Layouter<C::Base>,
        _bitstring: &Bitstring<C::Base, K>,
        _pub_input_rows: Vec<usize>,
    ) -> Result<(), Error> {
        todo!()
    }
}
