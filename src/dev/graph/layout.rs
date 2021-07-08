use ff::Field;
use plotters::{
    coord::Shift,
    prelude::{DrawingArea, DrawingAreaErrorKind, DrawingBackend},
};
use std::cmp;
use std::collections::HashSet;
use std::ops::Range;

use crate::plonk::{
    Advice, Any, Assigned, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed,
    FloorPlanner, Instance, Selector,
};

/// Graphical renderer for circuit layouts.
///
/// Cells that have been assigned to by the circuit will be shaded. If any cells are
/// assigned to more than once (which is usually a mistake), they will be shaded darker
/// than the surrounding cells.
///
/// # Examples
///
/// ```ignore
/// use halo2::dev::CircuitLayout;
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
/// CircuitLayout::default().render(&circuit, &drawing_area).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct CircuitLayout {
    hide_labels: bool,
    view_width: Option<Range<usize>>,
    view_height: Option<Range<usize>>,
}

impl CircuitLayout {
    /// Sets the visibility of region labels.
    ///
    /// The default is to show labels.
    pub fn show_labels(mut self, show: bool) -> Self {
        self.hide_labels = !show;
        self
    }

    /// Sets the view width for this layout, as a number of columns.
    pub fn view_width(mut self, width: Range<usize>) -> Self {
        self.view_width = Some(width);
        self
    }

    /// Sets the view height for this layout, as a number of rows.
    pub fn view_height(mut self, height: Range<usize>) -> Self {
        self.view_height = Some(height);
        self
    }

    /// Renders the given circuit on the given drawing area.
    pub fn render<F: Field, ConcreteCircuit: Circuit<F>, DB: DrawingBackend>(
        self,
        circuit: &ConcreteCircuit,
        drawing_area: &DrawingArea<DB, Shift>,
    ) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
        use plotters::coord::types::RangedCoordusize;
        use plotters::prelude::*;

        // Collect the layout details.
        let mut cs = ConstraintSystem::default();
        let config = ConcreteCircuit::configure(&mut cs);
        let mut layout = Layout::default();
        ConcreteCircuit::FloorPlanner::synthesize(&mut layout, circuit, config).unwrap();

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

        let view_width = self.view_width.unwrap_or(0..total_columns);
        let view_height = self.view_height.unwrap_or(0..layout.total_rows);
        let view_bottom = view_height.end;

        // Prepare the grid layout. We render a red background for advice columns, white for
        // instance columns, and blue for fixed columns (with a darker blue for selectors).
        let root =
            drawing_area.apply_coord_spec(Cartesian2d::<RangedCoordusize, RangedCoordusize>::new(
                view_width,
                view_height,
                drawing_area.get_pixel_range(),
            ));
        root.draw(&Rectangle::new(
            [(0, 0), (total_columns, view_bottom)],
            ShapeStyle::from(&WHITE).filled(),
        ))?;
        root.draw(&Rectangle::new(
            [(0, 0), (cs.num_advice_columns, view_bottom)],
            ShapeStyle::from(&RED.mix(0.2)).filled(),
        ))?;
        root.draw(&Rectangle::new(
            [
                (cs.num_advice_columns + cs.num_instance_columns, 0),
                (total_columns, view_bottom),
            ],
            ShapeStyle::from(&BLUE.mix(0.2)).filled(),
        ))?;
        for selector in layout.selectors {
            let index = selector.index();
            root.draw(&Rectangle::new(
                [
                    (cs.num_advice_columns + cs.num_instance_columns + index, 0),
                    (
                        cs.num_advice_columns + cs.num_instance_columns + index + 1,
                        view_bottom,
                    ),
                ],
                ShapeStyle::from(&BLUE.mix(0.1)).filled(),
            ))?;
        }
        root.draw(&Rectangle::new(
            [(0, 0), (total_columns, view_bottom)],
            &BLACK,
        ))?;

        let draw_region = |root: &DrawingArea<_, _>, top_left, bottom_right| {
            root.draw(&Rectangle::new(
                [top_left, bottom_right],
                ShapeStyle::from(&WHITE).filled(),
            ))?;
            root.draw(&Rectangle::new(
                [top_left, bottom_right],
                ShapeStyle::from(&RED.mix(0.2)).filled(),
            ))?;
            root.draw(&Rectangle::new(
                [top_left, bottom_right],
                ShapeStyle::from(&GREEN.mix(0.2)).filled(),
            ))?;
            root.draw(&Rectangle::new([top_left, bottom_right], &BLACK))?;
            Ok(())
        };

        let draw_cell = |root: &DrawingArea<_, _>, column, row| {
            root.draw(&Rectangle::new(
                [(column, row), (column + 1, row + 1)],
                ShapeStyle::from(&BLACK.mix(0.1)).filled(),
            ))
        };

        // Render the regions!
        let mut labels = if self.hide_labels { None } else { Some(vec![]) };
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
                            draw_region(&root, (start, offset), (end, offset + region.rows))?;
                            if let Some(labels) = &mut labels {
                                labels.push((region.name.clone(), (start, offset)));
                            }
                            width = Some((column, column + 1));
                        }
                        None => width = Some((column, column + 1)),
                    }
                }

                // Render the last part of the region.
                if let Some((start, end)) = width {
                    draw_region(&root, (start, offset), (end, offset + region.rows))?;
                    if let Some(labels) = &mut labels {
                        labels.push((region.name.clone(), (start, offset)));
                    }
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

        // Render labels last, on top of everything else.
        if let Some(labels) = labels {
            for (label, top_left) in labels {
                root.draw(
                    &(EmptyElement::at(top_left)
                        + Text::new(label, (10, 10), ("sans-serif", 15.0).into_font())),
                )?;
            }
        }
        Ok(())
    }
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
    /// Columns we have observed are actually Selectors.
    selectors: HashSet<Column<Any>>,
}

impl Layout {
    fn update(&mut self, column: Column<Any>, row: usize) {
        self.total_rows = cmp::max(self.total_rows, row + 1);

        if let Some(region) = self.current_region {
            let region = &mut self.regions[region];
            region.columns.insert(column);

            // The region offset is the earliest row assigned to.
            let mut offset = region.offset.unwrap_or(row);
            if row < offset {
                // The first row assigned was not at offset 0 within the region.
                region.rows += offset - row;
                offset = row;
            }
            // The number of rows in this region is the gap between the earliest and
            // latest rows assigned.
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

    fn enable_selector<A, AR>(
        &mut self,
        annotation: A,
        selector: &Selector,
        row: usize,
    ) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        // Remember that this column is a selector.
        self.selectors.insert(selector.0.into());
        // Selectors are just fixed columns.
        self.assign_fixed(annotation, selector.0, row, || Ok(F::one()))
    }

    /// Query the value of the cell of an instance column at a particular
    /// absolute row, if known.
    fn query_instance(&self, _: Column<Instance>, _: usize) -> Result<Option<F>, Error> {
        Ok(None)
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Advice>,
        row: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<VR, Error>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.update(column.into(), row);
        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Fixed>,
        row: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Result<VR, Error>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.update(column.into(), row);
        Ok(())
    }

    fn copy(
        &mut self,
        _: Column<Any>,
        _: usize,
        _: Column<Any>,
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
