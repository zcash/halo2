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
    use std::marker::PhantomData;
    const K: u32 = 5;

    // Initialize the polynomial commitment parameters
    let params: Params<EqAffine> = Params::new::<DummyHash<Fq>>(K);

    struct PLONKConfig {
        a: AdviceWire,
        b: AdviceWire,
        c: AdviceWire,

        sa: FixedWire,
        sb: FixedWire,
        sc: FixedWire,
        sm: FixedWire,
    }

    #[derive(Copy, Clone)]
    struct Variable(AdviceWire, usize);

    trait StandardCS<FF: Field> {
        fn raw_multiply<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>;
        fn raw_add<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>;
    }

    struct MyCircuit<F: Field> {
        a: Option<F>,
    }

    struct StandardPLONK<'a, F: Field, CS: ConstraintSystem<F> + 'a> {
        cs: &'a mut CS,
        config: PLONKConfig,
        current_gate: usize,
        _marker: PhantomData<F>,
    }

    impl<'a, FF: Field, CS: ConstraintSystem<FF>> StandardPLONK<'a, FF, CS> {
        fn new(cs: &'a mut CS, config: PLONKConfig) -> Self {
            StandardPLONK {
                cs,
                config,
                current_gate: 0,
                _marker: PhantomData,
            }
        }
    }

    impl<'a, FF: Field, CS: ConstraintSystem<FF>> StandardCS<FF> for StandardPLONK<'a, FF, CS> {
        fn raw_multiply<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>,
        {
            let index = self.current_gate;
            self.current_gate += 1;
            let mut value = None;
            self.cs.assign_advice(self.config.a, index, || {
                value = Some(f()?);
                Ok(value.ok_or(Error::SynthesisError)?.0)
            })?;
            self.cs.assign_advice(self.config.b, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.1)
            })?;
            self.cs.assign_advice(self.config.c, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.2)
            })?;

            self.cs
                .assign_fixed(self.config.sa, index, || Ok(FF::zero()))?;
            self.cs
                .assign_fixed(self.config.sb, index, || Ok(FF::zero()))?;
            self.cs
                .assign_fixed(self.config.sc, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(self.config.sm, index, || Ok(FF::one()))?;
            Ok((
                Variable(self.config.a, index),
                Variable(self.config.b, index),
                Variable(self.config.c, index),
            ))
        }
        fn raw_add<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>,
        {
            let index = self.current_gate;
            self.current_gate += 1;
            let mut value = None;
            self.cs.assign_advice(self.config.a, index, || {
                value = Some(f()?);
                Ok(value.ok_or(Error::SynthesisError)?.0)
            })?;
            self.cs.assign_advice(self.config.b, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.1)
            })?;
            self.cs.assign_advice(self.config.c, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.2)
            })?;

            self.cs
                .assign_fixed(self.config.sa, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(self.config.sb, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(self.config.sc, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(self.config.sm, index, || Ok(FF::zero()))?;
            Ok((
                Variable(self.config.a, index),
                Variable(self.config.b, index),
                Variable(self.config.c, index),
            ))
        }
    }

    impl<F: Field> Circuit<F> for MyCircuit<F> {
        type Config = PLONKConfig;

        fn configure(meta: &mut MetaCircuit<F>) -> PLONKConfig {
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

            PLONKConfig {
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
            config: PLONKConfig,
        ) -> Result<(), Error> {
            let mut cs = StandardPLONK::new(cs, config);

            for _ in 0..10 {
                let mut a_squared = None;
                let (_, _, _) = cs.raw_multiply(|| {
                    a_squared = self.a.map(|a| a.square());
                    Ok((
                        self.a.ok_or(Error::SynthesisError)?,
                        self.a.ok_or(Error::SynthesisError)?,
                        a_squared.ok_or(Error::SynthesisError)?,
                    ))
                })?;
                let (_, _, _) = cs.raw_add(|| {
                    let fin = a_squared.and_then(|a2| self.a.map(|a| a + a2));
                    Ok((
                        self.a.ok_or(Error::SynthesisError)?,
                        a_squared.ok_or(Error::SynthesisError)?,
                        fin.ok_or(Error::SynthesisError)?,
                    ))
                })?;
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
