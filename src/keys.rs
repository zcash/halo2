//! Key structures for Orchard.

use std::convert::TryInto;
use std::mem;

use aes::Aes256;
use fpe::ff1::{BinaryNumeralString, FF1};
use group::GroupEncoding;
use halo2::{arithmetic::FieldExt, pasta::pallas};
use subtle::CtOption;

use crate::{
    address::Address,
    spec::{
        commit_ivk, diversify_hash, extract_p, ka_orchard, prf_expand, prf_expand_vec, to_base,
        to_scalar,
    },
};

/// A spending key, from which all key material is derived.
///
/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][§4.2.3].
///
/// [§4.2.3]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
#[derive(Debug)]
pub struct SpendingKey([u8; 32]);

impl SpendingKey {
    /// Constructs an Orchard spending key from uniformly-random bytes.
    ///
    /// Returns `None` if the bytes do not correspond to a valid Orchard spending key.
    pub fn from_bytes(sk: [u8; 32]) -> CtOption<Self> {
        let sk = SpendingKey(sk);
        // If ask = 0, discard this key.
        let ask = SpendAuthorizingKey::derive_inner(&sk);
        CtOption::new(sk, !ask.ct_is_zero())
    }
}

/// A spending authorizing key, used to create spend authorization signatures.
///
/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][§4.2.3].
///
/// [§4.2.3]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
#[derive(Debug)]
pub(crate) struct SpendAuthorizingKey(reddsa::SigningKey<reddsa::orchard::SpendAuth>);

impl SpendAuthorizingKey {
    /// Derives ask from sk. Internal use only, does not enforce all constraints.
    fn derive_inner(sk: &SpendingKey) -> pallas::Scalar {
        to_scalar(prf_expand(&sk.0, &[0x06]))
    }
}

impl From<&SpendingKey> for SpendAuthorizingKey {
    fn from(sk: &SpendingKey) -> Self {
        let ask = Self::derive_inner(sk);
        // SpendingKey cannot be constructed such that this assertion would fail.
        assert!(!bool::from(ask.ct_is_zero()));
        // TODO: Add TryFrom<S::Scalar> for SpendAuthorizingKey.
        let ret = SpendAuthorizingKey(ask.to_bytes().try_into().unwrap());
        // If the last bit of repr_P(ak) is 1, negate ask.
        if (<[u8; 32]>::from(AuthorizingKey::from(&ret).0)[31] >> 7) == 1 {
            SpendAuthorizingKey((-ask).to_bytes().try_into().unwrap())
        } else {
            ret
        }
    }
}

/// TODO: This is its protocol spec name for Sapling, but I'd prefer a different name.
#[derive(Debug)]
pub(crate) struct AuthorizingKey(reddsa::VerificationKey<reddsa::orchard::SpendAuth>);

impl From<&SpendAuthorizingKey> for AuthorizingKey {
    fn from(ask: &SpendAuthorizingKey) -> Self {
        AuthorizingKey((&ask.0).into())
    }
}

/// A key used to derive [`Nullifier`]s from [`Note`]s.
///
/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][§4.2.3].
///
/// [`Nullifier`]: crate::note::Nullifier;
/// [`Note`]: crate::note::Note;
/// [§4.2.3]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
#[derive(Debug)]
pub(crate) struct NullifierDerivingKey(pallas::Base);

impl From<&SpendingKey> for NullifierDerivingKey {
    fn from(sk: &SpendingKey) -> Self {
        NullifierDerivingKey(to_base(prf_expand(&sk.0, &[0x07])))
    }
}

/// A key that provides the capability to view incoming and outgoing transactions.
///
/// This key is useful anywhere you need to maintain accurate balance, but do not want the
/// ability to spend funds (such as a view-only wallet).
#[derive(Debug)]
pub struct FullViewingKey {
    ak: AuthorizingKey,
    nk: NullifierDerivingKey,
    rivk: pallas::Scalar,
}

impl From<&SpendingKey> for FullViewingKey {
    fn from(sk: &SpendingKey) -> Self {
        FullViewingKey {
            ak: (&SpendAuthorizingKey::from(sk)).into(),
            nk: sk.into(),
            rivk: to_scalar(prf_expand(&sk.0, &[0x08])),
        }
    }
}

impl FullViewingKey {
    /// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][§4.2.3].
    ///
    /// [§4.2.3]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
    fn derive_dk_ovk(&self) -> (DiversifierKey, OutgoingViewingKey) {
        let k = self.rivk.to_bytes();
        let b = [self.ak.0.into(), self.nk.0.to_bytes()];
        let r = prf_expand_vec(&k, &[&[0x82], &b[0][..], &b[1][..]]);
        (
            DiversifierKey(r.as_bytes()[..32].try_into().unwrap()),
            OutgoingViewingKey(r.as_bytes()[32..].try_into().unwrap()),
        )
    }

    /// Returns the default payment address for this key.
    pub fn default_address(&self) -> Address {
        self.address(DiversifierKey::from(self).default_diversifier())
    }

    /// Returns the payment address for this key corresponding to the given diversifier.
    pub fn address(&self, d: Diversifier) -> Address {
        IncomingViewingKey::from(self).address(d)
    }
}

/// A key that provides the capability to derive a sequence of diversifiers.
#[derive(Debug)]
pub struct DiversifierKey([u8; 32]);

impl From<&FullViewingKey> for DiversifierKey {
    fn from(fvk: &FullViewingKey) -> Self {
        fvk.derive_dk_ovk().0
    }
}

/// The index for a particular diversifier.
#[derive(Clone, Copy, Debug)]
pub struct DiversifierIndex([u8; 11]);

macro_rules! di_from {
    ($n:ident) => {
        impl From<$n> for DiversifierIndex {
            fn from(j: $n) -> Self {
                let mut j_bytes = [0; 11];
                j_bytes[..mem::size_of::<$n>()].copy_from_slice(&j.to_le_bytes());
                DiversifierIndex(j_bytes)
            }
        }
    };
}
di_from!(u8);
di_from!(u16);
di_from!(u32);
di_from!(u64);
di_from!(usize);

impl DiversifierKey {
    /// Returns the diversifier at index 0.
    pub fn default_diversifier(&self) -> Diversifier {
        self.get(0u8)
    }

    /// Returns the diversifier at the given index.
    pub fn get(&self, j: impl Into<DiversifierIndex>) -> Diversifier {
        let ff = FF1::<Aes256>::new(&self.0, 2).expect("valid radix");
        let enc = ff
            .encrypt(&[], &BinaryNumeralString::from_bytes_le(&j.into().0[..]))
            .unwrap();
        Diversifier(enc.to_bytes_le().try_into().unwrap())
    }
}

/// A diversifier that can be used to derive a specific [`Address`] from a
/// [`FullViewingKey`] or [`IncomingViewingKey`].
#[derive(Debug)]
pub struct Diversifier([u8; 11]);

/// A key that provides the capability to detect and decrypt incoming notes from the block
/// chain, without being able to spend the notes or detect when they are spent.
///
/// This key is useful in situations where you only need the capability to detect inbound
/// payments, such as merchant terminals.
///
/// This key is not suitable for use on its own in a wallet, as it cannot maintain
/// accurate balance. You should use a [`FullViewingKey`] instead.
#[derive(Debug)]
pub struct IncomingViewingKey(pallas::Scalar);

impl From<&FullViewingKey> for IncomingViewingKey {
    fn from(fvk: &FullViewingKey) -> Self {
        let ak = pallas::Point::from_bytes(&fvk.ak.0.into()).unwrap();
        IncomingViewingKey(commit_ivk(&extract_p(&ak), &fvk.nk.0, &fvk.rivk))
    }
}

impl IncomingViewingKey {
    /// Returns the payment address for this key corresponding to the given diversifier.
    pub fn address(&self, d: Diversifier) -> Address {
        let g_d = diversify_hash(&d.0);
        Address::from_parts(d, ka_orchard(&self.0, &g_d))
    }
}

/// A key that provides the capability to recover outgoing transaction information from
/// the block chain.
///
/// This key is not suitable for use on its own in a wallet, as it cannot maintain
/// accurate balance. You should use a [`FullViewingKey`] instead.
#[derive(Debug)]
pub struct OutgoingViewingKey([u8; 32]);

impl From<&FullViewingKey> for OutgoingViewingKey {
    fn from(fvk: &FullViewingKey) -> Self {
        fvk.derive_dk_ovk().1
    }
}
