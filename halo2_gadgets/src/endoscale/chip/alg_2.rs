use ff::{Field, PrimeFieldBits};
use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{AssignedCell, Layouter, Value},
    plonk::{
        Advice, Assigned, Column, ConstraintSystem, Constraints, Error, Expression, Instance,
        Selector, TableColumn,
    },
    poly::Rotation,
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

impl<F: FieldExt + PrimeFieldBits, const K: usize> Bitstring<F, K> {
    #[cfg(test)]
    fn bitstring(&self) -> Value<Vec<bool>> {
        let num_bits = self.running_sum.num_bits();
        self.running_sum.zs()[0]
            .value()
            .map(|v| v.to_le_bits().iter().by_vals().take(num_bits).collect())
    }
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

        let config = Self {
            q_lookup: meta.complex_selector(),
            q_endoscale_scalar: meta.selector(),
            endoscalars,
            endoscalars_copy,
            acc,
            running_sum_chunks,
            table,
        };

        meta.create_gate("Endoscale scalar with lookup", |meta| {
            let q_endoscale_scalar = meta.query_selector(config.q_endoscale_scalar);
            let endo = meta.query_advice(config.endoscalars_copy, Rotation::cur());
            let acc = meta.query_advice(config.acc, Rotation::cur());
            let next_acc = meta.query_advice(config.acc, Rotation::next());

            // Check that next_acc = acc * 2^{K/2} + endo
            let expected_next_acc = acc * C::Base::from(1 << (K / 2)) + endo;

            Constraints::with_selector(q_endoscale_scalar, [next_acc - expected_next_acc])
        });

        meta.lookup(|meta| {
            let q_lookup = meta.query_selector(config.q_lookup);
            let word = config.running_sum_chunks.window_expr_be(meta);
            let endo = meta.query_advice(config.endoscalars_copy, Rotation::cur());

            let neg_q_lookup = Expression::Constant(C::Base::one()) - q_lookup.clone();
            let default_endo = {
                let val = compute_endoscalar_with_acc(Some(C::Base::zero()), &[false; K]);
                Expression::Constant(val)
            };

            let endo_expr = q_lookup.clone() * endo + neg_q_lookup * default_endo;

            vec![(q_lookup * word, table.bits), (endo_expr, table.endoscalar)]
        });

        config
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
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Bitstring<C::Base, K>,
    ) -> Result<AssignedCell<Assigned<C::Base>, C::Base>, Error> {
        // TODO: account for padding bits
        let num_bits = bitstring.running_sum.num_bits();
        // num_bits must be an even number not greater than MAX_BITSTRING_LENGTH.
        assert!(num_bits <= MAX_BITSTRING_LENGTH);

        layouter.assign_region(
            || "Endoscale scalar using bitstring (lookup optimisation)",
            |mut region| {
                let offset = 0;
                // The endoscalar is initialised to 2 * (ζ + 1).
                let mut acc = {
                    let init = (C::Base::ZETA + C::Base::one()).double();
                    region.assign_advice_from_constant(
                        || "initialise acc",
                        self.acc,
                        offset,
                        init,
                    )?
                };

                // Copy the running sum
                let running_sum_len = bitstring.running_sum.zs().len();
                for (idx, z) in bitstring.running_sum.zs().iter().rev().enumerate() {
                    z.copy_advice(
                        || format!("z[{:?}]", running_sum_len - idx),
                        &mut region,
                        self.running_sum_chunks.z(),
                        offset + idx,
                    )?;
                }

                // For each chunk, lookup the (chunk, endoscalar) pair and add
                // it to the accumulator.
                for (idx, chunk) in bitstring.running_sum.windows().iter().rev().enumerate() {
                    self.q_lookup.enable(&mut region, offset + idx)?;
                    self.q_endoscale_scalar.enable(&mut region, offset + idx)?;

                    let endoscalar = chunk.map(|c| {
                        compute_endoscalar_with_acc(
                            Some(C::Base::zero()),
                            &c.to_le_bits().iter().by_vals().take(K).collect::<Vec<_>>(),
                        )
                    });
                    // Witness endoscalar.
                    region.assign_advice(
                        || format!("Endoscalar for chunk {}", running_sum_len - idx),
                        self.endoscalars_copy,
                        offset + idx,
                        || endoscalar,
                    )?;

                    // Bitshift the accumulator by {K / 2} and add to endoscalar.
                    let acc_val = acc.value().zip(endoscalar).map(|(&acc, endo)| {
                        let two_pow_k_div2 = C::Base::from(1 << (K / 2));
                        acc * two_pow_k_div2 + endo
                    });
                    acc = region.assign_advice(
                        || format!("Endoscalar for chunk {}", running_sum_len - idx),
                        self.acc,
                        offset + idx + 1,
                        || acc_val,
                    )?;
                }

                #[cfg(test)]
                {
                    use crate::endoscale::util::compute_endoscalar;
                    let bitstring = bitstring.bitstring();
                    let expected_acc: Value<C::Base> = bitstring.map(|b| compute_endoscalar(&b));
                    acc.value()
                        .zip(expected_acc)
                        .map(|(&acc, expected_acc)| assert_eq!(acc, expected_acc));
                }

                Ok(acc.into())
            },
        )
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
