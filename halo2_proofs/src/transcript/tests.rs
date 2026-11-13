// Proof point layouts are lists of byte ranges; one-range lists are intended.
#![allow(clippy::single_range_in_vec_init)]

use super::*;
use crate::pasta::{pallas, vesta};
use group::{Curve, Group};

fn proof<C: CurveAffine>(runs: usize) -> Vec<u8>
where
    C::Scalar: FromUniformBytes<64>,
{
    let mut writer = Blake2bWrite::<_, C, Challenge255<C>>::init(Vec::new());
    for _ in 0..runs {
        writer
            .write_point(C::Curve::generator().to_affine())
            .unwrap();
        writer
            .write_point(C::Curve::generator().double().to_affine())
            .unwrap();
        writer.write_scalar(C::Scalar::from(7)).unwrap();
    }
    writer.finalize()
}

fn decodings<C: CurveAffine>(bytes: &[u8], runs: usize) -> Vec<(usize, Option<C>)> {
    let point_len = C::Repr::default().as_ref().len();
    let scalar_len = <C::Scalar as PrimeField>::Repr::default().as_ref().len();
    let stride = 2 * point_len + scalar_len;
    (0..runs)
        .flat_map(|run| [run * stride, run * stride + point_len])
        .filter_map(|offset| {
            let bytes = bytes.get(offset..offset + point_len)?;
            let mut encoding = C::Repr::default();
            encoding.as_mut().copy_from_slice(bytes);
            Some((offset, C::from_bytes(&encoding).into()))
        })
        .collect()
}

fn assert_read<T: PartialEq + std::fmt::Debug>(a: io::Result<T>, b: io::Result<T>) -> bool {
    match (a, b) {
        (Ok(a), Ok(b)) => {
            assert_eq!(a, b);
            true
        }
        (Err(a), Err(b)) => {
            assert_eq!(a.kind(), b.kind());
            assert_eq!(a.to_string(), b.to_string());
            false
        }
        (a, b) => panic!("different transcript reads: {a:?}, {b:?}"),
    }
}

fn exercise<C: CurveAffine>(bytes: &[u8], runs: usize, prefetched: Option<Vec<(usize, Option<C>)>>)
where
    C::Scalar: FromUniformBytes<64>,
{
    let mut plain = Blake2bRead::<_, C, Challenge255<C>>::init(bytes);
    let mut fast = match prefetched {
        Some(points) => PrefetchedBlake2bRead::<C>::with_prefetched(bytes, points),
        None => PrefetchedBlake2bRead::<C>::recording(bytes),
    };
    let point = C::Curve::generator().to_affine();
    plain.common_point(point).unwrap();
    fast.common_point(point).unwrap();
    plain.common_scalar(C::Scalar::from(9)).unwrap();
    fast.common_scalar(C::Scalar::from(9)).unwrap();
    for _ in 0..runs {
        for _ in 0..2 {
            let success = assert_read(plain.read_point(), fast.read_point());
            // Also compare the transcript state after malformed and identity reads.
            assert_eq!(*plain.squeeze_challenge(), *fast.squeeze_challenge());
            if !success {
                return;
            }
        }
        let success = assert_read(plain.read_scalar(), fast.read_scalar());
        assert_eq!(*plain.squeeze_challenge(), *fast.squeeze_challenge());
        if !success {
            return;
        }
    }
    assert_eq!(plain.reader, fast.inner.reader);
}

fn differential<C: CurveAffine>()
where
    C::Scalar: FromUniformBytes<64>,
{
    let bytes = proof::<C>(3);
    for end in 0..=bytes.len() {
        let bytes = &bytes[..end];
        exercise::<C>(bytes, 3, Some(decodings::<C>(bytes, 3)));
        exercise::<C>(bytes, 3, None);
        // A missing layout must take the ordinary decoding path.
        exercise::<C>(bytes, 3, Some(Vec::new()));
    }
    let len = C::Repr::default().as_ref().len();
    for offset in (0..bytes.len()).step_by(len) {
        let mut malformed = bytes.clone();
        malformed[offset..offset + len].fill(255);
        exercise::<C>(&malformed, 3, Some(decodings::<C>(&malformed, 3)));
        exercise::<C>(&malformed, 3, None);
    }
    let mut identity = bytes.clone();
    identity[..len].copy_from_slice(C::identity().to_bytes().as_ref());
    exercise::<C>(&identity, 3, Some(decodings::<C>(&identity, 3)));
    exercise::<C>(&identity, 3, None);

    // Prefetching at scalar offsets must not affect reads. Stale entries before
    // a point are skipped, while a matching later entry is still used.
    let mut extra = decodings::<C>(&bytes, 3);
    extra.push((2 * len, None));
    extra.sort_by_key(|(offset, _)| *offset);
    exercise::<C>(&bytes, 3, Some(extra));
    let mut trailing = bytes.clone();
    trailing.extend_from_slice(&[255; 7]);
    exercise::<C>(&trailing, 3, Some(decodings::<C>(&trailing, 3)));
}

#[test]
fn prefetched_and_cold_transcripts_match_bytes_challenges_and_errors() {
    differential::<pallas::Affine>();
    differential::<vesta::Affine>();
}

#[test]
fn recording_and_shape_caches_remain_bounded() {
    type C = vesta::Affine;
    let limit = crate::MAX_CACHED_PROOF_POINT_RUNS;
    for runs in [limit, limit + 1] {
        let bytes = proof::<C>(runs);
        let mut recording = PrefetchedBlake2bRead::<C>::recording(&bytes);
        for _ in 0..runs {
            recording.read_point().unwrap();
            recording.read_point().unwrap();
            recording.read_scalar().unwrap();
        }
        let ranges = recording.into_recorded();
        assert_eq!(ranges.is_some(), runs == limit);
        if let Some(ranges) = ranges {
            assert_eq!(
                ranges,
                (0..runs).map(|i| i * 96..i * 96 + 64).collect::<Vec<_>>()
            );
        }
    }
    let layouts = crate::plonk::ProofPointLayouts::default();
    layouts.insert(0, vec![0..32]);
    let in_flight = layouts.get(0).unwrap();
    layouts.insert(0, vec![32..64]);
    assert!(std::sync::Arc::ptr_eq(&in_flight, &layouts.get(0).unwrap()));
    for circuits in 1..=256 {
        layouts.insert(circuits, vec![32..64]);
    }
    assert!(layouts.get(0).is_none());
    assert_eq!(&*in_flight, &[0..32]);
    layouts.insert(300, (0..=limit).map(|i| i * 32..i * 32 + 32).collect());
    assert!(layouts.get(300).is_none());

    let installed = crate::plonk::ProofPointLayouts::default();
    layouts.install(&installed);
    assert!(installed.get(0).is_none());
    for circuits in 1..=256 {
        assert_eq!(&*installed.get(circuits).unwrap(), &[32..64]);
    }
}
