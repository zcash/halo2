//! Key structures for Orchard.

use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

use blake2b_simd::Params as Blake2bParams;
use subtle::{Choice, ConstantTimeEq};

use crate::{
    keys::{FullViewingKey, SpendingKey},
    spec::PrfExpand,
};

const ZIP32_ORCHARD_PERSONALIZATION: &[u8; 16] = b"ZcashIP32Orchard";
const ZIP32_ORCHARD_FVFP_PERSONALIZATION: &[u8; 16] = b"ZcashOrchardFVFP";

/// Errors produced in derivation of extended spending keys
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// A seed resulted in an invalid spending key
    InvalidSpendingKey,
    /// A child index in a derivation path exceeded 2^31
    InvalidChildIndex(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Seed produced invalid spending key.")
    }
}

//impl std::error::Error for Error {}

/// An Orchard full viewing key fingerprint
struct FvkFingerprint([u8; 32]);

impl From<&FullViewingKey> for FvkFingerprint {
    fn from(fvk: &FullViewingKey) -> Self {
        let mut h = Blake2bParams::new()
            .hash_length(32)
            .personal(ZIP32_ORCHARD_FVFP_PERSONALIZATION)
            .to_state();
        h.update(&fvk.to_bytes());
        let mut fvfp = [0u8; 32];
        fvfp.copy_from_slice(h.finalize().as_bytes());
        FvkFingerprint(fvfp)
    }
}

/// An Orchard full viewing key tag
#[derive(Clone, Copy, Debug, PartialEq)]
struct FvkTag([u8; 4]);

impl FvkFingerprint {
    fn tag(&self) -> FvkTag {
        let mut tag = [0u8; 4];
        tag.copy_from_slice(&self.0[..4]);
        FvkTag(tag)
    }
}

impl FvkTag {
    fn master() -> Self {
        FvkTag([0u8; 4])
    }
}

/// A hardened child index for a derived key.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChildIndex(u32);

impl TryFrom<u32> for ChildIndex {
    type Error = Error;

    /// `index` must be less than 2^31
    fn try_from(index: u32) -> Result<Self, Self::Error> {
        if index < (1 << 31) {
            Ok(Self(index + (1 << 31)))
        } else {
            Err(Error::InvalidChildIndex(32))
        }
    }
}

/// The chain code forming the second half of an Orchard extended key.
#[derive(Debug, Copy, Clone, PartialEq)]
struct ChainCode([u8; 32]);

/// An Orchard extended spending key.
///
/// Defined in [ZIP32: Orchard extended keys][orchardextendedkeys].
///
/// [orchardextendedkeys]: https://zips.z.cash/zip-0032#orchard-extended-keys
#[derive(Debug, Clone)]
pub(crate) struct ExtendedSpendingKey {
    depth: u8,
    parent_fvk_tag: FvkTag,
    child_index: ChildIndex,
    chain_code: ChainCode,
    sk: SpendingKey,
}

impl ConstantTimeEq for ExtendedSpendingKey {
    fn ct_eq(&self, rhs: &Self) -> Choice {
        self.depth.ct_eq(&rhs.depth)
            & self.parent_fvk_tag.0.ct_eq(&rhs.parent_fvk_tag.0)
            & self.child_index.0.ct_eq(&rhs.child_index.0)
            & self.chain_code.0.ct_eq(&rhs.chain_code.0)
            & self.sk.ct_eq(&rhs.sk)
    }
}

#[allow(non_snake_case)]
impl ExtendedSpendingKey {
    /// Returns the spending key of the child key corresponding to
    /// the path derived from the master key
    ///
    /// # Panics
    ///
    /// Panics if seed results in invalid spending key.
    pub fn from_path(seed: &[u8], path: &[ChildIndex]) -> Result<Self, Error> {
        let mut xsk = Self::master(seed)?;
        for i in path {
            xsk = xsk.derive_child(*i)?;
        }
        Ok(xsk)
    }

    /// Generates the master key of an Orchard extended spending key.
    ///
    /// Defined in [ZIP32: Orchard master key generation][orchardmasterkey].
    ///
    /// [orchardmasterkey]: https://zips.z.cash/zip-0032#orchard-master-key-generation
    ///
    /// # Panics
    ///
    /// Panics if the seed is shorter than 32 bytes or longer than 252 bytes.
    fn master(seed: &[u8]) -> Result<Self, Error> {
        assert!(seed.len() >= 32 && seed.len() <= 252);
        // I := BLAKE2b-512("ZcashIP32Orchard", seed)
        let I: [u8; 64] = {
            let mut I = Blake2bParams::new()
                .hash_length(64)
                .personal(ZIP32_ORCHARD_PERSONALIZATION)
                .to_state();
            I.update(seed);
            I.finalize().as_bytes().try_into().unwrap()
        };
        // I_L is used as the master spending key sk_m.
        let sk_m = SpendingKey::from_bytes(I[..32].try_into().unwrap());
        if sk_m.is_none().into() {
            return Err(Error::InvalidSpendingKey);
        }
        let sk_m = sk_m.unwrap();

        // I_R is used as the master chain code c_m.
        let c_m = ChainCode(I[32..].try_into().unwrap());

        // For the master extended spending key, depth is 0, parent_fvk_tag is 4 zero bytes, and i is 0.
        Ok(Self {
            depth: 0,
            parent_fvk_tag: FvkTag([0; 4]),
            child_index: ChildIndex(0),
            chain_code: c_m,
            sk: sk_m,
        })
    }

    /// Derives a child key from a parent key at a given index.
    ///
    /// Defined in [ZIP32: Orchard child key derivation][orchardchildkey].
    ///
    /// [orchardchildkey]: https://zips.z.cash/zip-0032#orchard-child-key-derivation
    ///
    /// Discards index if it results in an invalid sk
    fn derive_child(&self, index: ChildIndex) -> Result<Self, Error> {
        // I := PRF^Expand(c_par, [0x81] || sk_par || I2LEOSP(i))
        let I: [u8; 64] = PrfExpand::OrchardZip32Child.with_ad_slices(
            &self.chain_code.0,
            &[self.sk.to_bytes(), &index.0.to_le_bytes()],
        );

        // I_L is used as the child spending key sk_i.
        let sk_i = SpendingKey::from_bytes(I[..32].try_into().unwrap());
        if sk_i.is_none().into() {
            return Err(Error::InvalidSpendingKey);
        }
        let sk_i = sk_i.unwrap();

        // I_R is used as the child chain code c_i.
        let c_i = ChainCode(I[32..].try_into().unwrap());

        let fvk: FullViewingKey = self.into();

        Ok(Self {
            depth: self.depth + 1,
            parent_fvk_tag: FvkFingerprint::from(&fvk).tag(),
            child_index: index,
            chain_code: c_i,
            sk: sk_i,
        })
    }

    /// Returns sk of this ExtendedSpendingKey.
    pub fn sk(&self) -> SpendingKey {
        self.sk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_child() {
        let seed = [0; 32];
        let xsk_m = ExtendedSpendingKey::master(&seed).unwrap();

        let i_5 = 5;
        let xsk_5 = xsk_m.derive_child(i_5.try_into().unwrap());

        assert!(xsk_5.is_ok());
    }

    #[test]
    fn path() {
        let seed = [0; 32];
        let xsk_m = ExtendedSpendingKey::master(&seed).unwrap();

        let xsk_5h = xsk_m.derive_child(5.try_into().unwrap()).unwrap();
        assert!(bool::from(
            ExtendedSpendingKey::from_path(&seed, &[5.try_into().unwrap()])
                .unwrap()
                .ct_eq(&xsk_5h)
        ));

        let xsk_5h_7 = xsk_5h.derive_child(7.try_into().unwrap()).unwrap();
        assert!(bool::from(
            ExtendedSpendingKey::from_path(&seed, &[5.try_into().unwrap(), 7.try_into().unwrap()])
                .unwrap()
                .ct_eq(&xsk_5h_7)
        ));
    }
}
