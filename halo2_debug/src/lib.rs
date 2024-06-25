use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use tiny_keccak::Hasher;

pub fn test_rng() -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(0xdeadbeef)
}

/// Gets the hex representation of the keccak hash of the input data
pub fn keccak_hex<D: AsRef<[u8]>>(data: D) -> String {
    let mut hash = [0u8; 32];
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(data.as_ref());
    hasher.finalize(&mut hash);
    hex::encode(hash)
}

/// When the feature `vector-tests` is enabled, executes the test in a single thread and checks the result against the expected value.
/// When the feature `vector-tests` is disabled, just executes the test.
pub fn test_result<F: FnOnce() -> Vec<u8> + Send>(test: F, _expected: &str) -> Vec<u8> {
    #[cfg(feature = "vector-tests")]
    let result = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap()
        .install(|| {
            let result = test();
            assert_eq!(_expected, keccak_hex(result.clone()),);
            result
        });

    #[cfg(not(feature = "vector-tests"))]
    let result = test();

    result
}

pub mod display;
