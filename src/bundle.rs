//! Structs related to bundles of Orchard actions.

pub mod commitments;

use std::io;

use blake2b_simd::Hash as Blake2bHash;
use memuse::DynamicUsage;
use nonempty::NonEmpty;
use zcash_note_encryption::try_note_decryption;

use crate::{
    address::Address,
    bundle::commitments::{hash_bundle_auth_data, hash_bundle_txid_data},
    circuit::{Instance, Proof, VerifyingKey},
    keys::IncomingViewingKey,
    note::{ExtractedNoteCommitment, Note, Nullifier, TransmittedNoteCiphertext},
    note_encryption::OrchardDomain,
    primitives::redpallas::{self, Binding, SpendAuth},
    tree::Anchor,
    value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
};

/// An action applied to the global ledger.
///
/// Externally, this both creates a note (adding a commitment to the global ledger),
/// and consumes some note created prior to this action (adding a nullifier to the
/// global ledger).
///
/// Internally, this may both consume a note and create a note, or it may do only one of
/// the two. TODO: Determine which is more efficient (circuit size vs bundle size).
#[derive(Debug, Clone)]
pub struct Action<A> {
    /// The nullifier of the note being spent.
    nf: Nullifier,
    /// The randomized verification key for the note being spent.
    rk: redpallas::VerificationKey<SpendAuth>,
    /// A commitment to the new note being created.
    cmx: ExtractedNoteCommitment,
    /// The transmitted note ciphertext.
    encrypted_note: TransmittedNoteCiphertext,
    /// A commitment to the net value created or consumed by this action.
    cv_net: ValueCommitment,
    /// The authorization for this action.
    authorization: A,
}

impl<T> Action<T> {
    /// Constructs an `Action` from its constituent parts.
    pub fn from_parts(
        nf: Nullifier,
        rk: redpallas::VerificationKey<SpendAuth>,
        cmx: ExtractedNoteCommitment,
        encrypted_note: TransmittedNoteCiphertext,
        cv_net: ValueCommitment,
        authorization: T,
    ) -> Self {
        Action {
            nf,
            rk,
            cmx,
            encrypted_note,
            cv_net,
            authorization,
        }
    }

    /// Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> &Nullifier {
        &self.nf
    }

    /// Returns the randomized verification key for the note being spent.
    pub fn rk(&self) -> &redpallas::VerificationKey<SpendAuth> {
        &self.rk
    }

    /// Returns the commitment to the new note being created.
    pub fn cmx(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    /// Returns the encrypted note ciphertext.
    pub fn encrypted_note(&self) -> &TransmittedNoteCiphertext {
        &self.encrypted_note
    }

    /// Returns the commitment to the net value created or consumed by this action.
    pub fn cv_net(&self) -> &ValueCommitment {
        &self.cv_net
    }

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Transitions this action from one authorization state to another.
    pub fn map<U>(self, step: impl FnOnce(T) -> U) -> Action<U> {
        Action {
            nf: self.nf,
            rk: self.rk,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: self.cv_net,
            authorization: step(self.authorization),
        }
    }

    /// Transitions this action from one authorization state to another.
    pub fn try_map<U, E>(self, step: impl FnOnce(T) -> Result<U, E>) -> Result<Action<U>, E> {
        Ok(Action {
            nf: self.nf,
            rk: self.rk,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: self.cv_net,
            authorization: step(self.authorization)?,
        })
    }

    /// Prepares the public instance for this action, for creating and verifying the
    /// bundle proof.
    pub fn to_instance(&self, flags: Flags, anchor: Anchor) -> Instance {
        Instance {
            anchor,
            cv_net: self.cv_net.clone(),
            nf_old: self.nf,
            rk: self.rk.clone(),
            cmx: self.cmx,
            enable_spend: flags.spends_enabled,
            enable_output: flags.outputs_enabled,
        }
    }
}

impl DynamicUsage for Action<redpallas::Signature<SpendAuth>> {
    #[inline(always)]
    fn dynamic_usage(&self) -> usize {
        0
    }

    #[inline(always)]
    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

/// Orchard-specific flags.
#[derive(Clone, Copy, Debug)]
pub struct Flags {
    /// Flag denoting whether Orchard spends are enabled in the transaction.
    ///
    /// If `false`, spent notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the spent notes may be either real or
    /// dummy notes.
    spends_enabled: bool,
    /// Flag denoting whether Orchard outputs are enabled in the transaction.
    ///
    /// If `false`, created notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the created notes may be either real or
    /// dummy notes.
    outputs_enabled: bool,
}

const FLAG_SPENDS_ENABLED: u8 = 0b0000_0001;
const FLAG_OUTPUTS_ENABLED: u8 = 0b0000_0010;
const FLAGS_EXPECTED_UNSET: u8 = !(FLAG_SPENDS_ENABLED | FLAG_OUTPUTS_ENABLED);

impl Flags {
    /// Construct a set of flags from its constituent parts
    pub fn from_parts(spends_enabled: bool, outputs_enabled: bool) -> Self {
        Flags {
            spends_enabled,
            outputs_enabled,
        }
    }

    /// Flag denoting whether Orchard spends are enabled in the transaction.
    ///
    /// If `false`, spent notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the spent notes may be either real or
    /// dummy notes.
    pub fn spends_enabled(&self) -> bool {
        self.spends_enabled
    }

    /// Flag denoting whether Orchard outputs are enabled in the transaction.
    ///
    /// If `false`, created notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the created notes may be either real or
    /// dummy notes.
    pub fn outputs_enabled(&self) -> bool {
        self.outputs_enabled
    }

    /// Serialize flags to a byte as defined in [Zcash Protocol Spec § 7.1: Transaction
    /// Encoding And Consensus][txencoding].
    ///
    /// [txencoding]: https://zips.z.cash/protocol/protocol.pdf#txnencoding
    pub fn to_byte(&self) -> u8 {
        let mut value = 0u8;
        if self.spends_enabled {
            value |= FLAG_SPENDS_ENABLED;
        }
        if self.outputs_enabled {
            value |= FLAG_OUTPUTS_ENABLED;
        }
        value
    }

    /// Parse from a single byte as defined in [Zcash Protocol Spec § 7.1: Transaction
    /// Encoding And Consensus][txencoding].
    ///
    /// [txencoding]: https://zips.z.cash/protocol/protocol.pdf#txnencoding
    pub fn from_byte(value: u8) -> io::Result<Self> {
        if value & FLAGS_EXPECTED_UNSET == 0 {
            Ok(Self::from_parts(
                value & FLAG_SPENDS_ENABLED != 0,
                value & FLAG_OUTPUTS_ENABLED != 0,
            ))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unexpected bits set in Orchard flags value.",
            ))
        }
    }
}

/// Defines the authorization type of an Orchard bundle.
pub trait Authorization {
    /// The authorization type of an Orchard action.
    type SpendAuth;
}

/// A bundle of actions to be applied to the ledger.
#[derive(Debug, Clone)]
pub struct Bundle<T: Authorization, V> {
    /// The list of actions that make up this bundle.
    actions: NonEmpty<Action<T::SpendAuth>>,
    /// Orchard-specific transaction-level flags for this bundle.
    flags: Flags,
    /// The net value moved out of the Orchard shielded pool.
    ///
    /// This is the sum of Orchard spends minus the sum of Orchard outputs.
    value_balance: V,
    /// The root of the Orchard commitment tree that this bundle commits to.
    anchor: Anchor,
    /// The authorization for this bundle.
    authorization: T,
}

impl<T: Authorization, V> Bundle<T, V> {
    /// Constructs a `Bundle` from its constituent parts.
    pub fn from_parts(
        actions: NonEmpty<Action<T::SpendAuth>>,
        flags: Flags,
        value_balance: V,
        anchor: Anchor,
        authorization: T,
    ) -> Self {
        Bundle {
            actions,
            flags,
            value_balance,
            anchor,
            authorization,
        }
    }

    /// Returns the list of actions that make up this bundle.
    pub fn actions(&self) -> &NonEmpty<Action<T::SpendAuth>> {
        &self.actions
    }

    /// Returns the Orchard-specific transaction-level flags for this bundle.
    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    /// Returns the net value moved into or out of the Orchard shielded pool.
    ///
    /// This is the sum of Orchard spends minus the sum Orchard outputs.
    pub fn value_balance(&self) -> &V {
        &self.value_balance
    }

    /// Returns the root of the Orchard commitment tree that this bundle commits to.
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    /// Returns the authorization for this bundle.
    ///
    /// In the case of a `Bundle<Authorized>`, this is the proof and binding signature.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Computes a commitment to the effects of this bundle, suitable for inclusion within
    /// a transaction ID.
    pub fn commitment<'a>(&'a self) -> BundleCommitment
    where
        i64: From<&'a V>,
    {
        BundleCommitment(hash_bundle_txid_data(self))
    }

    /// Construct a new bundle by applying a transformation that might fail
    /// to the value balance.
    pub fn try_map_value_balance<V0, E, F: FnOnce(V) -> Result<V0, E>>(
        self,
        f: F,
    ) -> Result<Bundle<T, V0>, E> {
        Ok(Bundle {
            actions: self.actions,
            flags: self.flags,
            value_balance: f(self.value_balance)?,
            anchor: self.anchor,
            authorization: self.authorization,
        })
    }

    /// Transitions this bundle from one authorization state to another.
    pub fn authorize<R, U: Authorization>(
        self,
        context: &mut R,
        mut spend_auth: impl FnMut(&mut R, &T, T::SpendAuth) -> U::SpendAuth,
        step: impl FnOnce(&mut R, T) -> U,
    ) -> Bundle<U, V> {
        let authorization = self.authorization;
        Bundle {
            actions: self
                .actions
                .map(|a| a.map(|a_auth| spend_auth(context, &authorization, a_auth))),
            flags: self.flags,
            value_balance: self.value_balance,
            anchor: self.anchor,
            authorization: step(context, authorization),
        }
    }

    /// Transitions this bundle from one authorization state to another.
    pub fn try_authorize<R, U: Authorization, E>(
        self,
        context: &mut R,
        mut spend_auth: impl FnMut(&mut R, &T, T::SpendAuth) -> Result<U::SpendAuth, E>,
        step: impl FnOnce(&mut R, T) -> Result<U, E>,
    ) -> Result<Bundle<U, V>, E> {
        let authorization = self.authorization;
        let new_actions = self
            .actions
            .into_iter()
            .map(|a| a.try_map(|a_auth| spend_auth(context, &authorization, a_auth)))
            .collect::<Result<Vec<_>, E>>()?;

        Ok(Bundle {
            actions: NonEmpty::from_vec(new_actions).unwrap(),
            flags: self.flags,
            value_balance: self.value_balance,
            anchor: self.anchor,
            authorization: step(context, authorization)?,
        })
    }

    pub(crate) fn to_instances(&self) -> Vec<Instance> {
        self.actions
            .iter()
            .map(|a| a.to_instance(self.flags, self.anchor))
            .collect()
    }

    /// Perform trial decryption of each action in the bundle with each of the
    /// specified incoming viewing keys, and return the decrypted note contents
    /// along with the index of the action from which it was derived.
    pub fn decrypt_outputs_for_keys(
        &self,
        keys: &[IncomingViewingKey],
    ) -> Vec<(usize, IncomingViewingKey, Note, Address, [u8; 512])> {
        self.actions
            .iter()
            .enumerate()
            .filter_map(|(idx, action)| {
                let domain = OrchardDomain::for_action(action);
                keys.iter().find_map(move |ivk| {
                    try_note_decryption(&domain, ivk, action)
                        .map(|(n, a, m)| (idx, ivk.clone(), n, a, m))
                })
            })
            .collect()
    }

    /// Perform trial decryption of each action at `action_idx` in the bundle with the
    /// specified incoming viewing key, and return the decrypted note contents.
    pub fn decrypt_output_with_key(
        &self,
        action_idx: usize,
        key: &IncomingViewingKey,
    ) -> Option<(Note, Address, [u8; 512])> {
        self.actions.get(action_idx).and_then(move |action| {
            let domain = OrchardDomain::for_action(action);
            try_note_decryption(&domain, key, action)
        })
    }
}

impl<T: Authorization, V: Copy + Into<ValueSum>> Bundle<T, V> {
    /// Returns the transaction binding validating key for this bundle.
    ///
    /// This can be used to validate the [`Authorized::binding_signature`] returned from
    /// [`Bundle::authorization`].
    pub fn binding_validating_key(&self) -> redpallas::VerificationKey<Binding> {
        (self
            .actions
            .iter()
            .map(|a| a.cv_net())
            .sum::<ValueCommitment>()
            - ValueCommitment::derive(self.value_balance.into(), ValueCommitTrapdoor::zero()))
        .into_bvk()
    }
}

/// Authorizing data for a bundle of actions, ready to be committed to the ledger.
#[derive(Debug, Clone)]
pub struct Authorized {
    proof: Proof,
    binding_signature: redpallas::Signature<Binding>,
}

impl Authorization for Authorized {
    type SpendAuth = redpallas::Signature<SpendAuth>;
}

impl Authorized {
    /// Constructs the authorizing data for a bundle of actions from its constituent parts.
    pub fn from_parts(proof: Proof, binding_signature: redpallas::Signature<Binding>) -> Self {
        Authorized {
            proof,
            binding_signature,
        }
    }

    /// Return the proof component of the authorizing data.
    pub fn proof(&self) -> &Proof {
        &self.proof
    }

    /// Return the binding signature.
    pub fn binding_signature(&self) -> &redpallas::Signature<Binding> {
        &self.binding_signature
    }
}

impl<V> Bundle<Authorized, V> {
    /// Computes a commitment to the authorizing data within for this bundle.
    ///
    /// This together with `Bundle::commitment` bind the entire bundle.
    pub fn authorizing_commitment(&self) -> BundleAuthorizingCommitment {
        BundleAuthorizingCommitment(hash_bundle_auth_data(self))
    }

    /// Verifies the proof for this bundle.
    pub fn verify_proof(&self, vk: &VerifyingKey) -> Result<(), halo2::plonk::Error> {
        self.authorization()
            .proof()
            .verify(vk, &self.to_instances())
    }
}

impl<V: DynamicUsage> DynamicUsage for Bundle<Authorized, V> {
    fn dynamic_usage(&self) -> usize {
        self.actions.dynamic_usage()
            + self.value_balance.dynamic_usage()
            + self.authorization.proof.dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        let bounds = (
            self.actions.dynamic_usage_bounds(),
            self.value_balance.dynamic_usage_bounds(),
            self.authorization.proof.dynamic_usage_bounds(),
        );
        (
            bounds.0 .0 + bounds.1 .0 + bounds.2 .0,
            bounds
                .0
                 .1
                .zip(bounds.1 .1)
                .zip(bounds.2 .1)
                .map(|((a, b), c)| a + b + c),
        )
    }
}

/// A commitment to a bundle of actions.
///
/// This commitment is non-malleable, in the sense that a bundle's commitment will only
/// change if the effects of the bundle are altered.
#[derive(Debug)]
pub struct BundleCommitment(pub Blake2bHash);

/// A commitment to the authorizing data within a bundle of actions.
#[derive(Debug)]
pub struct BundleAuthorizingCommitment(pub Blake2bHash);

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use nonempty::NonEmpty;
    use pasta_curves::{arithmetic::FieldExt, pallas};
    use rand::{rngs::StdRng, SeedableRng};
    use reddsa::orchard::SpendAuth;

    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::{
        circuit::Proof,
        note::{
            commitment::ExtractedNoteCommitment, nullifier::testing::arb_nullifier,
            testing::arb_note, TransmittedNoteCiphertext,
        },
        primitives::redpallas::{
            self,
            testing::{
                arb_binding_signing_key, arb_spendauth_signing_key, arb_spendauth_verification_key,
            },
        },
        value::{
            testing::arb_note_value_bounded, NoteValue, ValueCommitTrapdoor, ValueCommitment,
            ValueSum, MAX_NOTE_VALUE,
        },
        Anchor,
    };

    use super::{Action, Authorization, Authorized, Bundle, Flags};

    /// Marker for an unauthorized bundle with no proofs or signatures.
    #[derive(Debug)]
    pub struct Unauthorized;

    impl Authorization for Unauthorized {
        type SpendAuth = ();
    }

    prop_compose! {
        /// Generate an action without authorization data.
        pub fn arb_unauthorized_action(spend_value: NoteValue, output_value: NoteValue)(
            nf in arb_nullifier(),
            rk in arb_spendauth_verification_key(),
            note in arb_note(output_value),
        ) -> Action<()> {
            let cmx = ExtractedNoteCommitment::from(note.commitment());
            let cv_net = ValueCommitment::derive(
                (spend_value - output_value).unwrap(),
                ValueCommitTrapdoor::zero()
            );
            // FIXME: make a real one from the note.
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: [0u8; 32],
                enc_ciphertext: [0u8; 580],
                out_ciphertext: [0u8; 80]
            };
            Action {
                nf,
                rk,
                cmx,
                encrypted_note,
                cv_net,
                authorization: ()
            }
        }
    }

    /// Generate an unauthorized action having spend and output values less than MAX_NOTE_VALUE / n_actions.
    pub fn arb_unauthorized_action_n(
        n_actions: usize,
        flags: Flags,
    ) -> impl Strategy<Value = (ValueSum, Action<()>)> {
        let spend_value_gen = if flags.spends_enabled {
            Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
        } else {
            Strategy::boxed(Just(NoteValue::zero()))
        };

        spend_value_gen.prop_flat_map(move |spend_value| {
            let output_value_gen = if flags.outputs_enabled {
                Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
            } else {
                Strategy::boxed(Just(NoteValue::zero()))
            };

            output_value_gen.prop_flat_map(move |output_value| {
                arb_unauthorized_action(spend_value, output_value)
                    .prop_map(move |a| ((spend_value - output_value).unwrap(), a))
            })
        })
    }

    prop_compose! {
        /// Generate an action with invalid (random) authorization data.
        pub fn arb_action(spend_value: NoteValue, output_value: NoteValue)(
            nf in arb_nullifier(),
            sk in arb_spendauth_signing_key(),
            note in arb_note(output_value),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
        ) -> Action<redpallas::Signature<SpendAuth>> {
            let cmx = ExtractedNoteCommitment::from(note.commitment());
            let cv_net = ValueCommitment::derive(
                (spend_value - output_value).unwrap(),
                ValueCommitTrapdoor::zero()
            );

            // FIXME: make a real one from the note.
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: [0u8; 32],
                enc_ciphertext: [0u8; 580],
                out_ciphertext: [0u8; 80]
            };

            let rng = StdRng::from_seed(rng_seed);

            Action {
                nf,
                rk: redpallas::VerificationKey::from(&sk),
                cmx,
                encrypted_note,
                cv_net,
                authorization: sk.sign(rng, &fake_sighash),
            }
        }
    }

    /// Generate an authorized action having spend and output values less than MAX_NOTE_VALUE / n_actions.
    pub fn arb_action_n(
        n_actions: usize,
        flags: Flags,
    ) -> impl Strategy<Value = (ValueSum, Action<redpallas::Signature<SpendAuth>>)> {
        let spend_value_gen = if flags.spends_enabled {
            Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
        } else {
            Strategy::boxed(Just(NoteValue::zero()))
        };

        spend_value_gen.prop_flat_map(move |spend_value| {
            let output_value_gen = if flags.outputs_enabled {
                Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
            } else {
                Strategy::boxed(Just(NoteValue::zero()))
            };

            output_value_gen.prop_flat_map(move |output_value| {
                arb_action(spend_value, output_value)
                    .prop_map(move |a| ((spend_value - output_value).unwrap(), a))
            })
        })
    }

    prop_compose! {
        /// Create an arbitrary set of flags.
        pub fn arb_flags()(spends_enabled in prop::bool::ANY, outputs_enabled in prop::bool::ANY) -> Flags {
            Flags::from_parts(spends_enabled, outputs_enabled)
        }
    }

    prop_compose! {
        fn arb_base()(bytes in prop::array::uniform32(0u8..)) -> pallas::Base {
            // Instead of rejecting out-of-range bytes, let's reduce them.
            let mut buf = [0; 64];
            buf[..32].copy_from_slice(&bytes);
            pallas::Base::from_bytes_wide(&buf)
        }
    }

    prop_compose! {
        /// Generate an arbitrary unauthorized bundle. This bundle does not
        /// necessarily respect consensus rules; for that use
        /// [`crate::builder::testing::arb_bundle`]
        pub fn arb_unauthorized_bundle(n_actions: usize)
        (
            flags in arb_flags(),
        )
        (
            acts in vec(arb_unauthorized_action_n(n_actions, flags), n_actions),
            anchor in arb_base().prop_map(Anchor::from),
            flags in Just(flags)
        ) -> Bundle<Unauthorized, ValueSum> {
            let (balances, actions): (Vec<ValueSum>, Vec<Action<_>>) = acts.into_iter().unzip();

            Bundle::from_parts(
                NonEmpty::from_vec(actions).unwrap(),
                flags,
                balances.into_iter().sum::<Result<ValueSum, _>>().unwrap(),
                anchor,
                Unauthorized
            )
        }
    }

    prop_compose! {
        /// Generate an arbitrary bundle with fake authorization data. This bundle does not
        /// necessarily respect consensus rules; for that use
        /// [`crate::builder::testing::arb_bundle`]
        pub fn arb_bundle(n_actions: usize)
        (
            flags in arb_flags(),
        )
        (
            acts in vec(arb_action_n(n_actions, flags), n_actions),
            anchor in arb_base().prop_map(Anchor::from),
            sk in arb_binding_signing_key(),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY),
            fake_proof in vec(prop::num::u8::ANY, 1973),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
            flags in Just(flags)
        ) -> Bundle<Authorized, ValueSum> {
            let (balances, actions): (Vec<ValueSum>, Vec<Action<_>>) = acts.into_iter().unzip();
            let rng = StdRng::from_seed(rng_seed);

            Bundle::from_parts(
                NonEmpty::from_vec(actions).unwrap(),
                flags,
                balances.into_iter().sum::<Result<ValueSum, _>>().unwrap(),
                anchor,
                Authorized {
                    proof: Proof::new(fake_proof),
                    binding_signature: sk.sign(rng, &fake_sighash),
                }
            )
        }
    }
}
