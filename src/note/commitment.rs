use std::iter;

use bitvec::{array::BitArray, order::Lsb0};
use ff::PrimeField;
use pasta_curves::pallas;

use crate::{
    constants::L_ORCHARD_BASE,
    primitives::sinsemilla,
    spec::{prf_expand, to_scalar},
    value::NoteValue,
};

use super::RandomSeed;

pub(super) struct NoteCommitTrapdoor(pallas::Scalar);

impl From<&RandomSeed> for NoteCommitTrapdoor {
    fn from(rseed: &RandomSeed) -> Self {
        NoteCommitTrapdoor(to_scalar(prf_expand(&rseed.0, &[0x05])))
    }
}

/// A commitment to a note.
#[derive(Debug)]
pub struct NoteCommitment(pallas::Point);

impl NoteCommitment {
    /// $NoteCommit^Orchard$.
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.8.4: Sinsemilla commitments][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub(super) fn derive(
        g_d: [u8; 32],
        pk_d: [u8; 32],
        v: NoteValue,
        rho: pallas::Base,
        psi: pallas::Base,
        rcm: NoteCommitTrapdoor,
    ) -> Self {
        let domain = sinsemilla::CommitDomain::new("z.cash:Orchard-NoteCommit");
        NoteCommitment(
            domain.commit(
                iter::empty()
                    .chain(BitArray::<Lsb0, _>::new(g_d).iter().by_val())
                    .chain(BitArray::<Lsb0, _>::new(pk_d).iter().by_val())
                    .chain(v.to_le_bits().iter().by_val())
                    .chain(rho.to_le_bits().iter().by_val().take(L_ORCHARD_BASE))
                    .chain(psi.to_le_bits().iter().by_val().take(L_ORCHARD_BASE)),
                &rcm.0,
            ),
        )
    }
}
