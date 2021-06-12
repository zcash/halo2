//! Make use of a K-bit lookup table to decompose a field element into K-bit
//! words.

use crate::primitives::sinsemilla::lebs2ip_k;
use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

use ff::PrimeFieldBits;

use super::*;

pub trait LookupRangeCheckInstructions<
    F: FieldExt + PrimeFieldBits + PrimeFieldBits,
    const K: usize,
>: UtilitiesInstructions<F>
{
    /// Decomposes a field element into K-bit words using a running sum.
    /// Constrains each word to K bits using a lookup table.
    ///
    /// Panics
    ///
    /// Panics if `num_words` is greater than F::NUM_BITS / K, i.e. there are
    /// more words than can fit in a field element.
    #[allow(clippy::type_complexity)]
    fn lookup_range_check(
        &self,
        layouter: impl Layouter<F>,
        words: CellValue<F>,
        num_words: usize,
    ) -> Result<Vec<CellValue<F>>, Error>;
}

#[derive(Clone, Debug)]
pub struct LookupRangeCheckChip<F: FieldExt + PrimeFieldBits + PrimeFieldBits, const K: usize> {
    config: LookupRangeCheckConfig<K>,
    _marker: PhantomData<F>,
}

impl<F: FieldExt + PrimeFieldBits + PrimeFieldBits, const K: usize> Chip<F>
    for LookupRangeCheckChip<F, K>
{
    type Config = LookupRangeCheckConfig<K>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

#[derive(Debug, Clone)]
pub struct LookupRangeCheckConfig<const K: usize> {
    q_lookup: Selector,
    q_decompose: Selector,
    running_sum: Column<Advice>,
    table_idx: Column<Fixed>,
    perm: Permutation,
}

impl<F: FieldExt + PrimeFieldBits + PrimeFieldBits, const K: usize> LookupRangeCheckChip<F, K> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        running_sum: Column<Advice>,
        table_idx: Column<Fixed>,
        perm: Permutation,
    ) -> LookupRangeCheckConfig<K> {
        let q_lookup = meta.selector();
        let q_decompose = meta.selector();

        let config = LookupRangeCheckConfig {
            q_lookup,
            q_decompose,
            running_sum,
            table_idx,
            perm,
        };

        meta.lookup(|meta| {
            let q_lookup = meta.query_selector(config.q_lookup);
            let z_cur = meta.query_advice(config.running_sum, Rotation::cur());
            let z_next = meta.query_advice(config.running_sum, Rotation::next());
            //    z_i = 2^{K}⋅z_{i + 1} + a_i
            // => a_i = z_i - 2^{K}⋅z_{i + 1}
            let word = z_cur - z_next * F::from_u64(1 << K);
            let table = meta.query_fixed(config.table_idx, Rotation::cur());

            vec![(q_lookup * word, table)]
        });

        config
    }

    pub fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        let config = self.config().clone();
        layouter.assign_region(
            || "table_idx",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                for index in 0..(1 << K) {
                    gate.assign_fixed(
                        || "table_idx",
                        config.table_idx,
                        index,
                        || Ok(F::from_u64(index as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }

    pub fn construct(config: LookupRangeCheckConfig<K>) -> Self {
        LookupRangeCheckChip {
            config,
            _marker: PhantomData,
        }
    }
}

impl<F: FieldExt + PrimeFieldBits + PrimeFieldBits, const K: usize> UtilitiesInstructions<F>
    for LookupRangeCheckChip<F, K>
{
    type Var = CellValue<F>;
}

impl<F: FieldExt + PrimeFieldBits + PrimeFieldBits, const K: usize>
    LookupRangeCheckInstructions<F, K> for LookupRangeCheckChip<F, K>
{
    fn lookup_range_check(
        &self,
        mut layouter: impl Layouter<F>,
        words: CellValue<F>,
        num_words: usize,
    ) -> Result<Vec<CellValue<F>>, Error> {
        // `num_words` must fit into a single field element.
        assert!(num_words <= F::NUM_BITS as usize / K);
        let num_bits = num_words * K;

        let config = self.config().clone();

        // Take first num_bits bits of `words`.
        let bits = words.value().map(|words| {
            words
                .to_le_bits()
                .into_iter()
                .take(num_bits)
                .collect::<Vec<_>>()
        });

        // Chunk the first num_bits bits into K-bit chunks.
        let bits: Option<Vec<Vec<bool>>> = bits.map(|bits| {
            bits.chunks_exact(K)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>()
        });

        let bits = if let Some(bits) = bits {
            bits.into_iter().map(Some).collect()
        } else {
            vec![None; num_words]
        };

        layouter.assign_region(
            || format!("{}-bit decomposition", K),
            |mut region| {
                let offset = 0;

                // Copy `words` and initialize running sum `z_0 = words` to decompose it.
                let z_0 = copy(
                    &mut region,
                    || "copy words",
                    config.running_sum,
                    offset,
                    &words,
                    &config.perm,
                )?;

                let mut zs = vec![z_0];

                // Assign cumulative sum such that
                //          z_i = 2^{K}⋅z_{i + 1} + a_i
                // => z_{i + 1} = (z_i - a_i) / (2^K)
                //
                // For `words` = a_0 + 2^10 a_1 + ... + 2^{120} a_{12}}, initialize z_0 = `words`.
                // If `words` fits in 130 bits, we end up with z_{13} = 0.
                let mut z = z_0;
                let inv_2_pow_k = F::from_u64(1u64 << K).invert().unwrap();
                for (idx, word) in bits.iter().enumerate() {
                    let word = word
                        .clone()
                        .map(|word| F::from_u64(lebs2ip_k(&word) as u64));

                    // Enable selector to activate lookup.
                    config.q_lookup.enable(&mut region, offset + idx)?;

                    // z_next = (z_cur - m_cur) / 2^K
                    z = {
                        let z_val = z
                            .value()
                            .zip(word)
                            .map(|(z, word)| (z - word) * inv_2_pow_k);

                        // Assign z_next
                        let z_cell = region.assign_advice(
                            || format!("z_{:?}", idx + 1),
                            config.running_sum,
                            offset + idx + 1,
                            || z_val.ok_or(Error::SynthesisError),
                        )?;

                        CellValue::new(z_cell, z_val)
                    };
                    zs.push(z);
                }

                Ok(zs)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::{UtilitiesInstructions, Var};
    use super::{LookupRangeCheckChip, LookupRangeCheckConfig, LookupRangeCheckInstructions};
    use crate::primitives::sinsemilla::{lebs2ip_k, K};
    use ff::PrimeFieldBits;
    use halo2::{
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::MockProver,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use std::marker::PhantomData;

    #[test]
    fn lookup_range_check() {
        struct MyCircuit<F: FieldExt + PrimeFieldBits> {
            _marker: PhantomData<F>,
        }

        impl<F: FieldExt + PrimeFieldBits> Circuit<F> for MyCircuit<F> {
            type Config = LookupRangeCheckConfig<K>;

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let running_sum = meta.advice_column();
                let table_idx = meta.fixed_column();

                let perm = meta.permutation(&[running_sum.into()]);

                LookupRangeCheckChip::<F, K>::configure(meta, running_sum, table_idx, perm)
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                config: Self::Config,
            ) -> Result<(), Error> {
                let mut layouter = SingleChipLayouter::new(cs)?;

                let chip = LookupRangeCheckChip::<F, K>::construct(config.clone());

                // Load table_idx
                chip.load(&mut layouter)?;

                let num_words = 6;

                // Test a word that is within num_words * K bits long.
                {
                    let words = F::from_u64((1 << (num_words * K)) - 1);
                    let expected_zs = expected_zs::<F, K>(words, num_words);

                    // Load the value to be decomposed into the circuit.
                    let words = chip.load_private(
                        layouter.namespace(|| "words"),
                        config.running_sum,
                        Some(words),
                    )?;
                    let zs = chip.lookup_range_check(
                        layouter.namespace(|| "range check"),
                        words,
                        num_words,
                    )?;

                    assert_eq!(expected_zs[expected_zs.len() - 1], F::zero());

                    for (expected_z, z) in expected_zs.into_iter().zip(zs.iter()) {
                        if let Some(z) = z.value() {
                            assert_eq!(expected_z, z);
                        }
                    }
                }

                // Test a word that is just over num_words * K bits long.
                {
                    let words = F::from_u64(1 << (num_words * K));
                    let expected_zs = expected_zs::<F, K>(words, num_words);

                    // Load the value to be decomposed into the circuit.
                    let words = chip.load_private(
                        layouter.namespace(|| "words"),
                        config.running_sum,
                        Some(words),
                    )?;
                    let zs = chip.lookup_range_check(
                        layouter.namespace(|| "range check"),
                        words,
                        num_words,
                    )?;

                    assert_eq!(expected_zs[expected_zs.len() - 1], F::one());

                    for (expected_z, z) in expected_zs.into_iter().zip(zs.iter()) {
                        if let Some(z) = z.value() {
                            assert_eq!(expected_z, z);
                        }
                    }
                }

                Ok(())
            }
        }

        {
            let circuit: MyCircuit<pallas::Base> = MyCircuit {
                _marker: PhantomData,
            };
            let prover = MockProver::<pallas::Base>::run(11, &circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }

    #[cfg(test)]
    fn expected_zs<F: FieldExt + PrimeFieldBits, const K: usize>(
        words: F,
        num_words: usize,
    ) -> Vec<F> {
        let chunks = {
            words
                .to_le_bits()
                .iter()
                .by_val()
                .take(num_words * K)
                .collect::<Vec<_>>()
                .chunks_exact(K)
                .collect::<Vec<_>>()
                .iter()
                .map(|chunk| {
                    let int = lebs2ip_k(&chunk);
                    F::from_u64(int as u64)
                })
                .collect::<Vec<_>>()
        };
        let expected_zs = {
            let inv_2_pow_k = F::from_u64(1u64 << K).invert().unwrap();
            chunks.iter().fold(vec![words], |mut zs, a_i| {
                // z_{i + 1} = (z_i - a_i) / 2^{K}
                let z = (zs[zs.len() - 1] - a_i) * inv_2_pow_k;
                zs.push(z);
                zs
            })
        };
        expected_zs
    }
}
