//! Make use of a K-bit lookup table to decompose a field element into K-bit
//! words.

use crate::primitives::sinsemilla::lebs2ip_k;
use halo2::{
    circuit::{Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation},
    poly::Rotation,
};
use std::marker::PhantomData;

use ff::PrimeFieldBits;

use super::*;

#[derive(Debug, Clone)]
pub struct LookupRangeCheckConfig<F: FieldExt + PrimeFieldBits, const K: usize> {
    q_lookup: Column<Fixed>, // This is passed in.
    running_sum: Column<Advice>,
    table_idx: Column<Fixed>,
    perm: Permutation,
    _marker: PhantomData<F>,
}

impl<F: FieldExt + PrimeFieldBits, const K: usize> LookupRangeCheckConfig<F, K> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        q_lookup: Column<Fixed>,
        running_sum: Column<Advice>,
        table_idx: Column<Fixed>,
        perm: Permutation,
    ) -> Self {
        let config = LookupRangeCheckConfig {
            q_lookup,
            running_sum,
            table_idx,
            perm,
            _marker: PhantomData,
        };

        meta.lookup(|meta| {
            let q_lookup = meta.query_fixed(config.q_lookup, Rotation::cur());
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
        layouter.assign_region(
            || "table_idx",
            |mut gate| {
                // We generate the row values lazily (we only need them during keygen).
                for index in 0..(1 << K) {
                    gate.assign_fixed(
                        || "table_idx",
                        self.table_idx,
                        index,
                        || Ok(F::from_u64(index as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }

    fn lookup_range_check(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        words: CellValue<F>,
        num_words: usize,
    ) -> Result<Vec<CellValue<F>>, Error> {
        // `num_words` must fit into a single field element.
        assert!(num_words <= F::NUM_BITS as usize / K);
        let num_bits = num_words * K;

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

        // Copy `words` and initialize running sum `z_0 = words` to decompose it.
        let z_0 = copy(
            region,
            || "copy words",
            self.running_sum,
            offset,
            &words,
            &self.perm,
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

            // z_next = (z_cur - m_cur) / 2^K
            z = {
                let z_val = z
                    .value()
                    .zip(word)
                    .map(|(z, word)| (z - word) * inv_2_pow_k);

                // Assign z_next
                let z_cell = region.assign_advice(
                    || format!("z_{:?}", idx + 1),
                    self.running_sum,
                    offset + idx + 1,
                    || z_val.ok_or(Error::SynthesisError),
                )?;

                CellValue::new(z_cell, z_val)
            };
            zs.push(z);
        }

        Ok(zs)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CellValue, UtilitiesInstructions, Var};
    use super::LookupRangeCheckConfig;
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

        impl<F: FieldExt + PrimeFieldBits> UtilitiesInstructions<F> for MyCircuit<F> {
            type Var = CellValue<F>;
        }

        impl<F: FieldExt + PrimeFieldBits> Circuit<F> for MyCircuit<F> {
            type Config = LookupRangeCheckConfig<F, K>;

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let running_sum = meta.advice_column();
                let table_idx = meta.fixed_column();
                let q_lookup = meta.fixed_column();

                let perm = meta.permutation(&[running_sum.into()]);

                LookupRangeCheckConfig::<F, K>::configure(
                    meta,
                    q_lookup,
                    running_sum,
                    table_idx,
                    perm,
                )
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                config: Self::Config,
            ) -> Result<(), Error> {
                let mut layouter = SingleChipLayouter::new(cs)?;

                // Load table_idx
                config.load(&mut layouter)?;

                let num_words = 6;
                let words_and_expected_final_zs = [
                    (F::from_u64((1 << (num_words * K)) - 1), F::zero()), // a word that is within num_words * K bits long
                    (F::from_u64(1 << (num_words * K)), F::one()), // a word that is just over num_words * K bits long
                ];

                for (words, expected_final_z) in words_and_expected_final_zs.iter() {
                    let expected_zs = expected_zs::<F, K>(*words, num_words);

                    // Load the value to be decomposed into the circuit.
                    let words = self.load_private(
                        layouter.namespace(|| "words"),
                        config.running_sum,
                        Some(*words),
                    )?;

                    let zs = layouter.assign_region(
                        || "word within range",
                        |mut region| {
                            for idx in 0..num_words {
                                // Assign fixed column to activate lookup.
                                region.assign_fixed(
                                    || format!("lookup on row {}", idx),
                                    config.q_lookup,
                                    idx,
                                    || Ok(F::one()),
                                )?;
                            }

                            config.lookup_range_check(&mut region, 0, words, num_words)
                        },
                    )?;

                    assert_eq!(expected_zs[expected_zs.len() - 1], *expected_final_z);

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
