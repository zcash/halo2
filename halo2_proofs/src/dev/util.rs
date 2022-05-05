use std::collections::BTreeMap;

use group::ff::Field;
use pasta_curves::arithmetic::FieldExt;

use super::{metadata, CellValue, Value};
use crate::{
    plonk::{Any, Column, ColumnType, Expression, Gate, VirtualCell},
    poly::Rotation,
};

pub(super) fn format_value<F: Field>(v: F) -> String {
    if v.is_zero_vartime() {
        "0".into()
    } else if v == F::one() {
        "1".into()
    } else if v == -F::one() {
        "-1".into()
    } else {
        // Format value as hex.
        let s = format!("{:?}", v);
        // Remove leading zeroes.
        let s = s.strip_prefix("0x").unwrap();
        let s = s.trim_start_matches('0');
        format!("0x{}", s)
    }
}

pub(super) fn load<'a, F: FieldExt, T: ColumnType>(
    n: i32,
    row: i32,
    queries: &'a [(Column<T>, Rotation)],
    cells: &'a [Vec<CellValue<F>>],
) -> impl Fn(usize, usize, Rotation) -> Value<F> + 'a {
    move |index, _, _| {
        let (column, at) = &queries[index];
        let resolved_row = (row + at.0) % n;
        cells[column.index()][resolved_row as usize].into()
    }
}

pub(super) fn load_instance<'a, F: FieldExt, T: ColumnType>(
    n: i32,
    row: i32,
    queries: &'a [(Column<T>, Rotation)],
    cells: &'a [Vec<F>],
) -> impl Fn(usize, usize, Rotation) -> Value<F> + 'a {
    move |index, _, _| {
        let (column, at) = &queries[index];
        let resolved_row = (row + at.0) % n;
        Value::Real(cells[column.index()][resolved_row as usize])
    }
}

fn cell_value<'a, F: FieldExt>(
    virtual_cells: &'a [VirtualCell],
    column_type: Any,
    load: impl Fn(usize, usize, Rotation) -> Value<F> + 'a,
) -> impl Fn(usize, usize, Rotation) -> BTreeMap<metadata::VirtualCell, String> + 'a {
    move |query_index, column_index, rotation| {
        virtual_cells
            .iter()
            .find(|c| {
                c.column.column_type() == &column_type
                    && c.column.index() == column_index
                    && c.rotation == rotation
            })
            // None indicates a selector, which we don't bother showing.
            .map(|cell| {
                (
                    cell.clone().into(),
                    match load(query_index, column_index, rotation) {
                        Value::Real(v) => format_value(v),
                        Value::Poison => unreachable!(),
                    },
                )
            })
            .into_iter()
            .collect()
    }
}

pub(super) fn cell_values<'a, F: FieldExt>(
    gate: &Gate<F>,
    poly: &Expression<F>,
    load_fixed: impl Fn(usize, usize, Rotation) -> Value<F> + 'a,
    load_advice: impl Fn(usize, usize, Rotation) -> Value<F> + 'a,
    load_instance: impl Fn(usize, usize, Rotation) -> Value<F> + 'a,
) -> Vec<(metadata::VirtualCell, String)> {
    let virtual_cells = gate.queried_cells();
    let cell_values = poly.evaluate(
        &|_| BTreeMap::default(),
        &|_| panic!("virtual selectors are removed during optimization"),
        &cell_value(virtual_cells, Any::Fixed, load_fixed),
        &cell_value(virtual_cells, Any::Advice, load_advice),
        &cell_value(virtual_cells, Any::Instance, load_instance),
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
    );
    cell_values.into_iter().collect()
}
