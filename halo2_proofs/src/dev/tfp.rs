use std::marker::PhantomData;

use ff::Field;

use crate::{
    circuit::{layouter::RegionLayouter, Cell, Layouter, Region, Table, Value},
    plonk::{
        Advice, Any, Assigned, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed,
        FloorPlanner, Instance, Selector,
    },
};

/// A helper type that augments a [`FloorPlanner`] with [`tracing`] spans and events.
#[derive(Debug)]
pub struct TracingFloorPlanner<P: FloorPlanner> {
    _phantom: PhantomData<P>,
}

impl<P: FloorPlanner> FloorPlanner for TracingFloorPlanner<P> {
    fn synthesize<F: Field, CS: Assignment<F>, C: Circuit<F>>(
        cs: &mut CS,
        circuit: &C,
        config: C::Config,
        constants: Vec<Column<Fixed>>,
    ) -> Result<(), Error> {
        P::synthesize(
            &mut TracingAssignment::new(cs),
            &TracingCircuit::borrowed(circuit),
            config,
            constants,
        )
    }
}

/// A helper type that augments a [`Circuit`] with [`tracing`] spans and events.
enum TracingCircuit<'c, F: Field, C: Circuit<F>> {
    Borrowed(&'c C, PhantomData<F>),
    Owned(C, PhantomData<F>),
}

impl<'c, F: Field, C: Circuit<F>> TracingCircuit<'c, F, C> {
    fn borrowed(circuit: &'c C) -> Self {
        Self::Borrowed(circuit, PhantomData)
    }

    fn owned(circuit: C) -> Self {
        Self::Owned(circuit, PhantomData)
    }

    fn inner_ref(&self) -> &C {
        match self {
            TracingCircuit::Borrowed(circuit, ..) => circuit,
            TracingCircuit::Owned(circuit, ..) => circuit,
        }
    }
}

impl<'c, F: Field, C: Circuit<F>> Circuit<F> for TracingCircuit<'c, F, C> {
    type Config = C::Config;
    type FloorPlanner = C::FloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::owned(self.inner_ref().without_witnesses())
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        C::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        self.inner_ref()
            .synthesize(config, TracingLayouter::new(layouter))
    }
}

/// A helper type that augments a [`Layouter`] with [`tracing`] spans and events.
struct TracingLayouter<F: Field, L: Layouter<F>> {
    layouter: L,
    _phantom: PhantomData<F>,
}

impl<F: Field, L: Layouter<F>> TracingLayouter<F, L> {
    fn new(layouter: L) -> Self {
        Self {
            layouter,
            _phantom: PhantomData,
        }
    }
}

impl<F: Field, L: Layouter<F>> Layouter<F> for TracingLayouter<F, L> {
    type Root = Self;

    fn assign_region<A, AR, N, NR>(&mut self, name: N, mut assignment: A) -> Result<AR, Error>
    where
        A: FnMut(Region<'_, F>) -> Result<AR, Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        self.layouter.assign_region(name, |region| {
            let mut region = TracingRegion(region);
            let region: &mut dyn RegionLayouter<F> = &mut region;
            assignment(region.into())
        })
    }

    fn assign_table<A, N, NR>(&mut self, name: N, assignment: A) -> Result<(), Error>
    where
        A: FnMut(Table<'_, F>) -> Result<(), Error>,
        N: Fn() -> NR,
        NR: Into<String>,
    {
        self.layouter.assign_table(name, assignment)
    }

    fn constrain_instance(
        &mut self,
        cell: Cell,
        column: Column<Instance>,
        row: usize,
    ) -> Result<(), Error> {
        self.layouter.constrain_instance(cell, column, row)
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.layouter.push_namespace(name_fn);
    }

    fn pop_namespace(&mut self, gadget_name: Option<String>) {
        self.layouter.pop_namespace(gadget_name);
    }
}

/// A helper type that augments a [`Region`] with [`tracing`] spans and events.
#[derive(Debug)]
struct TracingRegion<'r, F: Field>(Region<'r, F>);

impl<'r, F: Field> RegionLayouter<F> for TracingRegion<'r, F> {
    fn enable_selector<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        selector: &Selector,
        offset: usize,
    ) -> Result<(), Error> {
        self.0.enable_selector(annotation, selector, offset)
    }

    fn assign_advice<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Value<Assigned<F>> + 'v),
    ) -> Result<Cell, Error> {
        self.0
            .assign_advice(annotation, column, offset, to)
            .map(|value| value.cell())
    }

    fn assign_advice_from_constant<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Advice>,
        offset: usize,
        constant: Assigned<F>,
    ) -> Result<Cell, Error> {
        self.0
            .assign_advice_from_constant(annotation, column, offset, constant)
            .map(|value| value.cell())
    }

    fn assign_advice_from_instance<'v>(
        &mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        instance: Column<Instance>,
        row: usize,
        advice: Column<Advice>,
        offset: usize,
    ) -> Result<(Cell, Value<F>), Error> {
        self.0
            .assign_advice_from_instance(annotation, instance, row, advice, offset)
            .map(|value| (value.cell(), value.value().cloned()))
    }

    fn instance_value(
        &mut self,
        instance: Column<Instance>,
        row: usize,
    ) -> Result<Value<F>, Error> {
        self.0.instance_value(instance, row)
    }

    fn assign_fixed<'v>(
        &'v mut self,
        annotation: &'v (dyn Fn() -> String + 'v),
        column: Column<Fixed>,
        offset: usize,
        to: &'v mut (dyn FnMut() -> Value<Assigned<F>> + 'v),
    ) -> Result<Cell, Error> {
        self.0
            .assign_fixed(annotation, column, offset, to)
            .map(|value| value.cell())
    }

    fn constrain_constant(&mut self, cell: Cell, constant: Assigned<F>) -> Result<(), Error> {
        self.0.constrain_constant(cell, constant)
    }

    fn constrain_equal(&mut self, left: Cell, right: Cell) -> Result<(), Error> {
        self.0.constrain_equal(left, right)
    }
}

/// A helper type that augments an [`Assignment`] with [`tracing`] spans and events.
struct TracingAssignment<'cs, F: Field, CS: Assignment<F>> {
    cs: &'cs mut CS,
    _phantom: PhantomData<F>,
}

impl<'cs, F: Field, CS: Assignment<F>> TracingAssignment<'cs, F, CS> {
    fn new(cs: &'cs mut CS) -> Self {
        Self {
            cs,
            _phantom: PhantomData,
        }
    }
}

impl<'cs, F: Field, CS: Assignment<F>> Assignment<F> for TracingAssignment<'cs, F, CS> {
    fn enter_region<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.cs.enter_region(name_fn);
    }

    fn exit_region(&mut self) {
        self.cs.exit_region();
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
        self.cs.enable_selector(annotation, selector, row)
    }

    fn query_instance(&self, column: Column<Instance>, row: usize) -> Result<Value<F>, Error> {
        self.cs.query_instance(column, row)
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Advice>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.cs.assign_advice(annotation, column, row, to)
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        annotation: A,
        column: Column<Fixed>,
        row: usize,
        to: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.cs.assign_fixed(annotation, column, row, to)
    }

    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), Error> {
        self.cs.copy(left_column, left_row, right_column, right_row)
    }

    fn fill_from_row(
        &mut self,
        column: Column<Fixed>,
        row: usize,
        to: Value<Assigned<F>>,
    ) -> Result<(), Error> {
        self.cs.fill_from_row(column, row, to)
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.cs.push_namespace(name_fn)
    }

    fn pop_namespace(&mut self, gadget_name: Option<String>) {
        self.cs.pop_namespace(gadget_name);
    }
}
