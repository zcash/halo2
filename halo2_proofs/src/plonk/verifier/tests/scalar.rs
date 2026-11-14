use super::super::pow_by_power_of_two;
use super::*;
use crate::{pasta::Fq, plonk::ProvingKey};
use ff::{Field, PrimeField};

#[test]
fn power_of_two_matches_generic_exponentiation() {
    fn check<F: Field>() {
        let mut rng = UnwrapErr(SysRng);
        for value in [F::ZERO, F::ONE, -F::ONE, F::random(&mut rng)] {
            for k in 0..64 {
                assert_eq!(
                    pow_by_power_of_two(value, k),
                    value.pow([1u64 << k, 0, 0, 0])
                );
            }
        }
    }
    check::<Fp>();
    check::<Fq>();
}

pub(super) fn check_tampered_and_batch_verification(
    params: &Params<EqAffine>,
    pk: &ProvingKey<EqAffine>,
    proof: Vec<u8>,
    instances: &[&[&[Fp]]],
    #[cfg(feature = "batch")] public_inputs: &[Vec<Fp>],
) {
    // Alter the final IPA scalar without breaking its canonical encoding:
    // this must reach the verifier equation and fail, not just fail parsing.
    let mut tampered = proof.clone();
    let last = tampered.len() - 32;
    let scalar = Fp::from_repr(tampered[last..].try_into().unwrap()).unwrap();
    tampered[last..].copy_from_slice((scalar + Fp::ONE).to_repr().as_ref());
    let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&tampered[..]);
    assert!(verify_proof(
        params,
        pk.get_vk(),
        SingleVerifier::new(params),
        instances,
        &mut transcript,
    )
    .is_err());

    #[cfg(feature = "batch")]
    {
        let owned_instances = public_inputs
            .iter()
            .map(|v| vec![v.clone()])
            .collect::<Vec<_>>();
        for count in [0, 1, 4] {
            let mut batch = super::super::BatchVerifier::new();
            for _ in 0..count {
                batch.add_proof(owned_instances.clone(), proof.clone());
            }
            assert!(batch.finalize(params, pk.get_vk()));
        }
        for valid_count in [0, 1, 3] {
            let mut batch = super::super::BatchVerifier::new();
            for _ in 0..valid_count {
                batch.add_proof(owned_instances.clone(), proof.clone());
            }
            batch.add_proof(owned_instances.clone(), tampered.clone());
            assert!(!batch.finalize(params, pk.get_vk()));
        }
    }
}
