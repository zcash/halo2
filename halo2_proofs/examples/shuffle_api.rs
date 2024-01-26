use std::{marker::PhantomData, vec};

use ff::FromUniformBytes;
use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column,
        ConstraintSystem, Error, Fixed, Selector,
    },
    poly::Rotation,
    poly::{
        commitment::ParamsProver,
        ipa::{
            commitment::{IPACommitmentScheme, ParamsIPA},
            multiopen::{ProverIPA, VerifierIPA},
            strategy::AccumulatorStrategy,
        },
        VerificationStrategy,
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use halo2curves::{pasta::EqAffine, CurveAffine};
use rand_core::OsRng;

struct ShuffleChip<F: Field> {
    config: ShuffleConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone, Debug)]
struct ShuffleConfig {
    input_0: Column<Advice>,
    input_1: Column<Fixed>,
    shuffle_0: Column<Advice>,
    shuffle_1: Column<Advice>,
    s_input: Selector,
    s_shuffle: Selector,
}

impl<F: Field> ShuffleChip<F> {
    fn construct(config: ShuffleConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        input_0: Column<Advice>,
        input_1: Column<Fixed>,
        shuffle_0: Column<Advice>,
        shuffle_1: Column<Advice>,
    ) -> ShuffleConfig {
        let s_shuffle = meta.complex_selector();
        let s_input = meta.complex_selector();
        meta.shuffle("shuffle", |meta| {
            let s_input = meta.query_selector(s_input);
            let s_shuffle = meta.query_selector(s_shuffle);
            let input_0 = meta.query_advice(input_0, Rotation::cur());
            let input_1 = meta.query_fixed(input_1, Rotation::cur());
            let shuffle_0 = meta.query_advice(shuffle_0, Rotation::cur());
            let shuffle_1 = meta.query_advice(shuffle_1, Rotation::cur());
            vec![
                (s_input.clone() * input_0, s_shuffle.clone() * shuffle_0),
                (s_input * input_1, s_shuffle * shuffle_1),
            ]
        });
        ShuffleConfig {
            input_0,
            input_1,
            shuffle_0,
            shuffle_1,
            s_input,
            s_shuffle,
        }
    }
}

#[derive(Default)]
struct MyCircuit<F: Field> {
    input_0: Vec<Value<F>>,
    input_1: Vec<F>,
    shuffle_0: Vec<Value<F>>,
    shuffle_1: Vec<Value<F>>,
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    // Since we are using a single chip for everything, we can just reuse its config.
    type Config = ShuffleConfig;
    type FloorPlanner = SimpleFloorPlanner;
    #[cfg(feature = "circuit-params")]
    type Params = ();

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let input_0 = meta.advice_column();
        let input_1 = meta.fixed_column();
        let shuffle_0 = meta.advice_column();
        let shuffle_1 = meta.advice_column();
        ShuffleChip::configure(meta, input_0, input_1, shuffle_0, shuffle_1)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let ch = ShuffleChip::<F>::construct(config);
        layouter.assign_region(
            || "load inputs",
            |mut region| {
                for (i, (input_0, input_1)) in
                    self.input_0.iter().zip(self.input_1.iter()).enumerate()
                {
                    region.assign_advice(|| "input_0", ch.config.input_0, i, || *input_0)?;
                    region.assign_fixed(
                        || "input_1",
                        ch.config.input_1,
                        i,
                        || Value::known(*input_1),
                    )?;
                    ch.config.s_input.enable(&mut region, i)?;
                }
                Ok(())
            },
        )?;
        layouter.assign_region(
            || "load shuffles",
            |mut region| {
                for (i, (shuffle_0, shuffle_1)) in
                    self.shuffle_0.iter().zip(self.shuffle_1.iter()).enumerate()
                {
                    region.assign_advice(|| "shuffle_0", ch.config.shuffle_0, i, || *shuffle_0)?;
                    region.assign_advice(|| "shuffle_1", ch.config.shuffle_1, i, || *shuffle_1)?;
                    ch.config.s_shuffle.enable(&mut region, i)?;
                }
                Ok(())
            },
        )?;
        Ok(())
    }
}

fn test_prover<C: CurveAffine>(k: u32, circuit: MyCircuit<C::Scalar>, expected: bool)
where
    C::Scalar: FromUniformBytes<64>,
{
    let params = ParamsIPA::<C>::new(k);
    let vk = keygen_vk(&params, &circuit).unwrap();
    let pk = keygen_pk(&params, vk, &circuit).unwrap();

    let proof = {
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

        create_proof::<IPACommitmentScheme<C>, ProverIPA<C>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&[]],
            OsRng,
            &mut transcript,
        )
        .expect("proof generation should not fail");

        transcript.finalize()
    };

    let accepted = {
        let strategy = AccumulatorStrategy::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

        verify_proof::<IPACommitmentScheme<C>, VerifierIPA<C>, _, _, _>(
            &params,
            pk.get_vk(),
            strategy,
            &[&[]],
            &mut transcript,
        )
        .map(|strategy| strategy.finalize())
        .unwrap_or_default()
    };

    assert_eq!(accepted, expected);
}

fn main() {
    use halo2_proofs::dev::MockProver;
    use halo2curves::pasta::Fp;
    const K: u32 = 4;
    let input_0 = [1, 2, 4, 1]
        .map(|e: u64| Value::known(Fp::from(e)))
        .to_vec();
    let input_1 = [10, 20, 40, 10].map(Fp::from).to_vec();
    let shuffle_0 = [4, 1, 1, 2]
        .map(|e: u64| Value::known(Fp::from(e)))
        .to_vec();
    let shuffle_1 = [40, 10, 10, 20]
        .map(|e: u64| Value::known(Fp::from(e)))
        .to_vec();
    let circuit = MyCircuit {
        input_0,
        input_1,
        shuffle_0,
        shuffle_1,
    };
    let prover = MockProver::run(K, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
    test_prover::<EqAffine>(K, circuit, true);
}
