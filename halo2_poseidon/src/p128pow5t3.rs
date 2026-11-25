use alloc::vec::Vec;

use ff::Field;
use pasta_curves::{pallas::Base as Fp, vesta::Base as Fq};

use super::{
    mds::{PrecomputedT3, SparsePartialRound},
    Mds, Spec, State,
};

#[inline(always)]
fn pow5<F: Field>(val: F) -> F {
    let sq = val.square();
    sq.square() * val
}

macro_rules! derive_pasta_precomputed_t3 {
    ($fn_name:ident, $F:ty, $exp_minus_2:expr) => {
        const fn $fn_name(
            mds: Mds<$F, 3>,
            round_constants: [[$F; 3]; 64],
        ) -> PrecomputedT3<$F, 4, 56> {
            const fn invert_const(x: $F) -> $F {
                let exp: [u64; 4] = $exp_minus_2;
                let mut res = <$F>::one();
                let mut limb_idx = 4;
                while limb_idx > 0 {
                    limb_idx -= 1;
                    let limb = exp[limb_idx];
                    let mut bit = 64;
                    while bit > 0 {
                        bit -= 1;
                        res = res.square();
                        if ((limb >> bit) & 1) == 1 {
                            res = res.mul(&x);
                        }
                    }
                }
                res
            }

            const fn mat_vec(m: &Mds<$F, 3>, x: [$F; 3]) -> [$F; 3] {
                [
                    m[0][0]
                        .mul(&x[0])
                        .add(&m[0][1].mul(&x[1]))
                        .add(&m[0][2].mul(&x[2])),
                    m[1][0]
                        .mul(&x[0])
                        .add(&m[1][1].mul(&x[1]))
                        .add(&m[1][2].mul(&x[2])),
                    m[2][0]
                        .mul(&x[0])
                        .add(&m[2][1].mul(&x[1]))
                        .add(&m[2][2].mul(&x[2])),
                ]
            }

            const fn mat_mul(a: &Mds<$F, 3>, b: &Mds<$F, 3>) -> Mds<$F, 3> {
                let mut out = [[<$F>::zero(); 3]; 3];
                let mut i = 0;
                while i < 3 {
                    let mut j = 0;
                    while j < 3 {
                        out[i][j] = a[i][0]
                            .mul(&b[0][j])
                            .add(&a[i][1].mul(&b[1][j]))
                            .add(&a[i][2].mul(&b[2][j]));
                        j += 1;
                    }
                    i += 1;
                }
                out
            }

            let mut rc_work = round_constants;
            let mut sparse_rc = [<$F>::zero(); 56];
            let mut idx = 0;
            while idx < 56 {
                let r = 4 + idx;
                sparse_rc[idx] = rc_work[r][0];
                let pushed = mat_vec(&mds, [<$F>::zero(), rc_work[r][1], rc_work[r][2]]);
                rc_work[r + 1][0] = rc_work[r + 1][0].add(&pushed[0]);
                rc_work[r + 1][1] = rc_work[r + 1][1].add(&pushed[1]);
                rc_work[r + 1][2] = rc_work[r + 1][2].add(&pushed[2]);
                idx += 1;
            }

            let mut m00_arr = [<$F>::zero(); 56];
            let mut w_arr = [[<$F>::zero(); 2]; 56];
            let mut v_arr = [[<$F>::zero(); 2]; 56];
            let mut h_arr = [[<$F>::zero(); 4]; 56];
            let mut dets = [<$F>::zero(); 56];

            let mut acc = mds;
            let mut step = 56;
            while step > 0 {
                step -= 1;
                let h00 = acc[1][1];
                let h01 = acc[1][2];
                let h10 = acc[2][1];
                let h11 = acc[2][2];

                m00_arr[step] = acc[0][0];
                w_arr[step] = [acc[0][1], acc[0][2]];
                v_arr[step] = [acc[1][0], acc[2][0]];
                h_arr[step] = [h00, h01, h10, h11];
                dets[step] = h00.mul(&h11).sub(&h01.mul(&h10));

                let p_mat = [
                    [<$F>::one(), <$F>::zero(), <$F>::zero()],
                    [<$F>::zero(), h00, h01],
                    [<$F>::zero(), h10, h11],
                ];
                acc = mat_mul(&p_mat, &mds);
            }

            // Batch-invert all 56 2x2 submatrix determinants using a single exponentiation.
            let mut prefixes = [<$F>::one(); 56];
            let mut prod = <$F>::one();
            let mut i = 0;
            while i < 56 {
                prefixes[i] = prod;
                prod = prod.mul(&dets[i]);
                i += 1;
            }
            let mut inv_prod = invert_const(prod);
            let mut inv_dets = [<$F>::zero(); 56];
            let mut i = 56;
            while i > 0 {
                i -= 1;
                inv_dets[i] = inv_prod.mul(&prefixes[i]);
                inv_prod = inv_prod.mul(&dets[i]);
            }

            let mut partial_rounds = [SparsePartialRound {
                rc: <$F>::zero(),
                m00: <$F>::zero(),
                w_hat: [<$F>::zero(); 2],
                v: [<$F>::zero(); 2],
            }; 56];
            let mut i = 0;
            while i < 56 {
                let w = w_arr[i];
                let h = h_arr[i];
                let inv_det = inv_dets[i];
                let w_hat = [
                    w[0].mul(&h[3]).sub(&w[1].mul(&h[2])).mul(&inv_det),
                    w[1].mul(&h[0]).sub(&w[0].mul(&h[1])).mul(&inv_det),
                ];
                partial_rounds[i] = SparsePartialRound {
                    rc: sparse_rc[i],
                    m00: m00_arr[i],
                    w_hat,
                    v: v_arr[i],
                };
                i += 1;
            }

            let initial_round_constants = [rc_work[0], rc_work[1], rc_work[2], rc_work[3]];
            let terminal_round_constants = [rc_work[60], rc_work[61], rc_work[62], rc_work[63]];

            PrecomputedT3 {
                mds,
                pre_sparse_mds: acc,
                initial_round_constants,
                partial_rounds,
                terminal_round_constants,
            }
        }
    };
}

// p - 2 = 0x40000000000000000000000000000000224698fc094cf91b992d30ecffffffff
derive_pasta_precomputed_t3!(
    derive_precomputed_fp,
    Fp,
    [
        0x992d_30ec_ffff_ffff,
        0x2246_98fc_094c_f91b,
        0x0000_0000_0000_0000,
        0x4000_0000_0000_0000,
    ]
);

// q - 2 = 0x40000000000000000000000000000000224698fc0994a8dd8c46eb20ffffffff
derive_pasta_precomputed_t3!(
    derive_precomputed_fq,
    Fq,
    [
        0x8c46_eb20_ffff_ffff,
        0x2246_98fc_0994_a8dd,
        0x0000_0000_0000_0000,
        0x4000_0000_0000_0000,
    ]
);

static PRECOMPUTED_FP: PrecomputedT3<Fp, 4, 56> =
    derive_precomputed_fp(super::fp::MDS, super::fp::ROUND_CONSTANTS);
static PRECOMPUTED_FQ: PrecomputedT3<Fq, 4, 56> =
    derive_precomputed_fq(super::fq::MDS, super::fq::ROUND_CONSTANTS);

/// Poseidon-128 using the $x^5$ S-box, with a width of 3 field elements, and the
/// standard number of rounds for 128-bit security "with margin".
///
/// The standard specification for this set of parameters (on either of the Pasta
/// fields) uses $R_F = 8, R_P = 56$. This is conveniently an even number of
/// partial rounds, making it easier to construct a Halo 2 circuit.
#[derive(Debug)]
pub struct P128Pow5T3;

impl Spec<Fp, 3, 2> for P128Pow5T3 {
    fn full_rounds() -> usize {
        8
    }

    fn partial_rounds() -> usize {
        56
    }

    #[inline(always)]
    fn sbox(val: Fp) -> Fp {
        pow5(val)
    }

    fn secure_mds() -> usize {
        unimplemented!()
    }

    fn constants() -> (Vec<[Fp; 3]>, Mds<Fp, 3>, Mds<Fp, 3>) {
        (
            super::fp::ROUND_CONSTANTS[..].to_vec(),
            super::fp::MDS,
            super::fp::MDS_INV,
        )
    }

    #[inline]
    fn try_permute(state: State<Fp, 3>) -> Option<State<Fp, 3>> {
        Some(PRECOMPUTED_FP.permute(state, Self::sbox))
    }
}

impl Spec<Fq, 3, 2> for P128Pow5T3 {
    fn full_rounds() -> usize {
        8
    }

    fn partial_rounds() -> usize {
        56
    }

    #[inline(always)]
    fn sbox(val: Fq) -> Fq {
        pow5(val)
    }

    fn secure_mds() -> usize {
        unimplemented!()
    }

    fn constants() -> (Vec<[Fq; 3]>, Mds<Fq, 3>, Mds<Fq, 3>) {
        (
            super::fq::ROUND_CONSTANTS[..].to_vec(),
            super::fq::MDS,
            super::fq::MDS_INV,
        )
    }

    #[inline]
    fn try_permute(state: State<Fq, 3>) -> Option<State<Fq, 3>> {
        Some(PRECOMPUTED_FQ.permute(state, Self::sbox))
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::marker::PhantomData;

    use ff::{Field, FromUniformBytes, PrimeField};

    use super::{
        super::{fp, fq},
        Fp, Fq,
    };
    use crate::{generate_constants, permute, ConstantLength, Hash, Mds, Spec};

    /// The same Poseidon specification as poseidon::P128Pow5T3, but constructed
    /// such that its constants will be generated at runtime.
    #[derive(Debug)]
    pub struct P128Pow5T3Gen<F: Field, const SECURE_MDS: usize>(PhantomData<F>);

    impl<F: Field, const SECURE_MDS: usize> P128Pow5T3Gen<F, SECURE_MDS> {
        #![allow(dead_code)]
        pub fn new() -> Self {
            P128Pow5T3Gen(PhantomData)
        }
    }

    impl<F: FromUniformBytes<64> + Ord, const SECURE_MDS: usize> Spec<F, 3, 2>
        for P128Pow5T3Gen<F, SECURE_MDS>
    {
        fn full_rounds() -> usize {
            8
        }

        fn partial_rounds() -> usize {
            56
        }

        fn sbox(val: F) -> F {
            val.pow_vartime([5])
        }

        fn secure_mds() -> usize {
            SECURE_MDS
        }

        fn constants() -> (Vec<[F; 3]>, Mds<F, 3>, Mds<F, 3>) {
            generate_constants::<_, Self, 3, 2>()
        }
    }

    #[test]
    fn verify_constants() {
        fn verify_constants_helper<F: FromUniformBytes<64> + Ord>(
            expected_round_constants: [[F; 3]; 64],
            expected_mds: [[F; 3]; 3],
            expected_mds_inv: [[F; 3]; 3],
        ) {
            let (round_constants, mds, mds_inv) = P128Pow5T3Gen::<F, 0>::constants();

            for (actual, expected) in round_constants
                .iter()
                .flatten()
                .zip(expected_round_constants.iter().flatten())
            {
                assert_eq!(actual, expected);
            }

            for (actual, expected) in mds.iter().flatten().zip(expected_mds.iter().flatten()) {
                assert_eq!(actual, expected);
            }

            for (actual, expected) in mds_inv
                .iter()
                .flatten()
                .zip(expected_mds_inv.iter().flatten())
            {
                assert_eq!(actual, expected);
            }
        }

        verify_constants_helper(fp::ROUND_CONSTANTS, fp::MDS, fp::MDS_INV);
        verify_constants_helper(fq::ROUND_CONSTANTS, fq::MDS, fq::MDS_INV);

        assert_eq!(
            &super::PRECOMPUTED_FP,
            &crate::mds::derive_precomputed_t3(&fp::MDS, &fp::ROUND_CONSTANTS),
        );
        assert_eq!(
            &super::PRECOMPUTED_FQ,
            &crate::mds::derive_precomputed_t3(&fq::MDS, &fq::ROUND_CONSTANTS),
        );
    }

    #[test]
    fn sparse_precomputed_matches_dense_and_generic_sponge() {
        fn check_field<F: FromUniformBytes<64> + Ord>(
            mds: &Mds<F, 3>,
            round_constants: &[[F; 3]; 64],
            seeds: &[u64],
        ) where
            super::P128Pow5T3: Spec<F, 3, 2>,
        {
            let boundary_states = [
                [F::ZERO, F::ZERO, F::ZERO],
                [F::ONE, F::ZERO, F::ZERO],
                [F::ZERO, F::ONE, F::ZERO],
                [F::ZERO, F::ZERO, F::ONE],
                [-F::ONE, -F::ONE, -F::ONE],
                [F::ONE, -F::ONE, F::from(2)],
            ];

            for initial in boundary_states {
                let mut dense = initial;
                permute::<F, P128Pow5T3Gen<F, 0>, 3, 2>(&mut dense, mds, round_constants);
                let sparse = <super::P128Pow5T3 as Spec<F, 3, 2>>::try_permute(initial).unwrap();
                assert_eq!(sparse, dense);
            }

            let make_elem = |seed: u64| -> F {
                let mut bytes = [0u8; 64];
                let mut x = seed;
                for chunk in bytes.chunks_exact_mut(8) {
                    x = x
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    chunk.copy_from_slice(&x.to_le_bytes());
                }
                F::from_uniform_bytes(&bytes)
            };

            for &seed in seeds {
                let initial = [
                    make_elem(seed),
                    make_elem(seed.wrapping_add(1)),
                    make_elem(seed.wrapping_add(2)),
                ];
                let mut dense = initial;
                permute::<F, P128Pow5T3Gen<F, 0>, 3, 2>(&mut dense, mds, round_constants);
                let sparse = <super::P128Pow5T3 as Spec<F, 3, 2>>::try_permute(initial).unwrap();
                assert_eq!(sparse, dense);

                let m1 = [initial[0]];
                assert_eq!(
                    Hash::<F, super::P128Pow5T3, ConstantLength<1>, 3, 2>::init().hash(m1),
                    Hash::<F, P128Pow5T3Gen<F, 0>, ConstantLength<1>, 3, 2>::init().hash(m1),
                );

                let m2 = [initial[0], initial[1]];
                assert_eq!(
                    Hash::<F, super::P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(m2),
                    Hash::<F, P128Pow5T3Gen<F, 0>, ConstantLength<2>, 3, 2>::init().hash(m2),
                );

                let m4 = [
                    initial[0],
                    initial[1],
                    initial[2],
                    make_elem(seed.wrapping_add(3)),
                ];
                assert_eq!(
                    Hash::<F, super::P128Pow5T3, ConstantLength<4>, 3, 2>::init().hash(m4),
                    Hash::<F, P128Pow5T3Gen<F, 0>, ConstantLength<4>, 3, 2>::init().hash(m4),
                );

                let m5 = [
                    initial[0],
                    initial[1],
                    initial[2],
                    make_elem(seed.wrapping_add(3)),
                    make_elem(seed.wrapping_add(4)),
                ];
                assert_eq!(
                    Hash::<F, super::P128Pow5T3, ConstantLength<5>, 3, 2>::init().hash(m5),
                    Hash::<F, P128Pow5T3Gen<F, 0>, ConstantLength<5>, 3, 2>::init().hash(m5),
                );

                // Verify multi-squeeze beyond RATE = 2 against an independent raw `permute`
                // trajectory (exercises the `input = None` sponge re-permutation leg).
                let cap = <ConstantLength<3> as crate::Domain<F, 2>>::initial_capacity_element();
                let mut fast_sponge = crate::Sponge::<F, super::P128Pow5T3, _, 3, 2>::new(cap);
                let mut ref_sponge = crate::Sponge::<F, P128Pow5T3Gen<F, 0>, _, 3, 2>::new(cap);
                for v in initial
                    .into_iter()
                    .chain(<ConstantLength<3> as crate::Domain<F, 2>>::padding(3))
                {
                    fast_sponge.absorb(v);
                    ref_sponge.absorb(v);
                }
                let mut fast_sq = fast_sponge.finish_absorbing();
                let mut ref_sq = ref_sponge.finish_absorbing();

                let mut raw_state = [initial[0], initial[1], cap];
                permute::<F, P128Pow5T3Gen<F, 0>, 3, 2>(&mut raw_state, mds, round_constants);
                raw_state[0] += initial[2];
                permute::<F, P128Pow5T3Gen<F, 0>, 3, 2>(&mut raw_state, mds, round_constants);
                let s0 = raw_state;
                permute::<F, P128Pow5T3Gen<F, 0>, 3, 2>(&mut raw_state, mds, round_constants);
                let s1 = raw_state;
                permute::<F, P128Pow5T3Gen<F, 0>, 3, 2>(&mut raw_state, mds, round_constants);
                let s2 = raw_state;

                for expected in [s0[0], s0[1], s1[0], s1[1], s2[0]] {
                    assert_eq!(fast_sq.squeeze(), expected);
                    assert_eq!(ref_sq.squeeze(), expected);
                }
            }
        }

        let seeds: Vec<u64> = (0..64)
            .map(|i| 0x9e37_79b9_7f4a_7c15u64.wrapping_mul(i + 1))
            .collect();
        check_field(&fp::MDS, &fp::ROUND_CONSTANTS, &seeds);
        check_field(&fq::MDS, &fq::ROUND_CONSTANTS, &seeds);
    }

    #[test]
    fn test_against_reference() {
        {
            // <https://github.com/daira/pasta-hadeshash>, using parameters from
            // `generate_parameters_grain.sage 1 0 255 3 8 56 0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001`.
            // The test vector is generated by `sage poseidonperm_x5_pallas_3.sage --rust`

            let mut input = [
                Fp::from_raw([
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ]),
                Fp::from_raw([
                    0x0000_0000_0000_0001,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ]),
                Fp::from_raw([
                    0x0000_0000_0000_0002,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ]),
            ];

            let expected_output = [
                Fp::from_raw([
                    0xaeb1_bc02_4aec_a456,
                    0xf7e6_9a71_d0b6_42a0,
                    0x94ef_b364_f966_240f,
                    0x2a52_6acd_0b64_b453,
                ]),
                Fp::from_raw([
                    0x012a_3e96_28e5_b82a,
                    0xdcd4_2e7f_bed9_dafe,
                    0x76ff_7dae_343d_5512,
                    0x13c5_d156_8b4a_a430,
                ]),
                Fp::from_raw([
                    0x3590_29a1_d34e_9ddd,
                    0xf7cf_dfe1_bda4_2c7b,
                    0x256f_cd59_7984_561a,
                    0x0a49_c868_c697_6544,
                ]),
            ];

            permute::<Fp, P128Pow5T3Gen<Fp, 0>, 3, 2>(&mut input, &fp::MDS, &fp::ROUND_CONSTANTS);
            assert_eq!(input, expected_output);
        }

        {
            // <https://github.com/daira/pasta-hadeshash>, using parameters from
            // `generate_parameters_grain.sage 1 0 255 3 8 56 0x40000000000000000000000000000000224698fc0994a8dd8c46eb2100000001`.
            // The test vector is generated by `sage poseidonperm_x5_vesta_3.sage --rust`

            let mut input = [
                Fq::from_raw([
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ]),
                Fq::from_raw([
                    0x0000_0000_0000_0001,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ]),
                Fq::from_raw([
                    0x0000_0000_0000_0002,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                    0x0000_0000_0000_0000,
                ]),
            ];

            let expected_output = [
                Fq::from_raw([
                    0x0eb0_8ea8_13be_be59,
                    0x4d43_d197_3dd3_36c6,
                    0xeddd_74f2_2f8f_2ff7,
                    0x315a_1f4c_db94_2f7c,
                ]),
                Fq::from_raw([
                    0xf9f1_26e6_1ea1_65f1,
                    0x413e_e0eb_7bbd_2198,
                    0x642a_dee0_dd13_aa48,
                    0x3be4_75f2_d764_2bde,
                ]),
                Fq::from_raw([
                    0x14d5_4237_2a7b_a0d9,
                    0x5019_bfd4_e042_3fa0,
                    0x117f_db24_20d8_ea60,
                    0x25ab_8aec_e953_7168,
                ]),
            ];

            permute::<Fq, P128Pow5T3Gen<Fq, 0>, 3, 2>(&mut input, &fq::MDS, &fq::ROUND_CONSTANTS);
            assert_eq!(input, expected_output);
        }
    }

    #[test]
    fn permute_test_vectors() {
        {
            let (round_constants, mds, _) = super::P128Pow5T3::constants();

            for tv in crate::test_vectors::fp::permute() {
                let initial = [
                    Fp::from_repr(tv.initial_state[0]).unwrap(),
                    Fp::from_repr(tv.initial_state[1]).unwrap(),
                    Fp::from_repr(tv.initial_state[2]).unwrap(),
                ];
                let mut state = initial;

                permute::<Fp, super::P128Pow5T3, 3, 2>(&mut state, &mds, &round_constants);
                assert_eq!(
                    <super::P128Pow5T3 as Spec<Fp, 3, 2>>::try_permute(initial),
                    Some(state),
                );

                for (expected, actual) in tv.final_state.iter().zip(state.iter()) {
                    assert_eq!(&actual.to_repr(), expected);
                }
            }
        }

        {
            let (round_constants, mds, _) = super::P128Pow5T3::constants();

            for tv in crate::test_vectors::fq::permute() {
                let initial = [
                    Fq::from_repr(tv.initial_state[0]).unwrap(),
                    Fq::from_repr(tv.initial_state[1]).unwrap(),
                    Fq::from_repr(tv.initial_state[2]).unwrap(),
                ];
                let mut state = initial;

                permute::<Fq, super::P128Pow5T3, 3, 2>(&mut state, &mds, &round_constants);
                assert_eq!(
                    <super::P128Pow5T3 as Spec<Fq, 3, 2>>::try_permute(initial),
                    Some(state),
                );

                for (expected, actual) in tv.final_state.iter().zip(state.iter()) {
                    assert_eq!(&actual.to_repr(), expected);
                }
            }
        }
    }

    #[test]
    fn hash_test_vectors() {
        for tv in crate::test_vectors::fp::hash() {
            let message = [
                Fp::from_repr(tv.input[0]).unwrap(),
                Fp::from_repr(tv.input[1]).unwrap(),
            ];

            let result =
                Hash::<_, super::P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

            assert_eq!(result.to_repr(), tv.output);
        }

        for tv in crate::test_vectors::fq::hash() {
            let message = [
                Fq::from_repr(tv.input[0]).unwrap(),
                Fq::from_repr(tv.input[1]).unwrap(),
            ];

            let result =
                Hash::<_, super::P128Pow5T3, ConstantLength<2>, 3, 2>::init().hash(message);

            assert_eq!(result.to_repr(), tv.output);
        }
    }
}
