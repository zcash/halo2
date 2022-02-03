use halo2_proofs::{
    circuit::{Layouter, Region, AssignedCell},
    plonk::{Column, Advice, Error, Assigned},
};

use pasta_curves::pallas;
use std::convert::TryInto;

#[derive(Clone, Debug, Copy, Default)]
pub struct Chunk(u8);

impl Chunk {
    pub fn new(x: u8) -> Self {
        return Chunk(x) 
    }
}

impl std::ops::Deref for Chunk {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct AssignedChunk(AssignedCell<Chunk, pallas::Base>);

impl AssignedChunk {
    pub fn new(assigned_cell: AssignedCell<Chunk, pallas::Base>) -> Self {
        AssignedChunk(assigned_cell)
    }

    pub fn value_chunk(&self) -> Option<Chunk> {
        self.0.value().map(|v| *v)
    }
}

impl From<&Chunk> for Assigned<pallas::Base> {
    fn from(chunk: &Chunk) -> Assigned<pallas::Base> {
        pallas::Base::from(chunk.0 as u64).into()
    }
}


impl AssignedChunk {
    #![allow(dead_code)]
    fn assign_chunk(
        mut layouter: impl Layouter<pallas::Base>,
        column: Column<Advice>,
        value: Option<Chunk>,
    ) -> Result<AssignedChunk, Error> {
        layouter.assign_region(
            || "witness word",
            |mut region| {
                let assigned = region.assign_advice(
                    || "witness",
                    column,
                    0,
                    || value.ok_or(Error::Synthesis),
                )?;
                Ok(AssignedChunk::new(assigned))
            },
        )
    }
}


#[derive(Clone, Debug, Copy, Default)]
pub struct Word(u32);

impl Word {
    pub fn new(x: u32) -> Self {
        return Word(x) 
    }

    pub fn decompose_4(&self) -> [Chunk; 4] {
        let bytes = self.to_le_bytes();
        bytes.iter().map(|byte| Chunk(*byte)).collect::<Vec<Chunk>>().as_slice().try_into().unwrap()
    }

    pub fn compose(chunks: [u8; 4]) -> Self {
        return Word(u32::from_le_bytes(chunks))
    }
}

impl std::ops::Deref for Word {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct AssignedWord(AssignedCell<Word, pallas::Base>);

impl AssignedWord {
    pub fn new(assigned_cell: AssignedCell<Word, pallas::Base>) -> Self {
        AssignedWord(assigned_cell)
    }

    pub fn value_word(&self) -> Option<Word> {
        self.0.value().map(|word| *word)
    }
}

impl From<&Word> for Assigned<pallas::Base> {
    fn from(word: &Word) -> Assigned<pallas::Base> {
        pallas::Base::from(word.0 as u64).into()
    }
}


impl AssignedWord {
    pub fn assign_word(
        mut layouter: impl Layouter<pallas::Base>,
        column: Column<Advice>,
        value: Option<Word>,
    ) -> Result<AssignedWord, Error> {
        layouter.assign_region(
            || "witness word",
            |mut region| {
                let assigned = region.assign_advice(
                    || "witness",
                    column,
                    0,
                    || value.ok_or(Error::Synthesis),
                )?;
                Ok(AssignedWord::new(assigned))
            },
        )
    }

    pub fn copy<A, AR>(
        &self,
        annotation: A,
        region: &mut Region<'_, pallas::Base>,
        column: Column<Advice>,
        offset: usize,
    ) -> Result<Self, Error>
    where
        A: Fn() -> AR,
        AR: Into<String>,
    {
        let assigned_cell = &self.0;
        let copied = assigned_cell.copy_advice(annotation, region, column, offset)?;
        Ok(AssignedWord::new(copied))
    }
}

#[cfg(test)]
mod test {

    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::{Advice, Instance, Column, ConstraintSystem, Error},
        plonk,
        pasta::Fp,
        dev::MockProver,
    };
    use std::marker::PhantomData;
    use pasta_curves::pallas;

    use super::{Word, AssignedWord};

    #[derive(Clone, Debug)]
    pub struct Config {
        advice: [Column<Advice>; 4],
    }

    #[derive(Clone, Debug, Default)]
    pub struct Circuit {
        value: Option<Word>
    }

    impl plonk::Circuit<pallas::Base> for Circuit {
        type Config = Config;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

        fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {

            let advice = [
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column()
            ];

            for column in &advice {
                meta.enable_equality(*column);
            }

            Config {
                advice, 
            }
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            let config = config.clone();

            let assigned = AssignedWord::assign_word(layouter.namespace(|| "witness value"), config.advice[0], self.value).unwrap();
            Ok(())
        }
    }

    #[test]
    fn assign_word() {
        let value = Word::new(5);
        let circuit = Circuit {
            value: Some(value)
        };
        let k = 4;
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}