//! Helper functions defined in the Zcash Protocol Specification.

use std::iter;

use blake2b_simd::Params;
use ff::PrimeField;
use group::{Curve, Group};
use halo2::arithmetic::{CurveAffine, CurveExt, FieldExt};
use pasta_curves::pallas;

use crate::{constants::L_ORCHARD_BASE, primitives::sinsemilla};

const PRF_EXPAND_PERSONALIZATION: &[u8; 16] = b"Zcash_ExpandSeed";

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

/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][orchardkeycomponents].
///
/// [orchardkeycomponents]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
pub(crate) fn commit_ivk(
    ak: &pallas::Base,
    nk: &pallas::Base,
    rivk: &pallas::Scalar,
) -> pallas::Scalar {
    // We rely on the API contract that to_le_bits() returns at least PrimeField::NUM_BITS
    // bits, which is equal to L_ORCHARD_BASE.
    let ivk = sinsemilla::short_commit(
        "z.cash:Orchard-CommitIvk",
        iter::empty()
            .chain(ak.to_le_bits().iter().by_val().take(L_ORCHARD_BASE))
            .chain(nk.to_le_bits().iter().by_val().take(L_ORCHARD_BASE)),
        rivk,
    );

    // Convert from pallas::Base to pallas::Scalar. This requires no modular reduction
    // because Pallas' base field is smaller than its scalar field.
    pallas::Scalar::from_repr(ivk.to_repr()).unwrap()
}

/// Defined in [Zcash Protocol Spec § 5.4.1.6: DiversifyHash^Sapling and DiversifyHash^Orchard Hash Functions][concretediversifyhash].
///
/// [concretediversifyhash]: https://zips.z.cash/protocol/nu5.pdf#concretediversifyhash
pub(crate) fn diversify_hash(d: &[u8; 11]) -> pallas::Point {
    let hasher = pallas::Point::hash_to_curve("z.cash:Orchard-gd");
    let pk_d = hasher(d);
    if pk_d.is_identity().into() {
        // If the identity occurs, we replace it with a different fixed point.
        hasher(&[])
    } else {
        pk_d
    }
}

/// $PRF^\mathsf{expand}(sk, t) := BLAKE2b-512("Zcash_ExpandSeed", sk || t)$
///
/// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
///
/// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
pub(crate) fn prf_expand(sk: &[u8], t: &[u8]) -> [u8; 64] {
    prf_expand_vec(sk, &[t])
}

pub(crate) fn prf_expand_vec(sk: &[u8], ts: &[&[u8]]) -> [u8; 64] {
    let mut h = Params::new()
        .hash_length(64)
        .personal(PRF_EXPAND_PERSONALIZATION)
        .to_state();
    h.update(sk);
    for t in ts {
        h.update(t);
    }
    *h.finalize().as_array()
}

/// Defined in [Zcash Protocol Spec § 5.4.5.5: Orchard Key Agreement][concreteorchardkeyagreement].
///
/// [concreteorchardkeyagreement]: https://zips.z.cash/protocol/nu5.pdf#concreteorchardkeyagreement
pub(crate) fn ka_orchard(sk: &pallas::Scalar, b: &pallas::Point) -> pallas::Point {
    b * sk
}

/// Coordinate extractor for Pallas.
///
/// Defined in [Zcash Protocol Spec § 5.4.9.7: Coordinate Extractor for Pallas][concreteextractorpallas].
///
/// [concreteextractorpallas]: https://zips.z.cash/protocol/nu5.pdf#concreteextractorpallas
pub(crate) fn extract_p(point: &pallas::Point) -> pallas::Base {
    if let Some((x, _)) = point.to_affine().get_xy().into() {
        x
    } else {
        pallas::Base::zero()
    }
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
