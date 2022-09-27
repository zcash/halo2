use std::marker::PhantomData;

use crate::transcript::{Transcript, TranscriptInstructions};
use halo2_gadgets::endoscale::EndoscaleInstructions;
use halo2_proofs::{
    arithmetic::CurveAffine,
    circuit::{Layouter, Value},
    pasta::group::ff::PrimeFieldBits,
    plonk::{Error, VerifyingKey},
    transcript::{EncodedChallenge, TranscriptRead},
};

use super::{Accumulator, Instance};

/// Accumulator verifier
pub struct Verifier<C, E, EndoscaleChip, TranscriptChip, TR>
where
    C: CurveAffine,
    C::Base: PrimeFieldBits,
    E: EncodedChallenge<C>,
    EndoscaleChip: EndoscaleInstructions<C>,
    TranscriptChip: TranscriptInstructions<C>,
    TR: TranscriptRead<C, E> + Clone,
{
    vk: VerifyingKey<C>,
    transcript: Transcript<C, TranscriptChip>,
    endoscale_chip: EndoscaleChip,
    fixed_bases: Vec<EndoscaleChip::FixedBases>,
    _marker: PhantomData<(E, TR)>,
}

impl<
        C: CurveAffine,
        E: EncodedChallenge<C>,
        EndoscaleChip: EndoscaleInstructions<C>,
        TranscriptChip: TranscriptInstructions<C>,
        TR: TranscriptRead<C, E> + Clone,
    > Verifier<C, E, EndoscaleChip, TranscriptChip, TR>
where
    C::Base: PrimeFieldBits,
{
    pub fn new(
        vk: VerifyingKey<C>,
        transcript_chip: TranscriptChip,
        endoscale_chip: EndoscaleChip,
        fixed_bases: Vec<EndoscaleChip::FixedBases>,
    ) -> Self {
        Self {
            vk,
            transcript: Transcript::new(transcript_chip),
            endoscale_chip,
            fixed_bases,
            _marker: PhantomData,
        }
    }

    pub fn verify_proof<A: Accumulator<C>>(
        &mut self,
        mut layouter: impl Layouter<C::Base>,
        proof: Value<TR>,
        instances: &[Instance],
        is_base_case: Value<bool>,
    ) -> Result<(Value<bool>, A::Output), Error> {
        // Check that instances matches the expected number of instance columns
        if instances.len() != self.vk.cs().num_instance_columns() {
            return Err(Error::InvalidInstances);
        }

        let mut instance_commitments = vec![];
        for column in instances.iter() {
            let mut column_vec = vec![];
            for instance in column.iter() {
                let instance =
                    self.endoscale_chip
                        .witness_bitstring(&mut layouter, instance, false)?;
                let commitment = self.endoscale_chip.endoscale_fixed_base(
                    &mut layouter,
                    instance,
                    self.fixed_bases.clone(),
                )?;
                column_vec.push(commitment);
            }
            instance_commitments.push(column_vec);
        }

        // Hash verification key into transcript
        self.transcript.common_scalar(
            layouter.namespace(|| "vk"),
            Value::known(self.vk.transcript_repr()),
        )?;

        let cs = self.vk.cs();
        for _ in 0..cs.num_advice_columns() {
            let advice = proof.clone().map(|mut p| p.read_point().unwrap());
            self.transcript
                .common_point(layouter.namespace(|| ""), advice)?;
        }

        // Old accumulator
        let acc = A::read_instance(instances);
        // Proof of previous recursive circuit
        let new_instance = A::read_instance(instances);
        // FIXME: We can calculate this ourselves
        let new_acc = A::read_new_acc(instances);

        Ok((
            A::check_new_acc(&[acc, new_instance], new_acc, is_base_case),
            new_acc,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use halo2_gadgets::{
        ecc::EccInstructions,
        endoscale::EndoscaleInstructions,
        utilities::{bitstring::BitstringInstructions, UtilitiesInstructions},
    };
    use halo2_proofs::{
        arithmetic::{CurveAffine, Field, FieldExt},
        circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        pasta::{group::ff::PrimeFieldBits, EpAffine, EqAffine},
        plonk::{
            create_proof, keygen_pk, keygen_vk, Circuit, ConstraintSystem, Error, VerifyingKey,
        },
        poly::commitment::Params,
        transcript::{
            self, Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, TranscriptRead,
        },
    };
    use rand_core::OsRng;

    use super::super::Instance;
    use super::Verifier;
    use crate::{
        accumulator::{Accumulator, SplitAccumulator},
        transcript::{DuplexInstructions, TranscriptInstructions},
    };

    #[derive(Clone, Copy)]
    struct BaseCircuit;

    impl<F: Field> Circuit<F> for BaseCircuit {
        type Config = ();

        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            *self
        }

        fn configure(_meta: &mut ConstraintSystem<F>) -> Self::Config {}

        fn synthesize(
            &self,
            _config: Self::Config,
            _layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            Ok(())
        }
    }

    struct EndoscaleChip<C: CurveAffine>(PhantomData<C>);
    impl<C: CurveAffine> EndoscaleInstructions<C> for EndoscaleChip<C>
    where
        C::Base: PrimeFieldBits,
    {
        type NonIdentityPoint = ();

        type Bitstring = ();

        type FixedBases = ();

        const MAX_BITSTRING_LENGTH: usize = 0;

        const NUM_FIXED_BASES: usize = 0;

        fn fixed_bases(&self) -> Vec<Self::FixedBases> {
            vec![]
        }

        fn witness_bitstring(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            bits: &[Value<bool>],
            for_base: bool,
        ) -> Result<Vec<Self::Bitstring>, Error> {
            todo!()
        }

        fn endoscale_fixed_base(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            bitstring: Vec<Self::Bitstring>,
            bases: Vec<Self::FixedBases>,
        ) -> Result<Vec<Self::NonIdentityPoint>, Error> {
            todo!()
        }

        fn endoscale_var_base(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            bitstring: Vec<Self::Bitstring>,
            bases: Vec<Self::NonIdentityPoint>,
        ) -> Result<Vec<Self::NonIdentityPoint>, Error> {
            todo!()
        }

        fn compute_endoscalar(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            bitstring: &Self::Bitstring,
        ) -> Result<
            halo2_proofs::circuit::AssignedCell<
                halo2_proofs::plonk::Assigned<<C as CurveAffine>::Base>,
                <C as CurveAffine>::Base,
            >,
            Error,
        > {
            todo!()
        }

        fn constrain_bitstring(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            bitstring: &Self::Bitstring,
            pub_input_rows: Vec<usize>,
        ) -> Result<(), Error> {
            todo!()
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    struct TranscriptChip;
    impl<F: FieldExt> Chip<F> for TranscriptChip {
        type Config = ();

        type Loaded = ();

        fn config(&self) -> &Self::Config {
            todo!()
        }

        fn loaded(&self) -> &Self::Loaded {
            todo!()
        }
    }
    impl<F: FieldExt> DuplexInstructions<F> for TranscriptChip {
        fn absorb(
            &mut self,
            layouter: impl Layouter<F>,
            value: AssignedCell<F, F>,
        ) -> Result<(), Error> {
            todo!()
        }

        fn squeeze(&mut self, layouter: impl Layouter<F>) -> Result<AssignedCell<F, F>, Error> {
            todo!()
        }
    }
    impl<F: FieldExt> BitstringInstructions<F> for TranscriptChip {
        fn constrain(
            &self,
            layouter: impl Layouter<F>,
            witnessed: &AssignedCell<F, F>,
            num_bits: usize,
        ) -> Result<halo2_gadgets::utilities::RangeConstrained<F, AssignedCell<F, F>>, Error>
        {
            todo!()
        }

        fn extract_bitrange(
            &self,
            layouter: impl Layouter<F>,
            witnessed: &AssignedCell<F, F>,
            range: std::ops::Range<usize>,
        ) -> Result<halo2_gadgets::utilities::RangeConstrained<F, AssignedCell<F, F>>, Error>
        {
            todo!()
        }
    }
    impl<C: CurveAffine> TranscriptInstructions<C> for TranscriptChip where C::Base: PrimeFieldBits {}
    impl<F: FieldExt> UtilitiesInstructions<F> for TranscriptChip {
        type Var = AssignedCell<F, F>;
    }

    #[derive(PartialEq, Clone, Eq, Debug)]
    struct FixedPoints;
    impl<C: CurveAffine> halo2_gadgets::ecc::FixedPoints<C> for FixedPoints {
        type FullScalar = ();

        type ShortScalar = ();

        type Base = ();
    }
    impl<C: CurveAffine> EccInstructions<C> for TranscriptChip
    where
        C::Base: PrimeFieldBits,
    {
        type ScalarVar = ();
        type ScalarFixed = ();
        type ScalarFixedShort = ();
        type Point = ();
        type NonIdentityPoint = ();
        type X = ();
        type FixedPoints = FixedPoints;

        fn constrain_equal(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            a: &Self::Point,
            b: &Self::Point,
        ) -> Result<(), Error> {
            Ok(())
        }

        fn witness_point(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            value: Value<C>,
        ) -> Result<Self::Point, Error> {
            Ok(())
        }

        fn witness_point_non_id(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            value: Value<C>,
        ) -> Result<Self::NonIdentityPoint, Error> {
            Ok(())
        }

        fn witness_scalar_var(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            value: Value<<C>::Scalar>,
        ) -> Result<Self::ScalarVar, Error> {
            Ok(())
        }

        fn witness_scalar_fixed(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            value: Value<<C>::Scalar>,
        ) -> Result<Self::ScalarFixed, Error> {
            Ok(())
        }

        fn scalar_fixed_from_signed_short(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            magnitude_sign: (Self::Var, Self::Var),
        ) -> Result<Self::ScalarFixedShort, Error> {
            Ok(())
        }

        fn extract_p<Point: Into<Self::Point> + Clone>(point: &Point) -> Self::X {
            ()
        }

        fn double(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            a: &Self::NonIdentityPoint,
        ) -> Result<Self::NonIdentityPoint, Error> {
            Ok(())
        }

        fn add_incomplete(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            a: &Self::NonIdentityPoint,
            b: &Self::NonIdentityPoint,
        ) -> Result<Self::NonIdentityPoint, Error> {
            Ok(())
        }

        fn add<A: Into<Self::Point> + Clone, B: Into<Self::Point> + Clone>(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            a: &A,
            b: &B,
        ) -> Result<Self::Point, Error> {
            Ok(())
        }

        fn mul(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            scalar: &Self::ScalarVar,
            base: &Self::NonIdentityPoint,
        ) -> Result<(Self::Point, Self::ScalarVar), Error> {
            Ok(((), ()))
        }

        fn mul_fixed(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            scalar: &Self::ScalarFixed,
            base: &<Self::FixedPoints as halo2_gadgets::ecc::FixedPoints<C>>::FullScalar,
        ) -> Result<(Self::Point, Self::ScalarFixed), Error> {
            Ok(((), ()))
        }

        fn mul_fixed_short(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            scalar: &Self::ScalarFixedShort,
            base: &<Self::FixedPoints as halo2_gadgets::ecc::FixedPoints<C>>::ShortScalar,
        ) -> Result<(Self::Point, Self::ScalarFixedShort), Error> {
            Ok(((), ()))
        }

        fn mul_fixed_base_field_elem(
            &self,
            layouter: &mut impl Layouter<<C as CurveAffine>::Base>,
            base_field_elem: Self::Var,
            base: &<Self::FixedPoints as halo2_gadgets::ecc::FixedPoints<C>>::Base,
        ) -> Result<Self::Point, Error> {
            Ok(())
        }
    }

    fn transcript_chip<C: CurveAffine, T: TranscriptInstructions<C>>() -> T {
        todo!()
    }

    fn endoscale_chip<C: CurveAffine, E: EndoscaleInstructions<C>>() -> E
    where
        C::Base: PrimeFieldBits,
    {
        todo!()
    }

    #[derive(Clone)]
    struct VerifierCircuit<
        'a,
        C: CurveAffine,
        E: EncodedChallenge<C>,
        TR: TranscriptRead<C, E> + Clone,
    >
    where
        C::Base: PrimeFieldBits,
    {
        vk: VerifyingKey<C>,
        proof: Value<TR>,
        instances: &'a [Instance],
        _marker: PhantomData<(C, E)>,
    }

    impl<'a, C: CurveAffine, E: EncodedChallenge<C>, TR: TranscriptRead<C, E> + Clone>
        Circuit<C::Base> for VerifierCircuit<'a, C, E, TR>
    where
        C::Base: PrimeFieldBits,
    {
        type Config = ();

        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self {
                vk: self.vk.clone(),
                proof: self.proof.clone(),
                instances: &self.instances,
                _marker: PhantomData,
            }
        }

        fn configure(_meta: &mut ConstraintSystem<C::Base>) -> Self::Config {}

        fn synthesize(
            &self,
            _config: Self::Config,
            mut layouter: impl Layouter<C::Base>,
        ) -> Result<(), Error> {
            let mut verifier = Verifier::new(
                self.vk.clone(),
                TranscriptChip,
                EndoscaleChip(PhantomData),
                EndoscaleChip::<C>(PhantomData).fixed_bases(),
            );

            // FIXME: We need to output `acc` from `synthesize()`
            let (_, acc) = verifier.verify_proof::<SplitAccumulator<C>>(
                layouter.namespace(|| "instance 0"),
                self.proof.clone(),
                self.instances,
                Value::known(false),
            )?;

            Ok(())
        }
    }

    #[derive(Clone)]
    struct BaseVerifierCircuit<
        'a,
        C: CurveAffine,
        E: EncodedChallenge<C>,
        TR: TranscriptRead<C, E> + Clone,
    >
    where
        C::Base: PrimeFieldBits,
    {
        vk: VerifyingKey<C>,
        proof: Value<TR>,
        instances: &'a [Instance],
        _marker: PhantomData<(C, E)>,
    }

    impl<'a, C: CurveAffine, E: EncodedChallenge<C>, TR: TranscriptRead<C, E> + Clone>
        Circuit<C::Base> for BaseVerifierCircuit<'a, C, E, TR>
    where
        C::Base: PrimeFieldBits,
    {
        type Config = ();

        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self {
                vk: self.vk.clone(),
                proof: self.proof.clone(),
                instances: &self.instances,
                _marker: PhantomData,
            }
        }

        fn configure(_meta: &mut ConstraintSystem<C::Base>) -> Self::Config {}

        fn synthesize(
            &self,
            _config: Self::Config,
            mut layouter: impl Layouter<C::Base>,
        ) -> Result<(), Error> {
            let mut verifier = Verifier::new(
                self.vk.clone(),
                TranscriptChip,
                EndoscaleChip(PhantomData),
                EndoscaleChip::<C>(PhantomData).fixed_bases(),
            );

            // FIXME: We need to output `acc` from `synthesize()`
            let (_, acc) = verifier.verify_proof::<SplitAccumulator<C>>(
                layouter.namespace(|| "instance 0"),
                self.proof.clone(),
                self.instances,
                Value::known(true),
            )?;

            Ok(())
        }
    }

    #[test]
    fn test_verify_gadget() {
        let params: Params<EqAffine> = Params::new(3);
        let vk = keygen_vk(&params, &BaseCircuit).expect("keygen_vk should not fail");
        let pk = keygen_pk(&params, vk.clone(), &BaseCircuit).expect("keygen_pk should not fail");
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

        create_proof(&params, &pk, &[BaseCircuit], &[&[]], OsRng, &mut transcript).unwrap();
        let proof = transcript.finalize();
        let mut proof = Blake2bRead::init(&proof[..]);

        // FIXME: Calculate acc and new_acc for base case

        let base_verifier = BaseVerifierCircuit::<'_, EqAffine, Challenge255<EqAffine>, _> {
            vk,
            proof: Value::known(proof),
            instances: &[],
            _marker: PhantomData,
        };

        let k = 13;
        let prover = MockProver::run(k, &base_verifier, vec![]).unwrap();
        prover.assert_satisfied();

        let params: Params<EqAffine> = Params::new(3);
        let vk = keygen_vk(&params, &BaseCircuit).expect("keygen_vk should not fail");
        let pk = keygen_pk(&params, vk.clone(), &BaseCircuit).expect("keygen_pk should not fail");
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

        create_proof(&params, &pk, &[BaseCircuit], &[&[]], OsRng, &mut transcript).unwrap();
        let proof = transcript.finalize();
        let mut proof = Blake2bRead::init(&proof[..]);

        // FIXME: Calculate acc and new_acc for recursive case

        let verifier = VerifierCircuit::<'_, EqAffine, Challenge255<EqAffine>, _> {
            vk,
            proof: Value::known(proof),
            instances: &[],
            _marker: PhantomData,
        };

        let k = 13;
        let prover = MockProver::run(k, &verifier, vec![]).unwrap();
        prover.assert_satisfied();
    }
}
