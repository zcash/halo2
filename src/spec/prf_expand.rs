use blake2b_simd::Params;

const PRF_EXPAND_PERSONALIZATION: &[u8; 16] = b"Zcash_ExpandSeed";

/// The set of domains in which $PRF^\mathsf{expand}$ is defined.
pub(crate) enum PrfExpand {
    Esk,
    Rcm,
    OrchardAsk,
    OrchardNk,
    OrchardRivk,
    Psi,
    OrchardDkOvk,
}

impl PrfExpand {
    fn domain_separator(&self) -> u8 {
        match self {
            Self::Esk => 0x04,
            Self::Rcm => 0x05,
            Self::OrchardAsk => 0x06,
            Self::OrchardNk => 0x07,
            Self::OrchardRivk => 0x08,
            Self::Psi => 0x09,
            Self::OrchardDkOvk => 0x82,
        }
    }

    /// Expands the given secret key in this domain, with no additional data.
    ///
    /// $PRF^\mathsf{expand}(sk, dst) := BLAKE2b-512("Zcash_ExpandSeed", sk || dst)$
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
    ///
    /// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
    pub(crate) fn expand(self, sk: &[u8]) -> [u8; 64] {
        self.with_ad_slices(sk, &[])
    }

    /// Expands the given secret key in this domain, with the given additional data.
    ///
    /// $PRF^\mathsf{expand}(sk, dst, t) := BLAKE2b-512("Zcash_ExpandSeed", sk || dst || t)$
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
    ///
    /// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
    pub(crate) fn with_ad(self, sk: &[u8], t: &[u8]) -> [u8; 64] {
        self.with_ad_slices(sk, &[t])
    }

    /// Expands the given secret key in this domain, with additional data concatenated
    /// from the given slices.
    ///
    /// $PRF^\mathsf{expand}(sk, dst, a, b, ...) := BLAKE2b-512("Zcash_ExpandSeed", sk || dst || a || b || ...)$
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
    ///
    /// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
    pub(crate) fn with_ad_slices(self, sk: &[u8], ts: &[&[u8]]) -> [u8; 64] {
        let mut h = Params::new()
            .hash_length(64)
            .personal(PRF_EXPAND_PERSONALIZATION)
            .to_state();
        h.update(sk);
        h.update(&[self.domain_separator()]);
        for t in ts {
            h.update(t);
        }
        *h.finalize().as_array()
    }
}
