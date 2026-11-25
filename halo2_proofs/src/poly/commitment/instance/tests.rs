use std::sync::Arc;

use ff::Field;
use group::{Curve, Group};
use pasta_curves::{EpAffine, Eq, EqAffine, Fp};
use rand::rngs::SysRng;
use rand_core::UnwrapErr;

use super::{
    commit_instance_columns, table_pays_off, WindowTable, MAX_TABLE_ROWS, MIN_BATCH_NORMALIZE,
};
use crate::{
    arithmetic::{BoothDigit, CurveAffine},
    poly::{
        commitment::{Blind, Params},
        EvaluationDomain,
    },
};

/// Zero, one and minus one at fixed positions, random scalars elsewhere.
fn column<F>(len: usize, seed: usize) -> Vec<F>
where
    F: Field,
{
    let mut rng = UnwrapErr(SysRng);
    (0..len)
        .map(|row| match (row + seed) % 5 {
            0 => F::ZERO,
            1 => F::ONE,
            2 => -F::ONE,
            _ => F::random(&mut rng),
        })
        .collect()
}

fn generic<C>(params: &Params<C>, columns: &[&[C::Scalar]]) -> Vec<C>
where
    C: CurveAffine,
{
    columns
        .iter()
        .map(|column| params.commit_lagrange_instance(column).to_affine())
        .collect()
}

fn check_columns_match_the_generic_path<C>()
where
    C: CurveAffine,
{
    const K: u32 = 8;
    let params = Params::<C>::new(K);
    let domain = EvaluationDomain::new(1, K);
    let lengths = [0, 1, 10, 17, 63, MAX_TABLE_ROWS, MAX_TABLE_ROWS + 1, 127];

    // Column counts around the affine conversion threshold and on both sides
    // of the table's break-even, at both short and maximal row counts.
    let mut took_table = [false; 2];
    for rows in [10, MAX_TABLE_ROWS] {
        let break_even = (1..).find(|&count| table_pays_off(rows, count)).unwrap();
        for count in [
            1,
            MIN_BATCH_NORMALIZE - 1,
            MIN_BATCH_NORMALIZE,
            break_even - 1,
            break_even,
            break_even + 3,
        ] {
            // Mostly `rows`-row columns, plus every boundary length, some of
            // them too long for the table.
            let columns = (0..count)
                .map(|index| {
                    let len = if index % 3 == 0 {
                        lengths[index / 3 % lengths.len()]
                    } else {
                        rows
                    };
                    column::<C::Scalar>(len, index)
                })
                .collect::<Vec<_>>();
            // The path the commitment takes: a cached table that covers the
            // short columns, or a new one if it pays off; else generic.
            let short = columns.iter().filter(|c| c.len() <= MAX_TABLE_ROWS);
            let short_rows = short.clone().map(Vec::len).max().unwrap_or(0);
            let short_count = short.count();
            let cached = params
                .instance_windows
                .get(&params.g_lagrange[..short_rows], false)
                .is_some();
            let table = short_count > 0 && (cached || table_pays_off(short_rows, short_count));
            took_table[usize::from(table)] = true;
            check_columns(&params, &domain, &columns);
        }
    }
    assert_eq!(took_table, [true, true]);
    assert!(commit_instance_columns(&params, &[]).is_empty());
}

fn check_columns<C>(
    params: &Params<C>,
    domain: &EvaluationDomain<C::Scalar>,
    columns: &[Vec<C::Scalar>],
) where
    C: CurveAffine,
{
    let count = columns.len();
    let columns = columns.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let commitments = commit_instance_columns(params, &columns);
    assert_eq!(commitments, generic(params, &columns), "{count} columns");

    for (column, commitment) in columns.iter().zip(&commitments) {
        let mut padded = domain.empty_lagrange();
        for (coefficient, value) in padded.iter_mut().zip(*column) {
            *coefficient = *value;
        }
        assert_eq!(
            *commitment,
            params
                .commit_lagrange(&padded, Blind::default())
                .to_affine()
        );
    }
}

#[test]
fn columns_match_the_generic_path() {
    check_columns_match_the_generic_path::<EqAffine>();
    check_columns_match_the_generic_path::<EpAffine>();
}

fn check_table_handles_exceptional_points<C>()
where
    C: CurveAffine,
{
    // Bases that make the table's additions hit the identity and doublings.
    let g = C::Curve::generator().to_affine();
    let bases = [C::identity(), g, -g, g];
    let table = WindowTable::build(&bases);
    let (zero, one) = (C::Scalar::ZERO, C::Scalar::ONE);
    let columns = [
        vec![],
        vec![zero; 4],
        vec![one; 4],
        vec![zero, one, one, zero],
        vec![one, -one, zero, one],
        vec![-one; 4],
    ];
    let columns = columns.iter().map(Vec::as_slice).collect::<Vec<_>>();
    for blind in [C::identity(), g, -g] {
        for (column, commitment) in columns.iter().zip(table.commit(blind, &columns)) {
            let expected = column
                .iter()
                .zip(&bases)
                .fold(blind.to_curve(), |acc, (scalar, base)| acc + *base * scalar);
            assert_eq!(commitment, expected);
            assert_eq!(table.commit(blind, &[column])[0], expected);
        }
    }
}

#[test]
fn table_handles_exceptional_points() {
    check_table_handles_exceptional_points::<EqAffine>();
    check_table_handles_exceptional_points::<EpAffine>();
}

#[test]
fn table_multiples_follow_the_digit() {
    let g = <EqAffine as CurveAffine>::CurveExt::generator().to_affine();
    let table = WindowTable::build(&[g, (g * Fp::from(3)).to_affine()]);
    assert_eq!(table.rows(), 2);
    for base in 0..2 {
        let g_base = table.multiples[base * super::MULTIPLES_PER_BASE];
        assert!(table
            .multiple(
                base,
                BoothDigit {
                    magnitude: 0,
                    negative: true,
                }
            )
            .is_none());
        for magnitude in [1, 2, 255, super::MULTIPLES_PER_BASE] {
            let scalar = Fp::from(u64::try_from(magnitude).unwrap());
            for negative in [false, true] {
                let expected = if negative {
                    -(g_base * scalar)
                } else {
                    g_base * scalar
                };
                let digit = BoothDigit {
                    magnitude,
                    negative,
                };
                assert_eq!(
                    Eq::from(table.multiple(base, digit).unwrap()),
                    expected,
                    "base {base}, digit {digit:?}"
                );
            }
        }
    }
}

/// Columns of `rows` rows, enough of them that a table pays off.
fn enough_columns(rows: usize) -> Vec<Vec<Fp>> {
    let count = (1..).find(|&count| table_pays_off(rows, count)).unwrap();
    (0..count).map(|index| column::<Fp>(rows, index)).collect()
}

fn commit_all(params: &Params<EqAffine>, columns: &[Vec<Fp>]) -> Vec<EqAffine> {
    let columns = columns.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let commitments = commit_instance_columns(params, &columns);
    assert_eq!(commitments, generic(params, &columns));
    commitments
}

#[test]
fn clones_share_the_cached_table_and_copies_do_not() {
    const ROWS: usize = 10;
    let params = Params::<EqAffine>::new(7);
    let debug = format!("{params:?}");
    let mut serialized = vec![];
    params.write(&mut serialized).unwrap();
    let bases = &params.g_lagrange[..ROWS];

    // Too few columns build nothing; enough build and cache a table.
    commit_all(&params, &[column(ROWS, 0)]);
    assert!(params.instance_windows.get(bases, false).is_none());
    commit_all(&params, &enough_columns(ROWS));
    let table = params.instance_windows.get(bases, false).unwrap();

    let cloned = params.clone();
    assert!(Arc::ptr_eq(
        &table,
        &cloned.instance_windows.get(bases, false).unwrap()
    ));
    let copy = Params::<EqAffine>::read(&mut serialized.as_slice()).unwrap();
    assert!(copy.instance_windows.get(bases, false).is_none());

    assert_eq!(format!("{params:?}"), debug);
    let mut after = vec![];
    params.write(&mut after).unwrap();
    assert_eq!(after, serialized);
}

#[test]
fn concurrent_callers_share_one_build() {
    let params = Arc::new(Params::<EqAffine>::new(7));
    let tables = std::thread::scope(|scope| {
        let threads = (0..4)
            .map(|_| {
                let params = Arc::clone(&params);
                scope.spawn(move || {
                    params
                        .instance_windows
                        .get(&params.g_lagrange[..10], true)
                        .unwrap()
                })
            })
            .collect::<Vec<_>>();
        threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(tables.iter().all(|table| Arc::ptr_eq(&tables[0], table)));
}

#[test]
fn a_larger_table_serves_smaller_requests() {
    let params = Params::<EqAffine>::new(7);
    let cache = &params.instance_windows;
    let small = cache.get(&params.g_lagrange[..10], true).unwrap();
    assert!(Arc::ptr_eq(
        &small,
        &cache.get(&params.g_lagrange[..9], false).unwrap()
    ));
    assert!(cache.get(&params.g_lagrange[..11], false).is_none());

    let large = cache
        .get(&params.g_lagrange[..MAX_TABLE_ROWS], true)
        .unwrap();
    assert_eq!(large.rows(), MAX_TABLE_ROWS);
    assert_eq!(
        &large.multiples[..small.multiples.len()],
        small.multiples.as_slice()
    );
    assert!(Arc::ptr_eq(
        &large,
        &cache.get(&params.g_lagrange[..10], false).unwrap()
    ));
}

#[test]
fn a_table_for_other_bases_is_not_used() {
    const ROWS: usize = 10;
    let mut params = Params::<EqAffine>::new(7);
    commit_all(&params, &enough_columns(ROWS));
    assert!(params
        .instance_windows
        .get(&params.g_lagrange[..ROWS], false)
        .is_some());

    // Clones share the cache, so a clone whose bases differ must not use it.
    params.g_lagrange.swap(0, 1);
    assert!(params
        .instance_windows
        .get(&params.g_lagrange[..ROWS], false)
        .is_none());
    commit_all(&params, &enough_columns(ROWS));
    commit_all(&params, &[column(ROWS, 3)]);
}
