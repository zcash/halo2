//! Alternative `AirBuilderWithPublicValues` trait that uses `Self::Var` instead of `Self::F`.

use p3_air::AirBuilder;

pub trait AirBuilderWithPublicValues: AirBuilder {
    fn public_values(&self) -> &[Self::Var];
}
