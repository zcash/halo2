//! The Sinsemilla hash function and commitment scheme.

use std::iter;

use group::{Curve, Group};
use halo2::{
    arithmetic::{CurveAffine, CurveExt},
    pasta::pallas,
};

const GROUP_HASH_Q: &str = "z.cash:SinsemillaQ";
const GROUP_HASH_S: &str = "z.cash:SinsemillaS";

const K: usize = 10;
const C: usize = 253;

fn lebs2ip_32(bits: &[bool]) -> u32 {
    bits.iter()
        .enumerate()
        .fold(0u32, |acc, (i, b)| acc + if *b { 1 << i } else { 0 })
}

/// Hash extractor for Pallas, from [§ 5.4.8.7].
///
/// [§ 5.4.8.7]: https://zips.z.cash/protocol/orchard.pdf#concreteextractorpallas
fn extract(point: &pallas::Point) -> pallas::Base {
    // TODO: Should we return the actual bits in a Vec, or allow the caller to use
    // PrimeField::to_le_bits on the returned pallas::Base?
    if let Some((x, _)) = point.to_affine().get_xy().into() {
        x
    } else {
        pallas::Base::zero()
    }
}

#[allow(non_snake_case)]
fn Q(domain_prefix: &str) -> pallas::Point {
    pallas::Point::hash_to_curve(GROUP_HASH_Q)(domain_prefix.as_bytes())
}

/// `SinsemillaHashToPoint` from [§ 5.4.1.9].
///
/// [§ 5.4.1.9]: https://zips.z.cash/protocol/orchard.pdf#concretesinsemillahash
#[allow(non_snake_case)]
pub(crate) fn hash_to_point(
    domain_prefix: &str,
    msg: impl Iterator<Item = bool> + ExactSizeIterator,
) -> pallas::Point {
    assert!(msg.len() <= K * C);
    let pad = msg.len() % K;
    let padded: Vec<_> = msg.chain(iter::repeat(false).take(pad)).collect();

    let hasher_S = pallas::Point::hash_to_curve(GROUP_HASH_S);
    let S = |chunk: &[bool]| hasher_S(&lebs2ip_32(chunk).to_le_bytes());

    padded
        .chunks(K)
        .fold(Q(domain_prefix), |acc, chunk| acc.double() + S(chunk))
}

/// `SinsemillaHash` from [§ 5.4.1.9].
///
/// [§ 5.4.1.9]: https://zips.z.cash/protocol/orchard.pdf#concretesinsemillahash
pub(crate) fn hash(
    domain_prefix: &str,
    msg: impl Iterator<Item = bool> + ExactSizeIterator,
) -> pallas::Base {
    extract(&hash_to_point(domain_prefix, msg))
}

/// `SinsemillaCommit` from [§ 5.4.7.4].
///
/// [§ 5.4.7.4]: https://zips.z.cash/protocol/orchard.pdf#concretesinsemillacommit
#[allow(non_snake_case)]
pub(crate) fn commit(
    domain_prefix: &str,
    msg: impl Iterator<Item = bool> + ExactSizeIterator,
    r: &pallas::Scalar,
) -> pallas::Point {
    let m_prefix = domain_prefix.to_owned() + "-M";
    let r_prefix = domain_prefix.to_owned() + "-r";

    let hasher_r = pallas::Point::hash_to_curve(&r_prefix);

    hash_to_point(&m_prefix, msg) + hasher_r(&[]) * r
}

/// `SinsemillaShortCommit` from [§ 5.4.7.4].
///
/// [§ 5.4.7.4]: https://zips.z.cash/protocol/orchard.pdf#concretesinsemillacommit
pub(crate) fn short_commit(
    domain_prefix: &str,
    msg: impl Iterator<Item = bool> + ExactSizeIterator,
    r: &pallas::Scalar,
) -> pallas::Base {
    extract(&commit(domain_prefix, msg, r))
}
