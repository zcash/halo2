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
    Pair(alg_1::Bitstring<F>),
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
        layouter: &mut impl Layouter<C::Base>,
        bits: &[Value<bool>],
        for_base: bool,
    ) -> Result<Vec<Self::Bitstring>, Error> {
        assert_eq!(bits.len() % 2, 0);

        bits.chunks(Self::MAX_BITSTRING_LENGTH)
            .map(|bits| {
                if for_base {
                    self.alg_1
                        .witness_bitstring(layouter.namespace(|| "alg 1"), bits)
                        .map(Bitstring::Pair)
                } else {
                    self.alg_2
                        .witness_bitstring(layouter.namespace(|| "alg 2"), bits)
                        .map(Bitstring::KBit)
                }
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::super::util::compute_endoscalar_with_acc;
    use super::{EndoscaleConfig, EndoscaleInstructions};
    use crate::ecc::chip::NonIdentityEccPoint;

    use ff::PrimeFieldBits;
    use halo2_proofs::{
        arithmetic::CurveAffine,
        circuit::{Layouter, SimpleFloorPlanner, Value},
        plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
        poly::commitment::Params,
    };
    use pasta_curves::{pallas, vesta};

    use std::{convert::TryInto, marker::PhantomData};

    struct BaseCircuit<
        C: CurveAffine,
        const K: usize,
        const NUM_BITS: usize,
        const NUM_BITS_DIV_K_CEIL: usize,
    >
    where
        C::Base: PrimeFieldBits,
    {
        bitstring: Value<[bool; NUM_BITS]>,
        pub_input_rows: [usize; NUM_BITS_DIV_K_CEIL],
        _marker: PhantomData<C>,
    }

    impl<
            C: CurveAffine,
            const K: usize,
            const NUM_BITS: usize,
            const NUM_BITS_DIV_K_CEIL: usize,
        > Circuit<C::Base> for BaseCircuit<C, K, NUM_BITS, NUM_BITS_DIV_K_CEIL>
    where
        C::Base: PrimeFieldBits,
    {
        type Config = (EndoscaleConfig<C, K, 248>, Column<Advice>);
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self {
                bitstring: Value::unknown(),
                pub_input_rows: self.pub_input_rows,
                _marker: PhantomData,
            }
        }

        fn configure(meta: &mut ConstraintSystem<C::Base>) -> Self::Config {
            let constants = meta.fixed_column();
            meta.enable_constant(constants);

            let advices = (0..8)
                .map(|_| meta.advice_column())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let running_sum = meta.advice_column();
            let endoscalars = meta.instance_column();

            (
                EndoscaleConfig::configure(meta, advices, running_sum, endoscalars),
                running_sum,
            )
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<C::Base>,
        ) -> Result<(), Error> {
            config.0.alg_2.table.load(&mut layouter)?;

            let bitstring = config.0.witness_bitstring(
                &mut layouter,
                &self.bitstring.transpose_array(),
                true,
            )?;

            // Alg 1 (fixed base)
            let g_lagrange = Params::<C>::new(11).g_lagrange()[0];
            config
                .0
                .endoscale_fixed_base(&mut layouter, bitstring.clone(), vec![g_lagrange])?;

            // Alg 1 (variable base)
            let g_lagrange = layouter.assign_region(
                || "g_lagrange",
                |mut region| {
                    let x = region.assign_advice(
                        || "x",
                        config.1,
                        0,
                        || Value::known(*g_lagrange.coordinates().unwrap().x()),
                    )?;
                    let y = region.assign_advice(
                        || "y",
                        config.1,
                        1,
                        || Value::known(*g_lagrange.coordinates().unwrap().y()),
                    )?;

                    Ok(NonIdentityEccPoint::<C>::from_coordinates_unchecked(
                        x.into(),
                        y.into(),
                    ))
                },
            )?;
            config
                .0
                .endoscale_var_base(&mut layouter, bitstring, vec![g_lagrange])?;

            Ok(())
        }
    }

    struct ScalarCircuit<
        C: CurveAffine,
        const K: usize,
        const NUM_BITS: usize,
        const NUM_BITS_DIV_K_CEIL: usize,
    >
    where
        C::Base: PrimeFieldBits,
    {
        bitstring: Value<[bool; NUM_BITS]>,
        pub_input_rows: [usize; NUM_BITS_DIV_K_CEIL],
        _marker: PhantomData<C>,
    }

    impl<
            C: CurveAffine,
            const K: usize,
            const NUM_BITS: usize,
            const NUM_BITS_DIV_K_CEIL: usize,
        > Circuit<C::Base> for ScalarCircuit<C, K, NUM_BITS, NUM_BITS_DIV_K_CEIL>
    where
        C::Base: PrimeFieldBits,
    {
        type Config = EndoscaleConfig<C, K, 248>;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self {
                bitstring: Value::unknown(),
                pub_input_rows: self.pub_input_rows,
                _marker: PhantomData,
            }
        }

        fn configure(meta: &mut ConstraintSystem<C::Base>) -> Self::Config {
            let constants = meta.fixed_column();
            meta.enable_constant(constants);

            let advices = (0..8)
                .map(|_| meta.advice_column())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let running_sum = meta.advice_column();
            let endoscalars = meta.instance_column();

            EndoscaleConfig::configure(meta, advices, running_sum, endoscalars)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<C::Base>,
        ) -> Result<(), Error> {
            config.alg_2.table.load(&mut layouter)?;

            let bitstring = config.witness_bitstring(
                &mut layouter,
                &self.bitstring.transpose_array(),
                false,
            )?;

            // Alg 2 with lookup
            config.compute_endoscalar(&mut layouter, &bitstring[0])?;

            // Constrain bitstring
            config.constrain_bitstring(
                &mut layouter,
                &bitstring[0],
                self.pub_input_rows.to_vec(),
            )?;

            Ok(())
        }
    }

    fn test_endoscale_cycle<
        BaseCurve: CurveAffine,
        ScalarCurve: CurveAffine,
        const K: usize,
        const NUM_BITS: usize,
        const NUM_BITS_DIV_K_CEIL: usize,
    >()
    where
        BaseCurve::Base: PrimeFieldBits,
        ScalarCurve::Base: PrimeFieldBits,
    {
        use ff::Field;
        use halo2_proofs::dev::MockProver;

        let bitstring: [bool; NUM_BITS] = (0..NUM_BITS)
            .map(|_| rand::random::<bool>())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // Public input of endoscalars in the base field corresponding to the bits.
        let pub_inputs_base: Vec<BaseCurve::Base> = {
            // Pad bitstring to multiple of K.
            let bitstring = bitstring
                .iter()
                .copied()
                .chain(std::iter::repeat(false))
                .take(K * NUM_BITS_DIV_K_CEIL)
                .collect::<Vec<_>>();
            bitstring
                .chunks(K)
                .map(|chunk| compute_endoscalar_with_acc(Some(BaseCurve::Base::zero()), chunk))
                .collect()
        };

        // Public input of endoscalars in the base field corresponding to the bits.
        let pub_inputs_scalar: Vec<ScalarCurve::Base> = {
            // Pad bitstring to multiple of K.
            let bitstring = bitstring
                .iter()
                .copied()
                .chain(std::iter::repeat(false))
                .take(K * NUM_BITS_DIV_K_CEIL)
                .collect::<Vec<_>>();
            bitstring
                .chunks(K)
                .map(|chunk| compute_endoscalar_with_acc(Some(ScalarCurve::Base::zero()), chunk))
                .collect()
        };

        let base_circuit = BaseCircuit::<BaseCurve, K, NUM_BITS, NUM_BITS_DIV_K_CEIL> {
            bitstring: Value::known(bitstring),
            pub_input_rows: (0..NUM_BITS_DIV_K_CEIL)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            _marker: PhantomData,
        };
        let scalar_circuit = ScalarCircuit::<ScalarCurve, K, NUM_BITS, NUM_BITS_DIV_K_CEIL> {
            bitstring: Value::known(bitstring),
            pub_input_rows: (0..NUM_BITS_DIV_K_CEIL)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            _marker: PhantomData,
        };

        let base_prover = MockProver::run(11, &base_circuit, vec![pub_inputs_base]).unwrap();
        base_prover.assert_satisfied();

        let scalar_prover = MockProver::run(11, &scalar_circuit, vec![pub_inputs_scalar]).unwrap();
        scalar_prover.assert_satisfied();
    }

    #[test]
    fn test_endoscale() {
        test_endoscale_cycle::<pallas::Affine, vesta::Affine, 8, 64, 8>();
        test_endoscale_cycle::<vesta::Affine, pallas::Affine, 8, 64, 8>();

        test_endoscale_cycle::<pallas::Affine, vesta::Affine, 8, 66, 9>();
        test_endoscale_cycle::<vesta::Affine, pallas::Affine, 8, 66, 9>();
    }
}
