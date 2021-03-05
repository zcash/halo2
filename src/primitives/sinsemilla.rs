//! The Sinsemilla hash function and commitment scheme.

use group::Group;
use halo2::{arithmetic::CurveExt, pasta::pallas};

use crate::spec::extract_p;

const GROUP_HASH_Q: &str = "z.cash:SinsemillaQ";
const GROUP_HASH_S: &str = "z.cash:SinsemillaS";

const K: usize = 10;
const C: usize = 253;

fn lebs2ip_k(bits: &[bool]) -> u32 {
    assert!(bits.len() == K);
    bits.iter()
        .enumerate()
        .fold(0u32, |acc, (i, b)| acc + if *b { 1 << i } else { 0 })
}

struct Pad<I: Iterator<Item = bool>> {
    inner: I,
    len: usize,
    padding_left: Option<usize>,
}

impl<I: Iterator<Item = bool>> Pad<I> {
    fn new(inner: I) -> Self {
        Pad {
            inner,
            len: 0,
            padding_left: None,
        }
    }
}

impl<I: Iterator<Item = bool>> Iterator for Pad<I> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(n) = self.padding_left.as_mut() {
                if *n == 0 {
                    break None;
                } else {
                    *n -= 1;
                    break Some(false);
                }
            } else if let Some(ret) = self.inner.next() {
                self.len += 1;
                assert!(self.len <= K * C);
                break Some(ret);
            } else {
                // Inner iterator just ended.
                let rem = self.len % K;
                if rem > 0 {
                    self.padding_left = Some(K - rem);
                } else {
                    // No padding required.
                    self.padding_left = Some(0);
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Q(domain_prefix: &str) -> pallas::Point {
    pallas::Point::hash_to_curve(GROUP_HASH_Q)(domain_prefix.as_bytes())
}

/// `SinsemillaHashToPoint` from [§ 5.4.1.9].
///
/// [§ 5.4.1.9]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash
#[allow(non_snake_case)]
pub(crate) fn hash_to_point(domain_prefix: &str, msg: impl Iterator<Item = bool>) -> pallas::Point {
    let padded: Vec<_> = Pad::new(msg).collect();

    let hasher_S = pallas::Point::hash_to_curve(GROUP_HASH_S);
    let S = |chunk: &[bool]| hasher_S(&lebs2ip_k(chunk).to_le_bytes());

    padded
        .chunks(K)
        .fold(Q(domain_prefix), |acc, chunk| acc.double() + S(chunk))
}

/// `SinsemillaHash` from [§ 5.4.1.9].
///
/// [§ 5.4.1.9]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash
pub(crate) fn hash(domain_prefix: &str, msg: impl Iterator<Item = bool>) -> pallas::Base {
    extract_p(&hash_to_point(domain_prefix, msg))
}

/// `SinsemillaCommit` from [§ 5.4.7.4].
///
/// [§ 5.4.7.4]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
#[allow(non_snake_case)]
pub(crate) fn commit(
    domain_prefix: &str,
    msg: impl Iterator<Item = bool>,
    r: &pallas::Scalar,
) -> pallas::Point {
    let m_prefix = domain_prefix.to_owned() + "-M";
    let r_prefix = domain_prefix.to_owned() + "-r";

    let hasher_r = pallas::Point::hash_to_curve(&r_prefix);

    hash_to_point(&m_prefix, msg) + hasher_r(&[]) * r
}

/// `SinsemillaShortCommit` from [§ 5.4.7.4].
///
/// [§ 5.4.7.4]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
pub(crate) fn short_commit(
    domain_prefix: &str,
    msg: impl Iterator<Item = bool>,
    r: &pallas::Scalar,
) -> pallas::Base {
    extract_p(&commit(domain_prefix, msg, r))
}

#[cfg(test)]
mod tests {
    use super::Pad;

    #[test]
    fn pad() {
        assert_eq!(Pad::new([].iter().cloned()).collect::<Vec<_>>(), vec![]);
        assert_eq!(
            Pad::new([true].iter().cloned()).collect::<Vec<_>>(),
            vec![true, false, false, false, false, false, false, false, false, false]
        );
        assert_eq!(
            Pad::new([true, true].iter().cloned()).collect::<Vec<_>>(),
            vec![true, true, false, false, false, false, false, false, false, false]
        );
        assert_eq!(
            Pad::new([true, true, true].iter().cloned()).collect::<Vec<_>>(),
            vec![true, true, true, false, false, false, false, false, false, false]
        );
        assert_eq!(
            Pad::new(
                [true, true, false, true, false, true, false, true, false, true]
                    .iter()
                    .cloned()
            )
            .collect::<Vec<_>>(),
            vec![true, true, false, true, false, true, false, true, false, true]
        );
        assert_eq!(
            Pad::new(
                [true, true, false, true, false, true, false, true, false, true, true]
                    .iter()
                    .cloned()
            )
            .collect::<Vec<_>>(),
            vec![
                true, true, false, true, false, true, false, true, false, true, true, false, false,
                false, false, false, false, false, false, false
            ]
        );
    }
}
