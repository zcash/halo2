use std::{cmp, collections::HashMap};

use super::RegionShape;
use crate::{
    circuit::RegionStart,
    plonk::{Any, Column},
};

/// A simple single-pass layout approach.
///
/// Positions the regions starting at the earliest row for which none of the columns are
/// in use.
pub fn slide_up(region_shapes: Vec<RegionShape>) -> Vec<RegionStart> {
    let mut regions = Vec::with_capacity(region_shapes.len());

    // Stores the first empty row for each column.
    let mut columns: HashMap<Column<Any>, usize> = Default::default();

    for shape in region_shapes {
        let mut region_start = 0;
        for column in &shape.columns {
            region_start = cmp::max(region_start, columns.get(column).cloned().unwrap_or(0));
        }
        regions.push(region_start.into());

        // Update column usage information.
        for column in shape.columns {
            columns.insert(column, region_start + shape.row_count);
        }
    }

    regions
}
