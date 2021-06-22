use std::fmt;
use std::marker::PhantomData;

use ff::Field;

use super::{RegionLayouter, RegionShape};
use crate::plonk::Assigned;
use crate::{
    circuit::{Cell, Layouter, Region, RegionIndex, RegionStart},
    plonk::{Advice, Assignment, Column, Error, Fixed, Permutation, Selector},
};

mod strategy;

/// The version 1 [`Layouter`] provided by `halo2`.
///
/// It is a dual-pass layouter, that has visibility into the entire `Circuit::synthesize`
/// step.
pub struct V1<'a, F: Field, CS: Assignment<F> + 'a> {
    cs: &'a mut CS,
    /// Stores the starting row for each region.
    regions: Vec<RegionStart>,
    _marker: PhantomData<F>,
}

impl<'a, F: Field, CS: Assignment<F> + 'a> fmt::Debug for V1<'a, F, CS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("layouter::V1").finish()
    }
}

impl<'a, F: Field, CS: Assignment<F>> V1<'a, F, CS> {
    /// Creates a new v1 layouter.
    pub fn new(cs: &'a mut CS) -> Result<Self, Error> {
        let ret = V1 {
            cs,
            regions: vec![],
            _marker: PhantomData,
        };
        Ok(ret)
    }

    /// Runs the layouter to synthesize the circuit.
    ///
    /// Even though `synthesis` has `FnMut` bounds, any value-assignment closures
    /// contained within it are guaranteed to be called at most once.
    pub fn run<S>(&mut self, mut synthesis: S) -> Result<(), Error>
    where
        S: FnMut(V1Pass<F, CS>) -> Result<(), Error>,
    {
        // First pass: measure the regions within the circuit.
        let mut measure = MeasurementPass::new();
        {
            let pass = &mut measure;
            synthesis(V1Pass::measure(pass))?;
        }

        self.regions = strategy::slot_in_biggest_advice_first(measure.regions);

        // Second pass: assign the regions.
        let mut assign = AssignmentPass::new(self);
        {
            let pass = &mut assign;
            synthesis(V1Pass::assign(pass))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Pass<'p, 'a, F: Field, CS: Assignment<F> + 'a> {
    Measurement(&'p mut MeasurementPass),
    Assignment(&'p mut AssignmentPass<'p, 'a, F, CS>),
}

/// A single pass of the [`V1`] layouter.
#[derive(Debug)]
pub struct V1Pass<'p, 'a, F: Field, CS: Assignment<F> + 'a>(Pass<'p, 'a, F, CS>);

impl<'p, 'a, F: Field, CS: Assignment<F> + 'a> V1Pass<'p, 'a, F, CS> {
    fn measure(pass: &'p mut MeasurementPass) -> Self {
        V1Pass(Pass::Measurement(pass))
    }

    fn assign(pass: &'p mut AssignmentPass<'p, 'a, F, CS>) -> Self {
        V1Pass(Pass::Assignment(pass))
    }
}

impl<'p, 'a, F: Field, CS: Assignment<F> + 'a> Layouter<F> for V1Pass<'p, 'a, F, CS> {
    type Root = Self;

    fn assign_region<A, AR, N, NR>(&mut self, name: N, assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        match &mut self.0 {
            Pass::Measurement(pass) => pass.assign_region(assignment),
            Pass::Assignment(pass) => pass.assign_region(name, assignment),
        }
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        if let Pass::Assignment(pass) = &mut self.0 {
            pass.layouter.cs.push_namespace(name_fn);
        }
    }

    fn pop_namespace(&mut self, gadget_name: Option<String>) {
        if let Pass::Assignment(pass) = &mut self.0 {
            pass.layouter.cs.pop_namespace(gadget_name);
        }
    }
}

/// Measures the circuit.
#[derive(Debug)]
pub struct MeasurementPass {
    regions: Vec<RegionShape>,
}

impl MeasurementPass {
    fn new() -> Self {
        MeasurementPass { regions: vec![] }
    }

    fn assign_region<F: Field, A, AR>(&mut self, mut assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
    {
        let region_index = self.regions.len();

        // Get shape of the region.
        let mut shape = RegionShape::new(region_index.into());
        let result = {
            let region: &mut dyn RegionLayouter<F> = &mut shape;
            assignment(region.into())
        }?;
        self.regions.push(shape);

        Ok(result)
    }
}

/// Assigns the circuit.
#[derive(Debug)]
pub struct AssignmentPass<'p, 'a, F: Field, CS: Assignment<F> + 'a> {
    layouter: &'p mut V1<'a, F, CS>,
    /// Counter tracking which region we need to assign next.
    region_index: usize,
}

impl<'p, 'a, F: Field, CS: Assignment<F> + 'a> AssignmentPass<'p, 'a, F, CS> {
    fn new(layouter: &'p mut V1<'a, F, CS>) -> Self {
        AssignmentPass {
            layouter,
            region_index: 0,
        }
    }

    fn assign_region<A, AR, N, NR>(&mut self, name: N, mut assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        // Get the next region we are assigning.
        let region_index = self.region_index;
        self.region_index += 1;

        self.layouter.cs.enter_region(name);
        let mut region = V1Region::new(self.layouter, region_index.into());
        let result = {
            let region: &mut dyn RegionLayouter<F> = &mut region;
            assignment(region.into())
        }?;
        self.layouter.cs.exit_region();

        Ok(result)
    }
}

struct V1Region<'r, 'a, F: Field, CS: Assignment<F> + 'a> {
    layouter: &'r mut V1<'a, F, CS>,
    region_index: RegionIndex,
}

impl<'r, 'a, F: Field, CS: Assignment<F> + 'a> fmt::Debug for V1Region<'r, 'a, F, CS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("V1Region")
            .field("layouter", &self.layouter)
            .field("region_index", &self.region_index)
            .finish()
    }
}

impl<'r, 'a, F: Field, CS: Assignment<F> + 'a> V1Region<'r, 'a, F, CS> {
    fn new(layouter: &'r mut V1<'a, F, CS>, region_index: RegionIndex) -> Self {
        V1Region {
            layouter,
            region_index,
        }
    }
}

impl<'r, 'a, F: Field, CS: Assignment<F> + 'a> RegionLayouter<F> for V1Region<'r, 'a, F, CS> {
    fn enable_selector<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        selector: &Selector,
        offset: usize,
    ) -> Result<(), Error> {
        self.layouter.cs.enable_selector(
            annotation,
            selector,
            *self.layouter.regions[*self.region_index] + offset,
        )
    }

    fn assign_advice<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<Assigned<F>, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.layouter.cs.assign_advice(
            annotation,
            column,
            *self.layouter.regions[*self.region_index] + offset,
            to,
        )?;

        Ok(Cell {
            region_index: self.region_index,
            row_offset: offset,
            column: column.into(),
        })
    }

    fn assign_fixed<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Result<Assigned<F>, Error> + 'v),
    ) -> Result<Cell, Error> {
        self.layouter.cs.assign_fixed(
            annotation,
            column,
            *self.layouter.regions[*self.region_index] + offset,
            to,
        )?;

        Ok(Cell {
            region_index: self.region_index,
            row_offset: offset,
            column: column.into(),
        })
    }

    fn constrain_equal(
        &mut self,
        permutation: &Permutation,
        left: Cell,
        right: Cell,
    ) -> Result<(), Error> {
        self.layouter.cs.copy(
            permutation,
            left.column,
            *self.layouter.regions[*left.region_index] + left.row_offset,
            right.column,
            *self.layouter.regions[*right.region_index] + right.row_offset,
        )?;

        Ok(())
    }
}
