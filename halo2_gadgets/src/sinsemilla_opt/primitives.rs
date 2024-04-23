//! Implementation of Sinsemilla outside the circuit.

use group::Wnaf;
use halo2_proofs::arithmetic::CurveExt;
use pasta_curves::pallas;
use subtle::CtOption;

use crate::sinsemilla::primitives::{CommitDomain, HashDomain};

impl CommitDomain {
    /// Constructs a new `CommitDomain` from different values for `hash_domain` and `blind_domain`
    pub fn new_with_personalization(hash_domain: &str, blind_domain: &str) -> Self {
        let m_prefix = format!("{}-M", hash_domain);
        let r_prefix = format!("{}-r", blind_domain);
        let hasher_r = pallas::Point::hash_to_curve(&r_prefix);
        CommitDomain {
            M: HashDomain::new(&m_prefix),
            R: hasher_r(&[]),
        }
    }

    /// $\mathsf{SinsemillaHashToPoint}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash
    pub fn hash_to_point(&self, msg: impl Iterator<Item = bool>) -> CtOption<pallas::Point> {
        self.M.hash_to_point(msg)
    }

    /// Returns `SinsemillaCommit_r(personalization, msg) = hash_point + [r]R`
    /// where `SinsemillaHash(personalization, msg) = hash_point`
    /// and `R` is derived from the `personalization`.
    #[allow(non_snake_case)]
    pub fn commit_from_hash_point(
        &self,
        hash_point: CtOption<pallas::Point>,
        r: &pallas::Scalar,
    ) -> CtOption<pallas::Point> {
        // We use complete addition for the blinding factor.
        hash_point.map(|p| p + Wnaf::new().scalar(r).base(self.R))
    }
}

#[cfg(test)]
mod tests {
    use pasta_curves::pallas;

    #[test]
    fn commit_in_several_steps() {
        use rand::{rngs::OsRng, Rng};

        use ff::Field;

        use crate::sinsemilla::primitives::CommitDomain;

        let domain = CommitDomain::new("z.cash:ZSA-NoteCommit");

        let mut os_rng = OsRng::default();
        let msg: Vec<bool> = (0..36).map(|_| os_rng.gen::<bool>()).collect();

        let rcm = pallas::Scalar::random(&mut os_rng);

        // Evaluate the commitment with commit function
        let commit1 = domain.commit(msg.clone().into_iter(), &rcm);

        // Evaluate the commitment with the following steps
        // 1. hash msg
        // 2. evaluate the commitment from the hash point
        let hash_point = domain.M.hash_to_point(msg.into_iter());
        let commit2 = domain.commit_from_hash_point(hash_point, &rcm);

        // Test equality
        assert_eq!(commit1.unwrap(), commit2.unwrap());
    }
}
