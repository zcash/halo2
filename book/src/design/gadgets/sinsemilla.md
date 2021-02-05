# Sinsemilla

## Specification
Sinsemilla is a collision-resistant hash function and commitment scheme designed to be efficient in algebraic circuit models that support lookups, such as PLONK or Halo 2. It is specified in the Zcash protocol spec. [TODO: link to spec]

## Gadget interface
Sinsemilla hashes messages of length `kn` bits, `k` bits at a time in `n` rounds. It returns a curve point as output.

## Chip instructions
The Sinsemilla gadget requires a chip with the following instructions:

```rust
# extern crate halo2;
# use halo2::plonk::Error;
# use std::fmt;
#
# trait Chip: Sized {}
# trait Layouter<C: Chip> {}

/// The set of circuit instructions required to use the [`Sinsemilla`] gadget.
pub trait SinsemillaInstructions<F: FieldExt, C: CurveAffine<Base = F>>: Chip<Field = F> {
    /// A message of at most `kn` bits.
    type Message: Clone + fmt::Debug;
    /// A message padded to `kn` bits.
    type PaddedMessage: Clone + fmt::Debug;
    /// The output of `Hash`.
    type HashOutput: fmt::Debug;

    /// Return Q
    fn q() -> (F, F);

    /// Pads the given message to `kn` bits.
    fn pad(
        layouter: &mut impl Layouter<Self>,
        message: Self::Message,
    ) -> Result<Self::PaddedMessage, Error>;

    /// Hashes the given message.
    ///
    /// TODO: Since the output is always curve point, maybe this should return
    /// `<Self as EccInstructions>::Point` instead of an associated type.
    fn hash(
        layouter: &mut impl Layouter<Self>,
        message: Self::PaddedMessage,
    ) -> Result<Self::HashOutput, Error>;
}
```