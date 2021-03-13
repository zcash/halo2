use group::GroupEncoding;
use pasta_curves::pallas;

use crate::{
    keys::FullViewingKey,
    spec::{prf_expand, to_base, to_scalar},
    value::NoteValue,
    Address,
};

mod commitment;
pub use self::commitment::NoteCommitment;

/// The ZIP 212 seed randomness for a note.
#[derive(Debug)]
struct RandomSeed([u8; 32]);

impl RandomSeed {
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn psi(&self) -> pallas::Base {
        to_base(prf_expand(&self.0, &[0x09]))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk(&self) -> pallas::Scalar {
        to_scalar(prf_expand(&self.0, &[0x04]))
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
    /// Derives the commitment to this note.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    pub fn commitment(&self) -> NoteCommitment {
        let g_d = self.recipient.g_d();

        NoteCommitment::derive(
            g_d.to_bytes(),
            self.recipient.pk_d().to_bytes(),
            self.value,
            self.rho.0,
            self.rseed.psi(),
            (&self.rseed).into(),
        )
    }

    /// Derives the nullifier for this note.
    pub fn nullifier(&self, _: &FullViewingKey) -> Nullifier {
        todo!()
    }
}

/// An encrypted note.
#[derive(Debug)]
pub struct EncryptedNote;

/// A unique nullifier for a note.
#[derive(Debug)]
pub struct Nullifier(pallas::Base);
