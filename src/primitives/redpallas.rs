//! A minimal RedPallas implementation for use in Zcash.

use std::convert::{TryFrom, TryInto};

use pasta_curves::pallas;

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

/// A RedPallas verification key.
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Signature<T: SigType>(reddsa::Signature<T>);

pub(crate) mod private {
    use super::{Binding, SpendAuth};

    pub trait Sealed {}

    impl Sealed for SpendAuth {}

    impl Sealed for Binding {}
}
