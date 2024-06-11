//! Functions used for circuit test.

use std::{env, fs, path::Path};

use rand::rngs::OsRng;

use pasta_curves::{
    vesta::Affine,
    {pallas, vesta},
};

use halo2_proofs::{
    plonk::{
        self, {Circuit, SingleVerifier, VerifyingKey},
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite},
};

const TEST_DATA_DIR: &str = "src/tests/circuit_data";
const GEN_ENV_VAR: &str = "CIRCUIT_TEST_GENERATE_NEW_DATA";

#[derive(Clone, Debug)]
pub struct Proof(Vec<u8>);

impl AsRef<[u8]> for Proof {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Proof {
    /// Creates a proof for the given circuit and instances.
    pub fn create<C>(
        vk: &VerifyingKey<Affine>,
        params: &Params<Affine>,
        circuit: C,
    ) -> Result<Self, plonk::Error>
        where
            C: Circuit<pallas::Base>,
    {
        let pk = plonk::keygen_pk(params, vk.clone(), &circuit).unwrap();

        let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);
        plonk::create_proof(params, &pk, &[circuit], &[&[]], OsRng, &mut transcript)?;
        let proof = transcript.finalize();

        Ok(Proof(proof))
    }

    /// Verifies this proof with the instances.
    pub fn verify(
        &self,
        vk: &VerifyingKey<Affine>,
        params: &Params<Affine>,
    ) -> Result<(), plonk::Error> {
        let strategy = SingleVerifier::new(params);
        let mut transcript = Blake2bRead::init(&self.0[..]);
        plonk::verify_proof(params, vk, strategy, &[&[]], &mut transcript)
    }

    /// Constructs a new Proof value.
    pub fn new(bytes: Vec<u8>) -> Self {
        Proof(bytes)
    }
}

/// Test the generated vk against the stored vk.
///
/// If the env variable GEN_ENV_VAR is set, save `vk` into a file.
pub(crate) fn test_against_stored_vk<C: Circuit<pallas::Base>>(circuit: &C, circuit_name: &str) {
    let file_path = Path::new(TEST_DATA_DIR)
        .join(format!("vk_{circuit_name}"))
        .with_extension("rdata");

    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, circuit).unwrap();

    let vk_text = format!("{:#?}\n", vk.pinned());

    if env::var_os(GEN_ENV_VAR).is_some() {
        fs::write(&file_path, &vk_text).expect("Unable to write vk test file");
    } else {
        assert_eq!(
            vk_text,
            fs::read_to_string(file_path)
                .expect("Unable to read vk test file")
                .replace("\r\n", "\n")
        );
    }
}

/// Test the generated circuit against the stored proof.
///
/// If the env variable GEN_ENV_VAR is set, save `vk` into a file.
pub(crate) fn test_against_stored_proof<C: Circuit<pallas::Base>>(
    circuit: C,
    circuit_name: &str,
    index: usize,
) {
    let file_path = Path::new(TEST_DATA_DIR)
        .join(format!("proof_{circuit_name}_{index}"))
        .with_extension("bin");

    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, &circuit).unwrap();

    let proof = if env::var_os(GEN_ENV_VAR).is_some() {
        // Create the proof and save it into a file
        let proof = Proof::create(&vk, &params, circuit).unwrap();
        fs::write(&file_path, proof.as_ref()).expect("Unable to write proof test file");
        proof
    } else {
        // Read the proof from storage
        Proof::new(fs::read(file_path).expect("Unable to read proof test file"))
    };

    // Verify the stored proof with the generated vk
    assert!(proof.verify(&vk, &params).is_ok());
}
