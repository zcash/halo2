#![allow(clippy::many_single_char_names)]
#![allow(clippy::op_ref)]

use group::Curve;
use halo2::arithmetic::FieldExt;
use halo2::dev::MockProver;
use halo2::pasta::{EqAffine, Fp};
use halo2::plonk::{
    create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Assignment, Circuit, Column,
    ConstraintSystem, Error, Fixed, Permutation, VerifyingKey,
};
use halo2::poly::{
    commitment::{Blind, Params},
    Rotation,
};
use halo2::transcript::{Blake2bRead, Blake2bWrite};
use std::marker::PhantomData;

#[test]
fn plonk_api() {
    const K: u32 = 5;

    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    // Initialize the polynomial commitment parameters
    let params: Params<EqAffine> = Params::new(K);

    #[derive(Clone)]
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

        perm: Permutation,
        perm2: Permutation,
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
            self.cs.copy(
                &self.config.perm,
                left.0.into(),
                left.1,
                right.0.into(),
                right.1,
            )?;
            self.cs.copy(
                &self.config.perm2,
                left.0.into(),
                left.1,
                right.0.into(),
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
            let a_ = meta.query_any(a.into(), Rotation::cur());
            let b_ = meta.query_any(b.into(), Rotation::cur());
            let sl_ = meta.query_any(sl.into(), Rotation::cur());
            let sl2_ = meta.query_any(sl2.into(), Rotation::cur());
            meta.lookup(&[a_.clone()], &[sl_.clone()]);
            meta.lookup(&[a_ * b_], &[sl_ * sl2_]);

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

    for _ in 0..10 {
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

    // Check that the verification key has not changed unexpectedly
    {
        assert_eq!(
            format!("{:#?}", pk.get_vk().pinned()),
            r#####"PinnedVerificationKey {
    base_modulus: "0x40000000000000000000000000000000224698fc0994a8dd8c46eb2100000001",
    scalar_modulus: "0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001",
    domain: PinnedEvaluationDomain {
        k: 5,
        extended_k: 7,
        omega: 0x0cc3380dc616f2e1daf29ad1560833ed3baea3393eceb7bc8fa36376929b78cc,
    },
    cs: PinnedConstraintSystem {
        num_fixed_columns: 8,
        num_advice_columns: 5,
        num_instance_columns: 1,
        gates: [
            Sum(
                Sum(
                    Sum(
                        Sum(
                            Product(
                                Advice(
                                    0,
                                ),
                                Fixed(
                                    3,
                                ),
                            ),
                            Product(
                                Advice(
                                    1,
                                ),
                                Fixed(
                                    4,
                                ),
                            ),
                        ),
                        Product(
                            Product(
                                Advice(
                                    0,
                                ),
                                Advice(
                                    1,
                                ),
                            ),
                            Fixed(
                                6,
                            ),
                        ),
                    ),
                    Scaled(
                        Product(
                            Advice(
                                2,
                            ),
                            Fixed(
                                5,
                            ),
                        ),
                        0x40000000000000000000000000000000224698fc094cf91b992d30ed00000000,
                    ),
                ),
                Product(
                    Fixed(
                        2,
                    ),
                    Product(
                        Advice(
                            3,
                        ),
                        Advice(
                            4,
                        ),
                    ),
                ),
            ),
            Product(
                Fixed(
                    7,
                ),
                Sum(
                    Advice(
                        0,
                    ),
                    Scaled(
                        Instance(
                            0,
                        ),
                        0x40000000000000000000000000000000224698fc094cf91b992d30ed00000000,
                    ),
                ),
            ),
        ],
        advice_queries: [
            (
                Column {
                    index: 1,
                    column_type: Advice,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 2,
                    column_type: Advice,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 3,
                    column_type: Advice,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 4,
                    column_type: Advice,
                },
                Rotation(
                    1,
                ),
            ),
            (
                Column {
                    index: 0,
                    column_type: Advice,
                },
                Rotation(
                    -1,
                ),
            ),
        ],
        instance_queries: [
            (
                Column {
                    index: 0,
                    column_type: Instance,
                },
                Rotation(
                    0,
                ),
            ),
        ],
        fixed_queries: [
            (
                Column {
                    index: 6,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 7,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 0,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 2,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 3,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 4,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 1,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
            (
                Column {
                    index: 5,
                    column_type: Fixed,
                },
                Rotation(
                    0,
                ),
            ),
        ],
        permutations: [
            Argument {
                columns: [
                    Column {
                        index: 1,
                        column_type: Advice,
                    },
                    Column {
                        index: 2,
                        column_type: Advice,
                    },
                    Column {
                        index: 3,
                        column_type: Advice,
                    },
                ],
            },
            Argument {
                columns: [
                    Column {
                        index: 1,
                        column_type: Advice,
                    },
                    Column {
                        index: 2,
                        column_type: Advice,
                    },
                    Column {
                        index: 3,
                        column_type: Advice,
                    },
                ],
            },
        ],
        lookups: [
            Argument {
                input_expressions: [
                    Advice(
                        0,
                    ),
                ],
                table_expressions: [
                    Fixed(
                        0,
                    ),
                ],
            },
            Argument {
                input_expressions: [
                    Product(
                        Advice(
                            0,
                        ),
                        Advice(
                            1,
                        ),
                    ),
                ],
                table_expressions: [
                    Product(
                        Fixed(
                            0,
                        ),
                        Fixed(
                            1,
                        ),
                    ),
                ],
            },
        ],
    },
    fixed_commitments: [
        (0x046711bb0579a337420e33de9d54438e7c3a9cc47b6728b873d1fd0214d7eb58, 0x2416b30fadfacd828cf76891a2a5f0fe90d7ae0e5a8df947e98660ffbebf72e4),
        (0x241db4dcb35d3977d45a57a9c5053e8f2c2310fa98738feb48430254034e42bc, 0x3e9545f6b9aa955ce50450eb1b37fb69d5891bca9b5193e6e8288675abded312),
        (0x15a0f4deb421ccdfb7cebd60fe7055d406e8f24e9bf37d304327b2adb53e2f7a, 0x1811c4a5f95dc72b15e780bb76d5d0e91dc315c0726a361712bdcb7afd11dc6c),
        (0x15a0f4deb421ccdfb7cebd60fe7055d406e8f24e9bf37d304327b2adb53e2f7a, 0x1811c4a5f95dc72b15e780bb76d5d0e91dc315c0726a361712bdcb7afd11dc6c),
        (0x2c1e1e702ea5a876188a2e2d1f7fcbee31e5fba48ccd1d7d8dc000393da5b6cb, 0x302338ba3f31351e080311442a59fc9fd9cc30700ce33f4775741d6888df63ea),
        (0x3e6b7c66782b06e0e7cd5bd7930b0204dee22b44a25d7c405909d4ca4eb604a7, 0x19b69444de257eb1dd99020a8c615fdc6bed7308ea63b1d4b3c0430f15e71568),
        (0x05dfc2fbe7800a57610e7b61e4cd7e96f96026cc192a92750e50e9c35c2d262d, 0x3b2c6101d9a2bbf8982f84e2bd818952ea1d53c5a815c7a4d900cc27f67da390),
        (0x318668190ba5ac1d3a1f93b13dd611e4dd3d68b1ea2ae1fe15b99bfc0858cc94, 0x18edacbf7ad8d4b3e43d9cab81c696cb3671ac3a9007610a5c949d85f9790841),
    ],
    permutations: [
        VerifyingKey {
            commitments: [
                (0x02d8dce08483e705f124b2e3db76a8065bfd8d893a1de76fd4ba586acb8e2cd0, 0x1456b7e28d96b5f90f885d21fde2ed00d1774cdebc358a95383b95302a87e09d),
                (0x1d8a9751a63cbdf4c87787424b9c4a347483d5138943470dd1a73e1d1fd336b1, 0x2b1f6a54bff445799b6abf5bb0ed734d1cabdb46b4966556e753097ed87cef1b),
                (0x1592b59a2a90b155420abde2bcf6fb822d80a11e1b44306dc07fc45025f214e5, 0x3802666ef284d7db51cbd2f9be20014e19f0f6a22e1a4d3a0db124b7bdd7fa1b),
            ],
        },
        VerifyingKey {
            commitments: [
                (0x02d8dce08483e705f124b2e3db76a8065bfd8d893a1de76fd4ba586acb8e2cd0, 0x1456b7e28d96b5f90f885d21fde2ed00d1774cdebc358a95383b95302a87e09d),
                (0x1d8a9751a63cbdf4c87787424b9c4a347483d5138943470dd1a73e1d1fd336b1, 0x2b1f6a54bff445799b6abf5bb0ed734d1cabdb46b4966556e753097ed87cef1b),
                (0x1592b59a2a90b155420abde2bcf6fb822d80a11e1b44306dc07fc45025f214e5, 0x3802666ef284d7db51cbd2f9be20014e19f0f6a22e1a4d3a0db124b7bdd7fa1b),
            ],
        },
    ],
}"#####
        );
    }
}
