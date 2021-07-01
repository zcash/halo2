//! Utility functions for computing bundle commitments

use blake2b_simd::{Hash as Blake2bHash, Params, State};
use std::io::Write;

use crate::bundle::{Authorization, Authorized, Bundle};

const ZCASH_ORCHARD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActCHash";
const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActNHash";
const ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";

fn hasher(personal: &[u8; 16]) -> State {
    Params::new().hash_length(32).personal(personal).to_state()
}

/// Write disjoint parts of each Orchard shielded action as 3 separate hashes:
/// * \[(nullifier, cmx, ephemeral_key, enc_ciphertext\[..52\])*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION
/// * \[enc_ciphertext\[52..564\]*\] (memo ciphertexts) personalized
///   with ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION
/// * \[(cv, rk, enc_ciphertext\[564..\], out_ciphertext)*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION
/// as defined in [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// Then, hash these together along with (flags, value_balance_orchard, anchor_orchard),
/// personalized with ZCASH_ORCHARD_ACTIONS_HASH_PERSONALIZATION
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_txid_data<'a, A: Authorization, V>(bundle: &'a Bundle<A, V>) -> Blake2bHash
where
    i64: From<&'a V>,
{
    let mut h = hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION);
    let mut ch = hasher(ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION);
    let mut mh = hasher(ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION);
    let mut nh = hasher(ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION);

    for action in bundle.actions().iter() {
        ch.write_all(&action.nullifier().to_bytes()).unwrap();
        ch.write_all(&action.cmx().to_bytes()).unwrap();
        ch.write_all(&action.encrypted_note().epk_bytes).unwrap();
        ch.write_all(&action.encrypted_note().enc_ciphertext[..52])
            .unwrap();

        mh.write_all(&action.encrypted_note().enc_ciphertext[52..564])
            .unwrap();

        nh.write_all(&action.cv_net().to_bytes()).unwrap();
        nh.write_all(&<[u8; 32]>::from(action.rk())).unwrap();
        nh.write_all(&action.encrypted_note().enc_ciphertext[564..])
            .unwrap();
        nh.write_all(&action.encrypted_note().out_ciphertext)
            .unwrap();
    }

    h.write_all(&ch.finalize().as_bytes()).unwrap();
    h.write_all(&mh.finalize().as_bytes()).unwrap();
    h.write_all(&nh.finalize().as_bytes()).unwrap();
    h.write_all(&[bundle.flags().to_byte()]).unwrap();
    h.write_all(&<i64>::from(bundle.value_balance()).to_le_bytes())
        .unwrap();
    h.write_all(&bundle.anchor().to_bytes()).unwrap();
    h.finalize()
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_txid_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION).finalize()
}

/// Construct the commitment to the authorizing data of an
/// authorized bundle as defined in [ZIP-244: Transaction
/// Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_auth_data<V>(bundle: &Bundle<Authorized, V>) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION);
    h.write_all(bundle.authorization().proof().as_ref())
        .unwrap();
    for action in bundle.actions().iter() {
        h.write_all(&<[u8; 64]>::from(action.authorization()))
            .unwrap();
    }
    h.write_all(&<[u8; 64]>::from(
        bundle.authorization().binding_signature(),
    ))
    .unwrap();
    h.finalize()
}

/// Construct the commitment for an absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_auth_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION).finalize()
}
