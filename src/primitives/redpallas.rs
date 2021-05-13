//! A minimal RedPallas implementation for use in Zcash.

use std::convert::{TryFrom, TryInto};

use pasta_curves::pallas;
use rand_7::{CryptoRng, RngCore};

/// A RedPallas signature type.
pub trait SigType: reddsa::SigType + private::Sealed {}

/// A type variable corresponding to an Orchard spend authorization signature.
pub type SpendAuth = reddsa::orchard::SpendAuth;
impl SigType for SpendAuth {}

/// A type variable corresponding to an Orchard binding signature.
pub type Binding = reddsa::orchard::Binding;
impl SigType for Binding {}

/// A RedPallas signing key.
#[derive(Debug)]
pub struct SigningKey<T: SigType>(reddsa::SigningKey<T>);

impl<T: SigType> From<SigningKey<T>> for [u8; 32] {
    fn from(sk: SigningKey<T>) -> [u8; 32] {
        sk.0.into()
    }
}

impl<T: SigType> From<&SigningKey<T>> for [u8; 32] {
    fn from(sk: &SigningKey<T>) -> [u8; 32] {
        sk.0.into()
    }
}

impl<T: SigType> TryFrom<[u8; 32]> for SigningKey<T> {
    type Error = reddsa::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        bytes.try_into().map(SigningKey)
    }
}

impl<T: SigType> SigningKey<T> {
    /// Creates a signature of type `T` on `msg` using this `SigningKey`.
    pub fn sign<R: RngCore + CryptoRng>(&self, rng: R, msg: &[u8]) -> Signature<T> {
        Signature(self.0.sign(rng, msg))
    }
}

/// A RedPallas verification key.
#[derive(Clone, Debug)]
pub struct VerificationKey<T: SigType>(reddsa::VerificationKey<T>);

impl<T: SigType> From<VerificationKey<T>> for [u8; 32] {
    fn from(vk: VerificationKey<T>) -> [u8; 32] {
        vk.0.into()
    }
}

impl<T: SigType> From<&VerificationKey<T>> for [u8; 32] {
    fn from(vk: &VerificationKey<T>) -> [u8; 32] {
        vk.0.into()
    }
}

impl<T: SigType> TryFrom<[u8; 32]> for VerificationKey<T> {
    type Error = reddsa::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        bytes.try_into().map(VerificationKey)
    }
}

impl<'a, T: SigType> From<&'a SigningKey<T>> for VerificationKey<T> {
    fn from(sk: &'a SigningKey<T>) -> VerificationKey<T> {
        VerificationKey((&sk.0).into())
    }
}

impl<T: SigType> PartialEq for VerificationKey<T> {
    fn eq(&self, other: &Self) -> bool {
        <[u8; 32]>::from(self).eq(&<[u8; 32]>::from(other))
    }
}

impl VerificationKey<SpendAuth> {
    /// Randomizes this verification key with the given `randomizer`.
    ///
    /// Randomization is only supported for `SpendAuth` keys.
    pub fn randomize(&self, randomizer: &pallas::Scalar) -> Self {
        VerificationKey(self.0.randomize(randomizer))
    }
}

/// A RedPallas signature.
#[derive(Debug, Clone)]
pub struct Signature<T: SigType>(reddsa::Signature<T>);

impl<T: SigType> From<[u8; 64]> for Signature<T> {
    fn from(bytes: [u8; 64]) -> Self {
        Signature(bytes.into())
    }
}

impl<T: SigType> From<&Signature<T>> for [u8; 64] {
    fn from(sig: &Signature<T>) -> Self {
        sig.0.into()
    }
}

pub(crate) mod private {
    use super::{Binding, SpendAuth};

    pub trait Sealed {}

    impl Sealed for SpendAuth {}

    impl Sealed for Binding {}
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use std::convert::TryFrom;

    use proptest::prelude::*;

    use super::{Binding, SigningKey, SpendAuth, VerificationKey};

    prop_compose! {
        /// Generate a uniformly distributed RedDSA spend authorization signing key.
        pub fn arb_spendauth_signing_key()(
            sk in prop::array::uniform32(prop::num::u8::ANY)
                .prop_map(reddsa::SigningKey::try_from)
                .prop_filter("Values must be parseable as valid signing keys", |r| r.is_ok())
        ) -> SigningKey<SpendAuth> {
            SigningKey(sk.unwrap())
        }
    }

    prop_compose! {
        /// Generate a uniformly distributed RedDSA binding signing key.
        pub fn arb_binding_signing_key()(
            sk in prop::array::uniform32(prop::num::u8::ANY)
                .prop_map(reddsa::SigningKey::try_from)
                .prop_filter("Values must be parseable as valid signing keys", |r| r.is_ok())
        ) -> SigningKey<Binding> {
            SigningKey(sk.unwrap())
        }
    }

    prop_compose! {
        /// Generate a uniformly distributed RedDSA spend authorization verification key.
        pub fn arb_spendauth_verification_key()(sk in arb_spendauth_signing_key()) -> VerificationKey<SpendAuth> {
            VerificationKey::from(&sk)
        }
    }

    prop_compose! {
        /// Generate a uniformly distributed RedDSA binding verification key.
        pub fn arb_binding_verification_key()(sk in arb_binding_signing_key()) -> VerificationKey<Binding> {
            VerificationKey::from(&sk)
        }
    }
}
