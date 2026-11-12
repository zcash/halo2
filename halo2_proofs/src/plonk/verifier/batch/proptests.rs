//! Property tests for the instance-commitment fast path.
//!
//! The reference is `Params::commit_lagrange_instance`, an affine map
//! `x -> W + sum_i x_i G_i` from instance columns into the curve. Each law
//! below is a consequence of that specification; together they pin the
//! fast path to it from several independent directions, so a bug confined to
//! one code path (per-column or lockstep, cached or fallback) has to violate
//! a law that does not mention that path.
//!
//! 1. Recoding inverse: the signed digits recompose to the scalar, as an
//!    integer, with every digit inside the table.
//! 2. Differential: every output equals the reference.
//! 3. Concatenation: `commit_batch` is a monoid homomorphism for `++`, so the
//!    lockstep threshold is invisible.
//! 4. Affinity: `commit(0) = W`, `commit(e_i) = W + G_i`, and
//!    `commit(x + y) + W = commit(x) + commit(y)`.
//! 5. Zero padding: trailing zeros do not change a commitment.
//! 6. Permutation: reordering columns reorders outputs the same way.
//! 7. Cache: every table the cache returns is the multiples table of the
//!    current bases, checked by a linear-time certificate, including after a
//!    base changes under an already-built table.

use std::sync::{Arc, OnceLock};

use ff::{Field, FromUniformBytes, PrimeField};
use pasta_curves::{EpAffine, EqAffine, Fp, Fq};
use proptest::{collection::vec, prelude::*, sample::select};

use super::{signed_fixed_window_digit, signed_instance_window_count, InstanceFixedWindowTable};
use crate::{
    arithmetic::CurveAffine, poly::commitment::Params, INSTANCE_WINDOW_BITS,
    INSTANCE_WINDOW_ENTRIES_PER_BASE, MAX_CACHED_INSTANCE_ROWS,
};

const K: u32 = 7;
// Columns longer than the table take the generic MSM fallback.
const FALLBACK_SLACK: usize = 8;
const MAX_COLUMNS: usize = 8;

/// Scalars weighted towards the values a windowed recoding gets wrong: zero,
/// one, `p - 1`, powers of two and their neighbours (all-ones windows and
/// carries across every window boundary), small negatives, and uniform values.
fn arb_scalar<F: PrimeField + FromUniformBytes<64>>() -> impl Strategy<Value = F> {
    prop_oneof![
        select(vec![
            F::ZERO,
            F::ONE,
            -F::ONE,
            F::ONE.double(),
            -F::ONE.double()
        ]),
        (0u64..(F::NUM_BITS as u64), select(vec![-1i8, 0, 1])).prop_map(|(k, delta)| {
            let power = F::ONE.double().pow_vartime([k]);
            match delta {
                -1 => power - F::ONE,
                1 => power + F::ONE,
                _ => power,
            }
        }),
        any::<u64>().prop_map(|v| -F::from(v)),
        any::<u64>().prop_map(F::from),
        (
            prop::array::uniform32(any::<u8>()),
            prop::array::uniform32(any::<u8>())
        )
            .prop_map(|(low, high)| {
                let mut bytes = [0; 64];
                bytes[..32].copy_from_slice(&low);
                bytes[32..].copy_from_slice(&high);
                F::from_uniform_bytes(&bytes)
            }),
    ]
}

fn arb_column<F: PrimeField + FromUniformBytes<64>>(
    max_len: usize,
) -> impl Strategy<Value = Vec<F>> {
    vec(arb_scalar::<F>(), 0..=max_len)
}

/// A table size, and columns that are mostly cached but sometimes longer than
/// the table, so both the cached and the fallback path run.
fn arb_table_and_columns<F: PrimeField + FromUniformBytes<64>>(
) -> impl Strategy<Value = (usize, Vec<Vec<F>>)> {
    (0..=MAX_CACHED_INSTANCE_ROWS).prop_flat_map(|base_count| {
        (
            Just(base_count),
            vec(
                arb_column::<F>(base_count + FALLBACK_SLACK),
                0..=MAX_COLUMNS,
            ),
        )
    })
}

/// The integer value of the signed digits, reduced modulo `2^128` and modulo
/// `p`. The digits are bounded by `2^8` across `ceil(255 / 9) + 1` windows, so
/// any two candidate values differ by less than `2^262 < p * 2^128`; agreement
/// modulo both coprime moduli is therefore agreement as integers (CRT).
fn recompose<F: PrimeField>(repr: &[u8]) -> (u128, F) {
    let mut low = 0u128;
    let mut field = F::ZERO;
    let radix = F::from(1 << INSTANCE_WINDOW_BITS);
    let window_count = signed_instance_window_count(F::NUM_BITS as usize);
    for window in (0..window_count).rev() {
        let digit = signed_fixed_window_digit(repr, window, INSTANCE_WINDOW_BITS);
        assert!(digit.magnitude <= INSTANCE_WINDOW_ENTRIES_PER_BASE);
        assert!(digit.magnitude != 0 || !digit.negative);
        let magnitude = F::from(digit.magnitude as u64);
        field = field * radix
            + if digit.negative {
                -magnitude
            } else {
                magnitude
            };
        let shift = window * INSTANCE_WINDOW_BITS;
        if shift < 128 {
            let term = (digit.magnitude as u128) << shift;
            low = if digit.negative {
                low.wrapping_sub(term)
            } else {
                low.wrapping_add(term)
            };
        }
    }
    (low, field)
}

fn reference<C: CurveAffine>(params: &Params<C>, columns: &[Vec<C::Scalar>]) -> Vec<C::Curve> {
    columns
        .iter()
        .map(|column| params.commit_lagrange_instance(column))
        .collect()
}

fn fast<C: CurveAffine>(
    params: &Params<C>,
    base_count: usize,
    columns: &[Vec<C::Scalar>],
) -> Vec<C::Curve> {
    let table = InstanceFixedWindowTable::new(params, base_count);
    let slices = columns.iter().map(Vec::as_slice).collect::<Vec<_>>();
    table.commit_batch(params, &slices)
}

/// The same commitments, one column per call, which forces the per-column
/// (unsigned window) path rather than the lockstep path.
fn fast_one_by_one<C: CurveAffine>(
    params: &Params<C>,
    base_count: usize,
    columns: &[Vec<C::Scalar>],
) -> Vec<C::Curve> {
    let table = InstanceFixedWindowTable::new(params, base_count);
    columns
        .iter()
        .flat_map(|column| table.commit_batch(params, &[column.as_slice()]))
        .collect()
}

/// Linear-time certificate that `table` holds `G_i, 2 G_i, ..., 256 G_i` for
/// the first `base_count` bases: the first multiple is the base and each next
/// one differs from it by the base.
fn certify_table<C: CurveAffine>(params: &Params<C>, table: &[C], base_count: usize) {
    assert!(table.len() >= base_count * INSTANCE_WINDOW_ENTRIES_PER_BASE);
    for (base, multiples) in params.g_lagrange[..base_count]
        .iter()
        .zip(table.chunks(INSTANCE_WINDOW_ENTRIES_PER_BASE))
    {
        assert_eq!(multiples[0], *base);
        for pair in multiples.windows(2) {
            assert_eq!(C::Curve::from(pair[1]) - pair[0], C::Curve::from(*base));
        }
    }
}

macro_rules! curve_laws {
    ($module:ident, $curve:ty, $scalar:ty) => {
        mod $module {
            use super::*;
            use group::{Curve as _, CurveAffine as _, Group as _};

            type C = $curve;
            type S = $scalar;

            fn params() -> &'static Params<C> {
                static PARAMS: OnceLock<Params<C>> = OnceLock::new();
                PARAMS.get_or_init(|| Params::new(K))
            }

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(512))]

                #[test]
                fn recoding_inverts_to_the_integer(x in arb_scalar::<S>()) {
                    let repr = x.to_repr();
                    let (low, field) = recompose::<S>(repr.as_ref());
                    let expected_low = u128::from_le_bytes(repr.as_ref()[..16].try_into().unwrap());
                    prop_assert_eq!(low, expected_low);
                    prop_assert_eq!(field, x);
                }
            }

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(64))]

                #[test]
                fn matches_the_reference((base_count, columns) in arb_table_and_columns::<S>()) {
                    let params = params();
                    let expected = reference(params, &columns);
                    prop_assert_eq!(fast(params, base_count, &columns), expected.clone());
                    prop_assert_eq!(fast_one_by_one(params, base_count, &columns), expected);
                }

                #[test]
                fn concatenation_is_preserved(
                    (base_count, left) in arb_table_and_columns::<S>(),
                    right in vec(arb_column::<S>(FALLBACK_SLACK), 0..=MAX_COLUMNS),
                ) {
                    let params = params();
                    let mut joined = left.clone();
                    joined.extend(right.iter().cloned());
                    let mut separate = fast(params, base_count, &left);
                    separate.extend(fast(params, base_count, &right));
                    prop_assert_eq!(fast(params, base_count, &joined), separate);
                }

                #[test]
                fn commitment_is_affine(
                    (base_count, x, y, index) in (1..=MAX_CACHED_INSTANCE_ROWS)
                        .prop_flat_map(|len| (
                            Just(len),
                            vec(arb_scalar::<S>(), len),
                            vec(arb_scalar::<S>(), len),
                            0..len,
                        )),
                ) {
                    let params = params();
                    let sum = x.iter().zip(&y).map(|(a, b)| *a + b).collect::<Vec<_>>();
                    let zero = vec![S::ZERO; base_count];
                    let mut unit = zero.clone();
                    unit[index] = S::ONE;
                    let w = params.w.to_curve();
                    let columns = vec![x, y, sum, zero, unit];
                    for out in [fast(params, base_count, &columns), fast_one_by_one(params, base_count, &columns)] {
                        prop_assert_eq!(out[2] + w, out[0] + out[1]);
                        prop_assert_eq!(out[3], w);
                        prop_assert_eq!(out[4], w + params.g_lagrange[index]);
                    }
                }

                #[test]
                fn trailing_zeros_are_invisible(
                    (base_count, column, pad) in (1..=MAX_CACHED_INSTANCE_ROWS)
                        .prop_flat_map(|base_count| (
                            Just(base_count),
                            arb_column::<S>(base_count),
                            0..=FALLBACK_SLACK,
                        )),
                ) {
                    let params = params();
                    let mut padded = column.clone();
                    padded.resize(column.len() + pad, S::ZERO);
                    // Interleave so both columns run through the lockstep path
                    // together, and padding may push one over the table size.
                    let columns = vec![column.clone(), padded.clone(), column, padded];
                    for out in [fast(params, base_count, &columns), fast_one_by_one(params, base_count, &columns)] {
                        prop_assert_eq!(out[0], out[1]);
                        prop_assert_eq!(out[2], out[3]);
                    }
                }

                #[test]
                fn permutation_is_equivariant(
                    ((base_count, columns), order) in arb_table_and_columns::<S>()
                        .prop_flat_map(|(base_count, columns)| {
                            let order = (0..columns.len()).collect::<Vec<_>>();
                            (Just((base_count, columns)), Just(order).prop_shuffle())
                        }),
                ) {
                    let params = params();
                    let permuted = order.iter().map(|&i| columns[i].clone()).collect::<Vec<_>>();
                    let out = fast(params, base_count, &columns);
                    let permuted_out = fast(params, base_count, &permuted);
                    for (position, &i) in order.iter().enumerate() {
                        prop_assert_eq!(permuted_out[position], out[i]);
                    }
                }
            }

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(16))]

                #[test]
                fn cache_returns_certified_tables(
                    requests in vec(0..=MAX_CACHED_INSTANCE_ROWS, 1..8),
                ) {
                    // A fresh cache per case, so growth from empty is covered.
                    let params = Params::<C>::new(K);
                    let mut previous: Option<(usize, Arc<Vec<C>>)> = None;
                    for base_count in requests {
                        let table = params.instance_window_table(base_count);
                        certify_table(&params, &table, base_count);
                        if let Some((previous_count, previous_table)) = &previous {
                            if base_count <= *previous_count {
                                // A request the cache already covers reuses it.
                                prop_assert!(table.len() >= previous_table.len());
                            }
                        }
                        prop_assert!(Arc::ptr_eq(&table, &params.instance_window_table(base_count)));
                        previous = Some((base_count, table));
                    }
                }

                #[test]
                fn stale_tables_are_rebuilt(
                    (base_count, index, column) in (1..=MAX_CACHED_INSTANCE_ROWS)
                        .prop_flat_map(|base_count| (
                            Just(base_count),
                            0..base_count,
                            arb_column::<S>(base_count),
                        )),
                ) {
                    // A clone shares the cache, so it sees a table built from
                    // the bases as they were before the change.
                    let original = Params::<C>::new(K);
                    let _ = original.instance_window_table(base_count);
                    let mut params = original.clone();
                    params.g_lagrange[index] = params.g_lagrange[index].to_curve().double().to_affine();

                    certify_table(&params, &params.instance_window_table(base_count), base_count);
                    prop_assert_eq!(
                        fast(&params, base_count, &[column.clone()]),
                        reference(&params, &[column]),
                    );
                    // The original still gets a table for its own bases.
                    certify_table(&original, &original.instance_window_table(base_count), base_count);
                }
            }
        }
    };
}

curve_laws!(vesta, EqAffine, Fp);
curve_laws!(pallas, EpAffine, Fq);
