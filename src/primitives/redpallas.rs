//! TODO

use std::fmt;
use std::marker::PhantomData;

/// A RedPallas signature type.
pub trait SigType: private::Sealed + fmt::Debug {}

/// A type variable corresponding to an Orchard spend authorization signature.
#[derive(Debug)]
pub enum SpendAuth {}
impl SigType for SpendAuth {}

/// A type variable corresponding to an Orchard binding signature.
#[derive(Debug)]
pub enum Binding {}
impl SigType for Binding {}

/// A RedPallas signing key.
#[derive(Debug)]
pub struct SigningKey<T: SigType> {
    _t: PhantomData<T>,
}

/// A RedPallas verification key.
#[derive(Debug)]
pub struct VerificationKey<T: SigType> {
    _t: PhantomData<T>,
}

/// A RedPallas signature.
#[derive(Debug)]
pub struct Signature<T: SigType> {
    _t: PhantomData<T>,
}

pub(crate) mod private {
    use super::{Binding, SpendAuth};

    pub trait Sealed {}

    impl Sealed for SpendAuth {}

    impl Sealed for Binding {}
}
