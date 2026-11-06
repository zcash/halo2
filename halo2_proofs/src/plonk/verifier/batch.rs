use group::{
    Group,
    ff::{Field, FromUniformBytes, PrimeField},
};
use pasta_curves::arithmetic::{CurveAffine, CurveExt};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;
use std::sync::Arc;

use super::{VerificationStrategy, validate_instances, verify_proof_with_instance_commitments};
use crate::{
    INSTANCE_WINDOW_BITS, INSTANCE_WINDOW_ENTRIES_PER_BASE, InstanceWindowTable,
    MAX_CACHED_INSTANCE_ROWS,
    multicore::{IntoParallelIterator, TryFoldAndReduce},
    plonk::{Error, VerifyingKey},
    poly::commitment::{Guard, Params, MSM},
    transcript::{Blake2bRead, EncodedChallenge},
};

const SERIAL_INSTANCE_WINDOW_BITS: usize = 8;
// Lockstep bookkeeping pays off with three or more instance columns.
const MIN_LOCKSTEP_INSTANCE_COLUMNS: usize = 3;

#[cfg(feature = "multicore")]
use crate::multicore::{IndexedParallelIterator, ParallelIterator};

/// A proof verification strategy that returns the proof's MSM.
///
/// `BatchVerifier` handles the accumulation of the MSMs for the batched proofs.
#[derive(Debug)]
struct BatchStrategy<'params, C: CurveAffine> {
    msm: MSM<'params, C>,
    batching_scalar: C::Scalar,
}

impl<'params, C: CurveAffine> BatchStrategy<'params, C> {
    fn new(params: &'params Params<C>, batching_scalar: C::Scalar) -> Self {
        BatchStrategy {
            msm: MSM::new(params),
            batching_scalar,
        }
    }
}

impl<'params, C: CurveAffine> VerificationStrategy<'params, C> for BatchStrategy<'params, C> {
    type Output = MSM<'params, C>;

    fn process<E: EncodedChallenge<C>>(
        self,
        f: impl FnOnce(MSM<'params, C>) -> Result<Guard<'params, C, E>, Error>,
    ) -> Result<Self::Output, Error> {
        let guard = f(self.msm)?;
        Ok(guard.use_challenges_with_scale(self.batching_scalar))
    }
}

#[derive(Debug)]
struct BatchItem<C: CurveAffine> {
    instances: Vec<Vec<Vec<C::Scalar>>>,
    proof: Vec<u8>,
}

struct InstanceFixedWindowTable<C: CurveAffine> {
    base_count: usize,
    multiples: Arc<Vec<C>>,
}

impl<C: CurveAffine> InstanceFixedWindowTable<C> {
    fn new(params: &Params<C>, base_count: usize) -> Self {
        Self {
            base_count,
            multiples: params.instance_window_table(base_count),
        }
    }

    fn commit(&self, params: &Params<C>, scalars: &[C::Scalar]) -> C::Curve {
        assert!(scalars.len() <= self.base_count);

        let window_count = (C::Scalar::NUM_BITS as usize).div_ceil(SERIAL_INSTANCE_WINDOW_BITS);
        let scalar_reprs = scalars.iter().map(PrimeField::to_repr).collect::<Vec<_>>();
        let mut variable = C::Curve::identity();

        for window in (0..window_count).rev() {
            if window + 1 != window_count {
                for _ in 0..SERIAL_INSTANCE_WINDOW_BITS {
                    variable = variable.double();
                }
            }

            for (base_index, scalar) in scalar_reprs.iter().enumerate() {
                let digit = unsigned_fixed_window_digit(
                    scalar.as_ref(),
                    window,
                    SERIAL_INSTANCE_WINDOW_BITS,
                );
                if digit != 0 {
                    variable +=
                        self.multiples[base_index * INSTANCE_WINDOW_ENTRIES_PER_BASE + digit - 1];
                }
            }
        }

        let mut commitment = C::Curve::from(params.w);
        commitment += variable;
        commitment
    }

    fn commit_batch(&self, params: &Params<C>, instances: &[&[C::Scalar]]) -> Vec<C::Curve> {
        if instances.len() < MIN_LOCKSTEP_INSTANCE_COLUMNS {
            return instances
                .iter()
                .map(|instance| {
                    if instance.len() <= self.base_count {
                        self.commit(params, instance)
                    } else {
                        params.commit_lagrange_instance(instance)
                    }
                })
                .collect();
        }

        let mut commitments = vec![C::Curve::identity(); instances.len()];
        let cached_instances = instances
            .iter()
            .enumerate()
            .filter_map(|(output_index, instance)| {
                if instance.len() <= self.base_count {
                    Some((output_index, *instance))
                } else {
                    commitments[output_index] = params.commit_lagrange_instance(instance);
                    None
                }
            })
            .collect::<Vec<_>>();
        let cached_count = cached_instances.len();
        // Store representations base-major so every inner loop reads a
        // contiguous row while updating independent output accumulators.
        let mut scalar_reprs = vec![C::Scalar::ZERO.to_repr(); self.base_count * cached_count];
        for (cached_index, (_, instance)) in cached_instances.iter().enumerate() {
            for (base_index, scalar) in instance.iter().enumerate() {
                scalar_reprs[base_index * cached_count + cached_index] = scalar.to_repr();
            }
        }

        let window_count = signed_instance_window_count(C::Scalar::NUM_BITS as usize);
        for window in (0..window_count).rev() {
            if window + 1 != window_count {
                for _ in 0..INSTANCE_WINDOW_BITS {
                    for (output_index, _) in &cached_instances {
                        commitments[*output_index] = commitments[*output_index].double();
                    }
                }
            }

            for base_index in 0..self.base_count {
                for (cached_index, (output_index, _)) in cached_instances.iter().enumerate() {
                    let scalar = &scalar_reprs[base_index * cached_count + cached_index];
                    let digit =
                        signed_fixed_window_digit(scalar.as_ref(), window, INSTANCE_WINDOW_BITS);
                    if digit.magnitude != 0 {
                        let mut multiple = self.multiples
                            [base_index * INSTANCE_WINDOW_ENTRIES_PER_BASE + digit.magnitude - 1];
                        if digit.negative {
                            multiple = -multiple;
                        }
                        commitments[*output_index] += multiple;
                    }
                }
            }
        }

        let blind = C::Curve::from(params.w);
        for (output_index, _) in cached_instances {
            commitments[output_index] += blind;
        }
        commitments
    }

    #[cfg(test)]
    fn retained_bytes(&self) -> usize {
        self.multiples.len() * core::mem::size_of::<C>()
    }
}

#[cfg(test)]
fn commit_instance_with_table<C: CurveAffine>(
    params: &Params<C>,
    table: &InstanceFixedWindowTable<C>,
    instance: &[C::Scalar],
) -> C::Curve {
    if instance.len() <= table.base_count {
        table.commit(params, instance)
    } else {
        params.commit_lagrange_instance(instance)
    }
}

#[derive(Clone, Copy)]
struct FixedWindowDigit {
    magnitude: usize,
    negative: bool,
}

fn signed_instance_window_count(scalar_bits: usize) -> usize {
    // A full high window can carry into one more window. A partial high
    // window cannot carry, so it already occupies this final slot.
    scalar_bits / INSTANCE_WINDOW_BITS + 1
}

fn unsigned_fixed_window_digit(bytes: &[u8], window: usize, window_bits: usize) -> usize {
    let bit_start = window * window_bits;
    let byte_start = bit_start / u8::BITS as usize;
    let bit_offset = bit_start % u8::BITS as usize;
    let low = bytes.get(byte_start).copied().unwrap_or(0);
    let high = bytes.get(byte_start + 1).copied().unwrap_or(0);
    let encoded = u16::from(low) | (u16::from(high) << u8::BITS);
    usize::from((encoded >> bit_offset) & ((1 << window_bits) - 1))
}

fn signed_fixed_window_digit(bytes: &[u8], window: usize, window_bits: usize) -> FixedWindowDigit {
    let bit_start = window * window_bits;
    let byte_start = bit_start / u8::BITS as usize;
    let bit_offset = bit_start % u8::BITS as usize;
    let low = bytes.get(byte_start).copied().unwrap_or(0);
    let high = bytes.get(byte_start + 1).copied().unwrap_or(0);
    let encoded = u16::from(low) | (u16::from(high) << u8::BITS);
    let radix = 1usize << window_bits;
    let value = usize::from(encoded >> bit_offset) & (radix - 1);
    let overlap = if bit_start == 0 {
        0
    } else {
        let bit = bit_start - 1;
        bytes.get(bit / u8::BITS as usize).map_or(0, |byte| {
            usize::from((byte >> (bit % u8::BITS as usize)) & 1)
        })
    };

    // The bit below each window is its carry-in, while the window's high bit
    // is its carry-out. These cancel between adjacent windows.
    if value < radix / 2 {
        FixedWindowDigit {
            magnitude: value + overlap,
            negative: false,
        }
    } else {
        let magnitude = radix - value - overlap;
        FixedWindowDigit {
            magnitude,
            negative: magnitude != 0,
        }
    }
}

fn compute_batch_instance_commitments<C: CurveAffine>(
    params: &Params<C>,
    vk: &VerifyingKey<C>,
    items: &[BatchItem<C>],
) -> Result<Vec<Vec<Vec<C>>>, Error> {
    let mut item_column_counts = Vec::with_capacity(items.len());
    let mut max_cached_instance_len = 0;

    for item in items {
        let instance_columns = item
            .instances
            .iter()
            .map(|instances| instances.iter().map(Vec::as_slice).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let instances = instance_columns
            .iter()
            .map(Vec::as_slice)
            .collect::<Vec<_>>();
        validate_instances(params, vk, &instances)?;

        // Table construction costs INSTANCE_WINDOW_ENTRIES_PER_BASE points
        // per row. Longer, caller-sized columns use the generic MSM below.
        max_cached_instance_len = item
            .instances
            .iter()
            .flat_map(|instances| instances.iter())
            .map(Vec::len)
            .filter(|instance_len| *instance_len <= MAX_CACHED_INSTANCE_ROWS)
            .fold(max_cached_instance_len, usize::max);
        item_column_counts.push(item.instances.iter().map(Vec::len).collect::<Vec<_>>());
    }

    let table = InstanceFixedWindowTable::new(params, max_cached_instance_len);
    let instances = items
        .iter()
        .flat_map(|item| item.instances.iter())
        .flat_map(|instances| instances.iter())
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    let projective = table.commit_batch(params, &instances);
    let mut affine = vec![C::identity(); projective.len()];
    C::Curve::batch_normalize_vartime(&projective, &mut affine);
    let mut affine = affine.into_iter();
    let commitments = item_column_counts
        .into_iter()
        .map(|proof_column_counts| {
            proof_column_counts
                .into_iter()
                .map(|column_count| affine.by_ref().take(column_count).collect())
                .collect()
        })
        .collect();
    assert!(affine.next().is_none());

    Ok(commitments)
}

/// A verifier that checks multiple proofs in a batch. **This requires the
/// `batch` crate feature to be enabled.**
#[derive(Debug, Default)]
pub struct BatchVerifier<C: CurveAffine> {
    items: Vec<BatchItem<C>>,
}

impl<C: CurveAffine> BatchVerifier<C> {
    /// Constructs a new batch verifier.
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    /// Adds a proof to the batch.
    pub fn add_proof(&mut self, instances: Vec<Vec<Vec<C::Scalar>>>, proof: Vec<u8>) {
        self.items.push(BatchItem { instances, proof })
    }
}

impl<C: CurveAffine> BatchVerifier<C>
where
    C::Scalar: FromUniformBytes<64>,
{
    /// Finalizes the batch and checks its validity.
    ///
    /// Returns `false` if *some* proof was invalid. If the caller needs to identify
    /// specific failing proofs, it must re-process the proofs separately.
    ///
    /// This uses [`SysRng`] internally instead of taking an `R: Rng` argument, because
    /// the internal parallelization requires access to a RNG that is guaranteed to not
    /// clone its internal state when shared between threads.
    pub fn finalize(self, params: &Params<C>, vk: &VerifyingKey<C>) -> bool {
        fn accumulate_msm<'params, C: CurveAffine>(
            mut acc: MSM<'params, C>,
            msm: MSM<'params, C>,
        ) -> MSM<'params, C> {
            acc.add_msm_batch(msm);
            acc
        }

        let items = self.items;
        let instance_commitments = match compute_batch_instance_commitments(params, vk, &items) {
            Ok(instance_commitments) => instance_commitments,
            Err(_) => return false,
        };
        let items = items
            .into_iter()
            .zip(instance_commitments)
            .collect::<Vec<_>>();

        let final_msm = items
            .into_par_iter()
            .enumerate()
            .map(|(i, (item, instance_commitments))| {
                let instances: Vec<Vec<_>> = item
                    .instances
                    .iter()
                    .map(|i| i.iter().map(|c| &c[..]).collect())
                    .collect();
                let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

                // All proof equations are fixed before sampling these independent
                // weights. Fix the first weight at one; randomize every later
                // equation so invalid proofs cannot deliberately cancel.
                let rho = if i == 0 {
                    C::Scalar::ONE
                } else {
                    C::Scalar::random(&mut UnwrapErr(SysRng))
                };
                let strategy = BatchStrategy::new(params, rho);
                let mut transcript = Blake2bRead::init(&item.proof[..]);
                verify_proof_with_instance_commitments(
                    params,
                    vk,
                    strategy,
                    &instances,
                    instance_commitments,
                    &mut transcript,
                )
                .map_err(|e| {
                    tracing::debug!("Batch item {} failed verification: {}", i, e);
                    e
                })
            })
            .try_fold_and_reduce(
                || params.empty_msm(),
                |acc, res| res.map(|proof_msm| accumulate_msm(acc, proof_msm)),
            );

        match final_msm {
            Ok(msm) => msm.eval(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use ff::{Field, FromUniformBytes};
    use pasta_curves::{EpAffine, EqAffine, Fp, Fq};

    use super::{
        InstanceFixedWindowTable, commit_instance_with_table, signed_fixed_window_digit,
        signed_instance_window_count,
    };
    use crate::{
        INSTANCE_WINDOW_BITS, INSTANCE_WINDOW_ENTRIES_PER_BASE, MAX_CACHED_INSTANCE_ROWS,
        poly::commitment::Params,
    };

    #[test]
    fn signed_instance_windows_preserve_top_bit_carries() {
        for bits in [8, 9, 10, 17, 18, 19, 26, 27, 28, 35, 36, 37, 62, 63, 64] {
            let high_bit = 1u128 << (bits - 1);
            for value in [0, 1, high_bit - 1, high_bit, (1u128 << bits) - 1] {
                let bytes = (value as u64).to_le_bytes();
                let mut reconstructed = 0i128;
                for window in 0..signed_instance_window_count(bits) {
                    let digit = signed_fixed_window_digit(&bytes, window, INSTANCE_WINDOW_BITS);
                    assert!(digit.magnitude <= INSTANCE_WINDOW_ENTRIES_PER_BASE);
                    let magnitude = digit.magnitude as i128;
                    let signed = if digit.negative {
                        -magnitude
                    } else {
                        magnitude
                    };
                    reconstructed += signed << (window * INSTANCE_WINDOW_BITS);
                }
                assert_eq!(reconstructed, value as i128, "{bits}-bit value {value}");
            }
        }
    }

    #[test]
    fn fixed_window_instance_commitments_match_signed_booth() {
        macro_rules! check_curve {
            ($curve:ty, $scalar:ty) => {{
                const K: u32 = 7;

                let params = Params::<$curve>::new(K);
                let table = InstanceFixedWindowTable::new(&params, MAX_CACHED_INSTANCE_ROWS);
                assert_eq!(
                    table.retained_bytes(),
                    MAX_CACHED_INSTANCE_ROWS
                        * INSTANCE_WINDOW_ENTRIES_PER_BASE
                        * core::mem::size_of::<$curve>(),
                );

                let instances = [0, 1, 10, 17, 63, 64, 65, 127]
                    .into_iter()
                    .map(|len| {
                        let mut instance = (0..len)
                            .map(|index| {
                                let mut bytes = [0; 64];
                                for (offset, byte) in bytes.iter_mut().enumerate() {
                                    *byte = (index as u8)
                                        .wrapping_mul(73)
                                        .wrapping_add((offset as u8).wrapping_mul(29))
                                        .wrapping_add(17);
                                }
                                <$scalar as FromUniformBytes<64>>::from_uniform_bytes(&bytes)
                            })
                            .collect::<Vec<_>>();
                        if let Some(value) = instance.get_mut(0) {
                            *value = <$scalar>::ZERO;
                        }
                        if let Some(value) = instance.get_mut(1) {
                            *value = <$scalar>::ONE;
                        }
                        if let Some(value) = instance.get_mut(2) {
                            *value = -<$scalar>::ONE;
                        }
                        instance
                    })
                    .collect::<Vec<_>>();
                let instance_slices = instances.iter().map(Vec::as_slice).collect::<Vec<_>>();
                let batched = table.commit_batch(&params, &instance_slices);

                for (instance, batched) in instances.iter().zip(batched) {
                    assert_eq!(
                        commit_instance_with_table(&params, &table, &instance),
                        params.commit_lagrange_instance(&instance),
                    );
                    assert_eq!(batched, params.commit_lagrange_instance(&instance));
                }

                assert!(table.commit_batch(&params, &[]).is_empty());
                for column_count in [1, 2, 3] {
                    for columns in instance_slices.chunks(column_count) {
                        for (instance, commitment) in
                            columns.iter().zip(table.commit_batch(&params, columns))
                        {
                            assert_eq!(commitment, params.commit_lagrange_instance(instance));
                        }
                    }
                }
            }};
        }

        check_curve!(EqAffine, Fp);
        check_curve!(EpAffine, Fq);
    }
}
