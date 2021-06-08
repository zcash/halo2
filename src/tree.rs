use crate::{
    constants::{MERKLE_CRH_PERSONALIZATION, MERKLE_DEPTH_ORCHARD},
    note::commitment::ExtractedNoteCommitment,
    primitives::sinsemilla::{i2lebsp_k, HashDomain, K},
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use ff::{PrimeField, PrimeFieldBits};
use rand::RngCore;
use std::{convert::TryInto, iter};

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
        MerklePath {
            position: rng.next_u32(),
            auth_path: (0..MERKLE_DEPTH_ORCHARD)
                .map(|_| pallas::Base::rand())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        }
    }

    pub fn root(&self, cmx: ExtractedNoteCommitment) -> Anchor {
        // Initialize `node` to the first hash.
        let init_node = {
            let pos = self.position % 2 == 1;
            hash_layer(0, cond_swap(pos, *cmx, self.auth_path[0]))
        };
        let node = self.auth_path[1..]
            .iter()
            .enumerate()
            .fold(init_node, |node, (i, sibling)| {
                let l_star = i + 1;
                let swap = (self.position >> l_star) == 1;
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
                .chain(i2lebsp_k(l_star).iter().copied().take(K))
                .chain(
                    pair.left
                        .to_le_bits()
                        .iter()
                        .by_val()
                        .take(pallas::Base::NUM_BITS as usize),
                )
                .chain(
                    pair.right
                        .to_le_bits()
                        .iter()
                        .by_val()
                        .take(pallas::Base::NUM_BITS as usize),
                ),
        )
        .unwrap()
}
