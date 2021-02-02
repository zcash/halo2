//! This module contains implementations for the Pallas and Vesta elliptic curve
//! groups.

#[macro_use]
mod macros;
mod curves;
mod fields;

mod hashtocurve;
pub mod pallas;
pub mod vesta;

pub use curves::*;
pub use fields::*;
use hashtocurve::*;

#[test]
fn test_endo_consistency() {
    use crate::arithmetic::{Curve, FieldExt};

    let a = pallas::Point::one();
    assert_eq!(a * pallas::Scalar::ZETA, a.endo());
    let a = vesta::Point::one();
    assert_eq!(a * vesta::Scalar::ZETA, a.endo());
}
