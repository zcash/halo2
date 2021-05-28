use blake2b_simd::Params;

const PRF_EXPAND_PERSONALIZATION: &[u8; 16] = b"Zcash_ExpandSeed";

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
