//! Types related to Orchard note commitment trees and anchors.

use crate::{
    constants::{
        util::gen_const_array, L_ORCHARD_MERKLE, MERKLE_CRH_PERSONALIZATION, MERKLE_DEPTH_ORCHARD,
    },
    note::commitment::ExtractedNoteCommitment,
    primitives::sinsemilla::{i2lebsp_k, HashDomain},
};
use incrementalmerkletree::{Altitude, Hashable};
use pasta_curves::{arithmetic::FieldExt, pallas};

use ff::{Field, PrimeField, PrimeFieldBits};
use lazy_static::lazy_static;
use rand::RngCore;
use serde::de::{Deserializer, Error};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::iter;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

// The uncommitted leaf is defined as pallas::Base(2).
// <https://zips.z.cash/protocol/protocol.pdf#thmuncommittedorchard>
lazy_static! {
    static ref UNCOMMITTED_ORCHARD: pallas::Base = pallas::Base::from_u64(2);
    pub(crate) static ref EMPTY_ROOTS: Vec<pallas::Base> = {
        iter::empty()
            .chain(Some(*UNCOMMITTED_ORCHARD))
            .chain(
                (0..MERKLE_DEPTH_ORCHARD).scan(*UNCOMMITTED_ORCHARD, |state, l| {
                    *state = hash_with_l(
                        l,
                        Pair {
                            left: *state,
                            right: *state,
                        },
                    )
                    .unwrap();
                    Some(*state)
                }),
            )
            .collect()
    };
}

/// The root of an Orchard commitment tree.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct Anchor(pallas::Base);

impl From<pallas::Base> for Anchor {
    fn from(anchor_field: pallas::Base) -> Anchor {
        Anchor(anchor_field)
    }
}

impl Anchor {
    pub(crate) fn inner(&self) -> pallas::Base {
        self.0
    }
}

impl Anchor {
    /// Parses an Orchard anchor from a byte encoding.
    pub fn from_bytes(bytes: [u8; 32]) -> Option<Anchor> {
        pallas::Base::from_repr(bytes).map(Anchor)
    }

    /// Returns the byte encoding of this anchor.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_repr()
    }
}

/// The Merkle path from a leaf of the note commitment tree
/// to its anchor.
#[derive(Debug)]
pub struct MerklePath {
    position: u32,
    auth_path: [pallas::Base; MERKLE_DEPTH_ORCHARD],
}

impl MerklePath {
    /// Generates a dummy Merkle path for use in dummy spent notes.
    pub(crate) fn dummy(mut rng: &mut impl RngCore) -> Self {
        MerklePath {
            position: rng.next_u32(),
            auth_path: gen_const_array(|_| pallas::Base::random(&mut rng)),
        }
    }

    /// <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
    /// The layer with 2^n nodes is called "layer n":
    ///      - leaves are at layer MERKLE_DEPTH_ORCHARD = 32;
    ///      - the root is at layer 0.
    /// `l` is MERKLE_DEPTH_ORCHARD - layer - 1.
    ///      - when hashing two leaves, we produce a node on the layer above the leaves, i.e.
    ///        layer = 31, l = 0
    ///      - when hashing to the final root, we produce the anchor with layer = 0, l = 31.
    pub fn root(&self, cmx: ExtractedNoteCommitment) -> CtOption<Anchor> {
        self.auth_path
            .iter()
            .enumerate()
            .fold(
                CtOption::new(cmx.inner(), 1.into()),
                |node, (l, sibling)| {
                    let swap = self.position & (1 << l) != 0;
                    node.and_then(|n| hash_with_l(l, cond_swap(swap, n, *sibling)))
                },
            )
            .map(Anchor)
    }

    /// Returns the position of the leaf using this Merkle path.
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Returns the authentication path.
    pub fn auth_path(&self) -> [pallas::Base; MERKLE_DEPTH_ORCHARD] {
        self.auth_path
    }
}

struct Pair {
    left: pallas::Base,
    right: pallas::Base,
}

fn cond_swap(swap: bool, node: pallas::Base, sibling: pallas::Base) -> Pair {
    if swap {
        Pair {
            left: sibling,
            right: node,
        }
    } else {
        Pair {
            left: node,
            right: sibling,
        }
    }
}

/// Implements the function `hash` (internal to MerkleCRH^Orchard) defined
/// in <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
///
/// The layer with 2^n nodes is called "layer n":
///      - leaves are at layer MERKLE_DEPTH_ORCHARD = 32;
///      - the root is at layer 0.
/// `l` is MERKLE_DEPTH_ORCHARD - layer - 1.
///      - when hashing two leaves, we produce a node on the layer above the leaves, i.e.
///        layer = 31, l = 0
///      - when hashing to the final root, we produce the anchor with layer = 0, l = 31.
fn hash_with_l(l: usize, pair: Pair) -> CtOption<pallas::Base> {
    // MerkleCRH Sinsemilla hash domain.
    let domain = HashDomain::new(MERKLE_CRH_PERSONALIZATION);

    domain.hash(
        iter::empty()
            .chain(i2lebsp_k(l).iter().copied())
            .chain(
                pair.left
                    .to_le_bits()
                    .iter()
                    .by_val()
                    .take(L_ORCHARD_MERKLE),
            )
            .chain(
                pair.right
                    .to_le_bits()
                    .iter()
                    .by_val()
                    .take(L_ORCHARD_MERKLE),
            ),
    )
}

/// A newtype wrapper for leaves and internal nodes in the Orchard
/// incremental note commitment tree.
///
/// This wraps a CtOption<pallas::Base> because Sinsemilla hashes
/// can produce a bottom value which needs to be accounted for in
/// the production of a Merkle root. Leaf nodes are always wrapped
/// with the `Some` constructor.
#[derive(Copy, Clone, Debug)]
pub struct MerkleCrhOrchardOutput(pallas::Base);

impl MerkleCrhOrchardOutput {
    /// Creates an incremental tree leaf digest from the specified
    /// Orchard extracted note commitment.
    pub fn from_cmx(value: &ExtractedNoteCommitment) -> Self {
        MerkleCrhOrchardOutput(value.inner())
    }

    /// Convert this digest to its canonical byte representation.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Parses a incremental tree leaf digest from the bytes of
    /// a note commitment.
    ///
    /// Returns the empty `CtOption` if the provided bytes represent
    /// a non-canonical encoding.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Base::from_bytes(bytes).map(MerkleCrhOrchardOutput)
    }
}

/// This instance should only be used for hash table key comparisons.
impl std::cmp::PartialEq for MerkleCrhOrchardOutput {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

/// This instance should only be used for hash table key comparisons.
impl std::cmp::Eq for MerkleCrhOrchardOutput {}

/// This instance should only be used for hash table key hashing.
impl std::hash::Hash for MerkleCrhOrchardOutput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        <Option<pallas::Base>>::from(self.0)
            .map(|b| b.to_bytes())
            .hash(state)
    }
}

impl ConditionallySelectable for MerkleCrhOrchardOutput {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        MerkleCrhOrchardOutput(pallas::Base::conditional_select(&a.0, &b.0, choice))
    }
}

impl Hashable for MerkleCrhOrchardOutput {
    fn empty_leaf() -> Self {
        MerkleCrhOrchardOutput(*UNCOMMITTED_ORCHARD)
    }

    /// Implements `MerkleCRH^Orchard` as defined in
    /// <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
    fn combine(altitude: Altitude, left: &Self, right: &Self) -> Self {
        hash_with_l(
            altitude.into(),
            Pair {
                left: left.0,
                right: right.0,
            },
        )
        .map(MerkleCrhOrchardOutput)
        .unwrap_or_else(|| MerkleCrhOrchardOutput(pallas::Base::zero()))
    }

    fn empty_root(altitude: Altitude) -> Self {
        MerkleCrhOrchardOutput(EMPTY_ROOTS[<usize>::from(altitude)])
    }
}

impl Serialize for MerkleCrhOrchardOutput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_bytes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MerkleCrhOrchardOutput {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let parsed = <[u8; 32]>::deserialize(deserializer)?;
        <Option<_>>::from(Self::from_bytes(&parsed)).ok_or_else(|| {
            Error::custom(
            "Attempted to deserialize a non-canonical representation of a Pallas base field element.",
        )
        })
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    #[cfg(test)]
    use incrementalmerkletree::{
        bridgetree::Frontier as BridgeFrontier, Altitude, Frontier, Hashable,
    };

    use std::convert::TryInto;

    use crate::{
        constants::MERKLE_DEPTH_ORCHARD,
        note::{commitment::ExtractedNoteCommitment, testing::arb_note, Note},
        value::{testing::arb_positive_note_value, MAX_NOTE_VALUE},
    };
    #[cfg(test)]
    use pasta_curves::arithmetic::FieldExt;
    use pasta_curves::pallas;

    use proptest::collection::vec;
    use proptest::prelude::*;

    #[cfg(test)]
    use super::MerkleCrhOrchardOutput;
    use super::{hash_with_l, Anchor, MerklePath, Pair, EMPTY_ROOTS};

    #[test]
    fn test_vectors() {
        let tv_empty_roots = crate::test_vectors::commitment_tree::test_vectors().empty_roots;

        for (height, root) in EMPTY_ROOTS.iter().enumerate() {
            assert_eq!(tv_empty_roots[height], root.to_bytes());
        }
    }

    prop_compose! {
        /// Generates an arbitrary Merkle tree of with `n_notes` nonempty leaves.
        pub fn arb_tree(n_notes: usize)
        (
            // generate note values that we're certain won't exceed MAX_NOTE_VALUE in total
            notes in vec(
                arb_positive_note_value(MAX_NOTE_VALUE / n_notes as u64).prop_flat_map(arb_note),
                n_notes
            ),
        )
        -> (Vec<(Note, MerklePath)>, Anchor) {
            // Inefficient algorithm to build a perfect subtree containing all notes.
            let perfect_subtree_depth = (n_notes as f64).log2().ceil() as usize;
            let n_leaves = 1 << perfect_subtree_depth;

            let commitments: Vec<Option<ExtractedNoteCommitment>> = notes.iter().map(|note| {
                let cmx: ExtractedNoteCommitment = note.commitment().into();
                Some(cmx)
            }).collect();

            let padded_leaves = {
                let mut padded_leaves = commitments.clone();

                let pad = (0..(n_leaves - n_notes)).map(
                    |_| None
                ).collect::<Vec<_>>();

                padded_leaves.extend_from_slice(&pad);
                padded_leaves
            };

            let perfect_subtree = {
                let mut perfect_subtree: Vec<Vec<Option<pallas::Base>>> = vec![
                    padded_leaves.iter().map(|cmx| cmx.map(|cmx| cmx.inner())).collect()
                ];

                // <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
                // The layer with 2^n nodes is called "layer n":
                //      - leaves are at layer MERKLE_DEPTH_ORCHARD = 32;
                //      - the root is at layer 0.
                // `l` is MERKLE_DEPTH_ORCHARD - layer - 1.
                //      - when hashing two leaves, we produce a node on the layer above the leaves, i.e.
                //        layer = 31, l = 0
                //      - when hashing to the final root, we produce the anchor with layer = 0, l = 31.
                for l in 0..perfect_subtree_depth {
                    let inner_nodes = (0..(n_leaves >> (l + 1))).map(|pos| {
                        let left = perfect_subtree[l][pos * 2];
                        let right = perfect_subtree[l][pos * 2 + 1];
                        match (left, right) {
                            (None, None) => None,
                            (Some(left), None) => {
                                let right = EMPTY_ROOTS[l];
                                Some(hash_with_l(l, Pair {left, right}).unwrap())
                            },
                            (Some(left), Some(right)) => {
                                Some(hash_with_l(l, Pair {left, right}).unwrap())
                            },
                            (None, Some(_)) => {
                                unreachable!("The perfect subtree is left-packed.")
                            }
                        }
                    }).collect();
                    perfect_subtree.push(inner_nodes);
                };
                perfect_subtree
            };

            // Get Merkle path for each note commitment
            let auth_paths = {
                let mut auth_paths: Vec<MerklePath> = Vec::new();
                for (pos, _) in commitments.iter().enumerate() {

                    // Initialize the authentication path to the path for an empty tree.
                    let mut auth_path: [pallas::Base; MERKLE_DEPTH_ORCHARD] = (0..MERKLE_DEPTH_ORCHARD).map(|idx| EMPTY_ROOTS[idx]).collect::<Vec<_>>().try_into().unwrap();

                    let mut layer_pos = pos;
                    for height in 0..perfect_subtree_depth {
                        let is_right_sibling = layer_pos & 1 == 1;
                        let sibling = if is_right_sibling {
                            // This node is the right sibling, so we need its left sibling at the current height.
                            perfect_subtree[height][layer_pos - 1]
                        } else {
                            // This node is the left sibling, so we need its right sibling at the current height.
                            perfect_subtree[height][layer_pos + 1]
                        };
                        if let Some(sibling) = sibling {
                            auth_path[height] = sibling;
                        }
                        layer_pos = (layer_pos - is_right_sibling as usize) / 2;
                    };

                    let path = MerklePath {position: pos as u32, auth_path};
                    auth_paths.push(path);
                }
                auth_paths
            };

            // Compute anchor for this tree
            let anchor = auth_paths[0].root(notes[0].commitment().into()).unwrap();

            (
                notes.into_iter().zip(auth_paths.into_iter()).map(|(note, auth_path)| (note, auth_path)).collect(),
                anchor
            )
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        #[allow(clippy::redundant_closure)]
        #[test]
        fn tree(
            (notes_and_auth_paths, anchor) in (1usize..4).prop_flat_map(|n_notes| arb_tree(n_notes))
        ) {
            for (note, auth_path) in notes_and_auth_paths.iter() {
                let computed_anchor = auth_path.root(note.commitment().into()).unwrap();
                assert_eq!(anchor, computed_anchor);
            }
        }
    }

    #[test]
    fn empty_roots_incremental() {
        let tv_empty_roots = crate::test_vectors::commitment_tree::test_vectors().empty_roots;

        for (altitude, tv_root) in tv_empty_roots.iter().enumerate() {
            assert_eq!(
                MerkleCrhOrchardOutput::empty_root(Altitude::from(altitude as u8))
                    .0
                    .to_bytes(),
                *tv_root,
                "Empty root mismatch at altitude {}",
                altitude
            );
        }
    }

    #[test]
    fn anchor_incremental() {
        // These commitment values are derived from the bundle data that was generated for
        // testing commitment tree construction inside of zcashd here.
        // https://github.com/zcash/zcash/blob/ecec1f9769a5e37eb3f7fd89a4fcfb35bc28eed7/src/test/data/merkle_roots_orchard.h
        let commitments = [
            [
                0x68, 0x13, 0x5c, 0xf4, 0x99, 0x33, 0x22, 0x90, 0x99, 0xa4, 0x4e, 0xc9, 0x9a, 0x75,
                0xe1, 0xe1, 0xcb, 0x46, 0x40, 0xf9, 0xb5, 0xbd, 0xec, 0x6b, 0x32, 0x23, 0x85, 0x6f,
                0xea, 0x16, 0x39, 0x0a,
            ],
            [
                0x78, 0x31, 0x50, 0x08, 0xfb, 0x29, 0x98, 0xb4, 0x30, 0xa5, 0x73, 0x1d, 0x67, 0x26,
                0x20, 0x7d, 0xc0, 0xf0, 0xec, 0x81, 0xea, 0x64, 0xaf, 0x5c, 0xf6, 0x12, 0x95, 0x69,
                0x01, 0xe7, 0x2f, 0x0e,
            ],
            [
                0xee, 0x94, 0x88, 0x05, 0x3a, 0x30, 0xc5, 0x96, 0xb4, 0x30, 0x14, 0x10, 0x5d, 0x34,
                0x77, 0xe6, 0xf5, 0x78, 0xc8, 0x92, 0x40, 0xd1, 0xd1, 0xee, 0x17, 0x43, 0xb7, 0x7b,
                0xb6, 0xad, 0xc4, 0x0a,
            ],
            [
                0x9d, 0xdc, 0xe7, 0xf0, 0x65, 0x01, 0xf3, 0x63, 0x76, 0x8c, 0x5b, 0xca, 0x3f, 0x26,
                0x46, 0x60, 0x83, 0x4d, 0x4d, 0xf4, 0x46, 0xd1, 0x3e, 0xfc, 0xd7, 0xc6, 0xf1, 0x7b,
                0x16, 0x7a, 0xac, 0x1a,
            ],
            [
                0xbd, 0x86, 0x16, 0x81, 0x1c, 0x6f, 0x5f, 0x76, 0x9e, 0xa4, 0x53, 0x9b, 0xba, 0xff,
                0x0f, 0x19, 0x8a, 0x6c, 0xdf, 0x3b, 0x28, 0x0d, 0xd4, 0x99, 0x26, 0x16, 0x3b, 0xd5,
                0x3f, 0x53, 0xa1, 0x21,
            ],
        ];

        // This value was produced by the Python test vector generation code implemented here:
        // https://github.com/zcash-hackworks/zcash-test-vectors/blob/f4d756410c8f2456f5d84cedf6dac6eb8c068eed/orchard_merkle_tree.py
        let anchor = [
            0xc8, 0x75, 0xbe, 0x2d, 0x60, 0x87, 0x3f, 0x8b, 0xcd, 0xeb, 0x91, 0x28, 0x2e, 0x64,
            0x2e, 0x0c, 0xc6, 0x5f, 0xf7, 0xd0, 0x64, 0x2d, 0x13, 0x7b, 0x28, 0xcf, 0x28, 0xcc,
            0x9c, 0x52, 0x7f, 0x0e,
        ];

        let mut frontier = BridgeFrontier::<MerkleCrhOrchardOutput, 32>::empty();
        for commitment in commitments.iter() {
            let cmx = MerkleCrhOrchardOutput(pallas::Base::from_bytes(commitment).unwrap());
            frontier.append(&cmx);
        }
        assert_eq!(
            frontier.root().0,
            pallas::Base::from_bytes(&anchor).unwrap()
        );
    }
}
