//! functions used for circuit test

use std::{
    env, fs,
    io::{
        self, {Read, Write},
    },
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
    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, circuit).unwrap();

    // Test that the pinned verification key (representing the circuit)
    // is as expected.
    assert_eq!(
        format!("{:#?}\n", vk.pinned()),
        fs::read_to_string(file_name).unwrap().replace("\r\n", "\n")
    );
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

pub(crate) fn conditionally_save_proof_to_disk<C: Circuit<pallas::Base>>(
    vk: &VerifyingKey<Affine>,
    params: &Params<Affine>,
    circuit: C,
    file_name: &str,
) {
    // If the environment variable CIRCUIT_TEST_GENERATE_NEW_PROOF is set,
    // write the old proof in a file
    if env::var_os("CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
        let create_proof = || -> io::Result<()> {
            let proof = Proof::create(vk, params, circuit).unwrap();
            assert!(proof.verify(vk, params).is_ok());

            let file = fs::File::create(file_name)?;
            write_test_case(file, &proof)
        };
        create_proof().expect("should be able to write new proof");
    }
}

pub(crate) fn serialized_proof_test_case_with_circuit<C: Circuit<pallas::Base>>(
    circuit: C,
    file_name: &str,
) {
    // Setup phase: generate parameters, vk for the circuit.
    let params: Params<Affine> = Params::new(11);
    let vk = plonk::keygen_vk(&params, &circuit).unwrap();

    // Conditionally save proof to disk
    conditionally_save_proof_to_disk(&vk, &params, circuit, file_name);

    // Read proof from disk
    let proof = {
        let test_case_bytes = fs::read(file_name).unwrap();
        read_test_case(&test_case_bytes[..]).expect("proof must be valid")
    };

    // Verify the old proof with the new vk
    assert!(proof.verify(&vk, &params).is_ok());
}
