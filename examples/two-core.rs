extern crate halo2;

use std::marker::PhantomData;

use halo2::{
    arithmetic::FieldExt,
    circuit::{layouter::SingleCoreLayouter, Cell, Chip, Core, Layouter, Region},
    dev::VerifyFailure,
    plonk::{
        Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Instance, Permutation,
        Selector,
    },
    poly::Rotation,
};

struct FieldGadget<
    F: FieldExt,
    FieldCore: LoadInstructions<F> + AddInstructions<F> + MulInstructions<F>,
> {
    marker_f: PhantomData<F>,
    marker_core: PhantomData<FieldCore>,
}

impl<
        F: FieldExt,
        FieldCore: Core<F> + LoadInstructions<F> + AddInstructions<F> + MulInstructions<F>,
    > FieldGadget<F, FieldCore>
{
    // Given `a, b, c`, returns `(a + b) * c`
    fn add_and_mul(
        mut chip: impl Chip<F, FieldCore>,
        a: Option<F>,
        b: Option<F>,
        c: Option<F>,
    ) -> Result<(), Error> {
        // Load our private values into the circuit.
        let a = chip.namespace(|| "a").core().load_private(a)?;
        let b = chip.namespace(|| "b").core().load_private(b)?;
        let c = chip.namespace(|| "c").core().load_private(c)?;

        // Cast types to use for `AddInstructions`
        let a = chip.namespace(|| "a").core().expose_num_load(a);
        let a = chip.namespace(|| "a").core().new_num_add(a.0, a.1);
        let b = chip.namespace(|| "b").core().expose_num_load(b);
        let b = chip.namespace(|| "b").core().new_num_add(b.0, b.1);

        // `a + b`
        let ab = chip.namespace(|| "ab").core().add(a, b)?;

        // Cast types to use for `MulInstructions`
        let ab = chip.namespace(|| "ab").core().expose_num_add(ab);
        let ab = chip.namespace(|| "ab").core().new_num_mul(ab.0, ab.1);
        let c = chip.namespace(|| "c").core().expose_num_load(c);
        let c = chip.namespace(|| "c").core().new_num_mul(c.0, c.1);

        // `(a + b) * c`
        let abc = chip.namespace(|| "abc").core().mul(ab, c)?;

        // Return type as `LoadInstructions::Num`
        let abc = chip.namespace(|| "abc").core().expose_num_mul(abc);
        let abc = chip.namespace(|| "abc").core().new_num_load(abc.0, abc.1);

        chip.namespace(|| "abc").core().expose_public(abc)
    }
}

#[derive(Clone)]
struct Number<F: FieldExt> {
    cell: Cell,
    value: Option<F>,
}

trait LoadInstructions<F: FieldExt> {
    /// Variable representing a number.
    type Num;

    fn new_num_load(&mut self, cell: Cell, value: Option<F>) -> Self::Num;
    fn expose_num_load(&mut self, num: Self::Num) -> (Cell, Option<F>);

    /// Loads a number into the circuit as a private input.
    fn load_private(&mut self, a: Option<F>) -> Result<<Self as LoadInstructions<F>>::Num, Error>;

    /// Exposes a number as a public input to the circuit.
    fn expose_public(&mut self, num: <Self as LoadInstructions<F>>::Num) -> Result<(), Error>;
}
// ANCHOR_END: instructions

// ANCHOR: mul-instructions
trait MulInstructions<F: FieldExt> {
    /// Variable representing a number.
    type Num;

    fn new_num_mul(&mut self, cell: Cell, value: Option<F>) -> Self::Num;

    fn expose_num_mul(&mut self, num: Self::Num) -> (Cell, Option<F>);

    /// Returns `c = a * b`.
    fn mul(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;
}
// ANCHOR_END: mul-instructions

// ANCHOR: add-instructions
trait AddInstructions<F: FieldExt> {
    /// Variable representing a number.
    type Num;

    fn new_num_add(&mut self, cell: Cell, value: Option<F>) -> Self::Num;

    fn expose_num_add(&mut self, num: Self::Num) -> (Cell, Option<F>);

    /// Returns `c = a + b`.
    fn add(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;
}
// ANCHOR_END: add-instructions

// ANCHOR: core
/// The core that will implement our instructions! Cores do not store any persistent
/// state themselves, and usually only contain type markers if necessary.
struct FieldCore<F: FieldExt, L: Layouter<F>> {
    config: FieldConfig,
    layouter: L,
    marker: PhantomData<F>,
}
// ANCHOR_END: core

// ANCHOR: core-impl
impl<F: FieldExt, L: Layouter<F>> Core<F> for FieldCore<F, L> {
    type Config = FieldConfig;
    type Loaded = ();
    type Layouter = L;

    // fn new(config: Self::Config, mut layouter: Self::Layouter) -> Self {
    //     FieldCore {
    //         config: config.clone(),
    //         layouter,
    //         marker: PhantomData
    //     }
    // }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), halo2::plonk::Error> {
        // None of the instructions implemented by this core have any fixed state.
        // But if we required e.g. a lookup table, this is where we would load it.
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        &mut self.layouter
    }
}
// ANCHOR_END: core-impl

// ANCHOR: chip
/// The chip that will implement our instructions! Chips do not store any persistent
/// state themselves, and usually only contain type markers if necessary.
struct FieldChip<F: FieldExt, L: Layouter<F>> {
    core: FieldCore<F, L>,
}
// ANCHOR_END: chip

// ANCHOR: chip-impl
/// The chip that will implement our instructions! Chips do not store any persistent
/// state themselves, and usually only contain type markers if necessary.
impl<F: FieldExt, L: Layouter<F>> Chip<F, FieldCore<F, L>> for FieldChip<F, L> {
    type Config = FieldConfig;
    type Layouter = L;
    type Root = Self;

    fn new(config: Self::Config, layouter: Self::Layouter) -> Self::Root {
        FieldChip {
            core: FieldCore {
                config,
                layouter,
                marker: PhantomData,
            },
        }
    }

    fn root(&mut self) -> &mut Self::Root {
        self
    }

    fn core(&mut self) -> &mut FieldCore<F, L> {
        &mut self.core
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
    }

    fn pop_namespace(&mut self, _gadget_name: Option<String>) {}
}
// ANCHOR_END: chip-impl

// ANCHOR: config
/// Config state is stored in a separate config struct. This is generated by the config
/// during configuration, and then handed to the `Layouter`, which makes it available
/// to the config when it needs to implement its instructions.
#[derive(Clone, Debug)]
struct FieldConfig {
    s_pub: Selector,
    perm: Permutation,
    advice: [Column<Advice>; 2],
    add_config: AddConfig,
    mul_config: MulConfig,
}

impl<F: FieldExt, L: Layouter<F>> FieldCore<F, L> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 2],
        instance: Column<Instance>,
    ) -> FieldConfig {
        let perm = Permutation::new(
            meta,
            &advice
                .iter()
                .map(|column| (*column).into())
                .collect::<Vec<_>>(),
        );
        let s_pub = meta.selector();
        let s_add = meta.selector();
        let s_mul = meta.selector();

        let add_config = AddCore::<_, L>::configure(meta, perm.clone(), advice, s_add);
        let mul_config = MulCore::<_, L>::configure(meta, perm.clone(), advice, s_mul);

        // Define our public-input gate!
        meta.create_gate("public input", |meta| {
            // We choose somewhat-arbitrarily that we will use the second advice
            // column for exposing numbers as public inputs.
            let a = meta.query_advice(advice[1], Rotation::cur());
            let p = meta.query_instance(instance, Rotation::cur());
            let s = meta.query_selector(s_pub, Rotation::cur());

            // We simply constrain the advice cell to be equal to the instance cell,
            // when the selector is enabled.
            s * (p + a * -F::one())
        });

        FieldConfig {
            s_pub,
            perm,
            advice,
            add_config,
            mul_config,
        }
    }
}
// ANCHOR_END: config

// ANCHOR: instructions-impl
impl<F: FieldExt, L: Layouter<F>> LoadInstructions<F> for FieldCore<F, L> {
    type Num = Number<F>;

    fn new_num_load(&mut self, cell: Cell, value: Option<F>) -> Self::Num {
        Self::Num { cell, value }
    }

    fn expose_num_load(&mut self, num: Self::Num) -> (Cell, Option<F>) {
        (num.cell, num.value)
    }

    fn load_private(
        &mut self,
        value: Option<F>,
    ) -> Result<<Self as LoadInstructions<F>>::Num, Error> {
        let config = self.config().clone();
        let mut num = None;
        self.layouter().assign_region(
            || "load private",
            |mut region: Region<'_, F, Self>| {
                let cell = region.assign_advice(
                    || "private input",
                    config.advice[0],
                    0,
                    || value.ok_or(Error::SynthesisError),
                )?;
                num = Some(Number { cell, value });
                Ok(())
            },
        )?;
        Ok(num.unwrap())
    }

    fn expose_public(&mut self, num: <Self as LoadInstructions<F>>::Num) -> Result<(), Error> {
        let config = self.config().clone();
        self.layouter().assign_region(
            || "expose public",
            |mut region: Region<'_, F, Self>| {
                // Enable the public-input gate.
                config.s_pub.enable(&mut region, 0)?;

                // Load the output into the correct advice column.
                let out = region.assign_advice(
                    || "public advice",
                    config.advice[1],
                    0,
                    || num.value.ok_or(Error::SynthesisError),
                )?;
                region.constrain_equal(&config.perm, num.cell, out)?;

                // We don't assign to the instance column inside the circuit;
                // the mapping of public inputs to cells is provided to the prover.
                Ok(())
            },
        )
    }
}
// ANCHOR_END: instructions-impl

// ANCHOR: add-config
/// The config that will implement AddInstructions.
struct AddCore<'a, F: FieldExt, L: Layouter<F>> {
    config: AddConfig,
    layouter: &'a mut L,
    marker: PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<F>> Core<F> for AddCore<'_, F, L> {
    type Config = AddConfig;
    type Loaded = ();
    type Layouter = L;

    // fn new(config: Self::Config, mut layouter: Self::Layouter) -> Self {
    //     AddCore {
    //         config: config.clone(),
    //         layouter: &mut layouter,
    //         marker: PhantomData
    //     }
    // }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), halo2::plonk::Error> {
        // None of the instructions implemented by this core have any fixed state.
        // But if we required e.g. a lookup table, this is where we would load it.
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        &mut self.layouter
    }
}
// ANCHOR_END: add-config

// ANCHOR: add-config
#[derive(Clone, Debug)]
struct AddConfig {
    perm: Permutation,

    /// For this core, we will use two advice columns to implement our instructions.
    /// These are also the columns through which we communicate with other parts of
    /// the circuit.
    advice: [Column<Advice>; 2],

    // We need a selector to enable the addition gate, so that we aren't placing
    // any constraints on cells where `AddInstructions::add` is not being used.
    // This is important when building larger circuits, where columns are used by
    // multiple sets of instructions.
    s_add: Selector,
}

impl<F: FieldExt, L: Layouter<F>> AddCore<'_, F, L> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        perm: Permutation,
        advice: [Column<Advice>; 2],
        s_add: Selector,
    ) -> AddConfig {
        // Define our addition gate!
        meta.create_gate("add", |meta| {
            // To implement addition, we need three advice cells and a selector
            // cell. We arrange them like so:
            //
            // | a0  | a1  | s_add |
            // |-----|-----|-------|
            // | lhs | rhs | s_add |
            // | out |     |       |
            //
            // Gates may refer to any relative offsets we want, but each distinct
            // offset adds a cost to the proof. The most common offsets are 0 (the
            // current row), 1 (the next row), and -1 (the previous row), for which
            // `Rotation` has specific constructors.
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[0], Rotation::next());
            let s_add = meta.query_selector(s_add, Rotation::cur());

            // The polynomial expression returned from `create_gate` will be
            // constrained by the proving system to equal zero. Our expression
            // has the following properties:
            // - When s_add = 0, any value is allowed in lhs, rhs, and out.
            // - When s_add != 0, this constrains lhs + rhs = out.
            s_add * (lhs + rhs + out * -F::one())
        });

        AddConfig {
            perm,
            advice,
            s_add,
        }
    }
}
// ANCHOR_END: add-config

// ANCHOR: add-instructions-impl
impl<F: FieldExt, L: Layouter<F>> AddInstructions<F> for AddCore<'_, F, L> {
    type Num = Number<F>;

    fn new_num_add(&mut self, cell: Cell, value: Option<F>) -> Self::Num {
        Self::Num { cell, value }
    }

    fn expose_num_add(&mut self, num: Self::Num) -> (Cell, Option<F>) {
        (num.cell, num.value)
    }

    fn add(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let config = self.config().clone();
        let mut out = None;
        self.layouter().assign_region(
            || "add",
            |mut region: Region<'_, F, Self>| {
                // We only want to use a single addition gate in this region,
                // so we enable it at region offset 0; this means it will constrain
                // cells at offsets 0 and 1.
                config.s_add.enable(&mut region, 0)?;

                // The inputs we've been given could be located anywhere in the circuit,
                // but we can only rely on relative offsets inside this region. So we
                // assign new cells inside the region and constrain them to have the
                // same values as the inputs.
                let lhs = region.assign_advice(
                    || "lhs",
                    config.advice[0],
                    0,
                    || a.value.ok_or(Error::SynthesisError),
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    config.advice[1],
                    0,
                    || b.value.ok_or(Error::SynthesisError),
                )?;
                region.constrain_equal(&config.perm, a.cell, lhs)?;
                region.constrain_equal(&config.perm, b.cell, rhs)?;

                // Now we can assign the addition result into the output position.
                let value = a.value.and_then(|a| b.value.map(|b| a + b));
                let cell = region.assign_advice(
                    || "lhs + rhs",
                    config.advice[0],
                    1,
                    || value.ok_or(Error::SynthesisError),
                )?;

                // Finally, we return a variable representing the output,
                // to be used in another part of the circuit.
                out = Some(Number { cell, value });
                Ok(())
            },
        )?;

        Ok(out.unwrap())
    }
}

impl<F: FieldExt, L: Layouter<F>> AddInstructions<F> for FieldCore<F, L> {
    type Num = Number<F>;

    fn new_num_add(&mut self, cell: Cell, value: Option<F>) -> Self::Num {
        let mut add_core = AddCore {
            config: self.config().add_config.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        add_core.new_num_add(cell, value)
    }

    fn expose_num_add(&mut self, num: Self::Num) -> (Cell, Option<F>) {
        let mut add_core = AddCore {
            config: self.config().add_config.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        add_core.expose_num_add(num)
    }

    fn add(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let mut add_core = AddCore {
            config: self.config().add_config.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        add_core.add(a, b)
    }
}

// ANCHOR_END: add-instructions-impl

// ANCHOR: mul-config
/// The config that will implement MulInstructions.
struct MulCore<'a, F: FieldExt, L: Layouter<F>> {
    config: MulConfig,
    layouter: &'a mut L,
    marker: PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<F>> Core<F> for MulCore<'_, F, L> {
    type Config = MulConfig;
    type Loaded = ();
    type Layouter = L;

    // fn new(config: Self::Config, mut layouter: Self::Layouter) -> Self {
    //     MulCore {
    //         config: config.clone(),
    //         layouter: &mut layouter,
    //         marker: PhantomData
    //     }
    // }

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), halo2::plonk::Error> {
        // None of the instructions implemented by this core have any fixed state.
        // But if we required e.g. a lookup table, this is where we would load it.
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        &mut self.layouter
    }
}
// ANCHOR_END: mul-config

// ANCHOR: mul-config
#[derive(Clone, Debug)]
struct MulConfig {
    perm: Permutation,

    /// For this core, we will use two advice columns to implement our instructions.
    /// These are also the columns through which we communicate with other parts of
    /// the circuit.
    advice: [Column<Advice>; 2],

    // We need a selector to enable the multiplication gate, so that we aren't placing
    // any constraints on cells where `MulInstructions::mul` is not being used.
    // This is important when building larger circuits, where columns are used by
    // multiple sets of instructions.
    s_mul: Selector,
}

impl<F: FieldExt, L: Layouter<F>> MulCore<'_, F, L> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        perm: Permutation,
        advice: [Column<Advice>; 2],
        s_mul: Selector,
    ) -> MulConfig {
        // Define our multiplication gate!
        meta.create_gate("mul", |meta| {
            // To implement multiplication, we need three advice cells and a selector
            // cell. We arrange them like so:
            //
            // | a0  | a1  | s_mul |
            // |-----|-----|-------|
            // | lhs | rhs | s_mul |
            // | out |     |       |
            //
            // Gates may refer to any relative offsets we want, but each distinct
            // offset adds a cost to the proof. The most common offsets are 0 (the
            // current row), 1 (the next row), and -1 (the previous row), for which
            // `Rotation` has specific constructors.
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[0], Rotation::next());
            let s_mul = meta.query_selector(s_mul, Rotation::cur());

            // The polynomial expression returned from `create_gate` will be
            // constrained by the proving system to equal zero. Our expression
            // has the following properties:
            // - When s_mul = 0, any value is allowed in lhs, rhs, and out.
            // - When s_mul != 0, this constrains lhs * rhs = out.
            s_mul * (lhs * rhs + out * -F::one())
        });

        MulConfig {
            perm,
            advice,
            s_mul,
        }
    }
}
// ANCHOR_END: mul-config

// ANCHOR: mul-instructions-impl
impl<F: FieldExt, L: Layouter<F>> MulInstructions<F> for MulCore<'_, F, L> {
    type Num = Number<F>;

    fn new_num_mul(&mut self, cell: Cell, value: Option<F>) -> Self::Num {
        Self::Num { cell, value }
    }

    fn expose_num_mul(&mut self, num: Self::Num) -> (Cell, Option<F>) {
        (num.cell, num.value)
    }

    fn mul(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let config = self.config().clone();
        let mut out = None;
        self.layouter().assign_region(
            || "mul",
            |mut region: Region<'_, F, Self>| {
                // We only want to use a single multiplication gate in this region,
                // so we enable it at region offset 0; this means it will constrain
                // cells at offsets 0 and 1.
                config.s_mul.enable(&mut region, 0)?;

                // The inputs we've been given could be located anywhere in the circuit,
                // but we can only rely on relative offsets inside this region. So we
                // assign new cells inside the region and constrain them to have the
                // same values as the inputs.
                let lhs = region.assign_advice(
                    || "lhs",
                    config.advice[0],
                    0,
                    || a.value.ok_or(Error::SynthesisError),
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    config.advice[1],
                    0,
                    || b.value.ok_or(Error::SynthesisError),
                )?;
                region.constrain_equal(&config.perm, a.cell, lhs)?;
                region.constrain_equal(&config.perm, b.cell, rhs)?;

                // Now we can assign the multiplication result into the output position.
                let value = a.value.and_then(|a| b.value.map(|b| a * b));
                let cell = region.assign_advice(
                    || "lhs * rhs",
                    config.advice[0],
                    1,
                    || value.ok_or(Error::SynthesisError),
                )?;

                // Finally, we return a variable representing the output,
                // to be used in another part of the circuit.
                out = Some(Number { cell, value });
                Ok(())
            },
        )?;

        Ok(out.unwrap())
    }
}

impl<F: FieldExt, L: Layouter<F>> MulInstructions<F> for FieldCore<F, L> {
    type Num = Number<F>;

    fn new_num_mul(&mut self, cell: Cell, value: Option<F>) -> Self::Num {
        let mut mul_core = MulCore {
            config: self.config().mul_config.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        mul_core.new_num_mul(cell, value)
    }

    fn expose_num_mul(&mut self, num: Self::Num) -> (Cell, Option<F>) {
        let mut mul_core = MulCore {
            config: self.config().mul_config.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        mul_core.expose_num_mul(num)
    }

    fn mul(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let mut mul_core = MulCore {
            config: self.config().mul_config.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        mul_core.mul(a, b)
    }
}
// ANCHOR_END: mul-instructions-impl

// ANCHOR: circuit
/// The full circuit implementation.
///
/// In this struct we store the private input variables. We use `Option<F>` because
/// they won't have any value during key generation. During proving, if any of these
/// were `None` we would get an error.
struct MyCircuit<'a, F: FieldExt, CS: Assignment<F>> {
    a: Option<F>,
    b: Option<F>,
    c: Option<F>,
    marker: PhantomData<&'a CS>,
}

impl<'a, F: FieldExt, CS: Assignment<F> + 'a> Circuit<F> for MyCircuit<'a, F, CS> {
    // Since we are using a single chip for everything, we can just reuse its config.
    type Config = FieldConfig;
    type Chip = FieldChip<F, Self::Layouter>;
    type Core = FieldCore<F, Self::Layouter>;
    type Layouter = SingleCoreLayouter<'a, F, CS>;
    type CS = CS;

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // We create the two advice columns that FieldCore uses for I/O.
        let advice = [meta.advice_column(), meta.advice_column()];

        // We also need an instance column to store public inputs.
        let instance = meta.instance_column();

        FieldCore::<F, ()>::configure(meta, advice, instance)
    }

    fn synthesize(&self, cs: &mut Self::CS, config: Self::Config) -> Result<(), Error> {
        let layouter = SingleCoreLayouter::<'_, F, CS>::new(cs);
        let chip = FieldChip::<F, SingleCoreLayouter<'_, F, CS>>::new(config.clone(), layouter);

        FieldGadget::<F, Self::Core>::add_and_mul(chip, self.a, self.b, self.c)
    }
}
// ANCHOR_END: circuit

fn main() {
    use halo2::{dev::MockProver, pasta::Fp};

    // ANCHOR: test-circuit
    // The number of rows in our circuit cannot exceed 2^k. Since our example
    // circuit is very small, we can pick a very small value here.
    let k = 3;

    // Prepare the private and public inputs to the circuit!
    let a = Fp::from(2);
    let b = Fp::from(3);
    let c = Fp::from(4);

    // Instantiate the circuit with the private inputs.
    let circuit = MyCircuit::<'_, _, MockProver<Fp>> {
        a: Some(a),
        b: Some(b),
        c: Some(c),
        marker: PhantomData,
    };

    // Arrange the public input. We expose the multiplication result in row 6
    // of the instance column, so we position it there in our public inputs.
    let mut public_inputs = vec![Fp::zero(); 1 << k];
    public_inputs[7] = (a + b) * c;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    public_inputs[7] += Fp::one();
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert_eq!(
        prover.verify(),
        Err(VerifyFailure::Gate {
            gate_index: 2,
            gate_name: "public input",
            row: 7,
        })
    );
    // ANCHOR_END: test-circuit
}
