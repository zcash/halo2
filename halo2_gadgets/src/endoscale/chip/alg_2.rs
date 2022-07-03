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
        lookup_range_check::LookupRangeCheckConfig,
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
        let num_bits = self.running_sum.num_bits() - self.pad_len;
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
    C::Base: FieldExt + PrimeFieldBits,
{
    // Selector enabling a lookup in the (bitstring, endoscalar) table.
    q_lookup: Selector,
    // Selector to initialise Alg 2 endoscaling.
    q_init: Selector,
    // Selector for Alg 2 endoscaling.
    q_endoscale_scalar: Selector,
    // Selector checking that partial chunks are correctly shifted.
    q_partial_chunk: Selector,
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
    // Configuration for lookup range check of partial chunks.
    lookup_range_check: LookupRangeCheckConfig<C::Base, K>,
}

impl<C: CurveAffine, const K: usize, const MAX_BITSTRING_LENGTH: usize>
    Alg2Config<C, K, MAX_BITSTRING_LENGTH>
where
    C::Base: FieldExt + PrimeFieldBits,
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
        let lookup_range_check = LookupRangeCheckConfig::configure(meta, acc, table.bits);

        let config = Self {
            q_lookup: meta.complex_selector(),
            q_init: meta.selector(),
            q_endoscale_scalar: meta.selector(),
            q_partial_chunk: meta.selector(),
            endoscalars,
            endoscalars_copy,
            acc,
            running_sum_chunks,
            table,
            lookup_range_check,
        };

        let two_pow_k_div2 = Expression::Constant(C::Base::from(1u64 << (K / 2)));

        meta.create_gate("Endoscale scalar with lookup", |meta| {
            let q_endoscale_scalar = meta.query_selector(config.q_endoscale_scalar);
            let endo = meta.query_advice(config.endoscalars_copy, Rotation::cur());
            let acc = meta.query_advice(config.acc, Rotation::cur());
            let next_acc = meta.query_advice(config.acc, Rotation::next());

            // Check that next_acc = acc * 2^{K/2} + endo
            let expected_next_acc = acc * two_pow_k_div2.clone() + endo;

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

        meta.create_gate("Partial chunk", |meta| {
            let q_partial_chunk = meta.query_selector(config.q_partial_chunk);

            // z_pen - z_last * 2^K
            let expected_chunk = config.running_sum_chunks.window_expr_be(meta);
            let padded_chunk = meta.query_advice(config.acc, Rotation::cur());
            let chunk_check = expected_chunk - padded_chunk;

            let padded_endoscalar = meta.query_advice(config.endoscalars_copy, Rotation::cur());
            // 2^{K' / 2}
            let two_pow_k_prime_div2 = meta.query_advice(config.endoscalars_copy, Rotation::next());
            // shift = 2^{K'/2} - 2^{K/2}
            let shift = two_pow_k_prime_div2.clone() - two_pow_k_div2.clone();
            let shifted_endoscalar = padded_endoscalar - shift;

            // Initialise the accumulator to 2 * (ζ + 1).
            let init_acc = Expression::Constant((C::Base::ZETA + C::Base::one()).double());
            let acc_1 = meta.query_advice(config.acc, Rotation::next());
            // Check that acc_1 = init_acc * 2^{K' / 2} + shifted_endoscalar
            let expected_acc_1 = init_acc * two_pow_k_prime_div2 + shifted_endoscalar;
            let acc_check = acc_1 - expected_acc_1;

            Constraints::with_selector(
                q_partial_chunk,
                [("acc_check", acc_check), ("chunk_check", chunk_check)],
            )
        });

        meta.create_gate("Init chunk", |meta| {
            let q_init = meta.query_selector(config.q_init);

            let endoscalar = meta.query_advice(config.endoscalars_copy, Rotation::cur());

            let init_acc = meta.query_advice(config.endoscalars_copy, Rotation::next());
            let acc_1 = meta.query_advice(config.acc, Rotation::next());
            // Check that acc_1 = init_acc * 2^{K / 2} + endoscalar
            let expected_acc_1 = init_acc * two_pow_k_div2 + endoscalar;

            Constraints::with_selector(q_init, [("acc_check", acc_1 - expected_acc_1)])
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
        let pad_len = bitstring.pad_len;
        let num_bits = bitstring.running_sum.num_bits() - pad_len;
        // num_bits must be an even number not greater than MAX_BITSTRING_LENGTH.
        assert!(num_bits <= MAX_BITSTRING_LENGTH);

        // The bitstring will be broken into K-bit chunks with the first chunk
        // being a padded k_prime-bit partial chunk
        let k_prime = K - pad_len;

        // Interstitial running sum values
        let zs = bitstring.running_sum.zs();

        let init_acc = if pad_len > 0 {
            self.init_partial_chunk(
                layouter.namespace(|| "init partial chunk"),
                &(zs[zs.len() - 1]).clone().into(),
                &(zs[zs.len() - 2]).clone().into(),
                k_prime,
            )?
        } else {
            self.init_full_chunk(
                layouter.namespace(|| "init full chunk"),
                &(zs[zs.len() - 1]).clone().into(),
                &(zs[zs.len() - 2]).clone().into(),
            )?
        };

        layouter.assign_region(
            || "Endoscale scalar using bitstring (lookup optimisation)",
            |mut region| {
                let offset = 0;

                // Copy the running sum
                let running_sum_len = zs.len();
                for (idx, z) in zs.iter().rev().skip(1).enumerate() {
                    z.copy_advice(
                        || format!("z[{:?}]", running_sum_len - idx + 1),
                        &mut region,
                        self.running_sum_chunks.z(),
                        offset + idx,
                    )?;
                }

                // Copy in the accumulator
                init_acc.copy_advice(|| "copy acc", &mut region, self.acc, offset)?;

                let mut acc = init_acc.clone();
                // For each chunk, lookup the (chunk, endoscalar) pair and add
                // it to the accumulator.
                for (idx, chunk) in bitstring
                    .running_sum
                    .windows()
                    .iter()
                    .rev()
                    .skip(1)
                    .enumerate()
                {
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
                        .map(|(&acc, expected_acc)| assert_eq!(acc.evaluate(), expected_acc));
                }

                Ok(acc)
            },
        )
    }

    /// The first chunk is handled differently if it is padded:
    ///
    ///    |      z       |      acc     | endoscalars_copy  | q_partial | q_lookup|
    ///    -------------------------------------------------------------------------
    ///    |     z_last   | padded_chunk | padded_endoscalar |     1     |    1    |
    ///    |     z_pen    |    acc_1     |      2^{K'/2}     |     0     |    0    |
    ///
    fn init_partial_chunk(
        &self,
        mut layouter: impl Layouter<C::Base>,
        z_last: &AssignedCell<Assigned<C::Base>, C::Base>,
        z_pen: &AssignedCell<Assigned<C::Base>, C::Base>,
        k_prime: usize,
    ) -> Result<AssignedCell<Assigned<C::Base>, C::Base>, Error> {
        // Derive the padded chunk c_last = z_pen - z_last * 2^K
        let padded_chunk = z_pen
            .value()
            .zip(z_last.value())
            .map(|(z_pen, z_last)| *z_pen - *z_last * C::Base::from(1u64 << K));

        // Range-constrain the padded chunk to `k_prime` bits.
        let padded_chunk = self.lookup_range_check.witness_short_check(
            layouter.namespace(|| format!("Check that padded_chunk is {} bits", k_prime)),
            padded_chunk.evaluate(),
            k_prime,
        )?;

        layouter.assign_region(
            || "Init partial chunk",
            |mut region| {
                let offset = 0;

                // Enable q_partial_chunk on offset 0
                self.q_partial_chunk.enable(&mut region, offset)?;
                // Enable q_lookup on offset 0
                self.q_lookup.enable(&mut region, offset)?;

                // Copy z_last
                z_last.copy_advice(
                    || "copy z_last",
                    &mut region,
                    self.running_sum_chunks.z(),
                    offset,
                )?;
                // Copy z_pen
                z_pen.copy_advice(
                    || "copy z_pen",
                    &mut region,
                    self.running_sum_chunks.z(),
                    offset + 1,
                )?;
                // Copy padded_chunk
                padded_chunk.copy_advice(|| "copy padded chunk", &mut region, self.acc, offset)?;

                // Witness the endoscalar corresponding to the padded chunk.
                let padded_endoscalar = padded_chunk.value().map(|v| {
                    let bitstring = v.to_le_bits().iter().by_vals().take(K).collect::<Vec<_>>();
                    compute_endoscalar_with_acc(Some(C::Base::zero()), &bitstring)
                });
                region.assign_advice(
                    || "padded endoscalar",
                    self.endoscalars_copy,
                    offset,
                    || padded_endoscalar,
                )?;

                // Load the value 2^{K'/2} from constant.
                let two_pow_k_prime_div2: Assigned<C::Base> =
                    C::Base::from(1u64 << (k_prime / 2)).into();
                region.assign_advice_from_constant(
                    || "2^{K'/2}",
                    self.endoscalars_copy,
                    offset + 1,
                    two_pow_k_prime_div2,
                )?;

                // Bitshift the accumulator by {K' / 2} bits and add to adjusted endoscalar.
                let acc: Value<Assigned<C::Base>> = padded_endoscalar.map(|padded_endoscalar| {
                    let two_pow_k_div2: Assigned<C::Base> = C::Base::from(1 << (K / 2)).into();
                    let padded_endoscalar: Assigned<C::Base> = padded_endoscalar.into();

                    // shift = 2^{K'/2} - 2^{K/2}
                    let shift = two_pow_k_prime_div2 - two_pow_k_div2;
                    let actual_endoscalar = padded_endoscalar - shift;
                    let init_acc: Assigned<C::Base> =
                        (C::Base::ZETA + C::Base::one()).double().into();
                    init_acc * two_pow_k_prime_div2 + actual_endoscalar
                });
                region.assign_advice(
                    || "acc = init_acc * 2^{K'/2} + actual_endoscalar",
                    self.acc,
                    offset + 1,
                    || acc,
                )
            },
        )
    }

    /// If it is not padded, we lookup the endoscalar directly using the derived chunk:
    ///
    ///    |      z       |      acc     | endoscalars_copy  |  q_lookup |  q_init |
    ///    -------------------------------------------------------------------------
    ///    |     z_last   |              |    endoscalar     |     1     |    1    |
    ///    |     z_pen    |     acc_1    |    init_acc       |     0     |    0    |
    ///
    fn init_full_chunk(
        &self,
        mut layouter: impl Layouter<C::Base>,
        z_last: &AssignedCell<Assigned<C::Base>, C::Base>,
        z_pen: &AssignedCell<Assigned<C::Base>, C::Base>,
    ) -> Result<AssignedCell<Assigned<C::Base>, C::Base>, Error> {
        layouter.assign_region(
            || "Init full chunk",
            |mut region| {
                let offset = 0;

                // Enable q_lookup on offset 0
                self.q_lookup.enable(&mut region, offset)?;
                // Enable q_init on offset 0
                self.q_init.enable(&mut region, offset)?;

                // Copy z_last
                z_last.copy_advice(
                    || "copy z_last",
                    &mut region,
                    self.running_sum_chunks.z(),
                    offset,
                )?;
                // Copy z_pen
                z_pen.copy_advice(
                    || "copy z_pen",
                    &mut region,
                    self.running_sum_chunks.z(),
                    offset + 1,
                )?;

                // Initialise the accumulator to 2 * (ζ + 1).
                let init_acc = Assigned::from((C::Base::ZETA + C::Base::one()).double());
                region.assign_advice_from_constant(
                    || "initialise acc",
                    self.endoscalars_copy,
                    offset + 1,
                    init_acc,
                )?;

                // Derive chunk c_last = z_pen - z_last * 2^K
                let chunk = z_pen
                    .value()
                    .zip(z_last.value())
                    .map(|(z_pen, z_last)| *z_pen - *z_last * C::Base::from(1u64 << K));

                // Witness the endoscalar corresponding to the chunk.
                let endoscalar: Value<Assigned<C::Base>> = chunk
                    .map(|v| {
                        let bitstring = v
                            .evaluate()
                            .to_le_bits()
                            .iter()
                            .by_vals()
                            .take(K)
                            .collect::<Vec<_>>();
                        compute_endoscalar_with_acc(Some(C::Base::zero()), &bitstring)
                    })
                    .into();
                region.assign_advice(
                    || "actual endoscalar",
                    self.endoscalars_copy,
                    offset,
                    || endoscalar,
                )?;

                // Bitshift the accumulator by {K / 2} and add to endoscalar.
                let acc: Value<Assigned<C::Base>> = endoscalar.map(|endoscalar| {
                    let two_pow_k_div2 = C::Base::from(1 << (K / 2));
                    init_acc * two_pow_k_div2 + endoscalar
                });
                region.assign_advice(
                    || "acc = init_acc * 2^{K/2} + endoscalar",
                    self.acc,
                    offset + 1,
                    || acc,
                )
            },
        )
    }

    pub(super) fn constrain_bitstring(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Bitstring<C::Base, K>,
        pub_input_rows: Vec<usize>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "Recover bitstring from endoscalars",
            |mut region| {
                let offset = 0;

                // Copy the running sum.
                let running_sum_len = bitstring.running_sum.zs().len();
                for (idx, z) in bitstring.running_sum.zs().iter().rev().enumerate() {
                    z.copy_advice(
                        || format!("z[{:?}]", running_sum_len - idx),
                        &mut region,
                        self.running_sum_chunks.z(),
                        offset + idx,
                    )?;
                }

                // For each chunk, lookup the (chunk, endoscalar) pair.
                for (idx, pub_input_row) in pub_input_rows.iter().rev().enumerate() {
                    self.q_lookup.enable(&mut region, offset + idx)?;

                    // Copy endoscalar from given row on instance column
                    region.assign_advice_from_instance(
                        || format!("Endoscalar at row {:?}", pub_input_row),
                        self.endoscalars,
                        *pub_input_row,
                        self.endoscalars_copy,
                        offset + idx,
                    )?;
                }

                Ok(())
            },
        )
    }
}
