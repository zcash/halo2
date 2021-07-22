use group::Group;
use halo2::arithmetic::CurveExt;
use pasta_curves::{arithmetic::FieldExt, pallas};
use rand::RngCore;
use subtle::CtOption;

use super::NoteCommitment;
use crate::{
    keys::NullifierDerivingKey,
    spec::{extract_p, mod_r_p},
};

/// A unique nullifier for a note.
#[derive(Clone, Copy, Debug)]
pub struct Nullifier(pub(crate) pallas::Base);

impl Nullifier {
    /// Generates a dummy nullifier for use as $\rho$ in dummy spent notes.
    ///
    /// Nullifiers are required by consensus to be unique. For dummy output notes, we get
    /// this restriction as intended: the note's $\rho$ value is set to the nullifier of
    /// the accompanying spent note within the action, which is constrained by consensus
    /// to be unique. In the case of dummy spent notes, we get this restriction by
    /// following the chain backwards: the nullifier of the dummy spent note will be
    /// constrained by consensus to be unique, and the nullifier's uniqueness is derived
    /// from the uniqueness of $\rho$.
    ///
    /// Instead of explicitly sampling for a unique nullifier, we rely here on the size of
    /// the base field to make the chance of sapling a colliding nullifier negligible.
    pub(crate) fn dummy(rng: &mut impl RngCore) -> Self {
        Nullifier(extract_p(&pallas::Point::random(rng)))
    }

    /// Deserialize the nullifier from a byte array.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Base::from_bytes(bytes).map(Nullifier)
    }

    /// Serialize the nullifier to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// $DeriveNullifier$.
    ///
    /// Defined in [Zcash Protocol Spec § 4.16: Note Commitments and Nullifiers][commitmentsandnullifiers].
    ///
    /// [commitmentsandnullifiers]: https://zips.z.cash/protocol/nu5.pdf#commitmentsandnullifiers
    pub(super) fn derive(
        nk: &NullifierDerivingKey,
        rho: pallas::Base,
        psi: pallas::Base,
        cm: NoteCommitment,
    ) -> Self {
        let k = pallas::Point::hash_to_curve("z.cash:Orchard")(b"K");

        Nullifier(extract_p(&(k * mod_r_p(nk.prf_nf(rho) + psi) + cm.0)))
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use group::Group;
    use pasta_curves::{arithmetic::FieldExt, pallas};
    use proptest::collection::vec;
    use proptest::prelude::*;
    use std::convert::TryFrom;

    use super::Nullifier;
    use crate::spec::extract_p;

    prop_compose! {
        /// Generate a uniformly distributed nullifier value.
        pub fn arb_nullifier()(
            bytes in vec(any::<u8>(), 64)
        ) -> Nullifier {
            let point = pallas::Point::generator() * pallas::Scalar::from_bytes_wide(&<[u8; 64]>::try_from(bytes).unwrap());
            Nullifier(extract_p(&point))
        }
    }
}
