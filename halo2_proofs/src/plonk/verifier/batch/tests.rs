use ff::{Field, FromUniformBytes};
use group::CurveAffine as _;
use pasta_curves::{EpAffine, EqAffine, Fp, Fq};

use super::{
    compute_batch_instance_commitments, signed_fixed_window_digit, signed_instance_window_count,
    BatchItem, InstanceFixedWindowTable,
};
use crate::{
    arithmetic::CurveAffine,
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{keygen_vk, Circuit, ConstraintSystem, Error},
    poly::{
        commitment::{Blind, Params},
        EvaluationDomain,
    },
    transcript::{Blake2bWrite, Challenge255, EncodedChallenge, Transcript},
    INSTANCE_WINDOW_BITS, INSTANCE_WINDOW_ENTRIES_PER_BASE, MAX_CACHED_INSTANCE_ROWS,
};

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

#[test]
fn signed_instance_windows_preserve_top_bit_carries() {
    for bits in [8, 9, 10, 17, 18, 19, 26, 27, 28, 35, 36, 37, 62, 63, 64] {
        let high_bit = 1u128 << (bits - 1);
        for value in [0, 1, high_bit - 1, high_bit, (1u128 << bits) - 1] {
            let bytes = u64::try_from(value).unwrap().to_le_bytes();
            let mut reconstructed = 0i128;
            for window in 0..signed_instance_window_count(bits) {
                let digit = signed_fixed_window_digit(&bytes, window, INSTANCE_WINDOW_BITS);
                assert!(digit.magnitude <= INSTANCE_WINDOW_ENTRIES_PER_BASE);
                let magnitude = i128::try_from(digit.magnitude).unwrap();
                let signed = if digit.negative {
                    -magnitude
                } else {
                    magnitude
                };
                reconstructed += signed << (window * INSTANCE_WINDOW_BITS);
            }
            assert_eq!(
                reconstructed,
                i128::try_from(value).unwrap(),
                "{bits}-bit value {value}",
            );
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
                table.multiples.len() * core::mem::size_of::<$curve>(),
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
                                *byte = u8::try_from(index)
                                    .unwrap()
                                    .wrapping_mul(73)
                                    .wrapping_add(u8::try_from(offset).unwrap().wrapping_mul(29))
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
            let domain = EvaluationDomain::new(1, K);

            for (instance, batched) in instances.iter().zip(batched) {
                let expected = params.commit_lagrange_instance(instance);
                assert_eq!(
                    commit_instance_with_table(&params, &table, instance),
                    expected
                );
                assert_eq!(batched, expected);
                let mut padded = domain.empty_lagrange();
                for (coefficient, value) in padded.iter_mut().zip(instance) {
                    *coefficient = *value;
                }
                assert_eq!(batched, params.commit_lagrange(&padded, Blind::default()));
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

#[test]
fn fixed_window_instance_commitments_handle_exceptional_points() {
    macro_rules! check_curve {
        ($curve:ty, $scalar:ty) => {{
            let mut params = Params::<$curve>::new(3);
            let g = params.g_lagrange[0];
            params.g_lagrange[..4].copy_from_slice(&[<$curve>::identity(), g, -g, g]);
            let table = InstanceFixedWindowTable::new(&params, 4);
            let zero = <$scalar>::ZERO;
            let one = <$scalar>::ONE;
            let instances = [
                vec![],
                vec![zero; 4],
                vec![one; 4],
                vec![zero, one, one, zero],
                vec![one, -one, zero, one],
                vec![-one; 4],
            ];
            let slices = instances.iter().map(Vec::as_slice).collect::<Vec<_>>();
            for blind in [<$curve>::identity(), g, -g] {
                params.w = blind;
                for (instance, commitment) in
                    instances.iter().zip(table.commit_batch(&params, &slices))
                {
                    let expected = instance
                        .iter()
                        .zip(&params.g_lagrange)
                        .fold(blind.to_curve(), |acc, (scalar, base)| acc + *base * scalar);
                    assert_eq!(commitment, expected);
                    assert_eq!(table.commit(&params, instance), expected);
                }
            }
        }};
    }

    check_curve!(EqAffine, Fp);
    check_curve!(EpAffine, Fq);
}

#[derive(Clone)]
struct TwoInstanceColumns;

impl Circuit<Fp> for TwoInstanceColumns {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        meta.instance_column();
        meta.instance_column();
    }

    fn synthesize(&self, _: (), _: impl Layouter<Fp>) -> Result<(), Error> {
        Ok(())
    }
}

#[test]
fn batch_instance_commitments_preserve_order_validation_and_transcript() {
    let params = Params::<EqAffine>::new(7);
    let vk = keygen_vk(&params, &TwoInstanceColumns).unwrap();
    let items = vec![
        BatchItem {
            instances: vec![
                vec![vec![], vec![Fp::from(3); 10]],
                vec![vec![Fp::from(5); 65], vec![Fp::from(7); 64]],
            ],
            proof: vec![],
        },
        BatchItem {
            instances: vec![],
            proof: vec![],
        },
        BatchItem {
            instances: vec![vec![vec![Fp::from(11)], vec![Fp::ZERO; 17]]],
            proof: vec![],
        },
    ];
    let cached = compute_batch_instance_commitments(&params, &vk, &items).unwrap();
    let expected = items
        .iter()
        .map(|item| {
            let columns = item
                .instances
                .iter()
                .map(|columns| columns.iter().map(Vec::as_slice).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            let instances = columns.iter().map(Vec::as_slice).collect::<Vec<_>>();
            super::super::compute_instance_commitments(&params, &instances)
        })
        .collect::<Vec<_>>();
    assert_eq!(cached, expected);
    for (cached, expected) in cached.iter().zip(&expected) {
        let mut cached_transcript = Blake2bWrite::<_, _, Challenge255<EqAffine>>::init(vec![]);
        let mut expected_transcript = Blake2bWrite::<_, _, Challenge255<EqAffine>>::init(vec![]);
        vk.hash_into(&mut cached_transcript).unwrap();
        vk.hash_into(&mut expected_transcript).unwrap();
        for (cached, expected) in cached.iter().flatten().zip(expected.iter().flatten()) {
            cached_transcript.common_point(*cached).unwrap();
            expected_transcript.common_point(*expected).unwrap();
        }
        assert_eq!(
            cached_transcript.squeeze_challenge().get_scalar(),
            expected_transcript.squeeze_challenge().get_scalar(),
        );
    }
    assert!(compute_batch_instance_commitments(&params, &vk, &[])
        .unwrap()
        .is_empty());
    let max_len = usize::try_from(params.n).unwrap() - (vk.cs.blinding_factors() + 1);
    for (columns, expected_too_large) in [
        (vec![vec![]], false),
        (vec![vec![], vec![], vec![]], false),
        (vec![vec![Fp::ZERO; max_len + 1], vec![]], true),
    ] {
        let invalid = [BatchItem {
            instances: vec![columns],
            proof: vec![],
        }];
        let result = compute_batch_instance_commitments(&params, &vk, &invalid);
        if expected_too_large {
            assert!(matches!(result, Err(Error::InstanceTooLarge)));
        } else {
            assert!(matches!(result, Err(Error::InvalidInstances)));
        }
    }
    let maximal = [BatchItem {
        instances: vec![vec![vec![Fp::ZERO; max_len], vec![]]],
        proof: vec![],
    }];
    assert!(compute_batch_instance_commitments(&params, &vk, &maximal).is_ok());
}
