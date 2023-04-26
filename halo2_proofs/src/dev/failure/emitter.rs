use std::collections::BTreeMap;
use std::iter;

use group::ff::Field;

use super::FailureLocation;
use crate::{
    dev::{metadata, util},
    plonk::{Advice, Any, Expression},
};

fn padded(p: char, width: usize, text: &str) -> String {
    let pad = width - text.len();

    format!(
        "{}{}{}",
        iter::repeat(p).take(pad - pad / 2).collect::<String>(),
        text,
        iter::repeat(p).take(pad / 2).collect::<String>(),
    )
}

fn column_type_and_idx(column: &metadata::Column) -> String {
    format!(
        "{}{}",
        match column.column_type {
            Any::Advice(_) => "A",
            Any::Fixed => "F",
            Any::Instance => "I",
        },
        column.index
    )
}

/// Renders a cell layout around a given failure location.
///
/// `highlight_row` is called at the end of each row, with the offset of the active row
/// (if `location` is in a region), and the rotation of the current row relative to the
/// active row.
pub(super) fn render_cell_layout(
    prefix: &str,
    location: &FailureLocation,
    columns: &BTreeMap<metadata::Column, usize>,
    layout: &BTreeMap<i32, BTreeMap<metadata::Column, String>>,
    highlight_row: impl Fn(Option<i32>, i32),
) {
    let col_width = |cells: usize| cells.to_string().len() + 3;
    let mut col_headers = String::new();

    // If we are in a region, show rows at offsets relative to it. Otherwise, just show
    // the rotations directly.
    let offset = match location {
        FailureLocation::InRegion { region, offset } => {
            col_headers
                .push_str(format!("{}Cell layout in region '{}':\n", prefix, region.name).as_str());
            col_headers.push_str(format!("{}  | Offset |", prefix).as_str());
            Some(*offset as i32)
        }
        FailureLocation::OutsideRegion { row } => {
            col_headers.push_str(format!("{}Cell layout at row {}:\n", prefix, row).as_str());
            col_headers.push_str(format!("{}  |Rotation|", prefix).as_str());
            None
        }
    };
    eprint!("\n{}", col_headers);

    let widths: Vec<usize> = columns
        .iter()
        .map(|(col, _)| {
            let size = match location {
                FailureLocation::InRegion { region, offset: _ } => {
                    if let Some(column_ann) = region.column_annotations.as_ref() {
                        if let Some(ann) = column_ann.get(col) {
                            ann.len()
                        } else {
                            col_width(column_type_and_idx(col).as_str().len())
                        }
                    } else {
                        col_width(column_type_and_idx(col).as_str().len())
                    }
                }
                FailureLocation::OutsideRegion { row: _ } => {
                    col_width(column_type_and_idx(col).as_str().len())
                }
            };
            size
        })
        .collect();

    // Print the assigned cells, and their region offset or rotation + the column name at which they're assigned to.
    for ((column, _), &width) in columns.iter().zip(widths.iter()) {
        eprint!(
            "{}|",
            padded(
                ' ',
                width,
                &match location {
                    FailureLocation::InRegion { region, offset: _ } => {
                        region
                            .column_annotations
                            .as_ref()
                            .and_then(|column_ann| column_ann.get(column).cloned())
                            .unwrap_or_else(|| column_type_and_idx(column))
                    }
                    FailureLocation::OutsideRegion { row: _ } => {
                        column_type_and_idx(column)
                    }
                }
                .to_string()
            )
        );
    }

    eprintln!();
    eprint!("{}  +--------+", prefix);
    for &width in widths.iter() {
        eprint!("{}+", padded('-', width, ""));
    }
    eprintln!();
    for (rotation, row) in layout {
        eprint!(
            "{}  |{}|",
            prefix,
            padded(' ', 8, &(offset.unwrap_or(0) + rotation).to_string())
        );
        for ((col, _), &width) in columns.iter().zip(widths.iter()) {
            eprint!(
                "{}|",
                padded(
                    ' ',
                    width,
                    row.get(col).map(|s| s.as_str()).unwrap_or_default()
                )
            );
        }
        highlight_row(offset, *rotation);
        eprintln!();
    }
}

pub(super) fn expression_to_string<F: Field>(
    expr: &Expression<F>,
    layout: &BTreeMap<i32, BTreeMap<metadata::Column, String>>,
) -> String {
    expr.evaluate(
        &util::format_value,
        &|_| panic!("virtual selectors are removed during optimization"),
        &|query| {
            if let Some(label) = layout
                .get(&query.rotation.0)
                .and_then(|row| row.get(&(Any::Fixed, query.column_index).into()))
            {
                label.clone()
            } else if query.rotation.0 == 0 {
                // This is most likely a merged selector
                format!("S{}", query.index.unwrap())
            } else {
                // No idea how we'd get here...
                format!("F{}@{}", query.column_index, query.rotation.0)
            }
        },
        &|query| {
            layout
                .get(&query.rotation.0)
                .and_then(|map| {
                    map.get(
                        &(
                            Any::Advice(Advice { phase: query.phase }),
                            query.column_index,
                        )
                            .into(),
                    )
                })
                .cloned()
                .unwrap_or_default()
        },
        &|query| {
            layout
                .get(&query.rotation.0)
                .unwrap()
                .get(&(Any::Instance, query.column_index).into())
                .unwrap()
                .clone()
        },
        &|challenge| format!("C{}({})", challenge.index(), challenge.phase()),
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
}
