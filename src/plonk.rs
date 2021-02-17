//! This module provides an implementation of a variant of (Turbo)[PLONK][plonk]
//! that is designed specifically for the polynomial commitment scheme described
//! in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021
//! [plonk]: https://eprint.iacr.org/2019/953

use blake2b_simd::Params as Blake2bParams;

use crate::arithmetic::{CurveAffine, FieldExt};
use crate::poly::{
    commitment::Params, Coeff, EvaluationDomain, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial,
};
use crate::transcript::{ChallengeScalar, Transcript};
use std::convert::TryInto;

mod circuit;
mod keygen;
mod lookup;
pub(crate) mod permutation;
mod vanishing;

mod prover;
mod verifier;

pub use circuit::*;
pub use keygen::*;
pub use prover::*;
pub use verifier::*;

use std::io;

/// This is a verifying key which allows for the verification of proofs for a
/// particular circuit.
#[derive(Debug)]
pub struct VerifyingKey<C: CurveAffine> {
    domain: EvaluationDomain<C::Scalar>,
    fixed_commitments: Vec<C>,
    permutations: Vec<permutation::VerifyingKey<C>>,
    cs: ConstraintSystem<C::Scalar>,
}

impl<C: CurveAffine> VerifyingKey<C> {
    /// Writes a verifying key to a buffer.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        for commitment in &self.fixed_commitments {
            writer.write_all(&commitment.to_bytes())?;
        }
        for permutation in &self.permutations {
            permutation.write(writer)?;
        }

        Ok(())
    }

    /// Reads a verification key from a buffer.
    pub fn read<R: io::Read, ConcreteCircuit: Circuit<C::Scalar>>(
        reader: &mut R,
        params: &Params<C>,
    ) -> io::Result<Self> {
        let (domain, cs, _) = keygen::create_domain::<C, ConcreteCircuit>(params);

        let fixed_commitments: Vec<_> = (0..cs.num_fixed_columns)
            .map(|_| C::read(reader))
            .collect::<Result<_, _>>()?;

        let permutations: Vec<_> = cs
            .permutations
            .iter()
            .map(|argument| permutation::VerifyingKey::read(reader, argument))
            .collect::<Result<_, _>>()?;

        Ok(VerifyingKey {
            domain,
            fixed_commitments,
            permutations,
            cs,
        })
    }

    /// Hashes a verification key into a transcript.
    pub fn hash_into<T: Transcript<C>>(&self, transcript: &mut T) -> io::Result<()> {
        let mut hasher = Blake2bParams::new()
            .hash_length(64)
            .personal(C::BLAKE2B_PERSONALIZATION)
            .to_state();

        let s = format!("{}", self);

        hasher.update(&(s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());

        // Hash in final Blake2bState
        transcript.common_scalar(C::Scalar::from_bytes_wide(
            hasher.finalize().as_bytes().try_into().unwrap(),
        ))?;

        Ok(())
    }
}

impl<C: CurveAffine> std::fmt::Display for VerifyingKey<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "VerificationKey {{\
            base_modulus = {base_modulus}, \
            scalar_modulus = {scalar_modulus}, \
            domain = {domain}, \
            cs = {cs}, \
            fixed_commitments = {fixed_commitments:?}, \
            permutations = {permutations:?}\
        }}",
            base_modulus = C::Base::MODULUS,
            scalar_modulus = C::Scalar::MODULUS,
            domain = self.domain,
            cs = self.cs,
            fixed_commitments = self.fixed_commitments,
            permutations = self.permutations
        )
    }
}

/// This is a proving key which allows for the creation of proofs for a
/// particular circuit.
#[derive(Debug)]
pub struct ProvingKey<C: CurveAffine> {
    vk: VerifyingKey<C>,
    // TODO: get rid of this?
    l0: Polynomial<C::Scalar, ExtendedLagrangeCoeff>,
    fixed_values: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    fixed_polys: Vec<Polynomial<C::Scalar, Coeff>>,
    fixed_cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
    permutations: Vec<permutation::ProvingKey<C>>,
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
struct Theta;
type ChallengeTheta<F> = ChallengeScalar<F, Theta>;

#[derive(Clone, Copy, Debug)]
struct Beta;
type ChallengeBeta<F> = ChallengeScalar<F, Beta>;

#[derive(Clone, Copy, Debug)]
struct Gamma;
type ChallengeGamma<F> = ChallengeScalar<F, Gamma>;

#[derive(Clone, Copy, Debug)]
struct Y;
type ChallengeY<F> = ChallengeScalar<F, Y>;

#[derive(Clone, Copy, Debug)]
struct X;
type ChallengeX<F> = ChallengeScalar<F, X>;

#[test]
fn test_proving() {
    use crate::arithmetic::{Curve, FieldExt};
    use crate::dev::MockProver;
    use crate::pasta::{EqAffine, Fp};
    use crate::poly::{
        commitment::{Blind, Params},
        Rotation,
    };
    use crate::transcript::{Blake2bRead, Blake2bWrite};
    use circuit::{Advice, Column, Fixed};
    use std::marker::PhantomData;
    const K: u32 = 5;

    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    // Initialize the polynomial commitment parameters
    let params: Params<EqAffine> = Params::new(K);

    #[derive(Copy, Clone)]
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
        sl: Column<Fixed>,
        sl2: Column<Fixed>,

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
        fn lookup_table(&mut self, values: &[Vec<FF>]) -> Result<(), Error>;
    }

    #[derive(Clone)]
    struct MyCircuit<F: FieldExt> {
        a: Option<F>,
        lookup_tables: Vec<Vec<F>>,
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
            self.cs.assign_advice(
                || "lhs",
                self.config.a,
                index,
                || {
                    value = Some(f()?);
                    Ok(value.ok_or(Error::SynthesisError)?.0)
                },
            )?;
            self.cs.assign_advice(
                || "lhs^4",
                self.config.d,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.0.square().square()),
            )?;
            self.cs.assign_advice(
                || "rhs",
                self.config.b,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1),
            )?;
            self.cs.assign_advice(
                || "rhs^4",
                self.config.e,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1.square().square()),
            )?;
            self.cs.assign_advice(
                || "out",
                self.config.c,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.2),
            )?;

            self.cs
                .assign_fixed(|| "a", self.config.sa, index, || Ok(FF::zero()))?;
            self.cs
                .assign_fixed(|| "b", self.config.sb, index, || Ok(FF::zero()))?;
            self.cs
                .assign_fixed(|| "c", self.config.sc, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "a * b", self.config.sm, index, || Ok(FF::one()))?;
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
            self.cs.assign_advice(
                || "lhs",
                self.config.a,
                index,
                || {
                    value = Some(f()?);
                    Ok(value.ok_or(Error::SynthesisError)?.0)
                },
            )?;
            self.cs.assign_advice(
                || "lhs^4",
                self.config.d,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.0.square().square()),
            )?;
            self.cs.assign_advice(
                || "rhs",
                self.config.b,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1),
            )?;
            self.cs.assign_advice(
                || "rhs^4",
                self.config.e,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.1.square().square()),
            )?;
            self.cs.assign_advice(
                || "out",
                self.config.c,
                index,
                || Ok(value.ok_or(Error::SynthesisError)?.2),
            )?;

            self.cs
                .assign_fixed(|| "a", self.config.sa, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "b", self.config.sb, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "c", self.config.sc, index, || Ok(FF::one()))?;
            self.cs
                .assign_fixed(|| "a * b", self.config.sm, index, || Ok(FF::zero()))?;
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
            self.cs
                .assign_advice(|| "value", self.config.a, index, || f())?;
            self.cs
                .assign_fixed(|| "public", self.config.sp, index, || Ok(FF::one()))?;

            Ok(Variable(self.config.a, index))
        }
        fn lookup_table(&mut self, values: &[Vec<FF>]) -> Result<(), Error> {
            for (&value_0, &value_1) in values[0].iter().zip(values[1].iter()) {
                let index = self.current_gate;

                self.current_gate += 1;
                self.cs
                    .assign_fixed(|| "table col 1", self.config.sl, index, || Ok(value_0))?;
                self.cs
                    .assign_fixed(|| "table col 2", self.config.sl2, index, || Ok(value_1))?;
            }
            Ok(())
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
            let p = meta.instance_column();

            let perm = meta.permutation(&[a.into(), b.into(), c.into()]);
            let perm2 = meta.permutation(&[a.into(), b.into(), c.into()]);

            let sm = meta.fixed_column();
            let sa = meta.fixed_column();
            let sb = meta.fixed_column();
            let sc = meta.fixed_column();
            let sp = meta.fixed_column();
            let sl = meta.fixed_column();
            let sl2 = meta.fixed_column();

            /*
             *   A         B      ...  sl        sl2
             * [
             *   instance  0      ...  0         0
             *   a         a      ...  0         0
             *   a         a^2    ...  0         0
             *   a         a      ...  0         0
             *   a         a^2    ...  0         0
             *   ...       ...    ...  ...       ...
             *   ...       ...    ...  instance  0
             *   ...       ...    ...  a         a
             *   ...       ...    ...  a         a^2
             *   ...       ...    ...  0         0
             * ]
             */
            meta.lookup(&[a.into()], &[sl.into()]);
            meta.lookup(&[a.into(), b.into()], &[sl.into(), sl2.into()]);

            meta.create_gate("Combined add-mult", |meta| {
                let d = meta.query_advice(d, Rotation::next());
                let a = meta.query_advice(a, Rotation::cur());
                let sf = meta.query_fixed(sf, Rotation::cur());
                let e = meta.query_advice(e, Rotation::prev());
                let b = meta.query_advice(b, Rotation::cur());
                let c = meta.query_advice(c, Rotation::cur());

                let sa = meta.query_fixed(sa, Rotation::cur());
                let sb = meta.query_fixed(sb, Rotation::cur());
                let sc = meta.query_fixed(sc, Rotation::cur());
                let sm = meta.query_fixed(sm, Rotation::cur());

                a.clone() * sa + b.clone() * sb + a * b * sm + (c * sc * (-F::one())) + sf * (d * e)
            });

            meta.create_gate("Public input", |meta| {
                let a = meta.query_advice(a, Rotation::cur());
                let p = meta.query_instance(p, Rotation::cur());
                let sp = meta.query_fixed(sp, Rotation::cur());

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
                sl,
                sl2,
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

            cs.lookup_table(&self.lookup_tables)?;

            Ok(())
        }
    }

    let a = Fp::from_u64(2834758237) * Fp::ZETA;
    let a_squared = a * &a;
    let instance = Fp::one() + Fp::one();
    let lookup_table = vec![instance, a, a, Fp::zero()];
    let lookup_table_2 = vec![Fp::zero(), a, a_squared, Fp::zero()];

    let empty_circuit: MyCircuit<Fp> = MyCircuit {
        a: None,
        lookup_tables: vec![lookup_table.clone(), lookup_table_2.clone()],
    };

    let circuit: MyCircuit<Fp> = MyCircuit {
        a: Some(a),
        lookup_tables: vec![lookup_table, lookup_table_2],
    };

    // Initialize the proving key
    let vk = keygen_vk(&params, &empty_circuit).expect("keygen_vk should not fail");
    assert_eq!(format!("{}", vk), "VerificationKey {base_modulus = 0x40000000000000000000000000000000224698fc0994a8dd8c46eb2100000001, scalar_modulus = 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001, domain = Domain {k = 5, extended_k = 7, omega = 0x0cc3380dc616f2e1daf29ad1560833ed3baea3393eceb7bc8fa36376929b78cc }, cs = ConstraintSystem {num_fixed_columns: 8, num_advice_columns: 5, num_instance_columns: 1, fixed_queries: [(Column { index: 6, column_type: Fixed }, Rotation(0)), (Column { index: 7, column_type: Fixed }, Rotation(0)), (Column { index: 0, column_type: Fixed }, Rotation(0)), (Column { index: 2, column_type: Fixed }, Rotation(0)), (Column { index: 3, column_type: Fixed }, Rotation(0)), (Column { index: 4, column_type: Fixed }, Rotation(0)), (Column { index: 1, column_type: Fixed }, Rotation(0)), (Column { index: 5, column_type: Fixed }, Rotation(0))], advice_queries: [(Column { index: 1, column_type: Advice }, Rotation(0)), (Column { index: 2, column_type: Advice }, Rotation(0)), (Column { index: 3, column_type: Advice }, Rotation(0)), (Column { index: 4, column_type: Advice }, Rotation(1)), (Column { index: 0, column_type: Advice }, Rotation(-1))], instance_queries: [(Column { index: 0, column_type: Instance }, Rotation(0))], permutations: [Argument { columns: [Column { index: 1, column_type: Advice }, Column { index: 2, column_type: Advice }, Column { index: 3, column_type: Advice }] }, Argument { columns: [Column { index: 1, column_type: Advice }, Column { index: 2, column_type: Advice }, Column { index: 3, column_type: Advice }] }], lookups: [Argument { input_columns: [Column { index: 1, column_type: Advice }], table_columns: [Column { index: 6, column_type: Fixed }] }, Argument { input_columns: [Column { index: 1, column_type: Advice }, Column { index: 2, column_type: Advice }], table_columns: [Column { index: 6, column_type: Fixed }, Column { index: 7, column_type: Fixed }] }], gates: [Sum(Sum(Sum(Sum(Product(Advice(0), Fixed(3)), Product(Advice(1), Fixed(4))), Product(Product(Advice(0), Advice(1)), Fixed(6))), Scaled(Product(Advice(2), Fixed(5)), 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000000)), Product(Fixed(2), Product(Advice(3), Advice(4)))), Product(Fixed(7), Sum(Advice(0), Scaled(Instance(0), 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000000))), ]}, fixed_commitments = [(0x3710f15f98bf0a7421343fdf390b9519506c67431a5c78678fbcc4db815c8547, 0x0a3c77f30ab2a2741b21cd45326c87dc7f7050b1a5e8c181a534b9c84a09cb02), (0x115235d6bd2467772db857d66e7f44837cd38bb6ac0c7b412c2997dd47cbbc3c, 0x0f339c20c40e10bed476699e2ddb88256092447582b250f329b0cbf6c3f66f17), (0x2a9ba245dcce53752e1de999b45c975472bb33b58aed8bcebdfcd185627895f0, 0x1252bf596b7acd19625f68845749f6672b99e71238cfabe91ca545103168d0f0), (0x2a9ba245dcce53752e1de999b45c975472bb33b58aed8bcebdfcd185627895f0, 0x1252bf596b7acd19625f68845749f6672b99e71238cfabe91ca545103168d0f0), (0x241d6d9c2060ce821d4b05ff2f9566c3947541f3d14a9aabcdb96c19158e8bc7, 0x39582cc6bdb1a4a88e89c050ad6db0ade34f45ec5791b07de6e694e9627ca66a), (0x1ee805e20232ba31eeae1fa345bd88ac7df81dc43ffb3967a218ea4defc9d17d, 0x2838c83c064d44e87e5c8d05a234ad24d2d4a502a370acb514b430f516c0f0bf), (0x37ead9904c760201ec4734ef398f0bdb5fe5a5e6e9db19c85e6b5483bdeb0a0b, 0x1dc08c38ed713b14f7a21a891a83b52160a3ffb0dccfbd70db7c7eb235dd193e), (0x2dc3d20553691216c988ecbb596c4bda329f27d50bd8a7c2fb0d84b423da3cb4, 0x025b40e800020458e15e3a57268562e6c08c6971d71262bd67c72437cfc60b4c)], permutations = [VerifyingKey { commitments: [(0x289f468bca3471a3d240169ec65047d0c4bb5e1135e81822523c74b596139fed, 0x1a585c821c71fb49c883859f1389bcae45f17593ddb5f9fee1781b27129e1b06), (0x096ef96a7725c636e7ca645dfe539694cf2a988da1ca50a468320f419d008054, 0x1ac0b48a254f8e2311081f81aa4a8ff39e37e40aa4c003325f1ac1219d231818), (0x254c9f0088599aba37607cfd1700c653ef3ec21bfd60b98c3d725678540bc4df, 0x134d9818929589052f3cd80d50140b851db26231b2b784a6b2528a64805598dc)] }, VerifyingKey { commitments: [(0x289f468bca3471a3d240169ec65047d0c4bb5e1135e81822523c74b596139fed, 0x1a585c821c71fb49c883859f1389bcae45f17593ddb5f9fee1781b27129e1b06), (0x096ef96a7725c636e7ca645dfe539694cf2a988da1ca50a468320f419d008054, 0x1ac0b48a254f8e2311081f81aa4a8ff39e37e40aa4c003325f1ac1219d231818), (0x254c9f0088599aba37607cfd1700c653ef3ec21bfd60b98c3d725678540bc4df, 0x134d9818929589052f3cd80d50140b851db26231b2b784a6b2528a64805598dc)] }]}");
    let pk = keygen_pk(&params, vk, &empty_circuit).expect("keygen_pk should not fail");

    let mut pubinputs = pk.get_vk().get_domain().empty_lagrange();
    pubinputs[0] = instance;
    let pubinput = params
        .commit_lagrange(&pubinputs, Blind::default())
        .to_affine();

    // Check this circuit is satisfied.
    let prover = match MockProver::run(K, &circuit, vec![pubinputs.to_vec()]) {
        Ok(prover) => prover,
        Err(e) => panic!("{:?}", e),
    };
    assert_eq!(prover.verify(), Ok(()));

    for _ in 0..100 {
        let mut transcript = Blake2bWrite::init(vec![]);
        // Create a proof
        create_proof(
            &params,
            &pk,
            &[circuit.clone(), circuit.clone()],
            &[&[pubinputs.clone()], &[pubinputs.clone()]],
            &mut transcript,
        )
        .expect("proof generation should not fail");
        let proof: Vec<u8> = transcript.finalize();

        let pubinput_slice = &[pubinput];
        let pubinput_slice_copy = &[pubinput];
        let msm = params.empty_msm();
        let mut transcript = Blake2bRead::init(&proof[..]);
        let guard = verify_proof(
            &params,
            pk.get_vk(),
            msm,
            &[pubinput_slice, pubinput_slice_copy],
            &mut transcript,
        )
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
        let mut transcript = Blake2bRead::init(&proof[..]);
        let mut vk_buffer = vec![];
        pk.get_vk().write(&mut vk_buffer).unwrap();
        let vk = VerifyingKey::<EqAffine>::read::<_, MyCircuit<Fp>>(&mut &vk_buffer[..], &params)
            .unwrap();
        let guard = verify_proof(
            &params,
            &vk,
            msm,
            &[pubinput_slice, pubinput_slice_copy],
            &mut transcript,
        )
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
