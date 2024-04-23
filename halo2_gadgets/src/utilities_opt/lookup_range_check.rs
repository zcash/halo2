//! Make use of a K-bit lookup table to decompose a field element into K-bit
//! words.

use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{AssignedCell, Region},
    plonk::{
        Advice, Column, ConstraintSystem, Constraints, Error, Expression, Selector, TableColumn,
    },
    poly::Rotation,
};

#[cfg(test)]
use halo2_proofs::circuit::{Layouter, Value};

use ff::PrimeFieldBits;

use pasta_curves::pallas;

use crate::{
    sinsemilla::primitives as sinsemilla,
    utilities::lookup_range_check::{
        DefaultLookupRangeCheck, LookupRangeCheck, LookupRangeCheckConfig,
    },
};

/// Configuration that provides methods for a lookup range check.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct LookupRangeCheckConfigOptimized<F: PrimeFieldBits, const K: usize> {
    base: LookupRangeCheckConfig<F, K>,
    q_range_check_4: Selector,
    q_range_check_5: Selector,
    // FIXME: Instead of making it pub, add a method in LookupRangeCheckConfig that returns table_range_check_tag
    //pub(crate)
    table_range_check_tag: TableColumn,
}

impl<F: PrimeFieldBits, const K: usize> LookupRangeCheckConfigOptimized<F, K> {
    /// The `running_sum` advice column breaks the field element into `K`-bit
    /// words. It is used to construct the input expression to the lookup
    /// argument.
    ///
    /// The `table_idx` fixed column contains values from [0..2^K). Looking up
    /// a value in `table_idx` constrains it to be within this range. The table
    /// can be loaded outside this helper.
    ///
    /// # Side-effects
    ///
    /// Both the `running_sum` and `constants` columns will be equality-enabled.
    fn configure_with_tag(
        meta: &mut ConstraintSystem<F>,
        running_sum: Column<Advice>,
        table_idx: TableColumn,
        table_range_check_tag: TableColumn,
    ) -> Self {
        meta.enable_equality(running_sum);

        let q_lookup = meta.complex_selector();
        let q_running = meta.complex_selector();
        let q_bitshift = meta.selector();

        let q_range_check_4 = meta.complex_selector();
        let q_range_check_5 = meta.complex_selector();

        // FIXME: q_range_check_4 and q_range_check_5 need to be created here
        // if the order of the creation makes a difference
        let config = LookupRangeCheckConfigOptimized {
            base: LookupRangeCheckConfig {
                q_lookup,
                q_running,
                q_bitshift,
                running_sum,
                table_idx,
                _marker: PhantomData,
            },
            q_range_check_4,
            q_range_check_5,
            table_range_check_tag,
        };

        // https://p.z.cash/halo2-0.1:decompose-combined-lookup
        meta.lookup(|meta| {
            let q_lookup = meta.query_selector(config.base.q_lookup);
            let q_running = meta.query_selector(config.base.q_running);
            // FIXME: q_range_check_4 and q_range_check_5 need to be created here
            // if the order of the creation makes a difference
            let z_cur = meta.query_advice(config.base.running_sum, Rotation::cur());
            let one = Expression::Constant(F::ONE);

            // In the case of a running sum decomposition, we recover the word from
            // the difference of the running sums:
            //    z_i = 2^{K}⋅z_{i + 1} + a_i
            // => a_i = z_i - 2^{K}⋅z_{i + 1}
            let running_sum_lookup = {
                let running_sum_word = {
                    let z_next = meta.query_advice(config.base.running_sum, Rotation::next());
                    z_cur.clone() - z_next * F::from(1 << K)
                };

                q_running.clone() * running_sum_word
            };

            // In the short range check, the word is directly witnessed.
            let short_lookup = {
                let short_word = z_cur.clone();
                let q_short = one.clone() - q_running;

                q_short * short_word
            };

            let q_range_check_4 = meta.query_selector(config.q_range_check_4);
            let q_range_check_5 = meta.query_selector(config.q_range_check_5);

            // q_range_check is equal to
            // - 1 if q_range_check_4 = 1 or q_range_check_5 = 1
            // - 0 otherwise
            let q_range_check = one.clone()
                - (one.clone() - q_range_check_4.clone()) * (one.clone() - q_range_check_5.clone());

            // num_bits is equal to
            // - 5 if q_range_check_5 = 1
            // - 4 if q_range_check_4 = 1 and q_range_check_5 = 0
            // - 0 otherwise
            let num_bits = q_range_check_5.clone() * Expression::Constant(F::from(5_u64))
                + (one.clone() - q_range_check_5)
                    * q_range_check_4
                    * Expression::Constant(F::from(4_u64));

            // Combine the running sum, short lookups and optimized range checks:
            vec![
                (
                    q_lookup.clone()
                        * ((one - q_range_check.clone()) * (running_sum_lookup + short_lookup)
                            + q_range_check.clone() * z_cur),
                    config.base.table_idx,
                ),
                (
                    q_lookup * q_range_check * num_bits,
                    config.table_range_check_tag,
                ),
            ]
        });

        // For short lookups, check that the word has been shifted by the correct number of bits.
        // https://p.z.cash/halo2-0.1:decompose-short-lookup
        meta.create_gate("Short lookup bitshift", |meta| {
            let q_bitshift = meta.query_selector(config.base.q_bitshift);
            let word = meta.query_advice(config.base.running_sum, Rotation::prev());
            let shifted_word = meta.query_advice(config.base.running_sum, Rotation::cur());
            let inv_two_pow_s = meta.query_advice(config.base.running_sum, Rotation::next());

            let two_pow_k = F::from(1 << K);

            // shifted_word = word * 2^{K-s}
            //              = word * 2^K * inv_two_pow_s
            Constraints::with_selector(
                q_bitshift,
                Some(word * two_pow_k * inv_two_pow_s - shifted_word),
            )
        });

        config
    }

    pub(crate) fn table_range_check_tag(&self) -> TableColumn {
        self.table_range_check_tag
    }
}

impl<F: PrimeFieldBits, const K: usize> LookupRangeCheck<F, K>
    for LookupRangeCheckConfigOptimized<F, K>
{
    fn config(&self) -> &LookupRangeCheckConfig<F, K> {
        &self.base
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        running_sum: Column<Advice>,
        table_idx: TableColumn,
    ) -> Self {
        let table_range_check_tag = meta.lookup_table_column();
        Self::configure_with_tag(meta, running_sum, table_idx, table_range_check_tag)
    }

    #[cfg(test)]
    // Fill `table_idx` and `table_range_check_tag`.
    // This is only used in testing for now, since the Sinsemilla chip provides a pre-loaded table
    // in the Orchard context.
    fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "table_idx",
            |mut table| {
                let mut assign_cells =
                    |offset: usize, table_size, value: u64| -> Result<usize, Error> {
                        for index in 0..table_size {
                            let new_index = index + offset;
                            table.assign_cell(
                                || "table_idx",
                                self.base.table_idx,
                                new_index,
                                || Value::known(F::from(index as u64)),
                            )?;
                            table.assign_cell(
                                || "table_range_check_tag",
                                self.table_range_check_tag,
                                new_index,
                                || Value::known(F::from(value)),
                            )?;
                        }
                        Ok(offset + table_size)
                    };

                // We generate the row values lazily (we only need them during keygen).
                let mut offset = 0;

                //annotation: &'v (dyn Fn() -> String + 'v),
                //column: TableColumn,
                //offset: usize,
                //to: &'v mut (dyn FnMut() -> Value<Assigned<F>> + 'v),

                offset = assign_cells(offset, 1 << K, 0)?;
                offset = assign_cells(offset, 1 << 4, 4)?;
                assign_cells(offset, 1 << 5, 5)?;

                Ok(())
            },
        )
    }

    /// Constrain `x` to be a NUM_BITS word.
    ///
    /// `element` must have been assigned to `self.running_sum` at offset 0.
    fn short_range_check(
        &self,
        region: &mut Region<'_, F>,
        element: AssignedCell<F, F>,
        num_bits: usize,
    ) -> Result<(), Error> {
        match num_bits {
            4 => {
                self.base.q_lookup.enable(region, 0)?;
                self.q_range_check_4.enable(region, 0)?;
                Ok(())
            }

            5 => {
                self.base.q_lookup.enable(region, 0)?;
                self.q_range_check_5.enable(region, 0)?;
                Ok(())
            }

            _ => self.base.short_range_check(region, element, num_bits),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LookupRangeCheck, LookupRangeCheckConfigOptimized};

    use crate::sinsemilla::primitives::K;
    use crate::utilities::lebs2ip;

    use ff::{Field, PrimeFieldBits};
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::{FailureLocation, MockProver, VerifyFailure},
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::pallas;

    use std::{convert::TryInto, marker::PhantomData};

    #[test]
    fn lookup_range_check() {
        #[derive(Clone, Copy)]
        struct MyCircuit<F: PrimeFieldBits> {
            num_words: usize,
            _marker: PhantomData<F>,
        }

        impl<F: PrimeFieldBits> Circuit<F> for MyCircuit<F> {
            type Config = LookupRangeCheckConfigOptimized<F, K>;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                *self
            }

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let running_sum = meta.advice_column();
                let table_idx = meta.lookup_table_column();
                let table_range_check_tag = meta.lookup_table_column();
                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                LookupRangeCheckConfigOptimized::<F, K>::configure_with_tag(
                    meta,
                    running_sum,
                    table_idx,
                    table_range_check_tag,
                )
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<F>,
            ) -> Result<(), Error> {
                // Load table_idx
                config.load(&mut layouter)?;

                // Lookup constraining element to be no longer than num_words * K bits.
                let elements_and_expected_final_zs = [
                    (F::from((1 << (self.num_words * K)) - 1), F::ZERO, true), // a word that is within self.num_words * K bits long
                    (F::from(1 << (self.num_words * K)), F::ONE, false), // a word that is just over self.num_words * K bits long
                ];

                fn expected_zs<F: PrimeFieldBits, const K: usize>(
                    element: F,
                    num_words: usize,
                ) -> Vec<F> {
                    let chunks = {
                        element
                            .to_le_bits()
                            .iter()
                            .by_vals()
                            .take(num_words * K)
                            .collect::<Vec<_>>()
                            .chunks_exact(K)
                            .map(|chunk| F::from(lebs2ip::<K>(chunk.try_into().unwrap())))
                            .collect::<Vec<_>>()
                    };
                    let expected_zs = {
                        let inv_two_pow_k = F::from(1 << K).invert().unwrap();
                        chunks.iter().fold(vec![element], |mut zs, a_i| {
                            // z_{i + 1} = (z_i - a_i) / 2^{K}
                            let z = (zs[zs.len() - 1] - a_i) * inv_two_pow_k;
                            zs.push(z);
                            zs
                        })
                    };
                    expected_zs
                }

                for (element, expected_final_z, strict) in elements_and_expected_final_zs.iter() {
                    let expected_zs = expected_zs::<F, K>(*element, self.num_words);

                    let zs = config.witness_check(
                        layouter.namespace(|| format!("Lookup {:?}", self.num_words)),
                        Value::known(*element),
                        self.num_words,
                        *strict,
                    )?;

                    assert_eq!(*expected_zs.last().unwrap(), *expected_final_z);

                    for (expected_z, z) in expected_zs.into_iter().zip(zs.iter()) {
                        z.value().assert_if_known(|z| &&expected_z == z);
                    }
                }
                Ok(())
            }
        }

        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                num_words: 6,
                _marker: PhantomData,
            };

            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }

    #[test]
    fn short_range_check() {
        struct MyCircuit<F: PrimeFieldBits> {
            element: Value<F>,
            num_bits: usize,
        }

        impl<F: PrimeFieldBits> Circuit<F> for MyCircuit<F> {
            type Config = LookupRangeCheckConfigOptimized<F, K>;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                MyCircuit {
                    element: Value::unknown(),
                    num_bits: self.num_bits,
                }
            }

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let running_sum = meta.advice_column();
                let table_idx = meta.lookup_table_column();
                let table_range_check_tag = meta.lookup_table_column();
                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                LookupRangeCheckConfigOptimized::<F, K>::configure_with_tag(
                    meta,
                    running_sum,
                    table_idx,
                    table_range_check_tag,
                )
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<F>,
            ) -> Result<(), Error> {
                // Load table_idx
                config.load(&mut layouter)?;

                // Lookup constraining element to be no longer than num_bits.
                config.witness_short_check(
                    layouter.namespace(|| format!("Lookup {:?} bits", self.num_bits)),
                    self.element,
                    self.num_bits,
                )?;

                Ok(())
            }
        }

        // Edge case: zero bits
        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(pallas::Base::ZERO),
                num_bits: 0,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // Edge case: K bits
        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(pallas::Base::from((1 << K) - 1)),
                num_bits: K,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // Element within `num_bits`
        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(pallas::Base::from((1 << 6) - 1)),
                num_bits: 6,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // Element larger than `num_bits` but within K bits
        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(pallas::Base::from(1 << 6)),
                num_bits: 6,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Err(vec![VerifyFailure::Lookup {
                    lookup_index: 0,
                    location: FailureLocation::InRegion {
                        region: (1, "Range check 6 bits").into(),
                        offset: 1,
                    },
                }])
            );
        }

        // Element larger than K bits
        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(pallas::Base::from(1 << K)),
                num_bits: 6,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Err(vec![
                    VerifyFailure::Lookup {
                        lookup_index: 0,
                        location: FailureLocation::InRegion {
                            region: (1, "Range check 6 bits").into(),
                            offset: 0,
                        },
                    },
                    VerifyFailure::Lookup {
                        lookup_index: 0,
                        location: FailureLocation::InRegion {
                            region: (1, "Range check 6 bits").into(),
                            offset: 1,
                        },
                    },
                ])
            );
        }

        // Element which is not within `num_bits`, but which has a shifted value within
        // num_bits
        {
            let num_bits = 6;
            let shifted = pallas::Base::from((1 << num_bits) - 1);
            // Recall that shifted = element * 2^{K-s}
            //          => element = shifted * 2^{s-K}
            let element = shifted
                * pallas::Base::from(1 << (K as u64 - num_bits))
                    .invert()
                    .unwrap();
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(element),
                num_bits: num_bits as usize,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Err(vec![VerifyFailure::Lookup {
                    lookup_index: 0,
                    location: FailureLocation::InRegion {
                        region: (1, "Range check 6 bits").into(),
                        offset: 0,
                    },
                }])
            );
        }

        // Element within 4 bits
        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(pallas::Base::from((1 << 4) - 1)),
                num_bits: 4,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }

        // Element larger than 5 bits
        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                element: Value::known(pallas::Base::from(1 << 5)),
                num_bits: 5,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Err(vec![VerifyFailure::Lookup {
                    lookup_index: 0,
                    location: FailureLocation::InRegion {
                        region: (1, "Range check 5 bits").into(),
                        offset: 0,
                    },
                }])
            );
        }
    }
}

pub(crate) type DefaultLookupRangeCheckConfigOptimized =
    LookupRangeCheckConfigOptimized<pallas::Base, { sinsemilla::K }>;

impl DefaultLookupRangeCheck for DefaultLookupRangeCheckConfigOptimized {}
