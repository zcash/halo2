//! Gadgets for implementing a Merkle tree with Sinsemilla.

pub mod chip;

#[cfg(test)]
pub mod tests {

    use crate::{
        ecc::tests::TestFixedBases,
        sinsemilla::{
            tests::{TestCommitDomain, TestHashDomain},
            HashDomains,
        },
        utilities::{i2lebsp, UtilitiesInstructions},
    };

    use group::ff::{Field, PrimeField, PrimeFieldBits};
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        pasta::pallas,
        plonk::{Circuit, ConstraintSystem, Error},
    };

    use crate::sinsemilla::merkle::chip::MerkleConfig;
    use crate::sinsemilla::merkle::MerklePath;
    use crate::sinsemilla_opt::chip::SinsemillaChipOptimized;
    use crate::sinsemilla_opt::merkle::chip::MerkleChipOptimized;
    use crate::utilities_opt::lookup_range_check::LookupRangeCheckConfigOptimized;
    use rand::{rngs::OsRng, RngCore};
    use std::{convert::TryInto, iter};

    const MERKLE_DEPTH: usize = 32;

    #[derive(Default)]
    struct MyCircuit {
        leaf: Value<pallas::Base>,
        leaf_pos: Value<u32>,
        merkle_path: Value<[pallas::Base; MERKLE_DEPTH]>,
    }

    impl Circuit<pallas::Base> for MyCircuit {
        type Config = (
            MerkleConfig<
                TestHashDomain,
                TestCommitDomain,
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >,
            MerkleConfig<
                TestHashDomain,
                TestCommitDomain,
                TestFixedBases,
                LookupRangeCheckConfigOptimized<pallas::Base, { crate::sinsemilla::primitives::K }>,
            >,
        );
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            Self::default()
        }

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

            // NB: In the actual Action circuit, these fixed columns will be reused
            // by other chips. For this test, we are creating new fixed columns.
            let fixed_y_q_1 = meta.fixed_column();
            let fixed_y_q_2 = meta.fixed_column();

            // Fixed columns for the Sinsemilla generator lookup table
            let lookup = (
                meta.lookup_table_column(),
                meta.lookup_table_column(),
                meta.lookup_table_column(),
            );
            let table_range_check_tag = meta.lookup_table_column();

            let range_check = LookupRangeCheckConfigOptimized::configure_with_tag(
                meta,
                advices[9],
                lookup.0,
                table_range_check_tag,
            );

            let sinsemilla_config_1 = SinsemillaChipOptimized::configure(
                meta,
                advices[5..].try_into().unwrap(),
                advices[7],
                fixed_y_q_1,
                lookup,
                range_check,
            );
            let config1 = MerkleChipOptimized::configure(meta, sinsemilla_config_1);

            let sinsemilla_config_2 = SinsemillaChipOptimized::configure(
                meta,
                advices[..5].try_into().unwrap(),
                advices[2],
                fixed_y_q_2,
                lookup,
                range_check,
            );
            let config2 = MerkleChipOptimized::configure(meta, sinsemilla_config_2);

            (config1, config2)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            // Load generator table (shared across both configs) for SinsemillaChipOptimized
            SinsemillaChipOptimized::<TestHashDomain, TestCommitDomain, TestFixedBases>::load(
                config.0.sinsemilla_config.clone(),
                &mut layouter,
            )?;

            // Construct Merkle chips which will be placed side-by-side in the circuit.
            let chip_1 = MerkleChipOptimized::construct(config.0.clone());
            let chip_2 = MerkleChipOptimized::construct(config.1.clone());

            let leaf = chip_1.load_private(
                layouter.namespace(|| ""),
                config.0.cond_swap_config.a(),
                self.leaf,
            )?;

            let path = MerklePath {
                chips: [chip_1, chip_2],
                domain: TestHashDomain,
                leaf_pos: self.leaf_pos,
                path: self.merkle_path,
            };

            let computed_final_root =
                path.calculate_root(layouter.namespace(|| "calculate root"), leaf)?;

            self.leaf
                .zip(self.leaf_pos)
                .zip(self.merkle_path)
                .zip(computed_final_root.value())
                .assert_if_known(|(((leaf, leaf_pos), merkle_path), computed_final_root)| {
                    // The expected final root
                    let final_root =
                        merkle_path
                            .iter()
                            .enumerate()
                            .fold(*leaf, |node, (l, sibling)| {
                                let l = l as u8;
                                let (left, right) = if leaf_pos & (1 << l) == 0 {
                                    (&node, sibling)
                                } else {
                                    (sibling, &node)
                                };

                                use crate::sinsemilla::primitives as sinsemilla;
                                let merkle_crh =
                                    sinsemilla::HashDomain::from_Q(TestHashDomain.Q().into());

                                merkle_crh
                                    .hash(
                                        iter::empty()
                                            .chain(i2lebsp::<10>(l as u64).iter().copied())
                                            .chain(
                                                left.to_le_bits()
                                                    .iter()
                                                    .by_vals()
                                                    .take(pallas::Base::NUM_BITS as usize),
                                            )
                                            .chain(
                                                right
                                                    .to_le_bits()
                                                    .iter()
                                                    .by_vals()
                                                    .take(pallas::Base::NUM_BITS as usize),
                                            ),
                                    )
                                    .unwrap_or(pallas::Base::zero())
                            });

                    // Check the computed final root against the expected final root.
                    computed_final_root == &&final_root
                });

            Ok(())
        }
    }

    #[test]
    fn merkle_chip() {
        let mut rng = OsRng;

        // Choose a random leaf and position
        let leaf = pallas::Base::random(rng);
        let pos = rng.next_u32();

        // Choose a path of random inner nodes
        let path: Vec<_> = (0..(MERKLE_DEPTH))
            .map(|_| pallas::Base::random(rng))
            .collect();

        // The root is provided as a public input in the Orchard circuit.

        let circuit = MyCircuit {
            leaf: Value::known(leaf),
            leaf_pos: Value::known(pos),
            merkle_path: Value::known(path.try_into().unwrap()),
        };

        let prover = MockProver::run(11, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }

    #[cfg(feature = "test-dev-graph")]
    #[test]
    fn print_merkle_chip() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("merkle-path-layout.png", (1024, 7680)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("MerkleCRH Path", ("sans-serif", 60)).unwrap();

        let circuit = MyCircuit::default();
        halo2_proofs::dev::CircuitLayout::default()
            .show_labels(false)
            .render(11, &circuit, &root)
            .unwrap();
    }
}
