use halo2::arithmetic::*;
use halo2::gadgets::num::*;
use halo2::gadgets::*;
use halo2::plonk::*;
use halo2::poly::commitment::*;
use halo2::transcript::*;
struct SpendCircuit<F> {
    x: Option<F>,
}

use std::time::Instant;

impl<F: Field> Circuit<F> for SpendCircuit<F> {
    type Config = StandardConfig;

    fn configure(meta: &mut MetaCircuit<F>) -> StandardConfig {
        StandardConfig::new(meta)
    }

    fn synthesize(
        &self,
        cs: &mut impl ConstraintSystem<F>,
        config: StandardConfig,
    ) -> Result<(), Error> {
        let mut cs = Standard::new(cs, config);

        let mut x = AllocatedNum::alloc(&mut cs, || Ok(self.x.ok_or(Error::SynthesisError)?))?;
        for _ in 0..1000 {
            x = x.mul(&mut cs, &x)?;
            x = x.add(&mut cs, &x)?;
        }

        Ok(())
    }
}

fn main() {
    const K: u32 = 11;

    // Initialize the polynomial commitment parameters
    println!("Initializing polynomial commitment parameters");
    let params: Params<EqAffine> = Params::new::<DummyHash<Fq>>(K);

    let empty_circuit = SpendCircuit { x: None };

    // Initialize the SRS
    println!("Initializing SRS");
    let srs = SRS::generate(&params, &empty_circuit).expect("SRS generation should not fail");

    // Create a proof
    {
        let circuit = SpendCircuit {
            x: Some(Fp::from_u64(2)),
        };

        println!("Creating proof");
        let start = Instant::now();
        let proof = Proof::create::<DummyHash<Fq>, DummyHash<Fp>, _>(&params, &srs, &circuit)
            .expect("proof generation should not fail");
        let elapsed = start.elapsed();
        println!("Proof created in {:?}", elapsed);

        println!("Verifying proof");
        let start = Instant::now();
        assert!(proof.verify::<DummyHash<Fq>, DummyHash<Fp>>(&params, &srs));
        let elapsed = start.elapsed();
        println!("Proof verified in {:?}", elapsed);
    }
}
