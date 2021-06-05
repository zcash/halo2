//! Logic for building Orchard components of transactions.

use std::convert::TryFrom;
use std::iter;

use ff::Field;
use nonempty::NonEmpty;
use pasta_curves::pallas;
use rand::{CryptoRng, RngCore};

use crate::{
    address::Address,
    bundle::{Action, Authorization, Authorized, Bundle, Flags},
    circuit::{Circuit, Proof, ProvingKey},
    keys::{
        FullViewingKey, OutgoingViewingKey, SpendAuthorizingKey, SpendValidatingKey, SpendingKey,
    },
    note::{Note, TransmittedNoteCiphertext},
    primitives::redpallas::{self, Binding, SpendAuth},
    tree::{Anchor, MerklePath},
    value::{self, NoteValue, OverflowError, ValueCommitTrapdoor, ValueCommitment, ValueSum},
};

const MIN_ACTIONS: usize = 2;

/// An error type for the kinds of errors that can occur during bundle construction.
#[derive(Debug)]
pub enum Error {
    /// A bundle could not be built because required signatures were missing.
    MissingSignatures,
    /// An error occurred in the process of producing a proof for a bundle.
    Proof(halo2::plonk::Error),
    /// An overflow error occurred while attempting to construct the value
    /// for a bundle.
    ValueSum(value::OverflowError),
}

impl From<halo2::plonk::Error> for Error {
    fn from(e: halo2::plonk::Error) -> Self {
        Error::Proof(e)
    }
}

impl From<value::OverflowError> for Error {
    fn from(e: value::OverflowError) -> Self {
        Error::ValueSum(e)
    }
}

/// Information about a specific note to be spent in an [`Action`].
#[derive(Debug)]
struct SpendInfo {
    dummy_sk: Option<SpendingKey>,
    fvk: FullViewingKey,
    note: Note,
    merkle_path: MerklePath,
}

impl SpendInfo {
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    fn dummy(rng: &mut impl RngCore) -> Self {
        let (sk, fvk, note) = Note::dummy(rng, None);
        let merkle_path = MerklePath::dummy(rng);

        SpendInfo {
            dummy_sk: Some(sk),
            fvk,
            note,
            merkle_path,
        }
    }
}

/// Information about a specific recipient to receive funds in an [`Action`].
#[derive(Debug)]
struct RecipientInfo {
    ovk: Option<OutgoingViewingKey>,
    recipient: Address,
    value: NoteValue,
    memo: Option<()>,
}

impl RecipientInfo {
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    fn dummy(rng: &mut impl RngCore) -> Self {
        let fvk: FullViewingKey = (&SpendingKey::random(rng)).into();
        let recipient = fvk.default_address();

        RecipientInfo {
            ovk: None,
            recipient,
            value: NoteValue::zero(),
            memo: None,
        }
    }
}

/// Information about a specific [`Action`] we plan to build.
#[derive(Debug)]
struct ActionInfo {
    spend: SpendInfo,
    output: RecipientInfo,
    rcv: ValueCommitTrapdoor,
}

impl ActionInfo {
    fn new(spend: SpendInfo, output: RecipientInfo, rng: impl RngCore) -> Self {
        ActionInfo {
            spend,
            output,
            rcv: ValueCommitTrapdoor::random(rng),
        }
    }

    /// Returns the value sum for this action.
    fn value_sum(&self) -> Option<ValueSum> {
        self.spend.note.value() - self.output.value
    }

    /// Builds the action.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn build(self, mut rng: impl RngCore) -> (Action<SigningMetadata>, Circuit) {
        let v_net = self.value_sum().expect("already checked this");
        let cv_net = ValueCommitment::derive(v_net, self.rcv);

        let nf_old = self.spend.note.nullifier(&self.spend.fvk);
        let ak: SpendValidatingKey = self.spend.fvk.into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        let note = Note::new(self.output.recipient, self.output.value, nf_old, rng);
        let cm_new = note.commitment();

        // TODO: Note encryption
        let encrypted_note = TransmittedNoteCiphertext {
            epk_bytes: [0u8; 32],
            enc_ciphertext: [0u8; 580],
            out_ciphertext: [0u8; 80],
        };

        (
            Action::from_parts(
                nf_old,
                rk,
                cm_new.into(),
                encrypted_note,
                cv_net,
                SigningMetadata {
                    dummy_ask: self.spend.dummy_sk.as_ref().map(SpendAuthorizingKey::from),
                    parts: SigningParts { ak, alpha },
                },
            ),
            Circuit {},
        )
    }
}

/// A builder that constructs a [`Bundle`] from a set of notes to be spent, and recipients
/// to receive funds.
#[derive(Debug)]
pub struct Builder {
    spends: Vec<SpendInfo>,
    recipients: Vec<RecipientInfo>,
    flags: Flags,
    anchor: Anchor,
}

impl Builder {
    /// Constructs a new empty builder for an Orchard bundle.
    pub fn new(flags: Flags, anchor: Anchor) -> Self {
        Builder {
            spends: vec![],
            recipients: vec![],
            flags,
            anchor,
        }
    }

    /// Adds a note to be spent in this transaction.
    ///
    /// Returns an error if the given Merkle path does not have the required anchor for
    /// the given note.
    pub fn add_spend(
        &mut self,
        fvk: FullViewingKey,
        note: Note,
        merkle_path: MerklePath,
    ) -> Result<(), &'static str> {
        if !self.flags.spends_enabled() {
            return Err("Spends are not enabled for this builder");
        }

        // Consistency check: all anchors must be equal.
        let _cm = note.commitment();
        // TODO: Once we have tree logic.
        // let path_root: bls12_381::Scalar = merkle_path.root(cmu).into();
        // if path_root != anchor {
        //     return Err(Error::AnchorMismatch);
        // }

        self.spends.push(SpendInfo {
            dummy_sk: None,
            fvk,
            note,
            merkle_path,
        });

        Ok(())
    }

    /// Adds an address which will receive funds in this transaction.
    pub fn add_recipient(
        &mut self,
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        value: NoteValue,
        memo: Option<()>,
    ) -> Result<(), &'static str> {
        if !self.flags.outputs_enabled() {
            return Err("Outputs are not enabled for this builder");
        }

        self.recipients.push(RecipientInfo {
            ovk,
            recipient,
            value,
            memo,
        });

        Ok(())
    }

    /// Builds a bundle containing the given spent notes and recipients.
    ///
    /// This API assumes that none of the notes being spent are controlled by (threshold)
    /// multisignatures, and immediately constructs the bundle proof.
    fn build<V: TryFrom<i64>>(
        mut self,
        mut rng: impl RngCore,
        pk: &ProvingKey,
    ) -> Result<Bundle<Unauthorized, V>, Error> {
        // Pair up the spends and recipients, extending with dummy values as necessary.
        //
        // TODO: Do we want to shuffle the order like we do for Sapling? And if we do, do
        // we need the extra logic for mapping the user-provided input order to the
        // shuffled order?
        let pre_actions: Vec<_> = {
            let num_spends = self.spends.len();
            let num_recipients = self.recipients.len();
            let num_actions = [num_spends, num_recipients, MIN_ACTIONS]
                .iter()
                .max()
                .cloned()
                .unwrap();

            self.spends.extend(
                iter::repeat_with(|| SpendInfo::dummy(&mut rng)).take(num_actions - num_spends),
            );
            self.recipients.extend(
                iter::repeat_with(|| RecipientInfo::dummy(&mut rng))
                    .take(num_actions - num_recipients),
            );

            self.spends
                .into_iter()
                .zip(self.recipients.into_iter())
                .map(|(spend, recipient)| ActionInfo::new(spend, recipient, &mut rng))
                .collect()
        };

        // Move some things out of self that we will need.
        let flags = self.flags;
        let anchor = self.anchor;

        // Determine the value balance for this bundle, ensuring it is valid.
        let value_balance = pre_actions
            .iter()
            .fold(Some(ValueSum::zero()), |acc, action| {
                acc? + action.value_sum()?
            })
            .ok_or(OverflowError)?;

        let result_value_balance: V = i64::try_from(value_balance)
            .map_err(Error::ValueSum)
            .and_then(|i| V::try_from(i).map_err(|_| Error::ValueSum(value::OverflowError)))?;

        // Compute the transaction binding signing key.
        let bsk = pre_actions
            .iter()
            .map(|a| &a.rcv)
            .sum::<ValueCommitTrapdoor>()
            .into_bsk();

        // Create the actions.
        let (actions, circuits): (Vec<_>, Vec<_>) =
            pre_actions.into_iter().map(|a| a.build(&mut rng)).unzip();

        // Verify that bsk and bvk are consistent.
        let bvk = (actions.iter().map(|a| a.cv_net()).sum::<ValueCommitment>()
            - ValueCommitment::derive(value_balance, ValueCommitTrapdoor::zero()))
        .into_bvk();
        assert_eq!(redpallas::VerificationKey::from(&bsk), bvk);

        // Create the proof.
        let instances: Vec<_> = actions
            .iter()
            .map(|a| a.to_instance(flags, anchor.clone()))
            .collect();
        let proof = Proof::create(pk, &circuits, &instances)?;

        Ok(Bundle::from_parts(
            NonEmpty::from_vec(actions).unwrap(),
            flags,
            result_value_balance,
            anchor,
            Unauthorized { proof, bsk },
        ))
    }
}

/// The parts needed to sign an [`Action`].
#[derive(Debug)]
pub struct SigningParts {
    /// The spend validating key for this action. Used to match spend authorizing keys to
    /// actions they can create signatures for.
    ak: SpendValidatingKey,
    /// The randomization needed to derive the actual signing key for this note.
    alpha: pallas::Scalar,
}

/// Marker for an unauthorized bundle, with a proof but no signatures.
#[derive(Debug)]
pub struct Unauthorized {
    proof: Proof,
    bsk: redpallas::SigningKey<Binding>,
}

impl Authorization for Unauthorized {
    type SpendAuth = SigningMetadata;
}

/// Container for metadata needed to sign an [`Action`].
#[derive(Debug)]
pub struct SigningMetadata {
    /// If this action is spending a dummy note, this field holds that note's spend
    /// authorizing key.
    ///
    /// These keys are used automatically in [`Bundle<Unauthorized>::prepare`] or
    /// [`Bundle<Unauthorized>::apply_signatures`] to sign dummy spends.
    dummy_ask: Option<SpendAuthorizingKey>,
    parts: SigningParts,
}

/// Marker for a partially-authorized bundle, in the process of being signed.
#[derive(Debug)]
pub struct PartiallyAuthorized {
    proof: Proof,
    binding_signature: redpallas::Signature<Binding>,
    sighash: [u8; 32],
}

impl Authorization for PartiallyAuthorized {
    type SpendAuth = MaybeSigned;
}

/// A heisen[`Signature`] for a particular [`Action`].
///
/// [`Signature`]: redpallas::Signature
#[derive(Debug)]
pub enum MaybeSigned {
    /// The information needed to sign this [`Action`].
    SigningMetadata(SigningParts),
    /// The signature for this [`Action`].
    Signature(redpallas::Signature<SpendAuth>),
}

impl MaybeSigned {
    fn finalize(self) -> Result<redpallas::Signature<SpendAuth>, Error> {
        match self {
            Self::Signature(sig) => Ok(sig),
            _ => Err(Error::MissingSignatures),
        }
    }
}

impl<V> Bundle<Unauthorized, V> {
    /// Loads the sighash into this bundle, preparing it for signing.
    ///
    /// This API ensures that all signatures are created over the same sighash.
    pub fn prepare<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        sighash: [u8; 32],
    ) -> Bundle<PartiallyAuthorized, V> {
        self.authorize(
            &mut rng,
            |rng, _, SigningMetadata { dummy_ask, parts }| {
                // We can create signatures for dummy spends immediately.
                dummy_ask
                    .map(|ask| ask.randomize(&parts.alpha).sign(rng, &sighash))
                    .map(MaybeSigned::Signature)
                    .unwrap_or(MaybeSigned::SigningMetadata(parts))
            },
            |rng, unauth| PartiallyAuthorized {
                proof: unauth.proof,
                binding_signature: unauth.bsk.sign(rng, &sighash),
                sighash,
            },
        )
    }

    /// Applies signatures to this bundle, in order to authorize it.
    pub fn apply_signatures<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        sighash: [u8; 32],
        signing_keys: &[SpendAuthorizingKey],
    ) -> Result<Bundle<Authorized, V>, Error> {
        signing_keys
            .iter()
            .fold(self.prepare(&mut rng, sighash), |partial, ask| {
                partial.sign(&mut rng, ask)
            })
            .finalize()
    }
}

impl<V> Bundle<PartiallyAuthorized, V> {
    /// Signs this bundle with the given [`SpendAuthorizingKey`].
    ///
    /// This will apply signatures for all notes controlled by this spending key.
    pub fn sign<R: RngCore + CryptoRng>(self, mut rng: R, ask: &SpendAuthorizingKey) -> Self {
        let expected_ak = ask.into();
        self.authorize(
            &mut rng,
            |rng, partial, maybe| match maybe {
                MaybeSigned::SigningMetadata(parts) if parts.ak == expected_ak => {
                    MaybeSigned::Signature(ask.randomize(&parts.alpha).sign(rng, &partial.sighash))
                }
                s => s,
            },
            |_, partial| partial,
        )
    }

    /// Finalizes this bundle, enabling it to be included in a transaction.
    ///
    /// Returns an error if any signatures are missing.
    pub fn finalize(self) -> Result<Bundle<Authorized, V>, Error> {
        self.try_authorize(
            &mut (),
            |_, _, maybe| maybe.finalize(),
            |_, partial| {
                Ok(Authorized::from_parts(
                    partial.proof,
                    partial.binding_signature,
                ))
            },
        )
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use rand::{rngs::StdRng, CryptoRng, SeedableRng};
    use std::convert::TryFrom;
    use std::fmt::Debug;

    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::{
        address::testing::arb_address,
        bundle::{Authorized, Bundle, Flags},
        circuit::ProvingKey,
        keys::{
            testing::arb_spending_key, FullViewingKey, OutgoingViewingKey, SpendAuthorizingKey,
            SpendingKey,
        },
        note::testing::arb_note,
        tree::{Anchor, MerklePath},
        value::{testing::arb_positive_note_value, NoteValue, MAX_NOTE_VALUE},
        Address, Note,
    };

    use super::Builder;

    /// An intermediate type used for construction of arbitrary
    /// bundle values. This type is required because of a limitation
    /// of the proptest prop_compose! macro which does not correctly
    /// handle polymorphic generator functions. Instead of generating
    /// a bundle directly, we generate the bundle inputs, and then
    /// are able to use the `build` function to construct the bundle
    /// from these inputs, but using a `ValueBalance` implementation that
    /// is defined by the end user.
    #[derive(Debug)]
    struct ArbitraryBundleInputs<R> {
        rng: R,
        sk: SpendingKey,
        anchor: Anchor,
        notes: Vec<Note>,
        recipient_amounts: Vec<(Address, NoteValue)>,
    }

    impl<R: RngCore + CryptoRng> ArbitraryBundleInputs<R> {
        /// Create a bundle from the set of arbitrary bundle inputs.
        fn into_bundle<V: TryFrom<i64>>(mut self) -> Bundle<Authorized, V> {
            let fvk = FullViewingKey::from(&self.sk);
            let ovk = OutgoingViewingKey::from(&fvk);
            let flags = Flags::from_parts(true, true);
            let mut builder = Builder::new(flags, self.anchor);

            for note in self.notes.into_iter() {
                builder.add_spend(fvk.clone(), note, MerklePath).unwrap();
            }

            for (addr, value) in self.recipient_amounts.into_iter() {
                builder
                    .add_recipient(Some(ovk.clone()), addr, value, None)
                    .unwrap();
            }

            let pk = ProvingKey::build();
            builder
                .build(&mut self.rng, &pk)
                .unwrap()
                .prepare(&mut self.rng, [0; 32])
                .sign(&mut self.rng, &SpendAuthorizingKey::from(&self.sk))
                .finalize()
                .unwrap()
        }
    }

    prop_compose! {
        /// Produce a random valid Orchard bundle.
        fn arb_bundle_inputs(sk: SpendingKey)
        (
            n_notes in 1..30,
            n_recipients in 1..30,
        )
        (
            anchor in prop::array::uniform32(prop::num::u8::ANY).prop_map(Anchor),
            // generate note values that we're certain won't exceed MAX_NOTE_VALUE in total
            notes in vec(
                arb_positive_note_value(MAX_NOTE_VALUE / n_notes as u64).prop_flat_map(arb_note),
                n_notes as usize
            ),
            recipient_amounts in vec(
                arb_address().prop_flat_map(move |a| {
                    arb_positive_note_value(MAX_NOTE_VALUE / n_recipients as u64)
                        .prop_map(move |v| (a, v))
                }),
                n_recipients as usize
            ),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY)
        ) -> ArbitraryBundleInputs<StdRng> {
            ArbitraryBundleInputs {
                rng: StdRng::from_seed(rng_seed),
                sk: sk.clone(),
                anchor,
                notes,
                recipient_amounts
            }
        }
    }

    /// Produce an arbitrary valid Orchard bundle using a random spending key.
    pub fn arb_bundle<V: TryFrom<i64> + Debug>() -> impl Strategy<Value = Bundle<Authorized, V>> {
        arb_spending_key()
            .prop_flat_map(arb_bundle_inputs)
            .prop_map(|inputs| inputs.into_bundle::<V>())
    }

    /// Produce an arbitrary valid Orchard bundle using a specified spending key.
    pub fn arb_bundle_with_key<V: TryFrom<i64> + Debug>(
        k: SpendingKey,
    ) -> impl Strategy<Value = Bundle<Authorized, V>> {
        arb_bundle_inputs(k).prop_map(|inputs| inputs.into_bundle::<V>())
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;

    use super::Builder;
    use crate::{
        bundle::{Authorized, Bundle, Flags},
        circuit::ProvingKey,
        keys::{FullViewingKey, SpendingKey},
        tree::Anchor,
        value::NoteValue,
    };

    #[test]
    fn shielding_bundle() {
        let pk = ProvingKey::build();
        let mut rng = OsRng;

        let sk = SpendingKey::random(&mut rng);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.default_address();

        let mut builder = Builder::new(Flags::from_parts(true, true), Anchor([0; 32]));
        builder
            .add_recipient(None, recipient, NoteValue::from_raw(5000), None)
            .unwrap();
        let bundle: Bundle<Authorized, i64> = dbg!(builder
            .build(&mut rng, &pk)
            .unwrap()
            .prepare(&mut rng, [0; 32]))
        .finalize()
        .unwrap();
        assert_eq!(bundle.value_balance(), &(-5000))
    }
}
