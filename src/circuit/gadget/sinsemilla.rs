//! Gadget and chips for the Sinsemilla hash function.
use crate::circuit::gadget::{
    ecc::{self, EccInstructions},
    utilities::Var,
};
use halo2::{arithmetic::CurveAffine, circuit::Layouter, plonk::Error};
use std::fmt::Debug;

pub mod chip;
mod message;

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
/// This trait is bounded on two constant parameters: `K`, the number of bits
/// in each word accepted by the Sinsemilla hash, and `MAX_WORDS`, the maximum
/// number of words that a single hash instance can process.
pub trait SinsemillaInstructions<C: CurveAffine, const K: usize, const MAX_WORDS: usize> {
    /// A variable in the circuit.
    type CellValue: Var<C::Base>;

    /// A message composed of [`Self::MessagePiece`]s.
    type Message: From<Vec<Self::MessagePiece>>;

    /// A piece in a message containing a number of `K`-bit words.
    /// A [`Self::MessagePiece`] fits in a single base field element,
    /// which means it can only contain up to `N` words, where
    /// `N*K <= C::Base::NUM_BITS`.
    ///
    /// For example, in the case `K = 10`, `NUM_BITS = 255`, we can fit
    /// up to `N = 25` words in a single base field element.
    type MessagePiece;

    /// The x-coordinate of a point output of [`Self::hash_to_point`].
    type X;
    /// A point output of [`Self::hash_to_point`].
    type Point: Clone + Debug;

    /// HashDomains used in this instruction.
    type HashDomains: HashDomains<C>;

    /// Witness a message in the given bitstring.
    /// Returns a vector of [`Self::MessagePiece`]s encoding the given message.
    ///
    /// # Panics
    ///
    /// Panics if the message length is not a multiple of `K`.
    ///
    /// Panics if the message length exceeds `K * MAX_WORDS`.
    fn witness_message(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Vec<Option<bool>>,
    ) -> Result<Self::Message, Error>;

    /// Witnesses a message piece given a field element and the intended number of `K`-bit
    /// words it contains.
    ///
    /// Returns a [`Self::MessagePiece`] encoding the given message.
    ///
    /// # Panics
    ///
    /// Panics if the message length is not a multiple of `K`.
    ///
    /// Panics if the message length exceeds the maximum number of words
    /// that can fit in a field element.
    fn witness_message_piece_bitstring(
        &self,
        layouter: impl Layouter<C::Base>,
        message: &[Option<bool>],
    ) -> Result<Self::MessagePiece, Error>;

    /// Witness a message piece given a field element. Returns a [`Self::MessagePiece`]
    /// encoding the given message.
    ///
    /// # Panics
    ///
    /// Panics if `num_words` exceed the maximum number of `K`-bit words that
    /// can fit into a single base field element.
    fn witness_message_piece_field(
        &self,
        layouter: impl Layouter<C::Base>,
        value: Option<C::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error>;

    /// Hashes a message to an ECC curve point.
    /// This returns both the resulting point, as well as the message
    /// decomposition in the form of intermediate values in a cumulative
    /// sum.
    ///
    /// A cumulative sum `z` is used to decompose a Sinsemilla message. It
    /// produces intermediate values for each word in the message, such
    /// that `z_next` = (`z_cur` - `word_next`) / `2^K`.
    ///  
    /// These intermediate values are useful for range checks on subsets
    /// of the Sinsemilla message. Sinsemilla messages in the Orchard
    /// protocol are composed of field elements, and we need to check
    /// the canonicity of the field element encodings in certain cases.
    ///
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: C,
        message: Self::Message,
    ) -> Result<(Self::Point, Vec<Vec<Self::CellValue>>), Error>;

    /// Extracts the x-coordinate of the output of a Sinsemilla hash.
    fn extract(point: &Self::Point) -> Self::X;
}

/// A message to be hashed.
///
/// Composed of [`MessagePiece`]s with bitlength some multiple of `K`.
///
/// [`MessagePiece`]: SinsemillaInstructions::MessagePiece
#[derive(Clone, Debug)]
pub struct Message<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    chip: SinsemillaChip,
    inner: SinsemillaChip::Message,
}

impl<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
    Message<C, SinsemillaChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    fn from_bitstring(
        chip: SinsemillaChip,
        layouter: impl Layouter<C::Base>,
        bitstring: Vec<Option<bool>>,
    ) -> Result<Self, Error> {
        let inner = chip.witness_message(layouter, bitstring)?;
        Ok(Self { chip, inner })
    }

    /// Constructs a message from a vector of [`MessagePiece`]s.
    ///
    /// [`MessagePiece`]: SinsemillaInstructions::MessagePiece
    fn from_pieces(chip: SinsemillaChip, pieces: Vec<SinsemillaChip::MessagePiece>) -> Self {
        Self {
            chip,
            inner: pieces.into(),
        }
    }
}

/// A domain in which $\mathsf{SinsemillaHashToPoint}$ and $\mathsf{SinsemillaHash}$ can
/// be used.
#[allow(non_snake_case)]
pub struct HashDomain<
    C: CurveAffine,
    SinsemillaChip,
    EccChip,
    const K: usize,
    const MAX_WORDS: usize,
> where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            Point = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::Point,
        > + Clone
        + Debug
        + Eq,
{
    sinsemilla_chip: SinsemillaChip,
    ecc_chip: EccChip,
    Q: C,
}

impl<C: CurveAffine, SinsemillaChip, EccChip, const K: usize, const MAX_WORDS: usize>
    HashDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            Point = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::Point,
        > + Clone
        + Debug
        + Eq,
{
    #[allow(non_snake_case)]
    /// Constructs a new `HashDomain` for the given domain.
    pub fn new(
        sinsemilla_chip: SinsemillaChip,
        ecc_chip: EccChip,
        domain: &SinsemillaChip::HashDomains,
    ) -> Self {
        HashDomain {
            sinsemilla_chip,
            ecc_chip,
            Q: domain.Q(),
        }
    }

    /// $\mathsf{SinsemillaHashToPoint}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash
    pub fn hash_to_point(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<ecc::Point<C, EccChip>, Error> {
        assert_eq!(self.sinsemilla_chip, message.chip);
        self.sinsemilla_chip
            .hash_to_point(layouter, self.Q, message.inner)
            .map(|(point, _)| ecc::Point::from_inner(self.ecc_chip.clone(), point))
    }

    /// $\mathsf{SinsemillaHash}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash
    pub fn hash(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<ecc::X<C, EccChip>, Error> {
        assert_eq!(self.sinsemilla_chip, message.chip);
        let p = self.hash_to_point(layouter, message);
        p.map(|p| p.extract_p())
    }
}

/// Trait allowing circuit's Sinsemilla HashDomains to be enumerated.
#[allow(non_snake_case)]
pub trait HashDomains<C: CurveAffine>: Clone + Debug {
    fn Q(&self) -> C;
}

#[cfg(test)]
mod tests {
    use halo2::{
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::MockProver,
        pasta::pallas,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };

    use super::{
        chip::SinsemillaHashDomains,
        chip::{SinsemillaChip, SinsemillaConfig},
        HashDomain, Message, SinsemillaInstructions,
    };

    use crate::{
        circuit::gadget::ecc::{
            chip::{EccChip, EccConfig},
            Point,
        },
        constants::MERKLE_CRH_PERSONALIZATION,
        primitives::sinsemilla::{self, K},
    };

    use group::Curve;

    use std::convert::TryInto;

    struct MyCircuit {}

    impl Circuit<pallas::Base> for MyCircuit {
        type Config = (EccConfig, SinsemillaConfig, SinsemillaConfig);

        #[allow(non_snake_case)]
        fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
            let advices = [
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            ];

            let constants = meta.fixed_column();
            let perm = meta.permutation(
                &advices
                    .iter()
                    .map(|advice| (*advice).into())
                    .chain(Some(constants.into()))
                    .collect::<Vec<_>>(),
            );

            let ecc_config = EccChip::configure(meta, advices, perm.clone());

            // Fixed columns for the Sinsemilla generator lookup table
            let lookup = (
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
            );

            let config1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                lookup,
                constants,
                perm.clone(),
            );
            let config2 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                lookup,
                constants,
                perm,
            );
            (ecc_config, config1, config2)
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<pallas::Base>,
            config: Self::Config,
        ) -> Result<(), Error> {
            let mut layouter = SingleChipLayouter::new(cs)?;
            let ecc_chip = EccChip::construct(config.0);

            // The two `SinsemillaChip`s share the same lookup table.
            SinsemillaChip::load(config.1.clone(), &mut layouter)?;

            // This MerkleCRH example is purely for illustrative purposes.
            // It is not an implementation of the Orchard protocol spec.
            {
                let chip1 = SinsemillaChip::construct(config.1);

                let merkle_crh = HashDomain::new(
                    chip1.clone(),
                    ecc_chip.clone(),
                    &SinsemillaHashDomains::MerkleCrh,
                );

                // Layer 31, l = MERKLE_DEPTH_ORCHARD - 1 - layer = 0
                let l_bitstring = vec![Some(false); K];
                let l = chip1
                    .witness_message_piece_bitstring(layouter.namespace(|| "l"), &l_bitstring)?;

                // Left leaf
                let left_bitstring: Vec<Option<bool>> =
                    (0..250).map(|_| Some(rand::random::<bool>())).collect();
                let left = chip1.witness_message_piece_bitstring(
                    layouter.namespace(|| "left"),
                    &left_bitstring,
                )?;

                // Right leaf
                let right_bitstring: Vec<Option<bool>> =
                    (0..250).map(|_| Some(rand::random::<bool>())).collect();
                let right = chip1.witness_message_piece_bitstring(
                    layouter.namespace(|| "right"),
                    &right_bitstring,
                )?;

                let l_bitstring: Option<Vec<bool>> = l_bitstring.into_iter().collect();
                let left_bitstring: Option<Vec<bool>> = left_bitstring.into_iter().collect();
                let right_bitstring: Option<Vec<bool>> = right_bitstring.into_iter().collect();

                // Witness expected parent
                let expected_parent = {
                    let expected_parent = if let (Some(l), Some(left), Some(right)) =
                        (l_bitstring, left_bitstring, right_bitstring)
                    {
                        let merkle_crh = sinsemilla::HashDomain::new(MERKLE_CRH_PERSONALIZATION);
                        let point = merkle_crh
                            .hash_to_point(
                                l.into_iter()
                                    .chain(left.into_iter())
                                    .chain(right.into_iter()),
                            )
                            .unwrap();
                        Some(point.to_affine())
                    } else {
                        None
                    };

                    Point::new(
                        ecc_chip,
                        layouter.namespace(|| "Witness expected parent"),
                        expected_parent,
                    )?
                };

                // Parent
                let parent = {
                    let message = Message::from_pieces(chip1, vec![l, left, right]);
                    merkle_crh.hash_to_point(layouter.namespace(|| "parent"), message)?
                };

                parent.constrain_equal(
                    layouter.namespace(|| "parent == expected parent"),
                    &expected_parent,
                )?;
            }

            Ok(())
        }
    }

    #[test]
    fn sinsemilla_chip() {
        let k = 11;
        let circuit = MyCircuit {};
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }
}
