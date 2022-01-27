//! Gadget and chips for the Sinsemilla hash function.
use crate::circuit::gadget::{
    ecc::{self, EccInstructions, FixedPoints},
    utilities::Var,
};
use group::ff::{Field, PrimeField};
use halo2::{circuit::Layouter, plonk::Error};
use pasta_curves::arithmetic::CurveAffine;
use std::fmt::Debug;

pub mod chip;
pub mod merkle;
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
    /// `N*K <= C::Base::CAPACITY`.
    ///
    /// For example, in the case `K = 10`, `CAPACITY = 254`, we can fit
    /// up to `N = 25` words in a single base field element.
    type MessagePiece: Clone + Debug;

    /// A cumulative sum `z` is used to decompose a Sinsemilla message. It
    /// produces intermediate values for each word in the message, such
    /// that `z_next` = (`z_cur` - `word_next`) / `2^K`.
    ///
    /// These intermediate values are useful for range checks on subsets
    /// of the Sinsemilla message. Sinsemilla messages in the Orchard
    /// protocol are composed of field elements, and we need to check
    /// the canonicity of the field element encodings in certain cases.
    type RunningSum;

    /// The x-coordinate of a point output of [`Self::hash_to_point`].
    type X;
    /// A point output of [`Self::hash_to_point`].
    type NonIdentityPoint: Clone + Debug;
    /// A type enumerating the fixed points used in `CommitDomains`.
    type FixedPoints: FixedPoints<C>;

    /// HashDomains used in this instruction.
    type HashDomains: HashDomains<C>;
    /// CommitDomains used in this instruction.
    type CommitDomains: CommitDomains<C, Self::FixedPoints, Self::HashDomains>;

    /// Witness a message piece given a field element. Returns a [`Self::MessagePiece`]
    /// encoding the given message.
    ///
    /// # Panics
    ///
    /// Panics if `num_words` exceed the maximum number of `K`-bit words that
    /// can fit into a single base field element.
    fn witness_message_piece(
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
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: C,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Self::RunningSum>), Error>;

    /// Extracts the x-coordinate of the output of a Sinsemilla hash.
    fn extract(point: &Self::NonIdentityPoint) -> Self::X;
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
        mut layouter: impl Layouter<C::Base>,
        bitstring: Vec<Option<bool>>,
    ) -> Result<Self, Error> {
        // Message must be composed of `K`-bit words.
        assert_eq!(bitstring.len() % K, 0);

        // Message must have at most `MAX_WORDS` words.
        assert!(bitstring.len() / K <= MAX_WORDS);

        // Each message piece must have at most `floor(C::CAPACITY / K)` words.
        let piece_num_words = C::Base::CAPACITY as usize / K;
        let pieces: Result<Vec<_>, _> = bitstring
            .chunks(piece_num_words * K)
            .enumerate()
            .map(
                |(i, piece)| -> Result<MessagePiece<C, SinsemillaChip, K, MAX_WORDS>, Error> {
                    MessagePiece::from_bitstring(
                        chip.clone(),
                        layouter.namespace(|| format!("message piece {}", i)),
                        piece,
                    )
                },
            )
            .collect();

        pieces.map(|pieces| Self::from_pieces(chip, pieces))
    }

    /// Constructs a message from a vector of [`MessagePiece`]s.
    ///
    /// [`MessagePiece`]: SinsemillaInstructions::MessagePiece
    pub fn from_pieces(
        chip: SinsemillaChip,
        pieces: Vec<MessagePiece<C, SinsemillaChip, K, MAX_WORDS>>,
    ) -> Self {
        Self {
            chip,
            inner: pieces
                .into_iter()
                .map(|piece| piece.inner)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MessagePiece<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    chip: SinsemillaChip,
    inner: SinsemillaChip::MessagePiece,
}

impl<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
    MessagePiece<C, SinsemillaChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    /// Returns the inner MessagePiece contained in this gadget.
    pub fn inner(&self) -> SinsemillaChip::MessagePiece {
        self.inner.clone()
    }
}

impl<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
    MessagePiece<C, SinsemillaChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    fn from_bitstring(
        chip: SinsemillaChip,
        layouter: impl Layouter<C::Base>,
        bitstring: &[Option<bool>],
    ) -> Result<Self, Error> {
        // Message must be composed of `K`-bit words.
        assert_eq!(bitstring.len() % K, 0);
        let num_words = bitstring.len() / K;

        // Each message piece must have at most `floor(C::Base::CAPACITY / K)` words.
        // This ensures that the all-ones bitstring is canonical in the field.
        let piece_max_num_words = C::Base::CAPACITY as usize / K;
        assert!(num_words <= piece_max_num_words as usize);

        // Closure to parse a bitstring (little-endian) into a base field element.
        let to_base_field = |bits: &[Option<bool>]| -> Option<C::Base> {
            let bits: Option<Vec<bool>> = bits.iter().cloned().collect();
            bits.map(|bits| {
                bits.into_iter().rev().fold(C::Base::zero(), |acc, bit| {
                    if bit {
                        acc.double() + C::Base::one()
                    } else {
                        acc.double()
                    }
                })
            })
        };

        let piece_value = to_base_field(bitstring);
        Self::from_field_elem(chip, layouter, piece_value, num_words)
    }

    /// Constructs a MessagePiece from a field element.
    pub fn from_field_elem(
        chip: SinsemillaChip,
        layouter: impl Layouter<C::Base>,
        field_elem: Option<C::Base>,
        num_words: usize,
    ) -> Result<Self, Error> {
        let inner = chip.witness_message_piece(layouter, field_elem, num_words)?;
        Ok(Self { chip, inner })
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
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
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
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
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

    #[allow(clippy::type_complexity)]
    /// $\mathsf{SinsemillaHashToPoint}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash
    pub fn hash_to_point(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<(ecc::NonIdentityPoint<C, EccChip>, Vec<SinsemillaChip::RunningSum>), Error> {
        assert_eq!(self.sinsemilla_chip, message.chip);
        self.sinsemilla_chip
            .hash_to_point(layouter, self.Q, message.inner)
            .map(|(point, zs)| (ecc::NonIdentityPoint::from_inner(self.ecc_chip.clone(), point), zs))
    }

    /// $\mathsf{SinsemillaHash}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash
    #[allow(clippy::type_complexity)]
    pub fn hash(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<(ecc::X<C, EccChip>, Vec<SinsemillaChip::RunningSum>), Error> {
        assert_eq!(self.sinsemilla_chip, message.chip);
        let (p, zs) = self.hash_to_point(layouter, message)?;
        Ok((p.extract_p(), zs))
    }
}

/// Trait allowing circuit's Sinsemilla CommitDomains to be enumerated.
pub trait CommitDomains<C: CurveAffine, F: FixedPoints<C>, H: HashDomains<C>>:
    Clone + Debug
{
    /// Returns the fixed point corresponding to the R constant used for
    /// randomization in this CommitDomain.
    fn r(&self) -> F::FullScalar;

    /// Returns the HashDomain contained in this CommitDomain
    fn hash_domain(&self) -> H;
}

/// Trait allowing circuit's Sinsemilla HashDomains to be enumerated.
#[allow(non_snake_case)]
pub trait HashDomains<C: CurveAffine>: Clone + Debug {
    fn Q(&self) -> C;
}

#[allow(non_snake_case)]
pub struct CommitDomain<
    C: CurveAffine,
    SinsemillaChip,
    EccChip,
    const K: usize,
    const MAX_WORDS: usize,
> where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    M: HashDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>,
    R: ecc::FixedPoint<C, EccChip>,
}

impl<C: CurveAffine, SinsemillaChip, EccChip, const K: usize, const MAX_WORDS: usize>
    CommitDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    /// Constructs a new `CommitDomain` for the given domain.
    pub fn new(
        sinsemilla_chip: SinsemillaChip,
        ecc_chip: EccChip,
        // TODO: Instead of using SinsemilllaChip::CommitDomains, just use something that implements a CommitDomains trait
        domain: &SinsemillaChip::CommitDomains,
    ) -> Self {
        CommitDomain {
            M: HashDomain::new(sinsemilla_chip, ecc_chip.clone(), &domain.hash_domain()),
            R: ecc::FixedPoint::from_inner(ecc_chip, domain.r()),
        }
    }

    #[allow(clippy::type_complexity)]
    /// $\mathsf{SinsemillaCommit}$ from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub fn commit(
        &self,
        mut layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
        r: Option<C::Scalar>,
    ) -> Result<
        (
            ecc::Point<C, EccChip>,
            Vec<SinsemillaChip::RunningSum>,
        ),
        Error,
    > {
        assert_eq!(self.M.sinsemilla_chip, message.chip);
        let (blind, _) = self.R.mul(layouter.namespace(|| "[r] R"), r)?;
        let (p, zs) = self.M.hash_to_point(layouter.namespace(|| "M"), message)?;
        let commitment = p.add(layouter.namespace(|| "M + [r] R"), &blind)?;
        Ok((commitment, zs))
    }

    #[allow(clippy::type_complexity)]
    /// $\mathsf{SinsemillaShortCommit}$ from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub fn short_commit(
        &self,
        mut layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
        r: Option<C::Scalar>,
    ) -> Result<(ecc::X<C, EccChip>, Vec<SinsemillaChip::RunningSum>), Error> {
        assert_eq!(self.M.sinsemilla_chip, message.chip);
        let (p, zs) = self.commit(layouter.namespace(|| "commit"), message, r)?;
        Ok((p.extract_p(), zs))
    }
}

#[cfg(test)]
mod tests {
    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use rand::rngs::OsRng;

    use super::{
        chip::{SinsemillaChip, SinsemillaConfig},
        merkle::MERKLE_CRH_PERSONALIZATION,
        CommitDomain, HashDomain, Message, MessagePiece,
    };

    use crate::{
        circuit::gadget::{
            ecc::{
                chip::{EccChip, EccConfig},
                NonIdentityPoint,
            },
            utilities::lookup_range_check::LookupRangeCheckConfig,
        },
        primitives::sinsemilla::{self, K},
    };

    use group::{ff::Field, Curve};
    use pasta_curves::pallas;

    use std::convert::TryInto;

    struct MyCircuit {}

    impl Circuit<pallas::Base> for MyCircuit {
        #[allow(clippy::type_complexity)]
        type Config = (
            EccConfig<OrchardFixedBases>,
            SinsemillaConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>,
            SinsemillaConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>,
        );
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            MyCircuit {}
        }

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

            // Shared fixed column for loading constants
            let constants = meta.fixed_column();
            meta.enable_constant(constants);

            let table_idx = meta.lookup_table_column();
            let lagrange_coeffs = [
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
            ];

            // Fixed columns for the Sinsemilla generator lookup table
            let lookup = (
                table_idx,
                meta.lookup_table_column(),
                meta.lookup_table_column(),
            );

            let range_check = LookupRangeCheckConfig::configure(meta, advices[9], table_idx);

            let ecc_config = EccChip::<OrchardFixedBases>::configure(
                meta,
                advices,
                lagrange_coeffs,
                range_check,
            );

            let config1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                advices[2],
                lagrange_coeffs[0],
                lookup,
                range_check,
            );
            let config2 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                advices[7],
                lagrange_coeffs[1],
                lookup,
                range_check,
            );
            (ecc_config, config1, config2)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            let rng = OsRng;

            let ecc_chip = EccChip::construct(config.0);

            // The two `SinsemillaChip`s share the same lookup table.
            SinsemillaChip::<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>::load(
                config.1.clone(),
                &mut layouter,
            )?;

            // This MerkleCRH example is purely for illustrative purposes.
            // It is not an implementation of the Orchard protocol spec.
            {
                let chip1 = SinsemillaChip::construct(config.1);

                let merkle_crh = HashDomain::new(
                    chip1.clone(),
                    ecc_chip.clone(),
                    &OrchardHashDomains::MerkleCrh,
                );

                // Layer 31, l = MERKLE_DEPTH - 1 - layer = 0
                let l_bitstring = vec![Some(false); K];
                let l = MessagePiece::from_bitstring(
                    chip1.clone(),
                    layouter.namespace(|| "l"),
                    &l_bitstring,
                )?;

                // Left leaf
                let left_bitstring: Vec<Option<bool>> =
                    (0..250).map(|_| Some(rand::random::<bool>())).collect();
                let left = MessagePiece::from_bitstring(
                    chip1.clone(),
                    layouter.namespace(|| "left"),
                    &left_bitstring,
                )?;

                // Right leaf
                let right_bitstring: Vec<Option<bool>> =
                    (0..250).map(|_| Some(rand::random::<bool>())).collect();
                let right = MessagePiece::from_bitstring(
                    chip1.clone(),
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

                    NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "Witness expected parent"),
                        expected_parent,
                    )?
                };

                // Parent
                let (parent, _) = {
                    let message = Message::from_pieces(chip1, vec![l, left, right]);
                    merkle_crh.hash_to_point(layouter.namespace(|| "parent"), message)?
                };

                parent.constrain_equal(
                    layouter.namespace(|| "parent == expected parent"),
                    &expected_parent,
                )?;
            }

            {
                let chip2 = SinsemillaChip::construct(config.2);

                let commit_ivk = CommitDomain::new(
                    chip2.clone(),
                    ecc_chip.clone(),
                    &OrchardCommitDomains::CommitIvk,
                );
                let r_val = pallas::Scalar::random(rng);
                let message: Vec<Option<bool>> =
                    (0..500).map(|_| Some(rand::random::<bool>())).collect();

                let (result, _) = {
                    let message = Message::from_bitstring(
                        chip2,
                        layouter.namespace(|| "witness message"),
                        message.clone(),
                    )?;
                    commit_ivk.commit(layouter.namespace(|| "commit"), message, Some(r_val))?
                };

                // Witness expected result.
                let expected_result = {
                    let message: Option<Vec<bool>> = message.into_iter().collect();
                    let expected_result = if let Some(message) = message {
                        let domain = sinsemilla::CommitDomain::new(COMMIT_IVK_PERSONALIZATION);
                        let point = domain.commit(message.into_iter(), &r_val).unwrap();
                        Some(point.to_affine())
                    } else {
                        None
                    };

                    NonIdentityPoint::new(
                        ecc_chip,
                        layouter.namespace(|| "Witness expected result"),
                        expected_result,
                    )?
                };

                result.constrain_equal(
                    layouter.namespace(|| "result == expected result"),
                    &expected_result,
                )
            }
        }
    }

    #[test]
    fn sinsemilla_chip() {
        let k = 11;
        let circuit = MyCircuit {};
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_sinsemilla_chip() {
        use plotters::prelude::*;

        let root =
            BitMapBackend::new("sinsemilla-hash-layout.png", (1024, 7680)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("SinsemillaHash", ("sans-serif", 60)).unwrap();

        let circuit = MyCircuit {};
        halo2::dev::CircuitLayout::default()
            .render(11, &circuit, &root)
            .unwrap();
    }
}
