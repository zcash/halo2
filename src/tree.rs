use crate::{
    constants::{
        util::gen_const_array, L_ORCHARD_MERKLE, MERKLE_CRH_PERSONALIZATION, MERKLE_DEPTH_ORCHARD,
    },
    note::commitment::ExtractedNoteCommitment,
    primitives::sinsemilla::{i2lebsp_k, HashDomain},
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use ff::{Field, PrimeFieldBits};
use rand::RngCore;
use std::iter;

/// The root of an Orchard commitment tree.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Anchor(pub [u8; 32]);

impl From<pallas::Base> for Anchor {
    fn from(anchor_field: pallas::Base) -> Anchor {
        Anchor(anchor_field.to_bytes())
    }
}

#[derive(Debug)]
pub struct MerklePath {
    position: u32,
    auth_path: [pallas::Base; MERKLE_DEPTH_ORCHARD],
}

impl MerklePath {
    /// Generates a dummy Merkle path for use in dummy spent notes.
    pub(crate) fn dummy(rng: &mut impl RngCore) -> Self {
        fn dummy_inner(rng: &mut impl RngCore, _idx: usize) -> pallas::Base {
            pallas::Base::random(rng)
        }

        MerklePath {
            position: rng.next_u32(),
            auth_path: gen_const_array(rng, dummy_inner),
        }
    }

    /// <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
    /// The layer with 2^n nodes is called "layer n":
    ///      - leaves are at layer MERKLE_DEPTH_ORCHARD = 32;
    ///      - the root is at layer 0.
    /// `l_star` is MERKLE_DEPTH_ORCHARD - layer - 1.
    ///      - when hashing two leaves, we produce a node on the layer above the leaves, i.e.
    ///        layer = 31, l_star = 0
    ///      - when hashing to the final root, we produce the anchor with layer = 0, l_star = 31.
    pub fn root(&self, cmx: ExtractedNoteCommitment) -> Anchor {
        let node = self
            .auth_path
            .iter()
            .enumerate()
            .fold(*cmx, |node, (l_star, sibling)| {
                let swap = self.position & (1 << l_star) != 0;
                hash_layer(l_star, cond_swap(swap, node, *sibling))
            });
        Anchor(node.to_bytes())
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

// <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
// The layer with 2^n nodes is called "layer n":
//      - leaves are at layer MERKLE_DEPTH_ORCHARD = 32;
//      - the root is at layer 0.
// `l_star` is MERKLE_DEPTH_ORCHARD - layer - 1.
//      - when hashing two leaves, we produce a node on the layer above the leaves, i.e.
//        layer = 31, l_star = 0
//      - when hashing to the final root, we produce the anchor with layer = 0, l_star = 31.
fn hash_layer(l_star: usize, pair: Pair) -> pallas::Base {
    // MerkleCRH Sinsemilla hash domain.
    let domain = HashDomain::new(MERKLE_CRH_PERSONALIZATION);

    domain
        .hash(
            iter::empty()
                .chain(i2lebsp_k(l_star).iter().copied())
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
        .unwrap()
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use lazy_static::lazy_static;
    use std::convert::TryInto;
    use std::iter;

    use crate::{
        constants::MERKLE_DEPTH_ORCHARD,
        note::{commitment::ExtractedNoteCommitment, testing::arb_note, Note},
        value::{testing::arb_positive_note_value, MAX_NOTE_VALUE},
    };
    use pasta_curves::pallas;

    use proptest::collection::vec;
    use proptest::prelude::*;

    use super::{hash_layer, Anchor, MerklePath, Pair};

    lazy_static! {
        static ref EMPTY_ROOTS: Vec<pallas::Base> = {
            iter::empty()
                .chain(Some(pallas::Base::zero()))
                .chain(
                    (0..MERKLE_DEPTH_ORCHARD).scan(pallas::Base::zero(), |state, l_star| {
                        *state = hash_layer(
                            l_star,
                            Pair {
                                left: *state,
                                right: *state,
                            },
                        );
                        Some(*state)
                    }),
                )
                .collect()
        };
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
                    padded_leaves.iter().map(|cmx| cmx.map(|cmx| *cmx)).collect()
                ];

                // <https://zips.z.cash/protocol/protocol.pdf#orchardmerklecrh>
                // The layer with 2^n nodes is called "layer n":
                //      - leaves are at layer MERKLE_DEPTH_ORCHARD = 32;
                //      - the root is at layer 0.
                // `l_star` is MERKLE_DEPTH_ORCHARD - layer - 1.
                //      - when hashing two leaves, we produce a node on the layer above the leaves, i.e.
                //        layer = 31, l_star = 0
                //      - when hashing to the final root, we produce the anchor with layer = 0, l_star = 31.
                for height in 1..perfect_subtree_depth {
                    let l_star = height - 1;
                    let inner_nodes = (0..(n_leaves >> height)).map(|pos| {
                        let left = perfect_subtree[height - 1][pos * 2];
                        let right = perfect_subtree[height - 1][pos * 2 + 1];
                        match (left, right) {
                            (None, None) => None,
                            (Some(left), None) => {
                                let right = EMPTY_ROOTS[height - 1];
                                Some(hash_layer(l_star, Pair {left, right}))
                            },
                            (Some(left), Some(right)) => {
                                Some(hash_layer(l_star, Pair {left, right}))
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
            let anchor = auth_paths[0].root(notes[0].commitment().into());

            (
                notes.into_iter().zip(auth_paths.into_iter()).map(|(note, auth_path)| (note, auth_path)).collect(),
                anchor
            )
        }
    }

    proptest! {
        #[allow(clippy::redundant_closure)]
        #[test]
        fn tree(
            (notes_and_auth_paths, anchor) in (1usize..4).prop_flat_map(|n_notes| arb_tree(n_notes))
        ) {
            for (note, auth_path) in notes_and_auth_paths.iter() {
                let computed_anchor = auth_path.root(note.commitment().into());
                assert_eq!(anchor, computed_anchor);
            }
        }
    }
}
