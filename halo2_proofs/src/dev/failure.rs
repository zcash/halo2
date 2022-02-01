use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::iter;

use group::ff::Field;

use super::{metadata, util, Region};
use crate::plonk::{Any, Column, ConstraintSystem, Expression, Gate};

/// The location within the circuit at which a particular [`VerifyFailure`] occurred.
#[derive(Debug, PartialEq)]
pub enum FailureLocation {
    /// A location inside a region.
    InRegion {
        /// The region in which the failure occurred.
        region: metadata::Region,
        /// The offset (relative to the start of the region) at which the failure
        /// occurred.
        offset: usize,
    },
    /// A location outside of a region.
    OutsideRegion {
        /// The circuit row on which the failure occurred.
        row: usize,
    },
}

impl fmt::Display for FailureLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InRegion { region, offset } => write!(f, "in {} at offset {}", region, offset),
            Self::OutsideRegion { row } => {
                write!(f, "on row {}", row)
            }
        }
    }
}

impl FailureLocation {
    pub(super) fn find_expressions<'a, F: Field>(
        cs: &ConstraintSystem<F>,
        regions: &[Region],
        failure_row: usize,
        failure_expressions: impl Iterator<Item = &'a Expression<F>>,
    ) -> Self {
        let failure_columns: HashSet<Column<Any>> = failure_expressions
            .flat_map(|expression| {
                expression.evaluate(
                    &|_| vec![],
                    &|_| panic!("virtual selectors are removed during optimization"),
                    &|index, _, _| vec![cs.fixed_queries[index].0.into()],
                    &|index, _, _| vec![cs.advice_queries[index].0.into()],
                    &|index, _, _| vec![cs.instance_queries[index].0.into()],
                    &|a| a,
                    &|mut a, mut b| {
                        a.append(&mut b);
                        a
                    },
                    &|mut a, mut b| {
                        a.append(&mut b);
                        a
                    },
                    &|a, _| a,
                )
            })
            .collect();

        Self::find(regions, failure_row, failure_columns)
    }

    /// Figures out whether the given row and columns overlap an assigned region.
    fn find(regions: &[Region], failure_row: usize, failure_columns: HashSet<Column<Any>>) -> Self {
        regions
            .iter()
            .enumerate()
            .find(|(_, r)| {
                let (start, end) = r.rows.unwrap();
                // We match the region if any input columns overlap, rather than all of
                // them, because matching complex selector columns is hard. As long as
                // regions are rectangles, and failures occur due to assignments entirely
                // within single regions, "any" will be equivalent to "all". If these
                // assumptions change, we'll start getting bug reports from users :)
                (start..=end).contains(&failure_row) && !failure_columns.is_disjoint(&r.columns)
            })
            .map(|(r_i, r)| FailureLocation::InRegion {
                region: (r_i, r.name.clone()).into(),
                offset: failure_row as usize - r.rows.unwrap().0 as usize,
            })
            .unwrap_or_else(|| FailureLocation::OutsideRegion {
                row: failure_row as usize,
            })
    }
}

/// The reasons why a particular circuit is not satisfied.
#[derive(Debug, PartialEq)]
pub enum VerifyFailure {
    /// A cell used in an active gate was not assigned to.
    CellNotAssigned {
        /// The index of the active gate.
        gate: metadata::Gate,
        /// The region in which this cell should be assigned.
        region: metadata::Region,
        /// The column in which this cell should be assigned.
        column: Column<Any>,
        /// The offset (relative to the start of the region) at which this cell should be
        /// assigned. This may be negative (for example, if a selector enables a gate at
        /// offset 0, but the gate uses `Rotation::prev()`).
        offset: isize,
    },
    /// A constraint was not satisfied for a particular row.
    ConstraintNotSatisfied {
        /// The polynomial constraint that is not satisfied.
        constraint: metadata::Constraint,
        /// The location at which this constraint is not satisfied.
        ///
        /// `FailureLocation::OutsideRegion` is usually caused by a constraint that does
        /// not contain a selector, and as a result is active on every row.
        location: FailureLocation,
        /// The values of the virtual cells used by this constraint.
        cell_values: Vec<(metadata::VirtualCell, String)>,
    },
    /// A constraint was active on an unusable row, and is likely missing a selector.
    ConstraintPoisoned {
        /// The polynomial constraint that is not satisfied.
        constraint: metadata::Constraint,
    },
    /// A lookup input did not exist in its corresponding table.
    Lookup {
        /// The index of the lookup that is not satisfied. These indices are assigned in
        /// the order in which `ConstraintSystem::lookup` is called during
        /// `Circuit::configure`.
        lookup_index: usize,
        /// The location at which the lookup is not satisfied.
        ///
        /// `FailureLocation::InRegion` is most common, and may be due to the intentional
        /// use of a lookup (if its inputs are conditional on a complex selector), or an
        /// unintentional lookup constraint that overlaps the region (indicating that the
        /// lookup's inputs should be made conditional).
        ///
        /// `FailureLocation::OutsideRegion` is uncommon, and could mean that:
        /// - The input expressions do not correctly constrain a default value that exists
        ///   in the table when the lookup is not being used.
        /// - The input expressions use a column queried at a non-zero `Rotation`, and the
        ///   lookup is active on a row adjacent to an unrelated region.
        location: FailureLocation,
    },
    /// A permutation did not preserve the original value of a cell.
    Permutation {
        /// The column in which this permutation is not satisfied.
        column: metadata::Column,
        /// The row on which this permutation is not satisfied.
        row: usize,
    },
}

impl fmt::Display for VerifyFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CellNotAssigned {
                gate,
                region,
                column,
                offset,
            } => {
                write!(
                    f,
                    "{} uses {}, which requires cell in column {:?} at offset {} to be assigned.",
                    region, gate, column, offset
                )
            }
            Self::ConstraintNotSatisfied {
                constraint,
                location,
                cell_values,
            } => {
                writeln!(f, "{} is not satisfied {}", constraint, location)?;
                for (name, value) in cell_values {
                    writeln!(f, "- {} = {}", name, value)?;
                }
                Ok(())
            }
            Self::ConstraintPoisoned { constraint } => {
                write!(
                    f,
                    "{} is active on an unusable row - missing selector?",
                    constraint
                )
            }
            Self::Lookup {
                lookup_index,
                location,
            } => write!(f, "Lookup {} is not satisfied {}", lookup_index, location),
            Self::Permutation { column, row } => {
                write!(
                    f,
                    "Equality constraint not satisfied by cell ({:?}, {})",
                    column, row
                )
            }
        }
    }
}

/// Renders `VerifyFailure::ConstraintNotSatisfied`.
///
/// ```text
/// error: constraint not satisfied
///   Cell layout in region 'somewhere':
///     | Offset | A0 |
///     +--------+----+
///     |    0   | x0 | <--{ Gate 'foo' applied here
///     |    1   | x1 |
///
///   Constraint 'bar':
///     x1 + x1 * 0x100 + x1 * 0x10000 + x1 * 0x100_0000 - x0 = 0
///
///   Assigned cell values:
///     x0 = 0x5
///     x1 = 0x5
/// ```
fn render_constraint_not_satisfied<F: Field>(
    gates: &[Gate<F>],
    constraint: &metadata::Constraint,
    location: &FailureLocation,
    cell_values: &[(metadata::VirtualCell, String)],
) {
    // Collect the necessary rendering information:
    // - The columns involved in this constraint.
    // - How many cells are in each column.
    // - The grid of cell values, indexed by rotation.
    let mut columns = BTreeMap::<metadata::Column, usize>::default();
    let mut layout = BTreeMap::<i32, BTreeMap<metadata::Column, usize>>::default();
    for (i, (cell, _)) in cell_values.iter().enumerate() {
        *columns.entry(cell.column).or_default() += 1;
        layout
            .entry(cell.rotation)
            .or_default()
            .entry(cell.column)
            .or_insert(i);
    }

    let col_width = |cells: usize| cells.to_string().len() + 3;
    let padded = |p: char, width: usize, text: &str| {
        let pad = width - text.len();
        format!(
            "{}{}{}",
            iter::repeat(p).take(pad - pad / 2).collect::<String>(),
            text,
            iter::repeat(p).take(pad / 2).collect::<String>(),
        )
    };

    eprintln!("error: constraint not satisfied");

    // If we are in a region, show rows at offsets relative to it. Otherwise, just show
    // the rotations directly.
    let start = match location {
        FailureLocation::InRegion { region, offset } => {
            eprintln!("  Cell layout in region '{}':", region.name);
            eprint!("    | Offset |");
            *offset as i32
        }
        FailureLocation::OutsideRegion { row } => {
            eprintln!("  Cell layout at row {}:", row);
            eprint!("    |Rotation|");
            0
        }
    };

    // Print the assigned cells, and their region offset or rotation.
    for (column, cells) in &columns {
        let width = col_width(*cells);
        eprint!(
            "{}|",
            padded(
                ' ',
                width,
                &format!(
                    "{}{}",
                    match column.column_type {
                        Any::Advice => "A",
                        Any::Fixed => "F",
                        Any::Instance => "I",
                    },
                    column.index,
                )
            )
        );
    }
    eprintln!();
    eprint!("    +--------+");
    for cells in columns.values() {
        eprint!("{}+", padded('-', col_width(*cells), ""));
    }
    eprintln!();
    for (rotation, row) in &layout {
        eprint!("    |{}|", padded(' ', 8, &(start + rotation).to_string()));
        for (col, cells) in &columns {
            let width = col_width(*cells);
            eprint!(
                "{}|",
                padded(
                    ' ',
                    width,
                    &row.get(col).map(|i| format!("x{}", i)).unwrap_or_default()
                )
            );
        }
        if *rotation == 0 {
            eprint!(" <--{{ Gate '{}' applied here", constraint.gate.name);
        }
        eprintln!();
    }

    // Print the unsatisfied constraint, in terms of the local variables.
    eprintln!();
    eprintln!("  Constraint '{}':", constraint.name);
    eprintln!(
        "    {} = 0",
        gates[constraint.gate.index].polynomials()[constraint.index].evaluate(
            &util::format_value,
            &|_| panic!("virtual selectors are removed during optimization"),
            &|query, column, rotation| if let Some(i) = layout
                .get(&rotation.0)
                .and_then(|row| row.get(&(Any::Fixed, column).into()))
            {
                format!("x{}", i)
            } else if rotation.0 == 0 {
                // This is most likely a merged selector
                format!("S{}", query)
            } else {
                // No idea how we'd get here...
                format!("F{}@{}", column, rotation.0)
            },
            &|_, column, rotation| format!(
                "x{}",
                layout
                    .get(&rotation.0)
                    .unwrap()
                    .get(&(Any::Advice, column).into())
                    .unwrap()
            ),
            &|_, column, rotation| format!(
                "x{}",
                layout
                    .get(&rotation.0)
                    .unwrap()
                    .get(&(Any::Instance, column).into())
                    .unwrap()
            ),
            &|a| {
                if a.contains(' ') {
                    format!("-({})", a)
                } else {
                    format!("-{}", a)
                }
            },
            &|a, b| {
                if let Some(b) = b.strip_prefix('-') {
                    format!("{} - {}", a, b)
                } else {
                    format!("{} + {}", a, b)
                }
            },
            &|a, b| match (a.contains(' '), b.contains(' ')) {
                (false, false) => format!("{} * {}", a, b),
                (false, true) => format!("{} * ({})", a, b),
                (true, false) => format!("({}) * {}", a, b),
                (true, true) => format!("({}) * ({})", a, b),
            },
            &|a, s| {
                if a.contains(' ') {
                    format!("({}) * {}", a, util::format_value(s))
                } else {
                    format!("{} * {}", a, util::format_value(s))
                }
            },
        )
    );

    // Print the map from local variables to assigned values.
    eprintln!();
    eprintln!("  Assigned cell values:");
    for (i, (_, value)) in cell_values.iter().enumerate() {
        eprintln!("    x{} = {}", i, value);
    }
}

impl VerifyFailure {
    /// Emits this failure in pretty-printed format to stderr.
    pub(super) fn emit<F: Field>(&self, gates: &[Gate<F>]) {
        match self {
            Self::ConstraintNotSatisfied {
                constraint,
                location,
                cell_values,
            } => render_constraint_not_satisfied(gates, constraint, location, cell_values),
            _ => eprintln!("{}", self),
        }
    }
}
