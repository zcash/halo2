//! functions used for circuit test

use std::{
    env, fs,
    path::{Path, PathBuf},
};

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
const GEN_ENV_VAR_VK: &str = "CIRCUIT_TEST_GENERATE_NEW_VK";
const GEN_ENV_VAR_PROOF: &str = "CIRCUIT_TEST_GENERATE_NEW_PROOF";

/// A proof structure
#[derive(Clone, Debug)]
pub struct Proof(Vec<u8>);
impl AsRef<[u8]> for Proof {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Proof {
    /// Creates a proof for the given circuits and instances.
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
pub(crate) fn test_against_stored_vk<C: Circuit<pallas::Base>>(
    circuit: &C,
    circuit_name: &str,
) {
    let full_file_name = Path::new(TEST_DATA_DIR)
        .join(format!("vk_{circuit_name}"))
        .with_extension("rdata");

    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, circuit).unwrap();

    let vk_text = format!("{:#?}\n", vk.pinned());

    if env::var_os(GEN_ENV_VAR_VK).is_some() {
        fs::write(&full_file_name, &vk_text).expect("Unable to write vk test file")
    }

    assert_eq!(
        vk_text,
        fs::read_to_string(full_file_name)
            .expect("Unable to read vk test file")
            .replace("\r\n", "\n")
    );
}

/// If the environment variable GEN_ENV_VAR_PROOF is set,
/// write the old proof in a file
fn conditionally_save_proof_to_disk<C: Circuit<pallas::Base>>(
    vk: &VerifyingKey<Affine>,
    params: &Params<Affine>,
    circuit: C,
    full_file_name: &PathBuf,
) {
    if env::var_os(GEN_ENV_VAR_PROOF).is_some() {
        let proof = Proof::create(vk, params, circuit).unwrap();
        assert!(proof.verify(vk, params).is_ok());

        fs::write(full_file_name, proof.as_ref()).expect("Unable to write proof test file");
    }
}

/// Test the generated circuit against the stored proof.
pub(crate) fn test_against_stored_proof<C: Circuit<pallas::Base>>(
    circuit: C,
    circuit_name: &str,
) {
    let full_file_name = Path::new(TEST_DATA_DIR)
        .join(format!("proof_{circuit_name}"))
        .with_extension("bin");

    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, &circuit).unwrap();

    // Conditionally save proof to disk
    conditionally_save_proof_to_disk(&vk, &params, circuit, &full_file_name);

    // Read proof from disk
    let proof = Proof::new(fs::read(full_file_name).expect("Unable to read proof test file"));

    // Verify the old proof with the new vk
    assert!(proof.verify(&vk, &params).is_ok());
}
