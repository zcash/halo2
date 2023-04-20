#![allow(clippy::many_single_char_names)]
#![allow(clippy::op_ref)]

use assert_matches::assert_matches;
use ff::{FromUniformBytes, WithSmallOrderMulGroup};
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{Cell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::plonk::{
    create_proof as create_plonk_proof, keygen_pk, keygen_vk, verify_proof as verify_plonk_proof,
    Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed, ProvingKey, TableColumn,
    VerifyingKey,
};
use halo2_proofs::poly::commitment::{CommitmentScheme, ParamsProver, Prover, Verifier};
use halo2_proofs::poly::Rotation;
use halo2_proofs::poly::VerificationStrategy;
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, TranscriptReadBuffer,
    TranscriptWriterBuffer,
};
use rand_core::{OsRng, RngCore};
use std::marker::PhantomData;

#[test]
fn plonk_api() {
    const K: u32 = 5;

    /// This represents an advice column at a certain row in the ConstraintSystem
    #[derive(Copy, Clone, Debug)]
    pub struct Variable(Column<Advice>, usize);

    #[derive(Clone)]
    struct PlonkConfig {
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
        sl: TableColumn,
    }

    #[allow(clippy::type_complexity)]
    trait StandardCs<FF: Field> {
        fn raw_multiply<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Value<(Assigned<FF>, Assigned<FF>, Assigned<FF>)>;
        fn raw_add<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Value<(Assigned<FF>, Assigned<FF>, Assigned<FF>)>;
        fn copy(&self, layouter: &mut impl Layouter<FF>, a: Cell, b: Cell) -> Result<(), Error>;
        fn public_input<F>(&self, layouter: &mut impl Layouter<FF>, f: F) -> Result<Cell, Error>
        where
            F: FnMut() -> Value<FF>;
        fn lookup_table(
            &self,
            layouter: &mut impl Layouter<FF>,
            values: &[FF],
        ) -> Result<(), Error>;
    }

    #[derive(Clone)]
    struct MyCircuit<F: Field> {
        a: Value<F>,
        lookup_table: Vec<F>,
    }

    struct StandardPlonk<F: Field> {
        config: PlonkConfig,
        _marker: PhantomData<F>,
    }

    impl<FF: Field> StandardPlonk<FF> {
        fn new(config: PlonkConfig) -> Self {
            StandardPlonk {
                config,
                _marker: PhantomData,
            }
        }
    }

    impl<FF: Field> StandardCs<FF> for StandardPlonk<FF> {
        fn raw_multiply<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            mut f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Value<(Assigned<FF>, Assigned<FF>, Assigned<FF>)>,
        {
            layouter.assign_region(
                || "raw_multiply",
                |mut region| {
                    let mut value = None;
                    let lhs = region.assign_advice(
                        || "lhs",
                        self.config.a,
                        0,
                        || {
                            value = Some(f());
                            value.unwrap().map(|v| v.0)
                        },
                    )?;
                    region.assign_advice(
                        || "lhs^4",
                        self.config.d,
                        0,
                        || value.unwrap().map(|v| v.0).square().square(),
                    )?;
                    let rhs = region.assign_advice(
                        || "rhs",
                        self.config.b,
                        0,
                        || value.unwrap().map(|v| v.1),
                    )?;
                    region.assign_advice(
                        || "rhs^4",
                        self.config.e,
                        0,
                        || value.unwrap().map(|v| v.1).square().square(),
                    )?;
                    let out = region.assign_advice(
                        || "out",
                        self.config.c,
                        0,
                        || value.unwrap().map(|v| v.2),
                    )?;

                    region.assign_fixed(|| "a", self.config.sa, 0, || Value::known(FF::ZERO))?;
                    region.assign_fixed(|| "b", self.config.sb, 0, || Value::known(FF::ZERO))?;
                    region.assign_fixed(|| "c", self.config.sc, 0, || Value::known(FF::ONE))?;
                    region.assign_fixed(|| "a * b", self.config.sm, 0, || Value::known(FF::ONE))?;
                    Ok((lhs.cell(), rhs.cell(), out.cell()))
                },
            )
        }
        fn raw_add<F>(
            &self,
            layouter: &mut impl Layouter<FF>,
            mut f: F,
        ) -> Result<(Cell, Cell, Cell), Error>
        where
            F: FnMut() -> Value<(Assigned<FF>, Assigned<FF>, Assigned<FF>)>,
        {
            layouter.assign_region(
                || "raw_add",
                |mut region| {
                    let mut value = None;
                    let lhs = region.assign_advice(
                        || "lhs",
                        self.config.a,
                        0,
                        || {
                            value = Some(f());
                            value.unwrap().map(|v| v.0)
                        },
                    )?;
                    region.assign_advice(
                        || "lhs^4",
                        self.config.d,
                        0,
                        || value.unwrap().map(|v| v.0).square().square(),
                    )?;
                    let rhs = region.assign_advice(
                        || "rhs",
                        self.config.b,
                        0,
                        || value.unwrap().map(|v| v.1),
                    )?;
                    region.assign_advice(
                        || "rhs^4",
                        self.config.e,
                        0,
                        || value.unwrap().map(|v| v.1).square().square(),
                    )?;
                    let out = region.assign_advice(
                        || "out",
                        self.config.c,
                        0,
                        || value.unwrap().map(|v| v.2),
                    )?;

                    region.assign_fixed(|| "a", self.config.sa, 0, || Value::known(FF::ONE))?;
                    region.assign_fixed(|| "b", self.config.sb, 0, || Value::known(FF::ONE))?;
                    region.assign_fixed(|| "c", self.config.sc, 0, || Value::known(FF::ONE))?;
                    region.assign_fixed(
                        || "a * b",
                        self.config.sm,
                        0,
                        || Value::known(FF::ZERO),
                    )?;
                    Ok((lhs.cell(), rhs.cell(), out.cell()))
                },
            )
        }
        fn copy(
            &self,
            layouter: &mut impl Layouter<FF>,
            left: Cell,
            right: Cell,
        ) -> Result<(), Error> {
            layouter.assign_region(
                || "copy",
                |mut region| {
                    region.constrain_equal(left, right)?;
                    region.constrain_equal(left, right)
                },
            )
        }
        fn public_input<F>(&self, layouter: &mut impl Layouter<FF>, mut f: F) -> Result<Cell, Error>
        where
            F: FnMut() -> Value<FF>,
        {
            layouter.assign_region(
                || "public_input",
                |mut region| {
                    let value = region.assign_advice(|| "value", self.config.a, 0, &mut f)?;
                    region.assign_fixed(
                        || "public",
                        self.config.sp,
                        0,
                        || Value::known(FF::ONE),
                    )?;

                    Ok(value.cell())
                },
            )
        }
        fn lookup_table(
            &self,
            layouter: &mut impl Layouter<FF>,
            values: &[FF],
        ) -> Result<(), Error> {
            layouter.assign_table(
                || "",
                |mut table| {
                    for (index, &value) in values.iter().enumerate() {
                        table.assign_cell(
                            || "table col",
                            self.config.sl,
                            index,
                            || Value::known(value),
                        )?;
                    }
                    Ok(())
                },
            )?;
            Ok(())
        }
    }

    impl<F: Field> Circuit<F> for MyCircuit<F> {
        type Config = PlonkConfig;
        type FloorPlanner = SimpleFloorPlanner;
        #[cfg(feature = "circuit-params")]
        type Params = ();

        fn without_witnesses(&self) -> Self {
            Self {
                a: Value::unknown(),
                lookup_table: self.lookup_table.clone(),
            }
        }

        fn configure(meta: &mut ConstraintSystem<F>) -> PlonkConfig {
            let e = meta.advice_column();
            let a = meta.advice_column();
            let b = meta.advice_column();
            let sf = meta.fixed_column();
            let c = meta.advice_column();
            let d = meta.advice_column();
            let p = meta.instance_column();

            meta.enable_equality(a);
            meta.enable_equality(b);
            meta.enable_equality(c);

            let sm = meta.fixed_column();
            let sa = meta.fixed_column();
            let sb = meta.fixed_column();
            let sc = meta.fixed_column();
            let sp = meta.fixed_column();
            let sl = meta.lookup_table_column();

            /*
             *   A         B      ...  sl
             * [
             *   instance  0      ...  0
             *   a         a      ...  0
             *   a         a^2    ...  0
             *   a         a      ...  0
             *   a         a^2    ...  0
             *   ...       ...    ...  ...
             *   ...       ...    ...  instance
             *   ...       ...    ...  a
             *   ...       ...    ...  a
             *   ...       ...    ...  0
             * ]
             */

            meta.lookup("lookup", |meta| {
                let a_ = meta.query_any(a, Rotation::cur());
                vec![(a_, sl)]
            });

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

                vec![a.clone() * sa + b.clone() * sb + a * b * sm - (c * sc) + sf * (d * e)]
            });

            meta.create_gate("Public input", |meta| {
                let a = meta.query_advice(a, Rotation::cur());
                let p = meta.query_instance(p, Rotation::cur());
                let sp = meta.query_fixed(sp, Rotation::cur());

                vec![sp * (a - p)]
            });

            meta.enable_equality(sf);
            meta.enable_equality(e);
            meta.enable_equality(d);
            meta.enable_equality(p);
            meta.enable_equality(sm);
            meta.enable_equality(sa);
            meta.enable_equality(sb);
            meta.enable_equality(sc);
            meta.enable_equality(sp);

            PlonkConfig {
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
            }
        }

        fn synthesize(
            &self,
            config: PlonkConfig,
            mut layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            let cs = StandardPlonk::new(config);

            let _ = cs.public_input(&mut layouter, || Value::known(F::ONE + F::ONE))?;

            for _ in 0..10 {
                let a: Value<Assigned<_>> = self.a.into();
                let mut a_squared = Value::unknown();
                let (a0, _, c0) = cs.raw_multiply(&mut layouter, || {
                    a_squared = a.square();
                    a.zip(a_squared).map(|(a, a_squared)| (a, a, a_squared))
                })?;
                let (a1, b1, _) = cs.raw_add(&mut layouter, || {
                    let fin = a_squared + a;
                    a.zip(a_squared)
                        .zip(fin)
                        .map(|((a, a_squared), fin)| (a, a_squared, fin))
                })?;
                cs.copy(&mut layouter, a0, a1)?;
                cs.copy(&mut layouter, b1, c0)?;
            }

            cs.lookup_table(&mut layouter, &self.lookup_table)?;

            Ok(())
        }
    }

    macro_rules! common {
        ($scheme:ident) => {{
            let a = <$scheme as CommitmentScheme>::Scalar::from(2834758237)
                * <$scheme as CommitmentScheme>::Scalar::ZETA;
            let instance = <$scheme as CommitmentScheme>::Scalar::ONE
                + <$scheme as CommitmentScheme>::Scalar::ONE;
            let lookup_table = vec![instance, a, a, <$scheme as CommitmentScheme>::Scalar::ZERO];
            (a, instance, lookup_table)
        }};
    }

    macro_rules! bad_keys {
        ($scheme:ident) => {{
            let (_, _, lookup_table) = common!($scheme);
            let empty_circuit: MyCircuit<<$scheme as CommitmentScheme>::Scalar> = MyCircuit {
                a: Value::unknown(),
                lookup_table: lookup_table.clone(),
            };

            // Check that we get an error if we try to initialize the proving key with a value of
            // k that is too small for the minimum required number of rows.
            let much_too_small_params= <$scheme as CommitmentScheme>::ParamsProver::new(1);
            assert_matches!(
                keygen_vk(&much_too_small_params, &empty_circuit),
                Err(Error::NotEnoughRowsAvailable {
                    current_k,
                }) if current_k == 1
            );

            // Check that we get an error if we try to initialize the proving key with a value of
            // k that is too small for the number of rows the circuit uses.
            let slightly_too_small_params = <$scheme as CommitmentScheme>::ParamsProver::new(K-1);
            assert_matches!(
                keygen_vk(&slightly_too_small_params, &empty_circuit),
                Err(Error::NotEnoughRowsAvailable {
                    current_k,
                }) if current_k == K - 1
            );
        }};
    }

    fn keygen<Scheme: CommitmentScheme>(params: &Scheme::ParamsProver) -> ProvingKey<Scheme::Curve>
    where
        Scheme::Scalar: FromUniformBytes<64> + WithSmallOrderMulGroup<3>,
    {
        let (_, _, lookup_table) = common!(Scheme);
        let empty_circuit: MyCircuit<Scheme::Scalar> = MyCircuit {
            a: Value::unknown(),
            lookup_table,
        };

        // Initialize the proving key
        let vk = keygen_vk(params, &empty_circuit).expect("keygen_vk should not fail");

        keygen_pk(params, vk, &empty_circuit).expect("keygen_pk should not fail")
    }

    fn create_proof<
        'params,
        Scheme: CommitmentScheme,
        P: Prover<'params, Scheme>,
        E: EncodedChallenge<Scheme::Curve>,
        R: RngCore,
        T: TranscriptWriterBuffer<Vec<u8>, Scheme::Curve, E>,
    >(
        rng: R,
        params: &'params Scheme::ParamsProver,
        pk: &ProvingKey<Scheme::Curve>,
    ) -> Vec<u8>
    where
        Scheme::Scalar: Ord + WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        let (a, instance, lookup_table) = common!(Scheme);

        let circuit: MyCircuit<Scheme::Scalar> = MyCircuit {
            a: Value::known(a),
            lookup_table,
        };

        let mut transcript = T::init(vec![]);

        create_plonk_proof::<Scheme, P, _, _, _, _>(
            params,
            pk,
            &[circuit.clone(), circuit.clone()],
            &[&[&[instance]], &[&[instance]]],
            rng,
            &mut transcript,
        )
        .expect("proof generation should not fail");

        // Check this circuit is satisfied.
        let prover = match MockProver::run(K, &circuit, vec![vec![instance]]) {
            Ok(prover) => prover,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(prover.verify(), Ok(()));

        transcript.finalize()
    }

    fn verify_proof<
        'a,
        'params,
        Scheme: CommitmentScheme,
        V: Verifier<'params, Scheme>,
        E: EncodedChallenge<Scheme::Curve>,
        T: TranscriptReadBuffer<&'a [u8], Scheme::Curve, E>,
        Strategy: VerificationStrategy<'params, Scheme, V, Output = Strategy>,
    >(
        params_verifier: &'params Scheme::ParamsVerifier,
        vk: &VerifyingKey<Scheme::Curve>,
        proof: &'a [u8],
    ) where
        Scheme::Scalar: Ord + WithSmallOrderMulGroup<3> + FromUniformBytes<64>,
    {
        let (_, instance, _) = common!(Scheme);
        let pubinputs = vec![instance];

        let mut transcript = T::init(proof);

        let strategy = Strategy::new(params_verifier);
        let strategy = verify_plonk_proof(
            params_verifier,
            vk,
            strategy,
            &[&[&pubinputs[..]], &[&pubinputs[..]]],
            &mut transcript,
        )
        .unwrap();

        assert!(strategy.finalize());
    }

    fn test_plonk_api_gwc() {
        use halo2_proofs::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
        use halo2_proofs::poly::kzg::multiopen::{ProverGWC, VerifierGWC};
        use halo2_proofs::poly::kzg::strategy::AccumulatorStrategy;
        use halo2curves::bn256::Bn256;

        type Scheme = KZGCommitmentScheme<Bn256>;
        bad_keys!(Scheme);

        let params = ParamsKZG::<Bn256>::new(K);
        let rng = OsRng;

        let pk = keygen::<KZGCommitmentScheme<_>>(&params);

        let proof = create_proof::<_, ProverGWC<_>, _, _, Blake2bWrite<_, _, Challenge255<_>>>(
            rng, &params, &pk,
        );

        let verifier_params = params.verifier_params();

        verify_proof::<
            _,
            VerifierGWC<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, pk.get_vk(), &proof[..]);
    }

    fn test_plonk_api_shplonk() {
        use halo2_proofs::poly::kzg::commitment::{KZGCommitmentScheme, ParamsKZG};
        use halo2_proofs::poly::kzg::multiopen::{ProverSHPLONK, VerifierSHPLONK};
        use halo2_proofs::poly::kzg::strategy::AccumulatorStrategy;
        use halo2curves::bn256::Bn256;

        type Scheme = KZGCommitmentScheme<Bn256>;
        bad_keys!(Scheme);

        let params = ParamsKZG::<Bn256>::new(K);
        let rng = OsRng;

        let pk = keygen::<KZGCommitmentScheme<_>>(&params);

        let proof = create_proof::<_, ProverSHPLONK<_>, _, _, Blake2bWrite<_, _, Challenge255<_>>>(
            rng, &params, &pk,
        );

        let verifier_params = params.verifier_params();

        verify_proof::<
            _,
            VerifierSHPLONK<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, pk.get_vk(), &proof[..]);
    }

    fn test_plonk_api_ipa() {
        use halo2_proofs::poly::ipa::commitment::{IPACommitmentScheme, ParamsIPA};
        use halo2_proofs::poly::ipa::multiopen::{ProverIPA, VerifierIPA};
        use halo2_proofs::poly::ipa::strategy::AccumulatorStrategy;
        use halo2curves::pasta::EqAffine;

        type Scheme = IPACommitmentScheme<EqAffine>;
        bad_keys!(Scheme);

        let params = ParamsIPA::<EqAffine>::new(K);
        let rng = OsRng;

        let pk = keygen::<IPACommitmentScheme<EqAffine>>(&params);

        let proof = create_proof::<_, ProverIPA<_>, _, _, Blake2bWrite<_, _, Challenge255<_>>>(
            rng, &params, &pk,
        );

        let verifier_params = params.verifier_params();

        verify_proof::<
            _,
            VerifierIPA<_>,
            _,
            Blake2bRead<_, _, Challenge255<_>>,
            AccumulatorStrategy<_>,
        >(verifier_params, pk.get_vk(), &proof[..]);

        // Check that the verification key has not changed unexpectedly
        {
            //panic!("{:#?}", pk.get_vk().pinned());
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
        num_fixed_columns: 7,
        num_advice_columns: 5,
        num_instance_columns: 1,
        num_selectors: 0,
        gates: [
            Sum(
                Sum(
                    Sum(
                        Sum(
                            Product(
                                Advice {
                                    query_index: 0,
                                    column_index: 1,
                                    rotation: Rotation(
                                        0,
                                    ),
                                },
                                Fixed {
                                    query_index: 2,
                                    column_index: 2,
                                    rotation: Rotation(
                                        0,
                                    ),
                                },
                            ),
                            Product(
                                Advice {
                                    query_index: 1,
                                    column_index: 2,
                                    rotation: Rotation(
                                        0,
                                    ),
                                },
                                Fixed {
                                    query_index: 3,
                                    column_index: 3,
                                    rotation: Rotation(
                                        0,
                                    ),
                                },
                            ),
                        ),
                        Product(
                            Product(
                                Advice {
                                    query_index: 0,
                                    column_index: 1,
                                    rotation: Rotation(
                                        0,
                                    ),
                                },
                                Advice {
                                    query_index: 1,
                                    column_index: 2,
                                    rotation: Rotation(
                                        0,
                                    ),
                                },
                            ),
                            Fixed {
                                query_index: 5,
                                column_index: 1,
                                rotation: Rotation(
                                    0,
                                ),
                            },
                        ),
                    ),
                    Negated(
                        Product(
                            Advice {
                                query_index: 2,
                                column_index: 3,
                                rotation: Rotation(
                                    0,
                                ),
                            },
                            Fixed {
                                query_index: 4,
                                column_index: 4,
                                rotation: Rotation(
                                    0,
                                ),
                            },
                        ),
                    ),
                ),
                Product(
                    Fixed {
                        query_index: 1,
                        column_index: 0,
                        rotation: Rotation(
                            0,
                        ),
                    },
                    Product(
                        Advice {
                            query_index: 3,
                            column_index: 4,
                            rotation: Rotation(
                                1,
                            ),
                        },
                        Advice {
                            query_index: 4,
                            column_index: 0,
                            rotation: Rotation(
                                -1,
                            ),
                        },
                    ),
                ),
            ),
            Product(
                Fixed {
                    query_index: 6,
                    column_index: 5,
                    rotation: Rotation(
                        0,
                    ),
                },
                Sum(
                    Advice {
                        query_index: 0,
                        column_index: 1,
                        rotation: Rotation(
                            0,
                        ),
                    },
                    Negated(
                        Instance {
                            query_index: 0,
                            column_index: 0,
                            rotation: Rotation(
                                0,
                            ),
                        },
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
            (
                Column {
                    index: 0,
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
                    0,
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
        permutation: Argument {
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
                Column {
                    index: 0,
                    column_type: Fixed,
                },
                Column {
                    index: 0,
                    column_type: Advice,
                },
                Column {
                    index: 4,
                    column_type: Advice,
                },
                Column {
                    index: 0,
                    column_type: Instance,
                },
                Column {
                    index: 1,
                    column_type: Fixed,
                },
                Column {
                    index: 2,
                    column_type: Fixed,
                },
                Column {
                    index: 3,
                    column_type: Fixed,
                },
                Column {
                    index: 4,
                    column_type: Fixed,
                },
                Column {
                    index: 5,
                    column_type: Fixed,
                },
            ],
        },
        lookups: [
            Argument {
                input_expressions: [
                    Advice {
                        query_index: 0,
                        column_index: 1,
                        rotation: Rotation(
                            0,
                        ),
                    },
                ],
                table_expressions: [
                    Fixed {
                        query_index: 0,
                        column_index: 6,
                        rotation: Rotation(
                            0,
                        ),
                    },
                ],
            },
        ],
        constants: [],
        minimum_degree: None,
    },
    fixed_commitments: [
        (0x2bbc94ef7b22aebef24f9a4b0cc1831882548b605171366017d45c3e6fd92075, 0x082b801a6e176239943bfb759fb02138f47a5c8cc4aa7fa0af559fde4e3abd97),
        (0x2bf5082b105b2156ed0e9c5b8e42bf2a240b058f74a464d080e9585274dd1e84, 0x222ad83cee7777e7a160585e212140e5e770dd8d1df788d869b5ee483a5864fb),
        (0x374a656456a0aae7429b23336f825752b575dd5a44290ff614946ee59d6a20c0, 0x054491e187e6e3460e7601fb54ae10836d34d420026f96316f0c5c62f86db9b8),
        (0x374a656456a0aae7429b23336f825752b575dd5a44290ff614946ee59d6a20c0, 0x054491e187e6e3460e7601fb54ae10836d34d420026f96316f0c5c62f86db9b8),
        (0x02e62cd68370b13711139a08cbcdd889e800a272b9ea10acc90880fff9d89199, 0x1a96c468cb0ce77065d3a58f1e55fea9b72d15e44c01bba1e110bd0cbc6e9bc6),
        (0x224ef42758215157d3ee48fb8d769da5bddd35e5929a90a4a89736f5c4b5ae9b, 0x11bc3a1e08eb320cde764f1492ecef956d71e996e2165f7a9a30ad2febb511c1),
        (0x2d5415bf917fcac32bfb705f8ca35cb12d9bad52aa33ccca747350f9235d3a18, 0x2b2921f815fad504052512743963ef20ed5b401d20627793b006413e73fe4dd4),
    ],
    permutation: VerifyingKey {
        commitments: [
            (0x1347b4b385837977a96b87f199c6a9a81520015539d1e8fa79429bb4ca229a00, 0x2168e404cabef513654d6ff516cde73f0ba87e3dc84e4b940ed675b5f66f3884),
            (0x0e6d69cd2455ec43be640f6397ed65c9e51b1d8c0fd2216339314ff37ade122a, 0x222ed6dc8cfc9ea26dcc10b9d4add791ada60f2b5a63ee1e4635f88aa0c96654),
            (0x13c447846f48c41a5e0675ccf88ebc0cdef2c96c51446d037acb866d24255785, 0x1f0b5414fc5e8219dbfab996eed6129d831488b2386a8b1a63663938903bd63a),
            (0x1aae6470aa662b8fda003894ddef5fedd03af318b3231683039d2fac9cab05b9, 0x08832d91ae69e99cd07d096c7a4a284a69e6a16227cbb07932a0cdc56914f3a6),
            (0x0850521b0f8ac7dd0550fe3e25c840837076e9635067ed623b81d5cbac5944d9, 0x0c25d65d1038d0a92c72e5fccd96c1caf07801c3c8233290bb292e0c38c256fa),
            (0x12febcf696badd970750eabf75dd3ced4c2f54f93519bcee23849025177d2014, 0x0a05ab3cd42c9fbcc1bbfcf9269951640cc9920761c87cf8e211ba73c8d9f90f),
            (0x053904bdde8cfead3b517bb4f6ded3e699f8b94ca6156a9dd2f92a2a05a7ec5a, 0x16753ff97c0d82ff586bb7a07bf7f27a92df90b3617fa5e75d4f55c3b0ef8711),
            (0x3804548f6816452747a5b542fa5656353fc989db40d69e9e27d6f973b5deebb0, 0x389a44d5037866dd83993af75831a5f90a18ad5244255aa5bd2c922cc5853055),
            (0x003a9f9ca71c7c0b832c802220915f6fc8d840162bdde6b0ea05d25fb95559e3, 0x091247ca19d6b73887cd7f68908cbf0db0b47459b7c82276bbdb8a1c937e2438),
            (0x3eaa38689d9e391c8a8fafab9568f20c45816321d38f309d4cc37f4b1601af72, 0x247f8270a462ea88450221a56aa6b55d2bc352b80b03501e99ea983251ceea13),
            (0x394437571f9de32dccdc546fd4737772d8d92593c85438aa3473243997d5acc8, 0x14924ec6e3174f1fab7f0ce7070c22f04bbd0a0ecebdfc5c94be857f25493e95),
            (0x3d907e0591343bd285c2c846f3e871a6ac70d80ec29e9500b8cb57f544e60202, 0x1034e48df35830244cabea076be8a16d67d7896e27c6ac22b285d017105da9c3),
        ],
    },
}"#####
            );
        }
    }

    test_plonk_api_ipa();
    test_plonk_api_gwc();
    test_plonk_api_shplonk();
}
