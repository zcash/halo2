//! functions used for circuit test

use std::{
    env, fs,
    io::{
        self, {Read, Write},
    },
    path::Path,
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

const TEST_DATA_DIR: &str = "src/tests";

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

pub(crate) fn fixed_verification_key_test_with_circuit<C: Circuit<pallas::Base>>(
    circuit: &C,
    file_name: &str,
) {
    let full_file_name = Path::new(TEST_DATA_DIR).join(file_name);

    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, circuit).unwrap();

    let vk_text = format!("{:#?}\n", vk.pinned());

    if env::var_os("CIRCUIT_TEST_GENERATE_NEW_VK").is_some() {
        fs::write(full_file_name, vk_text).expect("Unable to write vk test file")
    } else {
        // Test that the pinned verification key (representing the circuit)
        // is as expected.
        assert_eq!(
            vk_text,
            fs::read_to_string(full_file_name)
                .expect("Unable to read vk test file")
                .replace("\r\n", "\n")
        );
    }
}

/// write proof to a file
fn write_test_case<W: Write>(mut w: W, proof: &Proof) -> io::Result<()> {
    w.write_all(proof.as_ref())?;
    Ok(())
}

/// read proof from a file
fn read_test_case<R: Read>(mut r: R) -> io::Result<Proof> {
    let mut proof_bytes = vec![];
    r.read_to_end(&mut proof_bytes)?;
    let proof = Proof::new(proof_bytes);

    Ok(proof)
}

fn conditionally_save_proof_to_disk<C: Circuit<pallas::Base>>(
    vk: &VerifyingKey<Affine>,
    params: &Params<Affine>,
    circuit: C,
    file_name: &str,
) {
    let full_file_name = Path::new(TEST_DATA_DIR)
        .join(file_name)
        .with_extension("bin");

    // If the environment variable CIRCUIT_TEST_GENERATE_NEW_PROOF is set,
    // write the old proof in a file
    if env::var_os("CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
        let create_proof = || -> io::Result<()> {
            let proof = Proof::create(vk, params, circuit).unwrap();
            assert!(proof.verify(vk, params).is_ok());

            let file = fs::File::create(full_file_name).expect("Unable to write proof test file");
            write_test_case(file, &proof)
        };
        create_proof().expect("should be able to write new proof");
    }
}

pub(crate) fn serialized_proof_test_case_with_circuit<C: Circuit<pallas::Base>>(
    circuit: C,
    file_name: &str,
) {
    let full_file_name = Path::new(TEST_DATA_DIR)
        .join(file_name)
        .with_extension("bin");

    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, &circuit).unwrap();

    // Conditionally save proof to disk
    conditionally_save_proof_to_disk(&vk, &params, circuit, file_name);

    // Read proof from disk
    let proof = {
        let test_case_bytes = fs::read(full_file_name).expect("Unable to read proof test file");
        read_test_case(&test_case_bytes[..]).expect("proof must be valid")
    };

    // Verify the old proof with the new vk
    assert!(proof.verify(&vk, &params).is_ok());
}
