//! A minimal RedPallas implementation for use in Zcash.

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

/// A RedPallas verification key.
#[derive(Debug)]
pub struct VerificationKey<T: SigType>(reddsa::VerificationKey<T>);

/// A RedPallas signature.
#[derive(Debug)]
pub struct Signature<T: SigType>(reddsa::Signature<T>);

pub(crate) mod private {
    use super::{Binding, SpendAuth};

    pub trait Sealed {}

    impl Sealed for SpendAuth {}

    impl Sealed for Binding {}
}
