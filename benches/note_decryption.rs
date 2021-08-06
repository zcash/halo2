use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use orchard::{
    builder::Builder,
    bundle::Flags,
    circuit::ProvingKey,
    keys::{FullViewingKey, IncomingViewingKey, SpendingKey},
    note_encryption::{CompactAction, OrchardDomain},
    value::NoteValue,
    Anchor, Bundle,
};
use rand::rngs::OsRng;
use zcash_note_encryption::{try_compact_note_decryption, try_note_decryption};

#[cfg(unix)]
use pprof::criterion::{Output, PProfProfiler};

fn bench_note_decryption(c: &mut Criterion) {
    let rng = OsRng;
    let pk = ProvingKey::build();

    let fvk = FullViewingKey::from(&SpendingKey::from_bytes([7; 32]).unwrap());
    let valid_ivk = IncomingViewingKey::from(&fvk);
    let recipient = fvk.default_address();

    // Compact actions don't have the full AEAD ciphertext, so ZIP 307 trial-decryption
    // relies on an invalid ivk resulting in random noise for which the note commitment
    // is invalid. However, in practice we still get early rejection:
    // - The version byte will be invalid in 255/256 instances.
    // - If the version byte is valid, one of either the note commitment check or the esk
    //   check will be invalid, saving us at least one scalar mul.
    //
    // Our fixed (action, invalid ivk) tuple will always fall into a specific rejection
    // case. In order to reflect the real behaviour in the benchmarks, we trial-decrypt
    // with 1000 invalid ivks (each of which will result in a different uniformly-random
    // plaintext); this is equivalent to trial-decrypting 1000 different actions with the
    // same ivk, but is faster to set up.
    let invalid_ivks: Vec<_> = (0u32..1000)
        .map(|i| {
            let mut sk = [0; 32];
            sk[..4].copy_from_slice(&i.to_le_bytes());
            IncomingViewingKey::from(&FullViewingKey::from(&SpendingKey::from_bytes(sk).unwrap()))
        })
        .collect();

    let bundle = {
        let mut builder = Builder::new(
            Flags::from_parts(true, true),
            Anchor::from_bytes([0; 32]).unwrap(),
        );
        builder
            .add_recipient(None, recipient, NoteValue::from_raw(10), None)
            .unwrap();
        let bundle: Bundle<_, i64> = builder.build(rng).unwrap();
        bundle
            .create_proof(&pk)
            .unwrap()
            .apply_signatures(rng, [0; 32], &[])
            .unwrap()
    };
    let action = bundle.actions().first();

    let domain = OrchardDomain::for_action(action);

    let compact = {
        let mut group = c.benchmark_group("note-decryption");

        group.bench_function("valid", |b| {
            b.iter(|| try_note_decryption(&domain, &valid_ivk, action).unwrap())
        });

        // Non-compact actions will always early-reject at the same point: AEAD decryption.
        group.bench_function("invalid", |b| {
            b.iter(|| try_note_decryption(&domain, &invalid_ivks[0], action))
        });

        let compact = CompactAction::from(action);

        group.bench_function("compact-valid", |b| {
            b.iter(|| try_compact_note_decryption(&domain, &valid_ivk, &compact).unwrap())
        });

        compact
    };

    {
        let mut group = c.benchmark_group("compact-note-decryption");
        group.throughput(Throughput::Elements(invalid_ivks.len() as u64));
        group.bench_function("invalid", |b| {
            b.iter(|| {
                for ivk in &invalid_ivks {
                    try_compact_note_decryption(&domain, ivk, &compact);
                }
            })
        });
    }
}

#[cfg(unix)]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_note_decryption
}
#[cfg(not(unix))]
criterion_group!(benches, bench_note_decryption);
criterion_main!(benches);
