//! Commitments to public instance columns.
//!
//! A column `x` commits to `W + sum_i x_i G_i`, where the `G_i` are the
//! parameters' Lagrange bases and the blind on `W` is one: what
//! [`Params::commit_lagrange_instance`] computes for one column. Both verifiers
//! commit through [`commit_instance_columns`].
//!
//! Every column shares the same bases, so a table of small multiples of those
//! bases replaces each column's multi-scalar multiplication by one lookup per
//! signed window digit. The table depends only on the parameters, so it is
//! built once, when enough short columns are committed together for it to pay
//! off, and cached in [`Params`] for every later commitment. All scalars and
//! points here are public.

use std::sync::{Arc, Mutex, PoisonError};

use group::{ff::PrimeField, Group};

use super::Params;
use crate::{
    arithmetic::{booth_digit, booth_window_count, BoothDigit, CurveAffine, CurveExt},
    multicore,
};

#[cfg(test)]
mod tests;

/// Width of the signed window digits read from each scalar.
const WINDOW_BITS: usize = 9;

/// Signed width-nine digits lie in `[-256, 256]`, so the table keeps the
/// multiples `1..=256` of each base and negates them for negative digits.
const MULTIPLES_PER_BASE: usize = 1 << (WINDOW_BITS - 1);

/// Each table row costs `MULTIPLES_PER_BASE` additions and points, so longer
/// columns, which only callers with unusual circuits supply, keep the generic
/// multi-scalar multiplication. This covers Orchard's ten-row columns.
const MAX_TABLE_ROWS: usize = 64;

/// Whether `columns` columns of at most `rows` rows are cheaper to commit
/// through a table built for them than one multi-scalar multiplication each.
///
/// Measured single-threaded on Pasta, building the table costs about 67 us
/// per row, and it saves about `61 + 3.3 rows` us per column, so it pays off
/// once `columns (rows + 18) >= 20 rows`: from 8 columns at Orchard's 10 rows,
/// and from 16 at 64 rows.
pub(crate) fn table_pays_off(rows: usize, columns: usize) -> bool {
    columns * (rows + 18) >= 20 * rows
}

/// Converting fewer points than this to affine form one at a time is cheaper
/// than a batched conversion.
pub(crate) const MIN_BATCH_NORMALIZE: usize = 4;

/// Returns the commitment to each column, in order: `W + sum_i x_i G_i` for
/// a column `x`. Each column must be no longer than `params.g_lagrange`.
pub(crate) fn commit_instance_columns<C>(params: &Params<C>, columns: &[&[C::Scalar]]) -> Vec<C>
where
    C: CurveAffine,
{
    let short = columns
        .iter()
        .enumerate()
        .filter(|(_, column)| column.len() <= MAX_TABLE_ROWS)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    let rows = short
        .iter()
        .map(|&index| columns[index].len())
        .max()
        .unwrap_or(0);
    let mut commitments = vec![C::Curve::identity(); columns.len()];
    let table = if short.is_empty() {
        None
    } else {
        params.instance_windows.get(
            &params.g_lagrange[..rows],
            table_pays_off(rows, short.len()),
        )
    };
    if let Some(table) = table {
        let short_columns = short
            .iter()
            .map(|&index| columns[index])
            .collect::<Vec<_>>();
        for (&index, commitment) in short.iter().zip(table.commit(params.w, &short_columns)) {
            commitments[index] = commitment;
        }
        for (column, commitment) in columns.iter().zip(&mut commitments) {
            if column.len() > MAX_TABLE_ROWS {
                *commitment = params.commit_lagrange_instance(column);
            }
        }
    } else {
        for (column, commitment) in columns.iter().zip(&mut commitments) {
            *commitment = params.commit_lagrange_instance(column);
        }
    }

    to_affine(&commitments)
}

fn to_affine<C>(points: &[C::Curve]) -> Vec<C>
where
    C: CurveAffine,
{
    if points.len() < MIN_BATCH_NORMALIZE {
        return points
            .iter()
            .map(|point| point.to_affine_vartime())
            .collect();
    }
    let mut affine = vec![C::identity(); points.len()];
    C::Curve::batch_normalize_vartime(points, &mut affine);
    affine
}

/// The window table of a set of [`Params`], once built. Clones of the
/// parameters share it.
pub(crate) struct WindowCache<C>(Arc<Mutex<Option<Arc<WindowTable<C>>>>>);

impl<C> Default for WindowCache<C> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

impl<C> Clone for WindowCache<C> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<C> WindowCache<C>
where
    C: CurveAffine,
{
    /// The cached table if it covers `bases`. Otherwise, if `build`, a table
    /// for `bases`, which replaces the cached one; else `None`.
    ///
    /// The lock is held while building, so concurrent callers share one build
    /// and wait for it: callers on a thread pool block their thread for the
    /// build, a few milliseconds once per `Params`.
    fn get(&self, bases: &[C], build: bool) -> Option<Arc<WindowTable<C>>> {
        let mut cached = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(table) = cached.as_ref().filter(|table| table.covers(bases)) {
            return Some(Arc::clone(table));
        }
        if !build {
            return None;
        }
        let table = Arc::new(WindowTable::build(bases));
        *cached = Some(Arc::clone(&table));
        Some(table)
    }
}

/// `j * G_i` for `j` in `1..=MULTIPLES_PER_BASE` and each base `G_i`, stored
/// base by base.
struct WindowTable<C> {
    multiples: Vec<C>,
}

impl<C> WindowTable<C>
where
    C: CurveAffine,
{
    /// Each base's multiples depend only on that base, so the rows are built,
    /// and converted to affine form, in parallel.
    fn build(bases: &[C]) -> Self {
        let mut multiples = vec![C::identity(); bases.len() * MULTIPLES_PER_BASE];
        for_each_chunk(&mut multiples, MULTIPLES_PER_BASE, |rows, first_row| {
            let mut projective = Vec::with_capacity(MULTIPLES_PER_BASE);
            for (row, multiples) in rows.chunks_mut(MULTIPLES_PER_BASE).enumerate() {
                let base = bases[first_row + row];
                let mut multiple = C::Curve::from(base);
                projective.clear();
                projective.push(multiple);
                for _ in 1..MULTIPLES_PER_BASE {
                    multiple += base;
                    projective.push(multiple);
                }
                C::Curve::batch_normalize_vartime(&projective, multiples);
            }
        });
        Self { multiples }
    }

    fn rows(&self) -> usize {
        self.multiples.len() / MULTIPLES_PER_BASE
    }

    /// Whether the table holds the multiples of every base in `bases`: at
    /// least as many rows, each starting with its base. A table built before
    /// the bases changed fails this check and is rebuilt.
    fn covers(&self, bases: &[C]) -> bool {
        self.rows() >= bases.len()
            && self
                .multiples
                .chunks(MULTIPLES_PER_BASE)
                .zip(bases)
                .all(|(multiples, base)| multiples[0] == *base)
    }

    /// `digit * G_base`, or `None` for a zero digit.
    fn multiple(&self, base: usize, digit: BoothDigit) -> Option<C> {
        let magnitude = digit.magnitude.checked_sub(1)?;
        let multiple = self.multiples[base * MULTIPLES_PER_BASE + magnitude];
        Some(if digit.negative { -multiple } else { multiple })
    }

    /// `blind + sum_i x_i G_i` for every column `x`, each no longer than the
    /// table. Horner's rule in radix `2^WINDOW_BITS` runs over a chunk of
    /// columns in lockstep, from the most significant window down; the
    /// columns are independent, so the chunks run in parallel.
    fn commit(&self, blind: C, columns: &[&[C::Scalar]]) -> Vec<C::Curve> {
        let digits = ColumnDigits::new(columns, self.rows());
        let mut commitments = vec![C::Curve::identity(); columns.len()];
        for_each_chunk(&mut commitments, 1, |commitments, first_column| {
            for window in (0..digits.window_count).rev() {
                if window + 1 != digits.window_count {
                    for commitment in commitments.iter_mut() {
                        for _ in 0..WINDOW_BITS {
                            *commitment = commitment.double();
                        }
                    }
                }
                for base in 0..self.rows() {
                    for (column, commitment) in commitments.iter_mut().enumerate() {
                        let digit = digits.get(base, first_column + column, window);
                        if let Some(multiple) = self.multiple(base, digit) {
                            *commitment += multiple;
                        }
                    }
                }
            }
            for commitment in commitments.iter_mut() {
                *commitment += blind;
            }
        });
        commitments
    }
}

/// Splits `items`, a sequence of units of `unit` items each, into one chunk
/// of whole units per thread, and runs `f` on every chunk in parallel with
/// the index of its first unit. Unlike `arithmetic::parallelize`, it splits
/// even when there are fewer units than threads squared: a table has at most
/// 64 rows, and every unit here is worth a thread.
fn for_each_chunk<T, F>(items: &mut [T], unit: usize, f: F)
where
    T: Send,
    F: Fn(&mut [T], usize) + Send + Sync,
{
    let units = items.len() / unit;
    if units == 0 {
        return;
    }
    let units_per_chunk = units.div_ceil(multicore::current_num_threads());
    let f = &f;
    multicore::scope(|scope| {
        for (chunk, items) in items.chunks_mut(units_per_chunk * unit).enumerate() {
            scope.spawn(move |_| f(items, chunk * units_per_chunk));
        }
    });
}

/// The scalars of several columns as byte representations, zero-padded to
/// `rows` and stored row by row, so that one row of every column is
/// contiguous: the order in which `WindowTable::commit` reads them.
struct ColumnDigits<R> {
    reprs: Vec<R>,
    columns: usize,
    window_count: usize,
}

impl<R> ColumnDigits<R>
where
    R: AsRef<[u8]> + Default + Clone,
{
    fn new<F>(columns: &[&[F]], rows: usize) -> Self
    where
        F: PrimeField<Repr = R>,
    {
        let mut reprs = vec![F::ZERO.to_repr(); rows * columns.len()];
        for (column, scalars) in columns.iter().enumerate() {
            assert!(scalars.len() <= rows);
            for (row, scalar) in scalars.iter().enumerate() {
                reprs[row * columns.len() + column] = scalar.to_repr();
            }
        }
        let window_count = booth_window_count(&[F::ZERO.to_repr()], WINDOW_BITS);
        Self {
            reprs,
            columns: columns.len(),
            window_count,
        }
    }

    fn get(&self, row: usize, column: usize, window: usize) -> BoothDigit {
        booth_digit(
            self.reprs[row * self.columns + column].as_ref(),
            WINDOW_BITS,
            window,
        )
    }
}
