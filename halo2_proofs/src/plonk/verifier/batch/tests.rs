use ff::Field;
use pasta_curves::{EqAffine, Fp};

use super::super::InstanceCommitments;
use crate::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{keygen_vk, Circuit, ConstraintSystem, Error, VerifyingKey},
    poly::commitment::{table_pays_off, Params},
    transcript::{Blake2bWrite, Challenge255, EncodedChallenge, Transcript},
};

#[derive(Clone)]
struct TwoInstanceColumns;

impl Circuit<Fp> for TwoInstanceColumns {
    type Config = ();
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        self.clone()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        meta.instance_column();
        meta.instance_column();
    }

    fn synthesize(&self, _: (), _: impl Layouter<Fp>) -> Result<(), Error> {
        Ok(())
    }
}

/// The challenge after absorbing `vk` and the commitments, as a proof's
/// transcript does before reading anything from the proof.
fn challenge(vk: &VerifyingKey<EqAffine>, commitments: &InstanceCommitments<EqAffine>) -> Fp {
    let mut transcript = Blake2bWrite::<_, _, Challenge255<EqAffine>>::init(vec![]);
    vk.hash_into(&mut transcript).unwrap();
    for commitment in commitments.0.iter().flatten() {
        transcript.common_point(*commitment).unwrap();
    }
    transcript.squeeze_challenge().get_scalar()
}

#[test]
fn shared_commitments_match_each_proof_alone() {
    let params = Params::<EqAffine>::new(7);
    let vk = keygen_vk(&params, &TwoInstanceColumns).unwrap();
    let mut proofs = vec![
        vec![
            vec![vec![], vec![Fp::from(3); 10]],
            vec![vec![Fp::from(5); 65], vec![Fp::from(7); 64]],
        ],
        vec![],
        vec![vec![vec![Fp::from(11)], vec![Fp::ZERO; 17]]],
    ];
    // The proofs above hold five columns of at most 64 rows, and each proof
    // added below two of nine and three rows: enough of them that the shared
    // commitment takes the table, while each proof alone does not.
    let extra = (1..)
        .find(|&proofs| table_pays_off(64, 5 + 2 * proofs))
        .unwrap();
    assert!(!table_pays_off(64, 3) && !table_pays_off(9, 2));
    proofs.extend((0..extra).map(|seed| {
        let seed = Fp::from(u64::try_from(seed).unwrap());
        vec![vec![vec![-seed; 9], vec![seed.square(); 3]]]
    }));

    let shared = InstanceCommitments::for_proofs(&params, &vk, &proofs).unwrap();
    assert_eq!(shared.len(), proofs.len());
    // The shared commitment cached a table on `params`, and clones share it,
    // so commit each proof alone on fresh parameters to take the generic path.
    let fresh = Params::<EqAffine>::new(7);
    for (proof, shared) in proofs.iter().zip(&shared) {
        let alone =
            InstanceCommitments::for_proofs(&fresh, &vk, core::slice::from_ref(proof)).unwrap();
        assert_eq!(shared.0, alone[0].0);
        assert_eq!(
            shared.0.iter().map(Vec::len).collect::<Vec<_>>(),
            proof.iter().map(Vec::len).collect::<Vec<_>>(),
        );
        assert_eq!(challenge(&vk, shared), challenge(&vk, &alone[0]));
    }

    let no_proofs: [Vec<Vec<Vec<Fp>>>; 0] = [];
    assert!(InstanceCommitments::for_proofs(&params, &vk, &no_proofs)
        .unwrap()
        .is_empty());
}

#[test]
fn instance_shapes_are_checked_before_committing() {
    let params = Params::<EqAffine>::new(7);
    let vk = keygen_vk(&params, &TwoInstanceColumns).unwrap();
    let max_len = usize::try_from(params.n).unwrap() - (vk.cs.blinding_factors() + 1);
    for (circuit, expected_too_large) in [
        (vec![vec![]], false),
        (vec![vec![], vec![], vec![]], false),
        (vec![vec![Fp::ZERO; max_len + 1], vec![]], true),
    ] {
        // A valid proof first: one invalid proof rejects the whole batch.
        let proofs = [vec![vec![vec![], vec![]]], vec![circuit]];
        let result = InstanceCommitments::for_proofs(&params, &vk, &proofs);
        if expected_too_large {
            assert!(matches!(result, Err(Error::InstanceTooLarge)));
        } else {
            assert!(matches!(result, Err(Error::InvalidInstances)));
        }
    }
    let maximal = [vec![vec![vec![Fp::ZERO; max_len], vec![]]]];
    assert!(InstanceCommitments::for_proofs(&params, &vk, &maximal).is_ok());
}
