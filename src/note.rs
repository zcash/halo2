//! Data structures used for note construction.
use group::GroupEncoding;
use pasta_curves::pallas;
use rand::RngCore;
use subtle::CtOption;

use crate::{
    keys::{FullViewingKey, SpendingKey},
    spec::{prf_expand_vec, to_base, to_scalar},
    value::NoteValue,
    Address,
};

pub(crate) mod commitment;
pub use self::commitment::{ExtractedNoteCommitment, NoteCommitment};

pub(crate) mod nullifier;
pub use self::nullifier::Nullifier;

/// The ZIP 212 seed randomness for a note.
#[derive(Clone, Debug)]
pub(crate) struct RandomSeed([u8; 32]);

impl From<[u8; 32]> for RandomSeed {
    fn from(rseed: [u8; 32]) -> Self {
        RandomSeed(rseed)
    }
}

impl RandomSeed {
    pub(crate) fn random(rng: &mut impl RngCore) -> Self {
        let mut bytes = [0; 32];
        rng.fill_bytes(&mut bytes);
        RandomSeed(bytes)
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn psi(&self, rho: &Nullifier) -> pallas::Base {
        to_base(prf_expand_vec(&self.0, &[&[0x09], &rho.to_bytes()[..]]))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk(&self, rho: &Nullifier) -> pallas::Scalar {
        to_scalar(prf_expand_vec(&self.0, &[&[0x04], &rho.to_bytes()[..]]))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn rcm(&self, rho: &Nullifier) -> commitment::NoteCommitTrapdoor {
        commitment::NoteCommitTrapdoor(to_scalar(prf_expand_vec(
            &self.0,
            &[&[0x05], &rho.to_bytes()[..]],
        )))
    }
}

/// A discrete amount of funds received by an address.
#[derive(Debug)]
pub struct Note {
    /// The recipient of the funds.
    recipient: Address,
    /// The value of this note.
    value: NoteValue,
    /// A unique creation ID for this note.
    ///
    /// This is set to the nullifier of the note that was spent in the [`Action`] that
    /// created this note.
    ///
    /// [`Action`]: crate::bundle::Action
    rho: Nullifier,
    /// The seed randomness for various note components.
    rseed: RandomSeed,
}

impl Note {
    #[cfg(test)]
    pub(crate) fn from_parts(
        recipient: Address,
        value: NoteValue,
        rho: Nullifier,
        rseed: RandomSeed,
    ) -> Self {
        Note {
            recipient,
            value,
            rho,
            rseed,
        }
    }

    /// Generates a new note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    pub(crate) fn new(
        recipient: Address,
        value: NoteValue,
        rho: Nullifier,
        mut rng: impl RngCore,
    ) -> Self {
        loop {
            let note = Note {
                recipient,
                value,
                rho,
                rseed: RandomSeed::random(&mut rng),
            };
            if note.commitment_inner().is_some().into() {
                break note;
            }
        }
    }

    /// Generates a dummy spent note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    pub(crate) fn dummy(
        rng: &mut impl RngCore,
        rho: Option<Nullifier>,
    ) -> (SpendingKey, FullViewingKey, Self) {
        let sk = SpendingKey::random(rng);
        let fvk: FullViewingKey = (&sk).into();
        let recipient = fvk.default_address();

        let note = Note::new(
            recipient,
            NoteValue::zero(),
            rho.unwrap_or_else(|| Nullifier::dummy(rng)),
            rng,
        );

        (sk, fvk, note)
    }

    /// Returns the value of this note.
    pub fn value(&self) -> NoteValue {
        self.value
    }

    /// Derives the commitment to this note.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    pub fn commitment(&self) -> NoteCommitment {
        // `Note` will always have a note commitment by construction.
        self.commitment_inner().unwrap()
    }

    /// Derives the commitment to this note.
    ///
    /// This is the internal fallible API, used to check at construction time that the
    /// note has a commitment. Once you have a [`Note`] object, use `note.commitment()`
    /// instead.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    fn commitment_inner(&self) -> CtOption<NoteCommitment> {
        let g_d = self.recipient.g_d();

        NoteCommitment::derive(
            g_d.to_bytes(),
            self.recipient.pk_d().to_bytes(),
            self.value,
            self.rho.0,
            self.rseed.psi(&self.rho),
            self.rseed.rcm(&self.rho),
        )
    }

    /// Derives the nullifier for this note.
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        Nullifier::derive(
            fvk.nk(),
            self.rho.0,
            self.rseed.psi(&self.rho),
            self.commitment(),
        )
    }
}

/// An encrypted note.
#[derive(Debug, Clone)]
pub struct TransmittedNoteCiphertext {
    /// The serialization of the ephemeral public key
    pub epk_bytes: [u8; 32],
    /// The encrypted note ciphertext
    pub enc_ciphertext: [u8; 580],
    /// An encrypted value that allows the holder of the outgoing cipher
    /// key for the note to recover the note plaintext.
    pub out_ciphertext: [u8; 80],
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use proptest::prelude::*;

    use crate::{
        address::testing::arb_address, note::nullifier::testing::arb_nullifier, value::NoteValue,
    };

    use super::{Note, RandomSeed};

    prop_compose! {
        /// Generate an arbitrary random seed
        pub(crate) fn arb_rseed()(elems in prop::array::uniform32(prop::num::u8::ANY)) -> RandomSeed {
            RandomSeed(elems)
        }
    }

    prop_compose! {
        /// Generate an action without authorization data.
        pub fn arb_note(value: NoteValue)(
            recipient in arb_address(),
            rho in arb_nullifier(),
            rseed in arb_rseed(),
        ) -> Note {
            Note {
                recipient,
                value,
                rho,
                rseed,
            }
        }
    }
}
