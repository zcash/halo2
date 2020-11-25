//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use crate::arithmetic::CurveAffine;
use crate::poly::{
    commitment::ChallengeScalar, multiopen, Coeff, EvaluationDomain, ExtendedLagrangeCoeff,
    LagrangeCoeff, Polynomial,
};

mod circuit;
mod keygen;
mod permutation;
mod prover;
mod verifier;

pub use circuit::*;
pub use keygen::*;
pub use prover::*;
pub use verifier::*;

/// This is a verifying key which allows for the verification of proofs for a
/// particular circuit.
#[derive(Debug)]
pub struct VerifyingKey<C: CurveAffine> {
    domain: EvaluationDomain<C::Scalar>,
    fixed_commitments: Vec<C>,
    permutation_commitments: Vec<Vec<C>>,
    cs: ConstraintSystem<C::Scalar>,
}

/// This is a proving key which allows for the creation of proofs for a
/// particular circuit.
#[derive(Debug)]
pub struct ProvingKey<C: CurveAffine> {
    vk: VerifyingKey<C>,
    // TODO: get rid of this?
    l0: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    fixed_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    fixed_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutations: Vec<Vec<Polynomial<C::Scalar, LagrangeCoeff>>>,
    permutation_polys: Vec<Vec<Polynomial<C::Scalar, Coeff>>>,
    permutation_cosets: Vec<Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>>,
}

/// This is an object which represents a (Turbo)PLONK proof.
// This structure must never allow points at infinity.
#[derive(Debug, Clone)]
pub struct Proof<C: CurveAffine> {
    advice_commitments: Vec<C>,
    h_commitments: Vec<C>,
    permutations: Option<permutation::Proof<C>>,
    advice_evals: Vec<C::Scalar>,
    aux_evals: Vec<C::Scalar>,
    fixed_evals: Vec<C::Scalar>,
    h_evals: Vec<C::Scalar>,
    multiopening: multiopen::Proof<C>,
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
    /// Opening error
    OpeningError,
    /// Transcript error
    TranscriptError,
}

impl<C: CurveAffine> ProvingKey<C> {
    /// Get the underlying [`VerifyingKey`].
    pub fn get_vk(&self) -> &VerifyingKey<C> {
        &self.vk
    }
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Get the underlying [`EvaluationDomain`].
    pub fn get_domain(&self) -> &EvaluationDomain<C::Scalar> {
        &self.domain
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Beta;
pub(crate) type ChallengeBeta<F> = ChallengeScalar<F, Beta>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Gamma;
pub(crate) type ChallengeGamma<F> = ChallengeScalar<F, Gamma>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Y;
pub(crate) type ChallengeY<F> = ChallengeScalar<F, Y>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct X;
pub(crate) type ChallengeX<F> = ChallengeScalar<F, X>;

#[test]
fn test_proving() {
    use crate::arithmetic::{Curve, FieldExt};
    use crate::poly::commitment::{Blind, Params};
    use crate::transcript::DummyHash;
    use crate::tweedle::{EqAffine, Fp, Fq};
    use circuit::{Advice, Column, Fixed};
    use std::marker::PhantomData;
    const K: u32 = 5;

    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    // Initialize the polynomial commitment parameters
    let params: Params<EqAffine> = Params::new::<DummyHash<Fq>>(K);

    struct PLONKConfig {
        a: Column<Advice>,
        b: Column<Advice>,
        c: Column<Advice>,
        d: Column<Advice>,
        e: Column<Advice>,

        sa: Column<Fixed>,
        sb: Column<Fixed>,
        sc: Column<Fixed>,
        sm: Column<Fixed>,
        sp: Column<Fixed>,

        perm: usize,
        perm2: usize,
    }

    trait StandardCS<FF: FieldExt> {
        fn raw_multiply<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>;
        fn raw_add<F>(&mut self, f: F) -> Result<(Variable, Variable, Variable), Error>
        where
            F: FnOnce() -> Result<(FF, FF, FF), Error>;
        fn copy(&mut self, a: Variable, b: Variable) -> Result<(), Error>;
        fn public_input<F>(&mut self, f: F) -> Result<Variable, Error>
        where
            F: FnOnce() -> Result<FF, Error>;
    }

    struct MyCircuit<F: FieldExt> {
        a: Option<F>,
    }

    struct StandardPLONK<'a, F: FieldExt, CS: Assignment<F> + 'a> {
        cs: &'a mut CS,
        config: PLONKConfig,
        current_gate: usize,
        _marker: PhantomData<F>,
    }

    impl<'a, FF: FieldExt, CS: Assignment<FF>> StandardPLONK<'a, FF, CS> {
        fn new(cs: &'a mut CS, config: PLONKConfig) -> Self {
            StandardPLONK {
                cs,
                config,
                current_gate: 0,
                _marker: PhantomData,
            }
        }
    }

    impl<'a, FF: FieldExt, CS: Assignment<FF>> StandardCS<FF> for StandardPLONK<'a, FF, CS> {
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
            self.cs.assign_advice(self.config.d, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.0.square().square())
            })?;
            self.cs.assign_advice(self.config.b, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.1)
            })?;
            self.cs.assign_advice(self.config.e, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.1.square().square())
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
            self.cs.assign_advice(self.config.d, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.0.square().square())
            })?;
            self.cs.assign_advice(self.config.b, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.1)
            })?;
            self.cs.assign_advice(self.config.e, index, || {
                Ok(value.ok_or(Error::SynthesisError)?.1.square().square())
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
        fn copy(&mut self, left: Variable, right: Variable) -> Result<(), Error> {
            let left_column = match left.0 {
                x if x == self.config.a => 0,
                x if x == self.config.b => 1,
                x if x == self.config.c => 2,
                _ => unreachable!(),
            };
            let right_column = match right.0 {
                x if x == self.config.a => 0,
                x if x == self.config.b => 1,
                x if x == self.config.c => 2,
                _ => unreachable!(),
            };

            self.cs
                .copy(self.config.perm, left_column, left.1, right_column, right.1)?;
            self.cs.copy(
                self.config.perm2,
                left_column,
                left.1,
                right_column,
                right.1,
            )
        }
        fn public_input<F>(&mut self, f: F) -> Result<Variable, Error>
        where
            F: FnOnce() -> Result<FF, Error>,
        {
            let index = self.current_gate;
            self.current_gate += 1;
            self.cs.assign_advice(self.config.a, index, || f())?;
            self.cs
                .assign_fixed(self.config.sp, index, || Ok(FF::one()))?;

            Ok(Variable(self.config.a, index))
        }
    }

    impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
        type Config = PLONKConfig;

        fn configure(meta: &mut ConstraintSystem<F>) -> PLONKConfig {
            let e = meta.advice_column();
            let a = meta.advice_column();
            let b = meta.advice_column();
            let sf = meta.fixed_column();
            let c = meta.advice_column();
            let d = meta.advice_column();
            let p = meta.aux_column();

            let perm = meta.permutation(&[a, b, c]);
            let perm2 = meta.permutation(&[a, b, c]);

            let sm = meta.fixed_column();
            let sa = meta.fixed_column();
            let sb = meta.fixed_column();
            let sc = meta.fixed_column();
            let sp = meta.fixed_column();

            meta.create_gate(|meta| {
                let d = meta.query_advice(d, 1);
                let a = meta.query_advice(a, 0);
                let sf = meta.query_fixed(sf, 0);
                let e = meta.query_advice(e, -1);
                let b = meta.query_advice(b, 0);
                let c = meta.query_advice(c, 0);

                let sa = meta.query_fixed(sa, 0);
                let sb = meta.query_fixed(sb, 0);
                let sc = meta.query_fixed(sc, 0);
                let sm = meta.query_fixed(sm, 0);

                a.clone() * sa + b.clone() * sb + a * b * sm + (c * sc * (-F::one())) + sf * (d * e)
            });

            meta.create_gate(|meta| {
                let a = meta.query_advice(a, 0);
                let p = meta.query_aux(p, 0);
                let sp = meta.query_fixed(sp, 0);

                sp * (a + p * (-F::one()))
            });

            PLONKConfig {
                a,
                b,
                c,
                d,
                e,
                sa,
                sb,
                sc,
                sm,
                sp,
                perm,
                perm2,
            }
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<F>,
            config: PLONKConfig,
        ) -> Result<(), Error> {
            let mut cs = StandardPLONK::new(cs, config);

            let _ = cs.public_input(|| Ok(F::one() + F::one()))?;

            for _ in 0..10 {
                let mut a_squared = None;
                let (a0, _, c0) = cs.raw_multiply(|| {
                    a_squared = self.a.map(|a| a.square());
                    Ok((
                        self.a.ok_or(Error::SynthesisError)?,
                        self.a.ok_or(Error::SynthesisError)?,
                        a_squared.ok_or(Error::SynthesisError)?,
                    ))
                })?;
                let (a1, b1, _) = cs.raw_add(|| {
                    let fin = a_squared.and_then(|a2| self.a.map(|a| a + a2));
                    Ok((
                        self.a.ok_or(Error::SynthesisError)?,
                        a_squared.ok_or(Error::SynthesisError)?,
                        fin.ok_or(Error::SynthesisError)?,
                    ))
                })?;
                cs.copy(a0, a1)?;
                cs.copy(b1, c0)?;
            }

            Ok(())
        }
    }

    let circuit: MyCircuit<Fp> = MyCircuit {
        a: Some(Fp::rand()),
    };

    let empty_circuit: MyCircuit<Fp> = MyCircuit { a: None };

    // Initialize the proving key
    let pk = keygen(&params, &empty_circuit).expect("keygen should not fail");

    let mut pubinputs = pk.get_vk().get_domain().empty_lagrange();
    pubinputs[0] = Fp::one();
    pubinputs[0] += Fp::one();
    let pubinput = params
        .commit_lagrange(&pubinputs, Blind::default())
        .to_affine();

    for _ in 0..100 {
        // Create a proof
        let proof = Proof::create::<DummyHash<Fq>, DummyHash<Fp>, _>(
            &params,
            &pk,
            &circuit,
            &[pubinputs.clone()],
        )
        .expect("proof generation should not fail");

        let pubinput_slice = &[pubinput];
        let msm = params.empty_msm();
        let guard = proof
            .verify::<DummyHash<Fq>, DummyHash<Fp>>(&params, pk.get_vk(), msm, pubinput_slice)
            .unwrap();
        {
            let msm = guard.clone().use_challenges();
            assert!(msm.eval());
        }
        {
            let g = guard.compute_g();
            let (msm, _) = guard.clone().use_g(g);
            assert!(msm.eval());
        }
        let msm = guard.clone().use_challenges();
        assert!(msm.clone().eval());
        let guard = proof
            .verify::<DummyHash<Fq>, DummyHash<Fp>>(&params, pk.get_vk(), msm, pubinput_slice)
            .unwrap();
        {
            let msm = guard.clone().use_challenges();
            assert!(msm.eval());
        }
        {
            let g = guard.compute_g();
            let (msm, _) = guard.clone().use_g(g);
            assert!(msm.eval());
        }
    }
}
