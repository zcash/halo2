//! The Sinsemilla hash function and commitment scheme.

use group::Group;
use halo2::arithmetic::CurveExt;
use pasta_curves::pallas;

use crate::spec::extract_p;

mod constants;
pub use constants::*;

fn lebs2ip_k(bits: &[bool]) -> u32 {
    assert!(bits.len() == K);
    bits.iter()
        .enumerate()
        .fold(0u32, |acc, (i, b)| acc + if *b { 1 << i } else { 0 })
}

/// Pads the given iterator (which MUST have length $\leq K * C$) with zero-bits to a
/// multiple of $K$ bits.
struct Pad<I: Iterator<Item = bool>> {
    /// The iterator we are padding.
    inner: I,
    /// The measured length of the inner iterator.
    ///
    /// This starts as a lower bound, and will be accurate once `padding_left.is_some()`.
    len: usize,
    /// The amount of padding that remains to be emitted.
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
            // If we have identified the required padding, the inner iterator has ended,
            // and we will never poll it again.
            if let Some(n) = self.padding_left.as_mut() {
                if *n == 0 {
                    // Either we already emitted all necessary padding, or there was no
                    // padding required.
                    break None;
                } else {
                    // Emit the next padding bit.
                    *n -= 1;
                    break Some(false);
                }
            } else if let Some(ret) = self.inner.next() {
                // We haven't reached the end of the inner iterator yet.
                self.len += 1;
                assert!(self.len <= K * C);
                break Some(ret);
            } else {
                // Inner iterator just ended, so we now know its length.
                let rem = self.len % K;
                if rem > 0 {
                    // The inner iterator requires padding in the range [1,K).
                    self.padding_left = Some(K - rem);
                } else {
                    // No padding required.
                    self.padding_left = Some(0);
                }
            }
        }
    }
}

/// A domain in which [`Self::hash_to_point()`] and
/// [`Self::hash()`] can be used.
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct HashDomain {
    Q: pallas::Point,
}

impl HashDomain {
    /// Construct a new HashDomain with a specific prefix string
    pub(crate) fn new(domain: &str) -> Self {
        HashDomain {
            Q: pallas::Point::hash_to_curve(Q_PERSONALIZATION)(domain.as_bytes()),
        }
    }

    /// `SinsemillaHashToPoint` from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash
    #[allow(non_snake_case)]
    pub(crate) fn hash_to_point(&self, msg: impl Iterator<Item = bool>) -> pallas::Point {
        let padded: Vec<_> = Pad::new(msg).collect();

        let hasher_S = pallas::Point::hash_to_curve(S_PERSONALIZATION);
        let S = |chunk: &[bool]| hasher_S(&lebs2ip_k(chunk).to_le_bytes());

        padded
            .chunks(K)
            .fold(self.Q, |acc, chunk| acc.double() + S(chunk))
    }

    /// `SinsemillaHash` from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash
    pub(crate) fn hash(&self, msg: impl Iterator<Item = bool>) -> pallas::Base {
        extract_p(&self.hash_to_point(msg))
    }

    /// Return `Q`
    #[allow(non_snake_case)]
    pub(crate) fn Q(&self) -> pallas::Point {
        self.Q
    }
}

/// A domain in which [`Self::commit`] and
/// [`Self::short_commit`] can be used.
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct CommitDomain {
    M: HashDomain,
    R: pallas::Point,
}

impl CommitDomain {
    /// Construct a new CommitDomain with a specific prefix string
    pub(crate) fn new(domain: &str) -> Self {
        let m_prefix = domain.to_owned() + "-M";
        let r_prefix = domain.to_owned() + "-r";
        CommitDomain {
            M: HashDomain::new(&m_prefix),
            R: pallas::Point::hash_to_curve(&r_prefix.clone())(&[]),
        }
    }

    /// `SinsemillaCommit` from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    #[allow(non_snake_case)]
    pub(crate) fn commit(
        &self,
        msg: impl Iterator<Item = bool>,
        r: &pallas::Scalar,
    ) -> pallas::Point {
        self.M.hash_to_point(msg) + self.R * r
    }

    /// `SinsemillaShortCommit` from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub(crate) fn short_commit(
        &self,
        msg: impl Iterator<Item = bool>,
        r: &pallas::Scalar,
    ) -> pallas::Base {
        extract_p(&self.commit(msg, r))
    }

    /// Return `R`
    #[allow(non_snake_case)]
    pub(crate) fn R(&self) -> pallas::Point {
        self.R
    }
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
