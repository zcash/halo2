//! Helper functions defined in the Zcash Protocol Specification.

use std::iter;
use std::ops::Deref;

use ff::{Field, PrimeField, PrimeFieldBits};
use group::GroupEncoding;
use group::{Curve, Group};
use halo2::arithmetic::{CurveAffine, CurveExt, FieldExt};
use pasta_curves::pallas;
use subtle::{ConditionallySelectable, CtOption};

use crate::{
    constants::L_ORCHARD_BASE,
    primitives::{poseidon, sinsemilla},
};

mod prf_expand;
pub(crate) use prf_expand::PrfExpand;

/// A Pallas point that is guaranteed to not be the identity.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NonIdentityPallasPoint(pallas::Point);

impl ConditionallySelectable for NonIdentityPallasPoint {
    fn conditional_select(a: &Self, b: &Self, choice: subtle::Choice) -> Self {
        NonIdentityPallasPoint(pallas::Point::conditional_select(&a.0, &b.0, choice))
    }
}

impl NonIdentityPallasPoint {
    pub(crate) fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Point::from_bytes(bytes)
            .and_then(|p| CtOption::new(NonIdentityPallasPoint(p), !p.is_identity()))
    }
}

impl Deref for NonIdentityPallasPoint {
    type Target = pallas::Point;

    fn deref(&self) -> &pallas::Point {
        &self.0
    }
}

/// An integer in [1..q_P].
pub(crate) struct NonZeroPallasBase(pallas::Base);

impl NonZeroPallasBase {
    /// Constructs a wrapper for a base field element that is guaranteed to be non-zero.
    ///
    /// # Panics
    ///
    /// Panics if `s.is_zero()`.
    fn guaranteed(s: pallas::Base) -> Self {
        assert!(!s.is_zero());
        NonZeroPallasBase(s)
    }
}

/// An integer in [1..r_P].
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NonZeroPallasScalar(pallas::Scalar);

impl From<NonZeroPallasBase> for NonZeroPallasScalar {
    fn from(s: NonZeroPallasBase) -> Self {
        NonZeroPallasScalar::guaranteed(mod_r_p(s.0))
    }
}

impl ConditionallySelectable for NonZeroPallasScalar {
    fn conditional_select(a: &Self, b: &Self, choice: subtle::Choice) -> Self {
        NonZeroPallasScalar(pallas::Scalar::conditional_select(&a.0, &b.0, choice))
    }
}

impl NonZeroPallasScalar {
    pub(crate) fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Scalar::from_bytes(bytes).and_then(NonZeroPallasScalar::from_scalar)
    }

    pub(crate) fn from_scalar(s: pallas::Scalar) -> CtOption<Self> {
        CtOption::new(NonZeroPallasScalar(s), !s.ct_is_zero())
    }

    /// Constructs a wrapper for a scalar field element that is guaranteed to be non-zero.
    ///
    /// # Panics
    ///
    /// Panics if `s.is_zero()`.
    fn guaranteed(s: pallas::Scalar) -> Self {
        assert!(!s.is_zero());
        NonZeroPallasScalar(s)
    }
}

impl Deref for NonZeroPallasScalar {
    type Target = pallas::Scalar;

    fn deref(&self) -> &pallas::Scalar {
        &self.0
    }
}

/// $\mathsf{ToBase}^\mathsf{Orchard}(x) := LEOS2IP_{\ell_\mathsf{PRFexpand}}(x) (mod q_P)$
///
/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][orchardkeycomponents].
///
/// [orchardkeycomponents]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
pub(crate) fn to_base(x: [u8; 64]) -> pallas::Base {
    pallas::Base::from_bytes_wide(&x)
}

/// $\mathsf{ToScalar}^\mathsf{Orchard}(x) := LEOS2IP_{\ell_\mathsf{PRFexpand}}(x) (mod r_P)$
///
/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][orchardkeycomponents].
///
/// [orchardkeycomponents]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
pub(crate) fn to_scalar(x: [u8; 64]) -> pallas::Scalar {
    pallas::Scalar::from_bytes_wide(&x)
}

/// Converts from pallas::Base to pallas::Scalar (aka $x \pmod{r_\mathbb{P}}$).
///
/// This requires no modular reduction because Pallas' base field is smaller than its
/// scalar field.
pub(crate) fn mod_r_p(x: pallas::Base) -> pallas::Scalar {
    pallas::Scalar::from_repr(x.to_repr()).unwrap()
}

/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][orchardkeycomponents].
///
/// [orchardkeycomponents]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
pub(crate) fn commit_ivk(
    ak: &pallas::Base,
    nk: &pallas::Base,
    rivk: &pallas::Scalar,
) -> CtOption<NonZeroPallasBase> {
    // We rely on the API contract that to_le_bits() returns at least PrimeField::NUM_BITS
    // bits, which is equal to L_ORCHARD_BASE.
    let domain = sinsemilla::CommitDomain::new(&"z.cash:Orchard-CommitIvk");
    domain
        .short_commit(
            iter::empty()
                .chain(ak.to_le_bits().iter().by_val().take(L_ORCHARD_BASE))
                .chain(nk.to_le_bits().iter().by_val().take(L_ORCHARD_BASE)),
            rivk,
        )
        // Commit^ivk.Output is specified as [1..q_P] ∪ {⊥}. We get this from
        // sinsemilla::CommitDomain::short_commit by construction:
        // - 0 is not a valid x-coordinate for any Pallas point.
        // - sinsemilla::CommitDomain::short_commit calls extract_p_bottom, which replaces
        //   the identity (which has no affine coordinates) with 0. but Sinsemilla is
        //   defined using incomplete addition, and thus will never produce the identity.
        .map(NonZeroPallasBase::guaranteed)
}

/// Defined in [Zcash Protocol Spec § 5.4.1.6: DiversifyHash^Sapling and DiversifyHash^Orchard Hash Functions][concretediversifyhash].
///
/// [concretediversifyhash]: https://zips.z.cash/protocol/nu5.pdf#concretediversifyhash
pub(crate) fn diversify_hash(d: &[u8; 11]) -> NonIdentityPallasPoint {
    let hasher = pallas::Point::hash_to_curve("z.cash:Orchard-gd");
    let pk_d = hasher(d);
    // If the identity occurs, we replace it with a different fixed point.
    // TODO: Replace the unwrap_or_else with a cached fixed point.
    NonIdentityPallasPoint(CtOption::new(pk_d, !pk_d.is_identity()).unwrap_or_else(|| hasher(&[])))
}

/// $PRF^\mathsf{nfOrchard}(nk, \rho) := Poseidon(nk, \rho)$
///
/// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
///
/// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
pub(crate) fn prf_nf(nk: pallas::Base, rho: pallas::Base) -> pallas::Base {
    poseidon::Hash::init(poseidon::OrchardNullifier, poseidon::ConstantLength).hash([nk, rho])
}

/// Defined in [Zcash Protocol Spec § 5.4.5.5: Orchard Key Agreement][concreteorchardkeyagreement].
///
/// [concreteorchardkeyagreement]: https://zips.z.cash/protocol/nu5.pdf#concreteorchardkeyagreement
pub(crate) fn ka_orchard(
    sk: &NonZeroPallasScalar,
    b: &NonIdentityPallasPoint,
) -> NonIdentityPallasPoint {
    NonIdentityPallasPoint(b.deref() * sk.deref())
}

/// Coordinate extractor for Pallas.
///
/// Defined in [Zcash Protocol Spec § 5.4.9.7: Coordinate Extractor for Pallas][concreteextractorpallas].
///
/// [concreteextractorpallas]: https://zips.z.cash/protocol/nu5.pdf#concreteextractorpallas
pub(crate) fn extract_p(point: &pallas::Point) -> pallas::Base {
    point
        .to_affine()
        .coordinates()
        .map(|c| *c.x())
        .unwrap_or_else(pallas::Base::zero)
}

/// Coordinate extractor for Pallas.
///
/// Defined in [Zcash Protocol Spec § 5.4.9.7: Coordinate Extractor for Pallas][concreteextractorpallas].
///
/// [concreteextractorpallas]: https://zips.z.cash/protocol/nu5.pdf#concreteextractorpallas
pub(crate) fn extract_p_bottom(point: CtOption<pallas::Point>) -> CtOption<pallas::Base> {
    point.map(|p| extract_p(&p))
}

#[cfg(test)]
mod tests {
    use group::Group;
    use halo2::arithmetic::CurveExt;
    use pasta_curves::pallas;

    #[test]
    fn diversify_hash_substitution() {
        assert!(!bool::from(
            pallas::Point::hash_to_curve("z.cash:Orchard-gd")(&[]).is_identity()
        ));
    }
}
