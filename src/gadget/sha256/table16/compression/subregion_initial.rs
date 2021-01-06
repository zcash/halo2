use super::super::{
    util::*, AbcdVar, BlockWord, EfghVar, MessagePiece, SpreadWord, Table16Chip, IV, ROUNDS,
};
use super::{Compression, State};
use crate::{arithmetic::FieldExt, gadget::Region, plonk::Error};

impl Compression {
    // Copy message schedule
    pub fn copy_message_schedule<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
        w_halves: [(MessagePiece, MessagePiece); ROUNDS],
    ) -> Result<(), Error> {
        todo!()
    }

    // First three rounds
    pub fn initialization_vector<F: FieldExt>(
        &self,
        region: &mut Region<'_, Table16Chip<F>>,
    ) -> Result<State, Error> {
        // s_upper_sigma_1(E)

        // Decompose F, G

        // Ch(E, F, G)

        // H' = H + Ch(E, F, G) + s_upper_sigma_1(E) + K + W

        // s_upper_sigma_0(A)

        todo!()
    }
}
