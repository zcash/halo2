//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use crate::arithmetic::CurveAffine;
use crate::polycommit::OpeningProof;
use crate::transcript::Hasher;

#[macro_use]
mod circuit;
mod domain;
mod prover;
mod srs;
mod verifier;

pub use circuit::*;
pub use prover::*;
pub use srs::*;
pub use verifier::*;

use domain::EvaluationDomain;

/// This is a structured reference string (SRS) that is (deterministically)
/// computed from a specific circuit and parameters for the polynomial
/// commitment scheme.
#[derive(Debug)]
pub struct SRS<C: CurveAffine> {
    domain: EvaluationDomain<C::Scalar>,
    fixed_commitments: Vec<C>,
    fixed_polys: Vec<Vec<C::Scalar>>,
    fixed_cosets: Vec<Vec<C::Scalar>>,
    meta: MetaCircuit<C::Scalar>,
}

/// This is an object which represents a (Turbo)PLONK proof.
// This structure must never allow points at infinity.
#[derive(Debug, Clone)]
pub struct Proof<C: CurveAffine> {
    advice_commitments: Vec<C>,
    h_commitments: Vec<C>,
    advice_evals: Vec<C::Scalar>,
    fixed_evals: Vec<C::Scalar>,
    h_evals: Vec<C::Scalar>,
    f_commitment: C,
    q_evals: Vec<C::Scalar>,
    opening: OpeningProof<C>,
}

/// This is an error that could occur during proving or circuit synthesis.
// TODO: these errors need to be cleaned up
#[derive(Debug)]
pub enum Error {
    /// This is an error that can occur during synthesis of the circuit, for
    /// example, when the witness is not present.
    SynthesisError,
    /// The structured reference string or the parameters are not compatible
    /// with the circuit being synthesized.
    IncompatibleParams,
    /// The constraint system is not satisfied.
    ConstraintSystemFailure,
    /// Out of bounds index passed to a backend
    BoundsFailure,
}

fn hash_point<C: CurveAffine, H: Hasher<C::Base>>(
    transcript: &mut H,
    point: &C,
) -> Result<(), Error> {
    let tmp = point.get_xy();
    if bool::from(tmp.is_none()) {
        return Err(Error::SynthesisError);
    };
    let tmp = tmp.unwrap();
    transcript.absorb(tmp.0);
    transcript.absorb(tmp.1);
    Ok(())
}

#[test]
fn test_proving() {
    use crate::arithmetic::{EqAffine, Field, Fp, Fq};
    use crate::polycommit::Params;
    use crate::transcript::DummyHash;
    const K: u32 = 5;

    // Initialize the polynomial commitment parameters
    let params: Params<EqAffine> = Params::new::<DummyHash<Fq>>(K);

    struct MyConfig {
        a: AdviceWire,
        b: AdviceWire,
        c: AdviceWire,

        sa: FixedWire,
        sb: FixedWire,
        sc: FixedWire,
        sm: FixedWire,
    }
    struct MyCircuit<F: Field> {
        a: Option<F>,
    }

    impl<F: Field> Circuit<F> for MyCircuit<F> {
        type Config = MyConfig;

        fn configure(meta: &mut MetaCircuit<F>) -> MyConfig {
            let a = meta.advice_wire();
            let b = meta.advice_wire();
            let c = meta.advice_wire();

            let sa = meta.fixed_wire();
            let sb = meta.fixed_wire();
            let sc = meta.fixed_wire();
            let sm = meta.fixed_wire();

            meta.create_gate(|meta| {
                let a = meta.query_advice(a, 0);
                let b = meta.query_advice(b, 0);
                let c = meta.query_advice(c, 0);

                let sa = meta.query_fixed(sa, 0);
                let sb = meta.query_fixed(sb, 0);
                let sc = meta.query_fixed(sc, 0);
                let sm = meta.query_fixed(sm, 0);

                a.clone() * sa + b.clone() * sb + a * b * sm + (c * sc * (-F::one()))
            });

            MyConfig {
                a,
                b,
                c,
                sa,
                sb,
                sc,
                sm,
            }
        }

        fn synthesize(
            &self,
            cs: &mut impl ConstraintSystem<F>,
            config: MyConfig,
        ) -> Result<(), Error> {
            // Similar to the above...
            let mut row = 0;
            for _ in 0..10 {
                cs.assign_advice(config.a, row, || self.a.ok_or(Error::SynthesisError))?;
                cs.assign_advice(config.b, row, || self.a.ok_or(Error::SynthesisError))?;
                let a_squared = self.a.map(|a| a.square());
                cs.assign_advice(config.c, row, || a_squared.ok_or(Error::SynthesisError))?;
                // Multiplication gate
                cs.assign_fixed(config.sa, row, || Ok(Field::zero()))?;
                cs.assign_fixed(config.sb, row, || Ok(Field::zero()))?;
                cs.assign_fixed(config.sc, row, || Ok(Field::one()))?;
                cs.assign_fixed(config.sm, row, || Ok(Field::one()))?;
                row += 1;

                cs.assign_advice(config.a, row, || self.a.ok_or(Error::SynthesisError))?;
                cs.assign_advice(config.b, row, || a_squared.ok_or(Error::SynthesisError))?;
                let fin = a_squared.and_then(|a_squared| self.a.map(|a| a + a_squared));
                cs.assign_advice(config.c, row, || fin.ok_or(Error::SynthesisError))?;
                // Addition gate
                cs.assign_fixed(config.sa, row, || Ok(Field::one()))?;
                cs.assign_fixed(config.sb, row, || Ok(Field::one()))?;
                cs.assign_fixed(config.sc, row, || Ok(Field::one()))?;
                cs.assign_fixed(config.sm, row, || Ok(Field::zero()))?;
                row += 1;
            }

            Ok(())
        }
    }

    let circuit: MyCircuit<Fp> = MyCircuit {
        a: Some((-Fp::from_u64(2) + Fp::ROOT_OF_UNITY).pow(&[100, 0, 0, 0])),
    };

    let empty_circuit: MyCircuit<Fp> = MyCircuit { a: None };

    // Initialize the SRS
    let srs = SRS::generate(&params, &empty_circuit).expect("SRS generation should not fail");

    // Create a proof
    let proof = Proof::create::<DummyHash<Fq>, DummyHash<Fp>, _>(&params, &srs, &circuit)
        .expect("proof generation should not fail");

    assert!(proof.verify::<DummyHash<Fq>, DummyHash<Fp>>(&params, &srs));
}
