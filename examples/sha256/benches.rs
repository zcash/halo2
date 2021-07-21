use halo2::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner},
    pasta::EqAffine,
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Circuit, ConstraintSystem, Error,
        VerifyingKey,
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};

use std::{
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
};

use criterion::{criterion_group, criterion_main, Criterion};

use crate::{BlockWord, Sha256, Table16Chip, Table16Config, BLOCK_SIZE};

#[allow(dead_code)]
fn bench(name: &str, k: u32, c: &mut Criterion) {
    #[derive(Default)]
    struct MyCircuit {}

    impl<F: FieldExt> Circuit<F> for MyCircuit {
        type Config = Table16Config;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
            Table16Chip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            Table16Chip::<F>::load(config.clone(), &mut layouter)?;
            let table16_chip = Table16Chip::<F>::construct(config);

            // Test vector: "abc"
            let test_input = [
                BlockWord::new(0b01100001011000100110001110000000),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
                BlockWord::new(0),
            ];

            // Create a message of length 31 blocks
            let mut input = Vec::with_capacity(31 * BLOCK_SIZE);
            for _ in 0..31 {
                input.extend_from_slice(&test_input);
            }

            Sha256::digest(table16_chip, layouter.namespace(|| "'abc' * 2"), &input)?;

            Ok(())
        }
    }

    // Initialize the polynomial commitment parameters
    let params_path = Path::new("./benches/sha256_assets/sha256_params");
    if File::open(&params_path).is_err() {
        let params: Params<EqAffine> = Params::new(k);
        let mut buf = Vec::new();

        params.write(&mut buf).expect("Failed to write params");
        let mut file = File::create(&params_path).expect("Failed to create sha256_params");

        file.write_all(&buf[..])
            .expect("Failed to write params to file");
    }

    let params_fs = File::open(&params_path).expect("couldn't load sha256_params");
    let params: Params<EqAffine> =
        Params::read::<_>(&mut BufReader::new(params_fs)).expect("Failed to read params");

    let empty_circuit: MyCircuit = MyCircuit {};

    // Initialize the proving key
    let vk_path = Path::new("./benches/sha256_assets/sha256_vk");
    if File::open(&vk_path).is_err() {
        let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
        let mut buf = Vec::new();

        vk.write(&mut buf).expect("Failed to write vk");
        let mut file = File::create(&vk_path).expect("Failed to create sha256_vk");

        file.write_all(&buf[..])
            .expect("Failed to write vk to file");
    }

    let vk_fs = File::open(&vk_path).expect("couldn't load sha256_params");
    let vk: VerifyingKey<EqAffine> =
        VerifyingKey::<EqAffine>::read::<_, MyCircuit>(&mut BufReader::new(vk_fs), &params)
            .expect("Failed to read vk");

    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let circuit: MyCircuit = MyCircuit {};

    // let prover_name = name.to_string() + "-prover";
    let verifier_name = name.to_string() + "-verifier";

    // /// Benchmark proof creation
    // c.bench_function(&prover_name, |b| {
    //     b.iter(|| {
    //         let mut transcript = Blake2bWrite::init(Fq::one());
    //         create_proof(&params, &pk, &circuit, &[], &mut transcript)
    //             .expect("proof generation should not fail");
    //         let proof: Vec<u8> = transcript.finalize();
    //     });
    // });

    // Create a proof
    let proof_path = Path::new("./benches/sha256_assets/sha256_proof");
    if File::open(&proof_path).is_err() {
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
        create_proof(&params, &pk, &[circuit], &[], &mut transcript)
            .expect("proof generation should not fail");
        let proof: Vec<u8> = transcript.finalize();
        let mut file = File::create(&proof_path).expect("Failed to create sha256_proof");
        file.write_all(&proof[..]).expect("Failed to write proof");
    }

    let mut proof_fs = File::open(&proof_path).expect("couldn't load sha256_proof");
    let mut proof = Vec::<u8>::new();
    proof_fs
        .read_to_end(&mut proof)
        .expect("Couldn't read proof");

    c.bench_function(&verifier_name, |b| {
        b.iter(|| {
            let msm = params.empty_msm();
            let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
            let guard = verify_proof(&params, pk.get_vk(), msm, &[], &mut transcript).unwrap();
            let msm = guard.clone().use_challenges();
            assert!(msm.eval());
        });
    });
}

#[allow(dead_code)]
fn criterion_benchmark(c: &mut Criterion) {
    bench("sha256", 17, c);
    // bench("sha256", 20, c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
