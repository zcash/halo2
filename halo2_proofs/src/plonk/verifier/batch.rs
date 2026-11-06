use group::{
    ff::{Field, FromUniformBytes, PrimeField},
    Group,
};
use pasta_curves::arithmetic::{CurveAffine, CurveExt};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;
use std::sync::Arc;

use super::{validate_instances, verify_proof_with_instance_commitments, VerificationStrategy};
use crate::{
    multicore::{IntoParallelIterator, TryFoldAndReduce},
    plonk::{Error, VerifyingKey},
    poly::commitment::{Guard, Params, MSM},
    transcript::{Blake2bRead, EncodedChallenge},
    INSTANCE_WINDOW_BITS, INSTANCE_WINDOW_ENTRIES_PER_BASE, MAX_CACHED_INSTANCE_ROWS,
};

#[cfg(feature = "multicore")]
use crate::multicore::{IndexedParallelIterator, ParallelIterator};

#[cfg(test)]
mod tests;

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

const SERIAL_INSTANCE_WINDOW_BITS: usize = 8;
// Lockstep bookkeeping pays off with three or more instance columns.
const MIN_LOCKSTEP_INSTANCE_COLUMNS: usize = 3;

// All scalars and points handled by this table are public instance data.
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

        // The scalar bit count fits usize on every supported target.
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

        // The default instance blind is one, including for an empty column.
        C::Curve::from(params.w) + variable
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

        // The scalar bit count fits usize on every supported target.
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
    // A byte has eight bits, which fits usize on every supported target.
    let byte_bits = u8::BITS as usize;
    let byte_start = bit_start / byte_bits;
    let bit_offset = bit_start % byte_bits;
    let low = bytes.get(byte_start).copied().unwrap_or(0);
    let high = bytes.get(byte_start + 1).copied().unwrap_or(0);
    let encoded = u16::from(low) | (u16::from(high) << u8::BITS);
    usize::from((encoded >> bit_offset) & ((1 << window_bits) - 1))
}

fn signed_fixed_window_digit(bytes: &[u8], window: usize, window_bits: usize) -> FixedWindowDigit {
    let bit_start = window * window_bits;
    let radix = 1usize << window_bits;
    let value = unsigned_fixed_window_digit(bytes, window, window_bits);
    let overlap = if bit_start == 0 {
        0
    } else {
        let bit = bit_start - 1;
        // A byte has eight bits, which fits usize on every supported target.
        let byte_bits = u8::BITS as usize;
        bytes
            .get(bit / byte_bits)
            .map_or(0, |byte| usize::from((byte >> (bit % byte_bits)) & 1))
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
    let mut max_cached_instance_len = 0;
    for item in items {
        validate_instances(params, vk, &item.instances)?;

        // Table construction costs 256 points per row. Longer, caller-sized
        // columns remain valid but use the generic MSM below.
        max_cached_instance_len = item
            .instances
            .iter()
            .flat_map(|instances| instances.iter())
            .map(Vec::len)
            .filter(|instance_len| *instance_len <= MAX_CACHED_INSTANCE_ROWS)
            .fold(max_cached_instance_len, usize::max);
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
    let commitments = items
        .iter()
        .map(|item| {
            item.instances
                .iter()
                .map(|columns| affine.by_ref().take(columns.len()).collect())
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

        let instance_commitments = match compute_batch_instance_commitments(params, vk, &self.items)
        {
            Ok(instance_commitments) => instance_commitments,
            Err(_) => return false,
        };

        let final_msm = self
            .items
            .into_par_iter()
            .zip(instance_commitments.into_par_iter())
            .enumerate()
            .map(|(i, (item, instance_commitments))| {
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
