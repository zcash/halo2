use halo2::arithmetic::Field;
use pasta_curves::pallas;

use super::{
    fp::{MDS, MDS_INV, ROUND_CONSTANTS},
    Mds, Spec,
};

/// Poseidon-128 using the $x^5$ S-box, with a width of 3 field elements, and the
/// standard number of rounds for 128-bit security "with margin".
///
/// The standard specification for this set of parameters uses $R_F = 8, R_P = 56$.
/// This is conveniently an even number of partial rounds, making it easier to
/// construct a Halo 2 circuit.
#[derive(Debug)]
pub struct P128Pow5T3;

impl Spec<pallas::Base, 3, 2> for P128Pow5T3 {
    fn full_rounds() -> usize {
        8
    }

    fn partial_rounds() -> usize {
        56
    }

    fn sbox(val: pallas::Base) -> pallas::Base {
        val.pow_vartime(&[5])
    }

    fn secure_mds(&self) -> usize {
        unimplemented!()
    }

    fn constants(
        &self,
    ) -> (
        Vec<[pallas::Base; 3]>,
        Mds<pallas::Base, 3>,
        Mds<pallas::Base, 3>,
    ) {
        (ROUND_CONSTANTS[..].to_vec(), MDS, MDS_INV)
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use ff::PrimeField;
    use halo2::arithmetic::FieldExt;
    use pasta_curves::pallas;

    use crate::primitives::poseidon::{permute, ConstantLength, Hash, Spec};

    use super::{MDS, MDS_INV, ROUND_CONSTANTS};

    /// The same Poseidon specification as poseidon::P128Pow5T3, but constructed
    /// such that its constants will be generated at runtime.
    #[derive(Debug)]
    pub struct P128Pow5T3<F: FieldExt> {
        secure_mds: usize,
        _field: PhantomData<F>,
    }

    impl<F: FieldExt> P128Pow5T3<F> {
        pub fn new(secure_mds: usize) -> Self {
            P128Pow5T3 {
                secure_mds,
                _field: PhantomData::default(),
            }
        }
    }

    impl<F: FieldExt> Spec<F, 3, 2> for P128Pow5T3<F> {
        fn full_rounds() -> usize {
            8
        }

        fn partial_rounds() -> usize {
            56
        }

        fn sbox(val: F) -> F {
            val.pow_vartime(&[5])
        }

        fn secure_mds(&self) -> usize {
            self.secure_mds
        }
    }

    #[test]
    fn verify_constants() {
        let poseidon = P128Pow5T3::<pallas::Base>::new(0);
        let (round_constants, mds, mds_inv) = poseidon.constants();

        for (actual, expected) in round_constants
            .iter()
            .flatten()
            .zip(ROUND_CONSTANTS.iter().flatten())
        {
            assert_eq!(actual, expected);
        }

        for (actual, expected) in mds.iter().flatten().zip(MDS.iter().flatten()) {
            assert_eq!(actual, expected);
        }

        for (actual, expected) in mds_inv.iter().flatten().zip(MDS_INV.iter().flatten()) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_against_reference() {
        // This is the test vector output by the reference code at
        // <https://extgit.iaik.tugraz.at/krypto/hadeshash>, using parameters from
        // `generate_parameters_grain.sage 1 0 255 3 8 56 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001`.

        let mut input = [
            pallas::Base::from_raw([
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
            ]),
            pallas::Base::from_raw([
                0x0000_0000_0000_0001,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
            ]),
            pallas::Base::from_raw([
                0x0000_0000_0000_0002,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
            ]),
        ];

        let expected_output = [
            pallas::Base::from_raw([
                0xaeb1_bc02_4aec_a456,
                0xf7e6_9a71_d0b6_42a0,
                0x94ef_b364_f966_240f,
                0x2a52_6acd_0b64_b453,
            ]),
            pallas::Base::from_raw([
                0x012a_3e96_28e5_b82a,
                0xdcd4_2e7f_bed9_dafe,
                0x76ff_7dae_343d_5512,
                0x13c5_d156_8b4a_a430,
            ]),
            pallas::Base::from_raw([
                0x3590_29a1_d34e_9ddd,
                0xf7cf_dfe1_bda4_2c7b,
                0x256f_cd59_7984_561a,
                0x0a49_c868_c697_6544,
            ]),
        ];

        permute::<pallas::Base, P128Pow5T3<pallas::Base>, 3, 2>(&mut input, &MDS, &ROUND_CONSTANTS);
        assert_eq!(input, expected_output);
    }

    #[test]
    fn permute_test_vectors() {
        let (round_constants, mds, _) = super::P128Pow5T3.constants();

        for tv in crate::primitives::poseidon::test_vectors::permute() {
            let mut state = [
                pallas::Base::from_repr(tv.initial_state[0]).unwrap(),
                pallas::Base::from_repr(tv.initial_state[1]).unwrap(),
                pallas::Base::from_repr(tv.initial_state[2]).unwrap(),
            ];

            permute::<pallas::Base, super::P128Pow5T3, 3, 2>(&mut state, &mds, &round_constants);

            for (expected, actual) in tv.final_state.iter().zip(state.iter()) {
                assert_eq!(&actual.to_repr(), expected);
            }
        }
    }

    #[test]
    fn hash_test_vectors() {
        for tv in crate::primitives::poseidon::test_vectors::hash() {
            let message = [
                pallas::Base::from_repr(tv.input[0]).unwrap(),
                pallas::Base::from_repr(tv.input[1]).unwrap(),
            ];

            let result = Hash::init(super::P128Pow5T3, ConstantLength).hash(message);

            assert_eq!(result.to_repr(), tv.output);
        }
    }
}
