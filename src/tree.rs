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

    use crate::{
        constants::{MERKLE_CRH_PERSONALIZATION, MERKLE_DEPTH_ORCHARD},
        note::{commitment::ExtractedNoteCommitment, testing::arb_note, Note},
        primitives::sinsemilla::{i2lebsp_k, HashDomain, K},
        value::{testing::arb_positive_note_value, MAX_NOTE_VALUE},
    };
    use ff::{PrimeField, PrimeFieldBits};
    use pasta_curves::pallas;

    use proptest::collection::vec;
    use proptest::prelude::*;

    use super::{hash_layer, Anchor, MerklePath, Pair};

    lazy_static! {
        static ref EMPTY_ROOTS: Vec<pallas::Base> = {
            // MerkleCRH Sinsemilla hash domain.
            let domain = HashDomain::new(MERKLE_CRH_PERSONALIZATION);

            let mut v = vec![pallas::Base::zero()];
            for l_star in 0..MERKLE_DEPTH_ORCHARD {
                let next = domain
                .hash(
                    std::iter::empty()
                        .chain(i2lebsp_k(l_star).iter().copied().take(K))
                        .chain(
                            v[l_star]
                            .to_le_bits()
                            .iter()
                            .by_val()
                            .take(pallas::Base::NUM_BITS as usize)
                        )
                        .chain(
                            v[l_star]
                            .to_le_bits()
                            .iter()
                            .by_val()
                            .take(pallas::Base::NUM_BITS as usize)
                        ),
                )
                .unwrap();
                v.push(next);
            }
            v
        };
    }

    prop_compose! {
        /// Generates an arbitrary Merkle tree of with `n_notes` nonempty leaves.
        pub fn arb_tree(n_notes: i32)
        (
            // generate note values that we're certain won't exceed MAX_NOTE_VALUE in total
            notes in vec(
                arb_positive_note_value(MAX_NOTE_VALUE / n_notes as u64).prop_flat_map(arb_note),
                n_notes as usize
            ),
        )
        -> (Vec<(Note, MerklePath)>, Anchor) {
            // Inefficient algorithm to build a perfect subtree containing all notes.
            let perfect_subtree_depth = (n_notes as f64).log2().ceil() as usize;
            let commitments: Vec<ExtractedNoteCommitment> = notes.iter().map(|note| {
                let cmx: ExtractedNoteCommitment = note.commitment().into();
                cmx
            }).collect();

            let padded_leaves = {
                let mut padded_leaves = commitments.clone();

                let pad = (0..((1 << perfect_subtree_depth) - n_notes as usize)).map(
                    |_| ExtractedNoteCommitment::uncommitted()
                ).collect::<Vec<_>>();

                padded_leaves.extend_from_slice(&pad);
                padded_leaves
            };

            let tree = {
                let mut tree: Vec<Vec<pallas::Base>> = vec![padded_leaves.into_iter().map(|leaf| *leaf).collect()];
                for height in 1..perfect_subtree_depth {
                    let inner_nodes = (0..(perfect_subtree_depth >> height)).map(|pos| {
                        hash_layer(height, Pair {
                            left: tree[height - 1][pos * 2],
                            right: tree[height - 1][pos * 2 + 1],
                        })
                    }).collect();
                    tree.push(inner_nodes);
                };
                tree
            };

            // Get Merkle path for each note commitment
            let auth_paths = {
                let mut auth_paths: Vec<MerklePath> = Vec::new();
                for (pos, _) in commitments.iter().enumerate() {
                    let mut auth_path: [pallas::Base; MERKLE_DEPTH_ORCHARD] = (0..MERKLE_DEPTH_ORCHARD).map(|idx| EMPTY_ROOTS[idx]).collect::<Vec<_>>().try_into().unwrap();

                    let mut layer_pos = pos;
                    for height in 0..perfect_subtree_depth {
                        let is_right_sibling = layer_pos & 1 == 1;
                        let sibling = if is_right_sibling {
                            tree[height][layer_pos - 1]
                        } else {
                            tree[height][layer_pos + 1]
                        };
                        auth_path[height] = sibling;
                        layer_pos = (layer_pos - is_right_sibling as usize) / 2;
                    };

                    let path = MerklePath {position: pos as u32, auth_path};
                    auth_paths.push(path);
                }
                auth_paths
            };

            // Compute anchor for this tree
            let anchor = auth_paths[0].root(commitments[0]);
            for (cmx, auth_path) in commitments.iter().zip(auth_paths.iter()) {
                let computed_anchor = auth_path.root(*cmx);
                assert_eq!(anchor, computed_anchor);
            }

            (
                notes.into_iter().zip(auth_paths.into_iter()).map(|(note, auth_path)| (note, auth_path)).collect(),
                anchor
            )
        }
    }
}
