//! Laws of proof verification, stated through the public API only, aimed at
//! the batch verifier's public-instance commitments: its cached windows, the
//! generic fallback for long columns, and the shared parameter cache.
#![cfg(feature = "batch")]

mod common;

use common::{
    arb_columns, arb_entries, fixture, mutate, verify_batch, verify_single, Mutation, Statement,
    COLUMNS, POOL, ROWS,
};
use halo2_proofs::{pasta::Fp, plonk::Error, poly::commitment::Params};
use proptest::{prelude::*, sample::Index};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Law: verify_batch(B) = AND over s in B of verify_single(s).is_ok().
    #[test]
    fn batch_is_the_conjunction_of_single_verifications(entries in arb_entries(4)) {
        let fixture = fixture();
        let statements = fixture.select(&entries);
        let expected = statements
            .iter()
            .all(|statement| verify_single(&fixture.params, fixture.vk(), statement).is_ok());
        prop_assert_eq!(verify_batch(&fixture.params, fixture.vk(), &statements), expected);
    }

    /// Law: verify_batch(B . sigma) = verify_batch(B) for every permutation sigma.
    #[test]
    fn batch_verdict_ignores_order(
        (entries, order) in arb_entries(4).prop_flat_map(|entries| {
            let order = (0..entries.len()).collect::<Vec<_>>();
            (Just(entries), Just(order).prop_shuffle())
        }),
    ) {
        let fixture = fixture();
        let statements = fixture.select(&entries);
        let permuted = order.iter().map(|&i| statements[i].clone()).collect::<Vec<_>>();
        prop_assert_eq!(
            verify_batch(&fixture.params, fixture.vk(), &permuted),
            verify_batch(&fixture.params, fixture.vk(), &statements),
        );
    }

    /// Law: verify_batch(A ++ B) = verify_batch(A) AND verify_batch(B), and verify_batch([]) = true.
    #[test]
    fn batch_verdict_splits_over_concatenation(a in arb_entries(3), b in arb_entries(3)) {
        let fixture = fixture();
        let (a, b) = (fixture.select(&a), fixture.select(&b));
        let joined = a.iter().chain(&b).cloned().collect::<Vec<_>>();
        prop_assert!(verify_batch(&fixture.params, fixture.vk(), &[]));
        prop_assert_eq!(
            verify_batch(&fixture.params, fixture.vk(), &joined),
            verify_batch(&fixture.params, fixture.vk(), &a)
                && verify_batch(&fixture.params, fixture.vk(), &b),
        );
    }

    /// Law: verify_batch on Params, its clone and its write-read copy agree, and leave write and Debug unchanged.
    #[test]
    fn parameter_cache_is_unobservable(entries in arb_entries(4)) {
        let fixture = fixture();
        let statements = fixture.select(&entries);
        let mut bytes = vec![];
        fixture.params.write(&mut bytes).unwrap();
        let fresh = Params::read(&mut bytes.as_slice()).unwrap();
        let debug = format!("{fresh:?}");

        let verdict = verify_batch(&fresh, fixture.vk(), &statements);
        prop_assert_eq!(verify_batch(&fresh.clone(), fixture.vk(), &statements), verdict);
        prop_assert_eq!(verify_batch(&fixture.params, fixture.vk(), &statements), verdict);

        let mut after = vec![];
        fresh.write(&mut after).unwrap();
        prop_assert_eq!(after, bytes);
        prop_assert_eq!(format!("{fresh:?}"), debug);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Law: an honest proof verifies at every column length, alone or batched, and fails once any public value changes.
    #[test]
    fn honest_proofs_verify_at_every_length(
        columns in arb_columns::<Fp>(),
        companion in 0..POOL,
        (column, row, delta) in (0..COLUMNS, any::<Index>(), 1..u64::MAX),
    ) {
        let fixture = fixture();
        let honest = fixture.prove(columns);
        let changed = Statement {
            proof: honest.proof.clone(),
            columns: mutate(&honest.columns, &Mutation::Change { column, row, delta }),
        };
        let companion = fixture.pool[companion].clone();

        prop_assert!(verify_single(&fixture.params, fixture.vk(), &honest).is_ok());
        prop_assert!(verify_batch(&fixture.params, fixture.vk(), &[honest.clone()]));
        prop_assert!(verify_batch(&fixture.params, fixture.vk(), &[honest, companion.clone()]));

        prop_assert!(verify_single(&fixture.params, fixture.vk(), &changed).is_err());
        prop_assert!(!verify_batch(&fixture.params, fixture.vk(), &[changed.clone()]));
        prop_assert!(!verify_batch(&fixture.params, fixture.vk(), &[changed, companion]));
    }
}

/// Law: for one M in ROWS..2^K, columns zero-padded to length <= M verify and longer ones give InstanceTooLarge.
#[test]
fn instance_length_has_one_threshold() {
    let fixture = fixture();
    let n = 1usize << fixture.params.k();
    let statement = &fixture.pool[0];
    let mut threshold = None;
    for len in ROWS..=n + 1 {
        let mut padded = statement.clone();
        padded.columns[0].resize(len, Fp::from(0));
        let result = verify_single(&fixture.params, fixture.vk(), &padded);
        match (threshold, result) {
            (None, Ok(())) => {}
            (None, Err(Error::InstanceTooLarge)) => threshold = Some(len - 1),
            (Some(_), Err(Error::InstanceTooLarge)) => {}
            (_, other) => panic!("length {len}: unexpected {other:?}"),
        }
        assert_eq!(
            verify_batch(&fixture.params, fixture.vk(), &[padded]),
            threshold.is_none(),
            "length {len}",
        );
    }
    let threshold = threshold.expect("some length is too large");
    assert!((ROWS..n).contains(&threshold));
}

/// Law: any number of instance columns other than COLUMNS gives InvalidInstances, and the batch rejects.
#[test]
fn wrong_column_count_is_rejected() {
    let fixture = fixture();
    for count in [0, 1, COLUMNS + 1] {
        let mut statement = fixture.pool[0].clone();
        statement.columns.resize(count, vec![]);
        assert!(matches!(
            verify_single(&fixture.params, fixture.vk(), &statement),
            Err(Error::InvalidInstances)
        ));
        assert!(!verify_batch(&fixture.params, fixture.vk(), &[statement]));
    }
}
