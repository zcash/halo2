//! Helper functions defined in the Zcash Protocol Specification.

use std::iter;

use blake2b_simd::{Hash, Params};
use ff::PrimeField;
use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, CurveExt, FieldExt},
    pasta::pallas,
};

use crate::{constants::L_ORCHARD_SCALAR, primitives::sinsemilla};

const PRF_EXPAND_PERSONALIZATION: &[u8; 16] = b"Zcash_ExpandSeed";

/// $\mathsf{ToBase}^\mathsf{Orchard}(x) := LEOS2IP_{\ell_\mathsf{PRFexpand}}(x) (mod q_P)$
///
/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][§4.2.3].
///
/// [§4.2.3]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
pub(crate) fn to_base(hash: Hash) -> pallas::Base {
    pallas::Base::from_bytes_wide(hash.as_array())
}

/// $\mathsf{ToScalar}^\mathsf{Orchard}(x) := LEOS2IP_{\ell_\mathsf{PRFexpand}}(x) (mod r_P)$
///
/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][§4.2.3].
///
/// [§4.2.3]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
pub(crate) fn to_scalar(hash: Hash) -> pallas::Scalar {
    pallas::Scalar::from_bytes_wide(hash.as_array())
}

/// Defined in [Zcash Protocol Spec § 4.2.3: Orchard Key Components][§4.2.3].
///
/// [§4.2.3]: https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents
pub(crate) fn commit_ivk(
    ak: &pallas::Base,
    nk: &pallas::Base,
    rivk: &pallas::Scalar,
) -> pallas::Scalar {
    let ivk = sinsemilla::short_commit(
        "z.cash:Orchard-CommitIvk",
        iter::empty()
            .chain(ak.to_le_bits().iter().by_val().take(L_ORCHARD_SCALAR))
            .chain(nk.to_le_bits().iter().by_val().take(L_ORCHARD_SCALAR)),
        rivk,
    );

    // Convert from pallas::Base to pallas::Scalar. This requires no modular reduction
    // because Pallas' base field is smaller than its scalar field.
    pallas::Scalar::from_repr(ivk.to_repr()).unwrap()
}

/// Defined in [Zcash Protocol Spec § 5.4.1.6: DiversifyHash^Sapling and DiversifyHash^Orchard Hash Functions][§5.4.1.6].
///
/// [§5.4.1.6]: https://zips.z.cash/protocol/nu5.pdf#concretediversifyhash
pub(crate) fn diversify_hash(d: &[u8; 11]) -> pallas::Point {
    pallas::Point::hash_to_curve("z.cash:Orchard-gd")(d)
}

/// $PRF^\mathsf{expand}(sk, t) := BLAKE2b-512("Zcash_ExpandSeed", sk || t)$
///
/// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][§5.4.2].
///
/// [§5.4.2]: https://zips.z.cash/protocol/orchard.pdf#concreteprfs
pub(crate) fn prf_expand(sk: &[u8], t: &[u8]) -> Hash {
    prf_expand_vec(sk, &[t])
}

pub(crate) fn prf_expand_vec(sk: &[u8], ts: &[&[u8]]) -> Hash {
    let mut h = Params::new()
        .hash_length(64)
        .personal(PRF_EXPAND_PERSONALIZATION)
        .to_state();
    h.update(sk);
    for t in ts {
        h.update(t);
    }
    h.finalize()
}

/// Defined in [Zcash Protocol Spec § 5.4.4.5: Orchard Key Agreement][§5.4.4.5].
///
/// [§5.4.4.5]: https://zips.z.cash/protocol/nu5.pdf#concreteorchardkeyagreement
pub(crate) fn ka_orchard(sk: &pallas::Scalar, b: &pallas::Point) -> pallas::Point {
    b * sk
}

/// Hash extractor for Pallas, from [§ 5.4.8.7].
///
/// [§ 5.4.8.7]: https://zips.z.cash/protocol/nu5.pdf#concreteextractorpallas
pub(crate) fn extract_p(point: &pallas::Point) -> pallas::Base {
    // TODO: Should we return the actual bits in a Vec, or allow the caller to use
    // PrimeField::to_le_bits on the returned pallas::Base?
    if let Some((x, _)) = point.to_affine().get_xy().into() {
        x
    } else {
        pallas::Base::zero()
    }
}
