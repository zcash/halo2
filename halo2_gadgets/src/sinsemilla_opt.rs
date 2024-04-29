//! The [Sinsemilla] hash function.
//!
//! [Sinsemilla]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash

use std::fmt::Debug;

use pasta_curves::arithmetic::CurveAffine;

use halo2_proofs::{circuit::Layouter, plonk::Error};

use crate::{
    ecc::{self, EccInstructions},
    sinsemilla::{CommitDomain, HashDomain, Message, SinsemillaInstructions},
};

pub mod chip;
pub mod merkle;
pub mod primitives;

/// `SinsemillaInstructionsOptimized` provides an optimized set of instructions
/// for implementing the Sinsemilla hash function and commitment scheme
/// on elliptic curves. This trait is an extension of the `SinsemillaInstructions` trait,
/// designed to enhance performance in specific cryptographic scenarios.ld

pub trait SinsemillaInstructionsOptimized<C: CurveAffine, const K: usize, const MAX_WORDS: usize>:
SinsemillaInstructions<C, K, MAX_WORDS>
{

    /// Hashes a message to an ECC curve point.
    /// This returns both the resulting point, as well as the message
    /// decomposition in the form of intermediate values in a cumulative
    /// sum.
    /// The initial point `Q` is a private point.
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point_with_private_init(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: &Self::NonIdentityPoint,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Self::RunningSum>), Error>;
}

impl<C: CurveAffine, SinsemillaChip, EccChip, const K: usize, const MAX_WORDS: usize>
    HashDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructionsOptimized<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    /// Evaluate the Sinsemilla hash of `message` from the private initial point `Q`.
    pub fn hash_to_point_with_private_init(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: &<SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<(ecc::NonIdentityPoint<C, EccChip>, Vec<SinsemillaChip::RunningSum>), Error> {
        assert_eq!(self.sinsemilla_chip, message.chip);
        self.sinsemilla_chip
            .hash_to_point_with_private_init(layouter, Q, message.inner)
            .map(|(point, zs)| (ecc::NonIdentityPoint::from_inner(self.ecc_chip.clone(), point), zs))
    }

}

impl<C: CurveAffine, SinsemillaChip, EccChip, const K: usize, const MAX_WORDS: usize>
    CommitDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructionsOptimized<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    #[allow(clippy::type_complexity)]
    /// Evaluates the Sinsemilla hash of `message` from the public initial point `Q` stored
    /// into `CommitDomain`.
    pub fn hash(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<
        (
            ecc::NonIdentityPoint<C, EccChip>,
            Vec<SinsemillaChip::RunningSum>,
        ),
        Error,
    > {
        assert_eq!(self.M.sinsemilla_chip, message.chip);
        self.M.hash_to_point(layouter, message)
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    /// Evaluates the Sinsemilla hash of `message` from the private initial point `Q`.
    pub fn hash_with_private_init(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: &<SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<
        (
            ecc::NonIdentityPoint<C, EccChip>,
            Vec<SinsemillaChip::RunningSum>,
        ),
        Error,
    > {
        assert_eq!(self.M.sinsemilla_chip, message.chip);
        self.M.hash_to_point_with_private_init(layouter, Q, message)
    }

    #[allow(clippy::type_complexity)]
    /// Returns the public initial point `Q` stored into `CommitDomain`.
    pub fn q_init(&self) -> C {
        self.M.Q
    }

    #[allow(clippy::type_complexity)]
    /// Evaluates the blinding factor equal to $\[r\] R$ where `r` is stored in the `CommitDomain`.
    pub fn blinding_factor(
        &self,
        mut layouter: impl Layouter<C::Base>,
        r: ecc::ScalarFixed<C, EccChip>,
    ) -> Result<
        ecc::Point<C, EccChip>,
        Error,
    > {
        let (blind, _) = self.R.mul(layouter.namespace(|| "[r] R"), r)?;
        Ok(blind)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use rand::rngs::OsRng;

    use crate::sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        CommitDomain, CommitDomains, HashDomain, HashDomains, Message, MessagePiece,
    };

    use crate::{
        ecc::ScalarFixed,
        sinsemilla::primitives::{self as sinsemilla, K},
        {
            ecc::{
                chip::{find_zs_and_us, EccChip, EccConfig, H, NUM_WINDOWS},
                tests::{FullWidth, TestFixedBases},
                NonIdentityPoint,
            },
        },
    };

    use group::{ff::Field, Curve};
    use lazy_static::lazy_static;
    use pasta_curves::pallas;

    use crate::sinsemilla_opt::chip::SinsemillaChipOptimized;
    use crate::utilities_opt::lookup_range_check::LookupRangeCheckConfigOptimized;
    use std::convert::TryInto;

    pub(crate) const PERSONALIZATION: &str = "MerkleCRH";

    lazy_static! {
        static ref COMMIT_DOMAIN: sinsemilla::CommitDomain =
            sinsemilla::CommitDomain::new(PERSONALIZATION);
        static ref Q: pallas::Affine = COMMIT_DOMAIN.Q().to_affine();
        static ref R: pallas::Affine = COMMIT_DOMAIN.R().to_affine();
        static ref R_ZS_AND_US: Vec<(u64, [pallas::Base; H])> =
            find_zs_and_us(*R, NUM_WINDOWS).unwrap();
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub(crate) struct TestHashDomain;
    impl HashDomains<pallas::Affine> for TestHashDomain {
        fn Q(&self) -> pallas::Affine {
            *Q
        }
    }

    // This test does not make use of the CommitDomain.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub(crate) struct TestCommitDomain;
    impl CommitDomains<pallas::Affine, TestFixedBases, TestHashDomain> for TestCommitDomain {
        fn r(&self) -> FullWidth {
            FullWidth::from_parts(*R, &R_ZS_AND_US)
        }

        fn hash_domain(&self) -> TestHashDomain {
            TestHashDomain
        }
    }

    struct MyCircuit {}

    impl Circuit<pallas::Base> for MyCircuit {
        #[allow(clippy::type_complexity)]
        type Config = (
            EccConfig<
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >,
            SinsemillaConfig<
                TestHashDomain,
                TestCommitDomain,
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >,
            SinsemillaConfig<
                TestHashDomain,
                TestCommitDomain,
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >,
        );
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            MyCircuit {}
        }

        #[allow(non_snake_case)]
        fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
            let advices = [
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            ];

            // Shared fixed column for loading constants
            let constants = meta.fixed_column();
            meta.enable_constant(constants);

            let table_idx = meta.lookup_table_column();
            let table_range_check_tag = meta.lookup_table_column();
            let lagrange_coeffs = [
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
            ];

            // Fixed columns for the Sinsemilla generator lookup table
            let lookup = (
                table_idx,
                meta.lookup_table_column(),
                meta.lookup_table_column(),
            );

            let range_check = LookupRangeCheckConfigOptimized::configure_with_tag(
                meta,
                advices[9],
                table_idx,
                table_range_check_tag,
            );

            let ecc_config = EccChip::<
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >::configure(meta, advices, lagrange_coeffs, range_check);

            let config1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                advices[2],
                lagrange_coeffs[0],
                lookup,
                range_check,
            );
            let config2 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                advices[7],
                lagrange_coeffs[1],
                lookup,
                range_check,
            );
            (ecc_config, config1, config2)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            let rng = OsRng;

            let ecc_chip = EccChip::construct(config.0);

            // The two `SinsemillaChip`s share the same lookup table.
            SinsemillaChipOptimized::<TestHashDomain, TestCommitDomain, TestFixedBases>::load(
                config.1.clone(),
                &mut layouter,
            )?;

            // This MerkleCRH example is purely for illustrative purposes.
            // It is not an implementation of the Orchard protocol spec.
            {
                let chip1 = SinsemillaChip::construct(config.1);

                let merkle_crh = HashDomain::new(chip1.clone(), ecc_chip.clone(), &TestHashDomain);

                // Layer 31, l = MERKLE_DEPTH - 1 - layer = 0
                let l_bitstring = vec![Value::known(false); K];
                let l = MessagePiece::from_bitstring(
                    chip1.clone(),
                    layouter.namespace(|| "l"),
                    &l_bitstring,
                )?;

                // Left leaf
                let left_bitstring: Vec<Value<bool>> = (0..250)
                    .map(|_| Value::known(rand::random::<bool>()))
                    .collect();
                let left = MessagePiece::from_bitstring(
                    chip1.clone(),
                    layouter.namespace(|| "left"),
                    &left_bitstring,
                )?;

                // Right leaf
                let right_bitstring: Vec<Value<bool>> = (0..250)
                    .map(|_| Value::known(rand::random::<bool>()))
                    .collect();
                let right = MessagePiece::from_bitstring(
                    chip1.clone(),
                    layouter.namespace(|| "right"),
                    &right_bitstring,
                )?;

                let l_bitstring: Value<Vec<bool>> = l_bitstring.into_iter().collect();
                let left_bitstring: Value<Vec<bool>> = left_bitstring.into_iter().collect();
                let right_bitstring: Value<Vec<bool>> = right_bitstring.into_iter().collect();

                // Witness expected parent
                let expected_parent = {
                    let expected_parent = l_bitstring.zip(left_bitstring.zip(right_bitstring)).map(
                        |(l, (left, right))| {
                            let merkle_crh = sinsemilla::HashDomain::from_Q((*Q).into());
                            let point = merkle_crh
                                .hash_to_point(
                                    l.into_iter()
                                        .chain(left.into_iter())
                                        .chain(right.into_iter()),
                                )
                                .unwrap();
                            point.to_affine()
                        },
                    );

                    NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "Witness expected parent"),
                        expected_parent,
                    )?
                };

                // Parent
                let (parent, _) = {
                    let message = Message::from_pieces(chip1, vec![l, left, right]);
                    merkle_crh.hash_to_point(layouter.namespace(|| "parent"), message)?
                };

                parent.constrain_equal(
                    layouter.namespace(|| "parent == expected parent"),
                    &expected_parent,
                )?;
            }

            {
                let chip2 = SinsemillaChip::construct(config.2);

                let test_commit =
                    CommitDomain::new(chip2.clone(), ecc_chip.clone(), &TestCommitDomain);
                let r_val = pallas::Scalar::random(rng);
                let message: Vec<Value<bool>> = (0..500)
                    .map(|_| Value::known(rand::random::<bool>()))
                    .collect();

                let (result, _) = {
                    let r = ScalarFixed::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "r"),
                        Value::known(r_val),
                    )?;
                    let message = Message::from_bitstring(
                        chip2,
                        layouter.namespace(|| "witness message"),
                        message.clone(),
                    )?;
                    test_commit.commit(layouter.namespace(|| "commit"), message, r)?
                };

                // Witness expected result.
                let expected_result = {
                    let message: Value<Vec<bool>> = message.into_iter().collect();
                    let expected_result = message.map(|message| {
                        let domain = sinsemilla::CommitDomain::new(PERSONALIZATION);
                        let point = domain.commit(message.into_iter(), &r_val).unwrap();
                        point.to_affine()
                    });

                    NonIdentityPoint::new(
                        ecc_chip,
                        layouter.namespace(|| "Witness expected result"),
                        expected_result,
                    )?
                };

                result.constrain_equal(
                    layouter.namespace(|| "result == expected result"),
                    &expected_result,
                )
            }
        }
    }

    #[test]
    fn sinsemilla_chip() {
        let k = 11;
        let circuit = MyCircuit {};
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }

    #[cfg(feature = "test-dev-graph")]
    #[test]
    fn print_sinsemilla_chip() {
        use plotters::prelude::*;

        let root =
            BitMapBackend::new("sinsemilla-hash-layout.png", (1024, 7680)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("SinsemillaHash", ("sans-serif", 60)).unwrap();

        let circuit = MyCircuit {};
        halo2_proofs::dev::CircuitLayout::default()
            .render(11, &circuit, &root)
            .unwrap();
    }
}
