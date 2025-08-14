#![cfg_attr(feature = "asm", feature(asm_const))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

#[macro_use]
mod macros;
mod curves;
mod fields;

pub use pasta_curves::arithmetic::{Coordinates, CurveAffine, CurveExt, FieldExt, Group};
pub mod arithmetic;
pub mod secp256k1;
pub mod secq256k1;

#[cfg(feature = "alloc")]
mod hashtocurve;

pub use curves::*;
pub use fields::*;

pub extern crate group;

#[cfg(test)]
pub mod tests;

#[cfg(all(feature = "prefetch", target_arch = "x86_64"))]
#[inline(always)]
pub fn prefetch<T>(data: &[T], offset: usize) {
    use core::arch::x86_64::_mm_prefetch;
    unsafe {
        _mm_prefetch(
            data.as_ptr().offset(offset as isize) as *const i8,
            core::arch::x86_64::_MM_HINT_T0,
        );
    }
}
