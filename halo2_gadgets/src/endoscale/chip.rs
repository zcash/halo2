use crate::{ecc::chip::NonIdentityEccPoint, utilities::decompose_running_sum::RunningSumConfig};

use super::EndoscaleInstructions;
use ff::PrimeFieldBits;
use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Assigned, Column, ConstraintSystem, Error, Instance},
};

mod alg_1;
mod alg_2;

use alg_1::Alg1Config;
use alg_2::Alg2Config;

/// Bitstring used in endoscaling.
#[derive(Clone, Debug)]
#[allow(clippy::type_complexity)]
pub enum Bitstring<F: FieldExt + PrimeFieldBits, const K: usize> {
    Pair(alg_1::Bitstring<F, 2>),
    KBit(alg_2::Bitstring<F, K>),
}

/// Config used in processing endoscalars.
#[derive(Clone, Debug)]
pub struct EndoscaleConfig<C: CurveAffine, const K: usize, const MAX_BITSTRING_LENGTH: usize>
where
    C::Base: PrimeFieldBits,
{
    alg_1: Alg1Config<C>,
    alg_2: Alg2Config<C, K, MAX_BITSTRING_LENGTH>,
}

impl<C: CurveAffine, const K: usize, const MAX_BITSTRING_LENGTH: usize>
    EndoscaleConfig<C, K, MAX_BITSTRING_LENGTH>
where
    C::Base: PrimeFieldBits,
{
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        // Advice columns not shared across alg_1 and alg_2
        advices: [Column<Advice>; 8],
        // Running sum column shared across alg_1 and alg_2
        running_sum: Column<Advice>,
        endoscalars: Column<Instance>,
    ) -> Self {
        let running_sum_pairs = {
            let q_pairs = meta.selector();
            RunningSumConfig::configure(meta, q_pairs, running_sum)
        };

        let alg_1 = Alg1Config::configure(
            meta,
            (advices[0], advices[1]),
            (advices[2], advices[3]),
            (advices[4], advices[5], advices[6], advices[7]),
            running_sum_pairs,
        );

        let running_sum_chunks = {
            let q_chunks = meta.complex_selector();
            RunningSumConfig::configure(meta, q_chunks, running_sum)
        };

        let alg_2 = Alg2Config::configure(
            meta,
            endoscalars,
            advices[0],
            advices[1],
            running_sum_chunks,
        );

        Self { alg_1, alg_2 }
    }
}

impl<C: CurveAffine, const K: usize, const N: usize> EndoscaleInstructions<C>
    for EndoscaleConfig<C, K, N>
where
    C::Base: PrimeFieldBits,
{
    type NonIdentityPoint = NonIdentityEccPoint<C>;
    type Bitstring = Bitstring<C::Base, K>;
    type FixedBases = C;
    const MAX_BITSTRING_LENGTH: usize = 248;
    const NUM_FIXED_BASES: usize = N;

    fn witness_bitstring(
        &self,
        _layouter: &mut impl Layouter<C::Base>,
        _bits: &[Value<bool>],
        _for_base: bool,
    ) -> Result<Vec<Self::Bitstring>, Error> {
        todo!()
    }

    #[allow(clippy::type_complexity)]
    fn endoscale_fixed_base(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: Vec<Self::Bitstring>,
        bases: Vec<Self::FixedBases>,
    ) -> Result<Vec<Self::NonIdentityPoint>, Error> {
        let mut points = Vec::new();
        for (bitstring, base) in bitstring.iter().zip(bases.iter()) {
            match bitstring {
                Bitstring::Pair(bitstring) => {
                    points.push(self.alg_1.endoscale_fixed_base(layouter, bitstring, base)?)
                }
                _ => unreachable!(),
            }
        }
        Ok(points)
    }

    fn endoscale_var_base(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: Vec<Self::Bitstring>,
        bases: Vec<Self::NonIdentityPoint>,
    ) -> Result<Vec<Self::NonIdentityPoint>, Error> {
        let mut points = Vec::new();
        for (bitstring, base) in bitstring.iter().zip(bases.iter()) {
            match bitstring {
                Bitstring::Pair(bitstring) => {
                    points.push(self.alg_1.endoscale_var_base(layouter, bitstring, base)?)
                }
                _ => unreachable!(),
            }
        }
        Ok(points)
    }

    fn compute_endoscalar(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Self::Bitstring,
    ) -> Result<AssignedCell<Assigned<C::Base>, C::Base>, Error> {
        match bitstring {
            Bitstring::KBit(bitstring) => self.alg_2.compute_endoscalar(layouter, bitstring),
            _ => unreachable!(),
        }
    }

    fn constrain_bitstring(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        bitstring: &Self::Bitstring,
        pub_input_rows: Vec<usize>,
    ) -> Result<(), Error> {
        match bitstring {
            Bitstring::KBit(bitstring) => {
                self.alg_2
                    .constrain_bitstring(layouter, bitstring, pub_input_rows)
            }
            _ => unreachable!(),
        }
    }
}
