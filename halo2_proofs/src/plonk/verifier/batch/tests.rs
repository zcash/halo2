use super::*;
use crate::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::{pallas, vesta},
    plonk::{
        create_proof, keygen_pk, keygen_vk, Advice, Circuit, Column, ConstraintSystem, Instance,
        ProvingKey, SingleVerifier,
    },
    transcript::{Blake2bRead, Blake2bWrite, Challenge255, TranscriptWrite},
};
use group::{ff::PrimeField, Curve, Group};

type C = vesta::Affine;
type F = vesta::Scalar;

#[derive(Clone)]
struct TestCircuit(Value<F>);

impl Circuit<F> for TestCircuit {
    type Config = (Column<Advice>, Column<Instance>);
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self(Value::unknown())
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = meta.advice_column();
        let instance = meta.instance_column();
        meta.enable_equality(advice);
        meta.enable_equality(instance);
        (advice, instance)
    }

    fn synthesize(
        &self,
        (advice, instance): Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let cell = layouter.assign_region(
            || "public value",
            |mut region| region.assign_advice(|| "value", advice, 0, || self.0),
        )?;
        layouter.constrain_instance(cell.cell(), instance, 0)
    }
}

fn make_proof(params: &Params<C>, pk: &ProvingKey<C>, circuits: usize) -> Vec<u8> {
    let values = [F::from(7)];
    let columns = [values.as_slice()];
    let instances = vec![columns.as_slice(); circuits];
    let circuits = vec![TestCircuit(Value::known(values[0])); circuits];
    let mut transcript = Blake2bWrite::<_, C, Challenge255<C>>::init(Vec::new());
    create_proof(
        params,
        pk,
        &circuits,
        &instances,
        UnwrapErr(SysRng),
        &mut transcript,
    )
    .unwrap();
    transcript.finalize()
}

fn instances(circuits: usize, value: u64) -> Vec<Vec<Vec<F>>> {
    vec![vec![vec![F::from(value)]]; circuits]
}

fn prefetch_differential<C: CurveAffine>()
where
    C::Scalar: FromUniformBytes<64>,
{
    let mut writer = Blake2bWrite::<_, C, Challenge255<C>>::init(Vec::new());
    let point = C::Curve::generator().to_affine();
    writer.write_point(point).unwrap();
    writer.write_scalar(C::Scalar::from(11)).unwrap();
    writer.write_point(point).unwrap();
    let bytes = writer.finalize();
    let len = C::Repr::default().as_ref().len();
    let scalar_len = <C::Scalar as group::ff::PrimeField>::Repr::default()
        .as_ref()
        .len();
    let second = len + scalar_len;
    let ranges = [0..len, second..second + len];
    for end in 0..=bytes.len() {
        let truncated = &bytes[..end];
        let prefetched = prefetch_points::<C>(truncated, &ranges);
        let expected: Vec<_> = [0, second]
            .into_iter()
            .filter(|offset| offset + len <= end)
            .map(|offset| (offset, Some(point)))
            .collect();
        assert_eq!(prefetched, expected);
    }
    let mut malformed = bytes.clone();
    malformed[..len].fill(255);
    assert_eq!(
        prefetch_points::<C>(&malformed, &ranges),
        vec![(0, None), (second, Some(point))]
    );
    malformed[..len].copy_from_slice(C::identity().to_bytes().as_ref());
    assert_eq!(
        prefetch_points::<C>(&malformed, &ranges)[0],
        (0, Some(C::identity()))
    );
}

#[test]
fn batched_decodings_match_individual_decodings_and_skip_partial_points() {
    prefetch_differential::<pallas::Affine>();
    prefetch_differential::<vesta::Affine>();
}

#[test]
fn only_successful_batches_install_layouts_and_clones_share_them() {
    let params = Params::<C>::new(4);
    let circuit = TestCircuit(Value::unknown());
    let vk = keygen_vk(&params, &circuit).unwrap();
    let pk = keygen_pk(&params, vk.clone(), &circuit).unwrap();
    let one = make_proof(&params, &pk, 1);
    let two = make_proof(&params, &pk, 2);

    // This proof parses completely, but its changed public input fails the MSM.
    let wrong_values = [F::from(8)];
    let wrong_columns = [wrong_values.as_slice()];
    let mut plain = Blake2bRead::<_, C, Challenge255<C>>::init(one.as_slice());
    let parsed = verify_proof(
        &params,
        &vk,
        BatchStrategy::new(&params),
        &[wrong_columns.as_slice()],
        &mut plain,
    )
    .unwrap();
    assert!(!parsed.eval());
    let mut invalid = BatchVerifier::new();
    invalid.add_proof(instances(1, 8), one.clone());
    assert!(!invalid.finalize(&params, &vk));
    assert!(vk.proof_point_layouts().get(1).is_none());

    // Even the valid member of a failing batch must not publish a layout.
    let mut mixed = BatchVerifier::new();
    mixed.add_proof(instances(1, 7), one.clone());
    mixed.add_proof(instances(2, 8), two.clone());
    assert!(!mixed.finalize(&params, &vk));
    assert!(vk.proof_point_layouts().get(1).is_none());
    assert!(vk.proof_point_layouts().get(2).is_none());

    let mut truncated = BatchVerifier::new();
    truncated.add_proof(instances(1, 7), one[..one.len() - 1].to_vec());
    assert!(!truncated.finalize(&params, &vk));
    assert!(vk.proof_point_layouts().get(1).is_none());

    let clone = vk.clone();
    let pinned = format!("{:?}", vk.pinned());
    let mut cold = BatchVerifier::new();
    cold.add_proof(instances(1, 7), one.clone());
    cold.add_proof(instances(2, 7), two.clone());
    assert!(cold.finalize(&params, &vk));
    for circuits in [1, 2] {
        let ranges = vk.proof_point_layouts().get(circuits).unwrap();
        assert!(ranges.len() <= crate::MAX_CACHED_PROOF_POINT_RUNS);
        assert!(std::sync::Arc::ptr_eq(
            &ranges,
            &clone.proof_point_layouts().get(circuits).unwrap()
        ));
    }
    assert_eq!(pinned, format!("{:?}", clone.pinned()));
    let mut warm = BatchVerifier::new();
    warm.add_proof(instances(2, 7), two);
    warm.add_proof(instances(1, 7), one.clone());
    assert!(warm.finalize(&params, &clone));

    // Differential full-verifier errors with a learned layout, including every
    // truncation and well-formed proofs rejected only by the final MSM check.
    let ranges = vk.proof_point_layouts().get(1).unwrap();
    let values = [F::from(7)];
    let columns = [values.as_slice()];
    let inputs = [columns.as_slice()];
    let compare = |bytes: &[u8]| {
        let mut plain = Blake2bRead::<_, C, Challenge255<C>>::init(bytes);
        let mut fast =
            PrefetchedBlake2bRead::with_prefetched(bytes, prefetch_points::<C>(bytes, &ranges));
        let a = verify_proof(
            &params,
            &vk,
            SingleVerifier::new(&params),
            &inputs,
            &mut plain,
        );
        let b = verify_proof(
            &params,
            &vk,
            SingleVerifier::new(&params),
            &inputs,
            &mut fast,
        );
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
    };
    for end in 0..=one.len() {
        compare(&one[..end]);
    }
    for range in ranges.iter() {
        let mut malformed = one.clone();
        malformed[range.start..range.start + 32].fill(255);
        compare(&malformed);
    }
    let mut changed_scalar = one;
    let scalar_start = ranges[0].end;
    changed_scalar[scalar_start..scalar_start + 32].copy_from_slice(F::from(23).to_repr().as_ref());
    compare(&changed_scalar);
}
