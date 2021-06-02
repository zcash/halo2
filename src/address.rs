use crate::{
    keys::{DiversifiedTransmissionKey, Diversifier},
    spec::{diversify_hash, NonIdentityPallasPoint},
};

/// A shielded payment address.
///
/// # Examples
///
/// ```
/// use orchard::keys::{SpendingKey, FullViewingKey};
///
/// let sk = SpendingKey::from_bytes([7; 32]).unwrap();
/// let address = FullViewingKey::from(&sk).default_address();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Address {
    d: Diversifier,
    pk_d: DiversifiedTransmissionKey,
}

impl Address {
    pub(crate) fn from_parts(d: Diversifier, pk_d: DiversifiedTransmissionKey) -> Self {
        // We assume here that pk_d is correctly-derived from d. We ensure this for
        // internal APIs. For parsing from raw byte encodings, we assume that users aren't
        // modifying internals of encoded address formats. If they do, that can result in
        // lost funds, but we can't defend against that from here.
        Address { d, pk_d }
    }

    pub(crate) fn diversifer(&self) -> Diversifier {
        self.d
    }

    pub(crate) fn g_d(&self) -> NonIdentityPallasPoint {
        diversify_hash(self.d.as_array())
    }

    pub(crate) fn pk_d(&self) -> &DiversifiedTransmissionKey {
        &self.pk_d
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use proptest::prelude::*;

    use crate::keys::{testing::arb_spending_key, FullViewingKey};

    use super::Address;

    prop_compose! {
        /// Generates an arbitrary payment address.
        pub(crate) fn arb_address()(sk in arb_spending_key()) -> Address {
            let fvk = FullViewingKey::from(&sk);
            fvk.default_address()
        }
    }
}
