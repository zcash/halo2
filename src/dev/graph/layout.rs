use ff::Field;
use plotters::{
    coord::Shift,
    prelude::{DrawingArea, DrawingAreaErrorKind, DrawingBackend},
};
use std::cmp;
use std::collections::HashSet;

use crate::plonk::{Advice, Any, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed};

/// Renders the circuit layout on the given drawing area.
///
/// Cells that have been assigned to by the circuit will be shaded. If any cells are
/// assigned to more than once (which is usually a mistake), they will be shaded darker
/// than the surrounding cells.
///
/// # Examples
///
/// ```ignore
/// use halo2::dev::circuit_layout;
/// use plotters::prelude::*;
///
/// let drawing_area = BitMapBackend::new("example-circuit-layout.png", (1024, 768))
///     .into_drawing_area();
/// drawing_area.fill(&WHITE).unwrap();
/// let drawing_area = drawing_area
///     .titled("Example Circuit Layout", ("sans-serif", 60))
///     .unwrap();
///
/// let circuit = MyCircuit::default();
/// circuit_layout(&circuit, &drawing_area).unwrap();
/// ```
pub fn circuit_layout<F: Field, ConcreteCircuit: Circuit<F>, DB: DrawingBackend>(
    circuit: &ConcreteCircuit,
    drawing_area: &DrawingArea<DB, Shift>,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    use plotters::coord::types::RangedCoordusize;
    use plotters::prelude::*;

    // Collect the layout details.
    let mut cs = ConstraintSystem::default();
    let config = ConcreteCircuit::configure(&mut cs);
    let mut layout = Layout::default();
    circuit.synthesize(&mut layout, config).unwrap();

    // Figure out what order to render the columns in.
    // TODO: For now, just render them in the order they were configured.
    let total_columns = cs.num_advice_columns + cs.num_instance_columns + cs.num_fixed_columns;
    let column_index = |column: &Column<Any>| {
        column.index()
            + match column.column_type() {
                Any::Advice => 0,
                Any::Instance => cs.num_advice_columns,
                Any::Fixed => cs.num_advice_columns + cs.num_instance_columns,
            }
    };

    // Prepare the grid layout. We render a red background for advice columns, white for
    // instance columns, and blue for fixed columns.
    let root =
        drawing_area.apply_coord_spec(Cartesian2d::<RangedCoordusize, RangedCoordusize>::new(
            0..total_columns,
            0..layout.total_rows,
            drawing_area.get_pixel_range(),
        ));
    root.draw(&Rectangle::new(
        [(0, 0), (total_columns, layout.total_rows)],
        ShapeStyle::from(&WHITE).filled(),
    ))?;
    root.draw(&Rectangle::new(
        [(0, 0), (cs.num_advice_columns, layout.total_rows)],
        ShapeStyle::from(&RED.mix(0.2)).filled(),
    ))?;
    root.draw(&Rectangle::new(
        [
            (cs.num_advice_columns + cs.num_instance_columns, 0),
            (total_columns, layout.total_rows),
        ],
        ShapeStyle::from(&BLUE.mix(0.2)).filled(),
    ))?;
    root.draw(&Rectangle::new(
        [(0, 0), (total_columns, layout.total_rows)],
        &BLACK,
    ))?;

    let draw_region = |root: &DrawingArea<_, _>, top_left, bottom_right, label| {
        root.draw(&Rectangle::new(
            [top_left, bottom_right],
            ShapeStyle::from(&GREEN.mix(0.2)).filled(),
        ))?;
        root.draw(&Rectangle::new([top_left, bottom_right], &BLACK))?;
        root.draw(
            &(EmptyElement::at(top_left)
                + Text::new(label, (10, 10), ("sans-serif", 15.0).into_font())),
        )
    };

    let draw_cell = |root: &DrawingArea<_, _>, column, row| {
        root.draw(&Rectangle::new(
            [(column, row), (column + 1, row + 1)],
            ShapeStyle::from(&BLACK.mix(0.1)).filled(),
        ))
    };

    // Render the regions!
    for region in layout.regions {
        if let Some(offset) = region.offset {
            // Sort the region's columns according to the defined ordering.
            let mut columns: Vec<_> = region.columns.into_iter().collect();
            columns.sort_unstable_by_key(|a| column_index(a));

            // Render contiguous parts of the same region as a single box.
            let mut width = None;
            for column in columns {
                let column = column_index(&column);
                match width {
                    Some((start, end)) if end == column => width = Some((start, end + 1)),
                    Some((start, end)) => {
                        draw_region(
                            &root,
                            (start, offset),
                            (end, offset + region.rows),
                            region.name.clone(),
                        )?;
                        width = Some((column, column + 1));
                    }
                    None => width = Some((column, column + 1)),
                }
            }

            // Render the last part of the region.
            if let Some((start, end)) = width {
                draw_region(
                    &root,
                    (start, offset),
                    (end, offset + region.rows),
                    region.name.clone(),
                )?;
            }

            // Darken the cells of the region that have been assigned to.
            for (column, row) in region.cells {
                draw_cell(&root, column_index(&column), row)?;
            }
        }
    }

    // Darken any loose cells that have been assigned to.
    for (column, row) in layout.loose_cells {
        draw_cell(&root, column_index(&column), row)?;
    }

    Ok(())
}

#[derive(Debug)]
struct Region {
    /// The name of the region. Not required to be unique.
    name: String,
    /// The columns used by this region.
    columns: HashSet<Column<Any>>,
    /// The row that this region starts on, if known.
    offset: Option<usize>,
    /// The number of rows that this region takes up.
    rows: usize,
    /// The cells assigned in this region. We store this as a `Vec` so that if any cells
    /// are double-assigned, they will be visibly darker.
    cells: Vec<(Column<Any>, usize)>,
}

#[derive(Default)]
struct Layout {
    regions: Vec<Region>,
    current_region: Option<usize>,
    total_rows: usize,
    /// Any cells assigned outside of a region. We store this as a `Vec` so that if any
    /// cells are double-assigned, they will be visibly darker.
    loose_cells: Vec<(Column<Any>, usize)>,
}

impl Layout {
    fn update(&mut self, column: Column<Any>, row: usize) {
        self.total_rows = cmp::max(self.total_rows, row + 1);

        if let Some(region) = self.current_region {
            let region = &mut self.regions[region];
            region.columns.insert(column);
            let offset = region.offset.unwrap_or(row);
            region.rows = cmp::max(region.rows, row - offset + 1);
            region.offset = Some(offset);
            region.cells.push((column, row));
        } else {
            self.loose_cells.push((column, row));
        }
    }
}

impl<F: Field> Assignment<F> for Layout {
    fn enter_region<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        assert!(self.current_region.is_none());
        self.current_region = Some(self.regions.len());
        self.regions.push(Region {
            name: name_fn().into(),
            columns: HashSet::default(),
            offset: None,
            rows: 0,
            cells: vec![],
        })
    }

    fn exit_region(&mut self) {
        assert!(self.current_region.is_some());
        self.current_region = None;
    }

    fn assign_advice<V, A, AR>(
        &mut self,
        _: A,
        column: Column<Advice>,
        row: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<F, Error>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.update(column.into(), row);
        Ok(())
    }

    fn assign_fixed<V, A, AR>(
        &mut self,
        _: A,
        column: Column<Fixed>,
        row: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<F, Error>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.update(column.into(), row);
        Ok(())
    }

    fn copy(
        &mut self,
        _: usize,
        _: usize,
        _: usize,
        _: usize,
        _: usize,
    ) -> Result<(), crate::plonk::Error> {
        // Do nothing; we don't care about permutations in this context.
        Ok(())
    }

    fn push_namespace<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // Do nothing; we don't care about namespaces in this context.
    }

    fn pop_namespace(&mut self, _: Option<String>) {
        // Do nothing; we don't care about namespaces in this context.
    }
}
