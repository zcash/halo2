//! This module contains implementations for the two finite fields of the
//! Tweedledum and Tweedledee curves.

mod fp;
mod fq;

pub use fp::*;
pub use fq::*;

#[cfg(test)]
use crate::arithmetic::Field;

#[test]
fn test_extract() {
    let a = Fq::random();
    let a = a.square();
    let (t, s) = a.extract_radix2_vartime().unwrap();
    assert_eq!(
        t.pow_vartime(&[1 << Fq::S, 0, 0, 0]) * Fq::ROOT_OF_UNITY.pow_vartime(&[s, 0, 0, 0]),
        a
    );
    assert_eq!(a.deterministic_sqrt().unwrap().square(), a);
}
