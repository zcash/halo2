//! Gadget and chips for the [Sinsemilla] hash function.
//!
//! [Sinsemilla]: https://hackmd.io/iOw7-HpFQY6dPF1aFY8pAw

use std::fmt;

use crate::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter},
    plonk::Error,
};

/// Domain prefix used in SWU hash-to-curve to generate S_i's.
pub const S_DOMAIN_PREFIX: &str = "z.cash:SinsemillaS";

/// Domain prefix used in SWU hash-to-curve to generate Q.
pub const Q_DOMAIN_PREFIX: &str = "z.cash:SinsemillaQ";

/// Personalization input used to generate Q
/// TODO: Decide on personalization
pub const Q_PERSONALIZATION: [u8; 4] = [0u8; 4];

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
pub trait SinsemillaInstructions<F: FieldExt>: Chip<Field = F> {
    /// A message of at most `kn` bits.
    type Message: Clone + fmt::Debug;
    /// The output of `Hash`.
    type HashOutput: fmt::Debug;

    /// Return Q
    fn q() -> (F, F);

    /// Hashes the given message.
    ///
    /// TODO: Since the output is always curve point, maybe this should return
    /// `<Self as EccInstructions>::Point` instead of an associated type.
    fn hash(
        layouter: &mut impl Layouter<Self>,
        message: Self::Message,
    ) -> Result<Self::HashOutput, Error>;
}
