use std::convert::TryInto;

use crate::constants::{self, compute_lagrange_coeffs, H, NUM_WINDOWS, NUM_WINDOWS_SHORT};
use halo2::arithmetic::{CurveAffine, FieldExt};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrchardFixedBasesFull<C: CurveAffine> {
    CommitIvkR(PhantomData<C>),
    NoteCommitR(PhantomData<C>),
    NullifierK(PhantomData<C>),
    ValueCommitR(PhantomData<C>),
    SpendAuthG(PhantomData<C>),
}

/// A fixed base to be used in scalar multiplication with a full-width scalar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchardFixedBase<C: CurveAffine> {
    pub generator: C,
    pub lagrange_coeffs: LagrangeCoeffs<C::Base>,
    pub z: Z<C::Base>,
    pub u: U<C::Base>,
}

impl<C: CurveAffine> From<OrchardFixedBasesFull<C>> for OrchardFixedBase<C> {
    fn from(base: OrchardFixedBasesFull<C>) -> Self {
        let (generator, z, u) = match base {
            OrchardFixedBasesFull::CommitIvkR(_) => (
                super::commit_ivk_r::generator(),
                super::commit_ivk_r::Z.into(),
                super::commit_ivk_r::U.into(),
            ),
            OrchardFixedBasesFull::NoteCommitR(_) => (
                super::note_commit_r::generator(),
                super::note_commit_r::Z.into(),
                super::note_commit_r::U.into(),
            ),
            OrchardFixedBasesFull::NullifierK(_) => (
                super::nullifier_k::generator(),
                super::nullifier_k::Z.into(),
                super::nullifier_k::U.into(),
            ),
            OrchardFixedBasesFull::ValueCommitR(_) => (
                super::value_commit_r::generator(),
                super::value_commit_r::Z.into(),
                super::value_commit_r::U.into(),
            ),
            OrchardFixedBasesFull::SpendAuthG(_) => (
                super::spend_auth_g::generator(),
                super::spend_auth_g::Z.into(),
                super::spend_auth_g::U.into(),
            ),
        };

        Self {
            generator,
            lagrange_coeffs: compute_lagrange_coeffs(generator, NUM_WINDOWS).into(),
            z,
            u,
        }
    }
}

/// A fixed base to be used in scalar multiplication with a short signed exponent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValueCommitV<C: CurveAffine> {
    pub generator: C,
    pub lagrange_coeffs_short: LagrangeCoeffsShort<C::Base>,
    pub z_short: ZShort<C::Base>,
    pub u_short: UShort<C::Base>,
}

impl<C: CurveAffine> ValueCommitV<C> {
    pub fn get() -> Self {
        let generator = super::value_commit_v::generator();
        Self {
            generator,
            lagrange_coeffs_short: compute_lagrange_coeffs(generator, NUM_WINDOWS_SHORT).into(),
            z_short: super::value_commit_v::Z_SHORT.into(),
            u_short: super::value_commit_v::U_SHORT.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 8 coefficients per window
pub struct WindowLagrangeCoeffs<F: FieldExt>(pub Box<[F; H]>);

impl<F: FieldExt> From<&[F; H]> for WindowLagrangeCoeffs<F> {
    fn from(array: &[F; H]) -> Self {
        Self(Box::new(*array))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 85 windows per base (with the exception of ValueCommitV)
pub struct LagrangeCoeffs<F: FieldExt>(pub Box<[WindowLagrangeCoeffs<F>; constants::NUM_WINDOWS]>);

impl<F: FieldExt> From<Vec<WindowLagrangeCoeffs<F>>> for LagrangeCoeffs<F> {
    fn from(windows: Vec<WindowLagrangeCoeffs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<Vec<[F; H]>> for LagrangeCoeffs<F> {
    fn from(arrays: Vec<[F; H]>) -> Self {
        let windows: Vec<WindowLagrangeCoeffs<F>> =
            arrays.iter().map(|array| array.into()).collect();
        windows.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 22 windows for ValueCommitV
pub struct LagrangeCoeffsShort<F: FieldExt>(pub Box<[WindowLagrangeCoeffs<F>; NUM_WINDOWS_SHORT]>);

impl<F: FieldExt> From<Vec<WindowLagrangeCoeffs<F>>> for LagrangeCoeffsShort<F> {
    fn from(windows: Vec<WindowLagrangeCoeffs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<Vec<[F; H]>> for LagrangeCoeffsShort<F> {
    fn from(arrays: Vec<[F; H]>) -> Self {
        let windows: Vec<WindowLagrangeCoeffs<F>> =
            arrays.iter().map(|array| array.into()).collect();
        windows.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 85 Z's per base (with the exception of ValueCommitV)
pub struct Z<F: FieldExt>(pub Box<[F; NUM_WINDOWS]>);

impl<F: FieldExt> From<[u64; NUM_WINDOWS]> for Z<F> {
    fn from(zs: [u64; NUM_WINDOWS]) -> Self {
        Self(
            zs.iter()
                .map(|z| F::from_u64(*z))
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 22 Z's for ValueCommitV
pub struct ZShort<F: FieldExt>(pub Box<[F; NUM_WINDOWS_SHORT]>);

impl<F: FieldExt> From<[u64; NUM_WINDOWS_SHORT]> for ZShort<F> {
    fn from(zs: [u64; NUM_WINDOWS_SHORT]) -> Self {
        Self(
            zs.iter()
                .map(|z| F::from_u64(*z))
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 8 u's per window
pub struct WindowUs<F: FieldExt>(pub Box<[F; H]>);

impl<F: FieldExt> From<&[[u8; 32]; H]> for WindowUs<F> {
    fn from(window_us: &[[u8; 32]; H]) -> Self {
        Self(
            window_us
                .iter()
                .map(|u| F::from_bytes(&u).unwrap())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 85 windows per base (with the exception of ValueCommitV)
pub struct U<F: FieldExt>(pub Box<[WindowUs<F>; NUM_WINDOWS]>);

impl<F: FieldExt> From<Vec<WindowUs<F>>> for U<F> {
    fn from(windows: Vec<WindowUs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<[[[u8; 32]; H]; NUM_WINDOWS]> for U<F> {
    fn from(window_us: [[[u8; 32]; H]; NUM_WINDOWS]) -> Self {
        let windows: Vec<WindowUs<F>> = window_us.iter().map(|us| us.into()).collect();
        windows.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 22 windows for ValueCommitV
pub struct UShort<F: FieldExt>(pub Box<[WindowUs<F>; NUM_WINDOWS_SHORT]>);

impl<F: FieldExt> From<Vec<WindowUs<F>>> for UShort<F> {
    fn from(windows: Vec<WindowUs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<[[[u8; 32]; H]; NUM_WINDOWS_SHORT]> for UShort<F> {
    fn from(window_us: [[[u8; 32]; H]; NUM_WINDOWS_SHORT]) -> Self {
        let windows: Vec<WindowUs<F>> = window_us.iter().map(|us| us.into()).collect();
        windows.into()
    }
}
