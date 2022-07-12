use ff::BatchInvert;
use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{floor_planner::V1, Layouter, Value},
    dev::{metadata, FailureLocation, MockProver, VerifyFailure},
    plonk::{Phase::*, *},
    poly::{commitment::Params, Rotation},
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use pasta_curves::EqAffine;
use rand_core::{OsRng, RngCore};
use std::{
    iter,
    panic::{self, AssertUnwindSafe},
};

fn catch_unwind_silent<F: FnOnce() -> R, R>(f: F) -> std::thread::Result<R> {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(AssertUnwindSafe(f));
    panic::set_hook(prev_hook);
    result
}

fn rand_2d_array<F: FieldExt, R: RngCore, const W: usize, const H: usize>(
    rng: &mut R,
) -> [[F; H]; W] {
    [(); W].map(|_| [(); H].map(|_| F::random(&mut *rng)))
}

fn shuffled<F: FieldExt, R: RngCore, const W: usize, const H: usize>(
    original: [[F; H]; W],
    rng: &mut R,
) -> [[F; H]; W] {
    let mut shuffled = original;

    for row in (1..H).rev() {
        let rand_row = (rng.next_u32() as usize) % row;
        for column in shuffled.iter_mut() {
            column.swap(row, rand_row);
        }
    }

    shuffled
}

#[derive(Clone)]
struct MyConfig<const W: usize> {
    q_shuffle: Selector,
    q_first: Selector,
    q_last: Selector,
    original: [Column<Advice>; W],
    shuffled: [Column<Advice>; W],
    theta: Challenge,
    gamma: Challenge,
    z: Column<Advice>,
}

impl<const W: usize> MyConfig<W> {
    fn configure<F: FieldExt>(meta: &mut ConstraintSystem<F>) -> Self {
        let [q_shuffle, q_first, q_last] = [(); 3].map(|_| meta.selector());
        // Fist phase
        let original = [(); W].map(|_| meta.advice_column_in(First));
        let shuffled = [(); W].map(|_| meta.advice_column_in(First));
        let [theta, gamma] = [(); 2].map(|_| meta.challenge_usable_after(First));
        // Any phase skipped will cause a panic
        assert!(catch_unwind_silent(|| meta.challenge_usable_after(Second)).is_err());
        assert!(catch_unwind_silent(|| meta.advice_column_in(Third)).is_err());
        // Second phase
        let z = meta.advice_column_in(Second);

        meta.create_gate("z should start with 1", |meta| {
            let q_first = meta.query_selector(q_first);
            let z = meta.query_advice(z, Rotation::cur());
            let one = Expression::Constant(F::one());

            vec![q_first * (one - z)]
        });

        meta.create_gate("z should end with 1", |meta| {
            let q_last = meta.query_selector(q_last);
            let z = meta.query_advice(z, Rotation::cur());
            let one = Expression::Constant(F::one());

            vec![q_last * (one - z)]
        });

        meta.create_gate("z should have valid transition", |meta| {
            let q_shuffle = meta.query_selector(q_shuffle);
            let original = original.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let shuffled = shuffled.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let [theta, gamma] = [theta, gamma].map(|challenge| meta.query_challenge(challenge));
            let [z, z_w] =
                [Rotation::cur(), Rotation::next()].map(|rotation| meta.query_advice(z, rotation));

            // Compress
            let original = original
                .iter()
                .cloned()
                .reduce(|acc, a| acc * theta.clone() + a)
                .unwrap();
            let shuffled = shuffled
                .iter()
                .cloned()
                .reduce(|acc, a| acc * theta.clone() + a)
                .unwrap();

            vec![q_shuffle * (z * (original + gamma.clone()) - z_w * (shuffled + gamma))]
        });

        Self {
            q_shuffle,
            q_first,
            q_last,
            original,
            shuffled,
            theta,
            gamma,
            z,
        }
    }
}

#[derive(Clone, Default)]
struct MyCircuit<F: FieldExt, const W: usize, const H: usize> {
    original: Value<[[F; H]; W]>,
    shuffled: Value<[[F; H]; W]>,
}

impl<F: FieldExt, const W: usize, const H: usize> MyCircuit<F, W, H> {
    fn rand<R: RngCore>(rng: &mut R) -> Self {
        let original = rand_2d_array::<F, _, W, H>(rng);
        let shuffled = shuffled(original, rng);

        Self {
            original: Value::known(original),
            shuffled: Value::known(shuffled),
        }
    }
}

impl<F: FieldExt, const W: usize, const H: usize> Circuit<F> for MyCircuit<F, W, H> {
    type Config = MyConfig<W>;
    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        MyConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let theta = layouter.get_challenge(config.theta);
        let gamma = layouter.get_challenge(config.gamma);

        layouter.assign_region(
            || "Shuffle original into shuffled",
            |mut region| {
                // Keygen
                config.q_first.enable(&mut region, 0)?;
                config.q_last.enable(&mut region, H)?;
                for offset in 0..H {
                    config.q_shuffle.enable(&mut region, offset)?;
                }

                // First phase
                for (idx, (&column, values)) in config
                    .original
                    .iter()
                    .zip(self.original.transpose_array().iter())
                    .enumerate()
                {
                    for (offset, &value) in values.transpose_array().iter().enumerate() {
                        region.assign_advice(
                            || format!("original[{}][{}]", idx, offset),
                            column,
                            offset,
                            || value,
                        )?;
                    }
                }
                for (idx, (&column, values)) in config
                    .shuffled
                    .iter()
                    .zip(self.shuffled.transpose_array().iter())
                    .enumerate()
                {
                    for (offset, &value) in values.transpose_array().iter().enumerate() {
                        region.assign_advice(
                            || format!("shuffled[{}][{}]", idx, offset),
                            column,
                            offset,
                            || value,
                        )?;
                    }
                }

                // Second phase
                let z = self.original.zip(self.shuffled).zip(theta).zip(gamma).map(
                    |(((original, shuffled), theta), gamma)| {
                        let mut product = vec![F::zero(); H];
                        for (idx, product) in product.iter_mut().enumerate() {
                            let mut compressed = F::zero();
                            for value in shuffled.iter() {
                                compressed *= theta;
                                compressed += value[idx];
                            }

                            *product = compressed + gamma
                        }

                        product.iter_mut().batch_invert();

                        for (idx, product) in product.iter_mut().enumerate() {
                            let mut compressed = F::zero();
                            for value in original.iter() {
                                compressed *= theta;
                                compressed += value[idx];
                            }

                            *product *= compressed + gamma
                        }

                        #[allow(clippy::let_and_return)]
                        let z = iter::once(F::one())
                            .chain(product)
                            .scan(F::one(), |state, cur| {
                                *state *= &cur;
                                Some(*state)
                            })
                            .take(H + 1)
                            .collect::<Vec<_>>();

                        #[cfg(feature = "sanity-checks")]
                        assert_eq!(F::one(), *z.last().unwrap());

                        z
                    },
                );
                for (offset, value) in z.transpose_vec(H + 1).into_iter().enumerate() {
                    region.assign_advice(
                        || format!("z[{}]", offset),
                        config.z,
                        offset,
                        || value,
                    )?;
                }

                Ok(())
            },
        )
    }
}

fn test_mock_prover<F: FieldExt, const W: usize, const H: usize>(
    k: u32,
    circuit: MyCircuit<F, W, H>,
    expected: Result<(), Vec<(metadata::Constraint, FailureLocation)>>,
) {
    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    match (prover.verify(), expected) {
        (Ok(_), Ok(_)) => {}
        (Err(err), Err(expected)) => {
            assert_eq!(
                err.into_iter()
                    .map(|failure| match failure {
                        VerifyFailure::ConstraintNotSatisfied {
                            constraint,
                            location,
                            ..
                        } => (constraint, location),
                        _ => panic!("MockProver::verify has result unmatching expected"),
                    })
                    .collect::<Vec<_>>(),
                expected
            )
        }
        (_, _) => panic!("MockProver::verify has result unmatching expected"),
    };
}

fn test_prover<C: CurveAffine, const W: usize, const H: usize>(
    k: u32,
    circuit: MyCircuit<C::Scalar, W, H>,
    expected: bool,
) {
    let params = Params::<C>::new(k);
    let vk = keygen_vk(&params, &circuit).unwrap();
    let pk = keygen_pk(&params, vk, &circuit).unwrap();

    let proof = {
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

        create_proof(&params, &pk, &[circuit], &[&[]], OsRng, &mut transcript)
            .expect("proof generation should not fail");

        transcript.finalize()
    };

    let accepted = {
        let strategy = SingleVerifier::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

        verify_proof(&params, pk.get_vk(), strategy, &[&[]], &mut transcript).is_ok()
    };

    assert_eq!(accepted, expected);
}

fn main() {
    const W: usize = 4;
    const H: usize = 32;
    const K: u32 = 8;

    let circuit = &MyCircuit::<_, W, H>::rand(&mut OsRng);

    {
        test_mock_prover(K, circuit.clone(), Ok(()));
        test_prover::<EqAffine, W, H>(K, circuit.clone(), true);
    }

    #[cfg(not(feature = "sanity-checks"))]
    {
        use std::ops::IndexMut;

        let mut circuit = circuit.clone();
        circuit.shuffled = circuit.shuffled.map(|mut shuffled| {
            shuffled.index_mut(0).swap(0, 1);
            shuffled
        });

        test_mock_prover(
            K,
            circuit.clone(),
            Err(vec![(
                ((1, "z should end with 1").into(), 0, "").into(),
                FailureLocation::InRegion {
                    region: (0, "Shuffle original into shuffled").into(),
                    offset: 32,
                },
            )]),
        );
        test_prover::<EqAffine, W, H>(K, circuit, false);
    }
}
