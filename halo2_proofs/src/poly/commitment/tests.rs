use std::sync::Arc;

use super::Params;
use crate::{pasta::EqAffine, INSTANCE_WINDOW_ENTRIES_PER_BASE, MAX_CACHED_INSTANCE_ROWS};

#[test]
fn instance_window_cache_is_shared_by_clones_only() {
    const K: u32 = 7;
    const BASE_COUNT: usize = 10;

    let params = Arc::new(Params::<EqAffine>::new(K));
    let debug_before = format!("{params:?}");
    let mut serialized_before = vec![];
    params.write(&mut serialized_before).unwrap();

    let tables = std::thread::scope(|scope| {
        (0..4)
            .map(|_| {
                let params = Arc::clone(&params);
                scope.spawn(move || params.instance_window_table(BASE_COUNT))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        tables[0].len(),
        BASE_COUNT * INSTANCE_WINDOW_ENTRIES_PER_BASE
    );
    assert!(tables
        .iter()
        .skip(1)
        .all(|table| Arc::ptr_eq(&tables[0], table)));

    let cloned = params.as_ref().clone();
    let cloned_table = cloned.instance_window_table(BASE_COUNT);
    assert!(Arc::ptr_eq(&tables[0], &cloned_table));
    assert_eq!(format!("{params:?}"), debug_before);

    let mut serialized_after = vec![];
    params.write(&mut serialized_after).unwrap();
    assert_eq!(serialized_after, serialized_before);

    let deserialized = Params::<EqAffine>::read(&mut serialized_before.as_slice()).unwrap();
    let smaller_table = params.instance_window_table(BASE_COUNT - 1);
    assert!(Arc::ptr_eq(&tables[0], &smaller_table));

    let larger_table = params.instance_window_table(BASE_COUNT + 1);
    assert!(!Arc::ptr_eq(&tables[0], &larger_table));
    let original_prefix = params.instance_window_table(BASE_COUNT);
    assert!(Arc::ptr_eq(&larger_table, &original_prefix));
    assert_eq!(&larger_table[..tables[0].len()], tables[0].as_slice());

    let deserialized_table = deserialized.instance_window_table(BASE_COUNT);
    assert!(!Arc::ptr_eq(&larger_table, &deserialized_table));
    assert_eq!(tables[0], deserialized_table);

    let bounded = params.instance_window_table(MAX_CACHED_INSTANCE_ROWS);
    assert_eq!(
        bounded.len(),
        MAX_CACHED_INSTANCE_ROWS * INSTANCE_WINDOW_ENTRIES_PER_BASE,
    );
    assert!(Arc::ptr_eq(&bounded, &cloned.instance_window_table(0)));
}
