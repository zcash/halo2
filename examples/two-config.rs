extern crate halo2;

use std::marker::PhantomData;

use halo2::{
    arithmetic::FieldExt,
    circuit::{layouter::SingleConfigLayouter, Cell, Config, Layouter, Region},
    dev::VerifyFailure,
    plonk::{
        Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Instance, Permutation,
        Selector,
    },
    poly::Rotation,
};

#[derive(Clone)]
struct Number<F: FieldExt> {
    cell: Cell,
    value: Option<F>,
}

trait NumericInstructions: Config {
    /// Variable representing a number.
    type Num;

    /// Loads a number into the circuit as a private input.
    fn load_private(&mut self, a: Option<Self::Field>) -> Result<Self::Num, Error>;

    /// Returns `c = a * b`.
    fn mul(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;

    /// Returns `c = a + b`.
    fn add(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;

    /// Exposes a number as a public input to the circuit.
    fn expose_public(&mut self, num: Self::Num) -> Result<(), Error>;
}
// ANCHOR_END: instructions

// ANCHOR: mul-instructions
trait MulInstructions: Config {
    /// Variable representing a number.
    type Num;

    /// Returns `c = a * b`.
    fn mul(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;
}
// ANCHOR_END: mul-instructions

// ANCHOR: add-instructions
trait AddInstructions: Config {
    /// Variable representing a number.
    type Num;

    /// Returns `c = a + b`.
    fn add(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error>;
}
// ANCHOR_END: add-instructions

// ANCHOR: config
/// The config that will implement our instructions! Configs do not store any persistent
/// state themselves, and usually only contain type markers if necessary.
struct FieldConfig<'a, F: FieldExt, L: Layouter<F>> {
    configured: FieldConfigured,
    layouter: &'a mut L,
    marker: PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<F>> Config for FieldConfig<'_, F, L> {
    type Root = Self;
    type Configured = FieldConfigured;
    type Loaded = ();
    type Field = F;
    type Layouter = L;

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn configured(&self) -> &Self::Configured {
        &self.configured
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), halo2::plonk::Error> {
        // None of the instructions implemented by this chip have any fixed state.
        // But if we required e.g. a lookup table, this is where we would load it.
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        self.layouter
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // TODO
    }

    /// Exits out of the existing namespace.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn pop_namespace(&mut self, _gadget_name: Option<String>) {
        // TODO
    }
}
// ANCHOR_END: config

// ANCHOR: configured
/// Configured state is stored in a separate config struct. This is generated by the config
/// during configuration, and then handed to the `Layouter`, which makes it available
/// to the config when it needs to implement its instructions.
#[derive(Clone, Debug)]
struct FieldConfigured {
    s_pub: Selector,
    perm: Permutation,
    advice: [Column<Advice>; 2],
    add_configured: AddConfigured,
    mul_configured: MulConfigured,
}

impl<F: FieldExt, L: Layouter<F>> FieldConfig<'_, F, L> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 2],
        instance: Column<Instance>,
    ) -> FieldConfigured {
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

        let add_configured = AddConfig::<'_, _, L>::configure(meta, perm.clone(), advice, s_add);
        let mul_configured = MulConfig::<'_, _, L>::configure(meta, perm.clone(), advice, s_mul);

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

        FieldConfigured {
            s_pub,
            perm,
            advice,
            add_configured,
            mul_configured,
        }
    }
}
// ANCHOR_END: configured

// ANCHOR: instructions-impl
impl<F: FieldExt, L: Layouter<F>> NumericInstructions for FieldConfig<'_, F, L> {
    type Num = Number<F>;

    fn load_private(&mut self, value: Option<Self::Field>) -> Result<Self::Num, Error> {
        let configured = self.configured().clone();
        let mut num = None;
        self.layouter().assign_region(
            || "load private",
            |mut region: Region<'_, Self>| {
                let cell = region.assign_advice(
                    || "private input",
                    configured.advice[0],
                    0,
                    || value.ok_or(Error::SynthesisError),
                )?;
                num = Some(Number { cell, value });
                Ok(())
            },
        )?;
        Ok(num.unwrap())
    }

    fn add(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let mut add_config = AddConfig {
            configured: self.configured().add_configured.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        add_config.add(a, b)
    }

    fn mul(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let mut mul_config = MulConfig {
            configured: self.configured().mul_configured.clone(),
            layouter: self.layouter(),
            marker: PhantomData,
        };
        mul_config.mul(a, b)
    }

    fn expose_public(&mut self, num: Self::Num) -> Result<(), Error> {
        let configured = self.configured().clone();
        self.layouter().assign_region(
            || "expose public",
            |mut region: Region<'_, Self>| {
                // Enable the public-input gate.
                configured.s_pub.enable(&mut region, 0)?;

                // Load the output into the correct advice column.
                let out = region.assign_advice(
                    || "public advice",
                    configured.advice[1],
                    0,
                    || num.value.ok_or(Error::SynthesisError),
                )?;
                region.constrain_equal(&configured.perm, num.cell, out)?;

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
struct AddConfig<'a, F: FieldExt, L: Layouter<F>> {
    configured: AddConfigured,
    layouter: &'a mut L,
    marker: PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<F>> Config for AddConfig<'_, F, L> {
    type Root = Self;
    type Configured = AddConfigured;
    type Loaded = ();
    type Field = F;
    type Layouter = L;

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn configured(&self) -> &Self::Configured {
        &self.configured
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), halo2::plonk::Error> {
        // None of the instructions implemented by this chip have any fixed state.
        // But if we required e.g. a lookup table, this is where we would load it.
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        self.layouter
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // TODO
    }

    /// Exits out of the existing namespace.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn pop_namespace(&mut self, _gadget_name: Option<String>) {
        // TODO
    }
}
// ANCHOR_END: add-config

// ANCHOR: add-configured
#[derive(Clone, Debug)]
struct AddConfigured {
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

impl<F: FieldExt, L: Layouter<F>> AddConfig<'_, F, L> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        perm: Permutation,
        advice: [Column<Advice>; 2],
        s_add: Selector,
    ) -> AddConfigured {
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

        AddConfigured {
            perm,
            advice,
            s_add,
        }
    }
}
// ANCHOR_END: add-configured

// ANCHOR: add-instructions-impl
impl<F: FieldExt, L: Layouter<F>> AddInstructions for AddConfig<'_, F, L> {
    type Num = Number<F>;

    fn add(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let configured = self.configured().clone();
        let mut out = None;
        self.layouter().assign_region(
            || "add",
            |mut region: Region<'_, Self>| {
                // We only want to use a single addition gate in this region,
                // so we enable it at region offset 0; this means it will constrain
                // cells at offsets 0 and 1.
                configured.s_add.enable(&mut region, 0)?;

                // The inputs we've been given could be located anywhere in the circuit,
                // but we can only rely on relative offsets inside this region. So we
                // assign new cells inside the region and constrain them to have the
                // same values as the inputs.
                let lhs = region.assign_advice(
                    || "lhs",
                    configured.advice[0],
                    0,
                    || a.value.ok_or(Error::SynthesisError),
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    configured.advice[1],
                    0,
                    || b.value.ok_or(Error::SynthesisError),
                )?;
                region.constrain_equal(&configured.perm, a.cell, lhs)?;
                region.constrain_equal(&configured.perm, b.cell, rhs)?;

                // Now we can assign the addition result into the output position.
                let value = a.value.and_then(|a| b.value.map(|b| a + b));
                let cell = region.assign_advice(
                    || "lhs + rhs",
                    configured.advice[0],
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

// ANCHOR_END: add-instructions-impl

// ANCHOR: mul-config
/// The config that will implement MulInstructions.
struct MulConfig<'a, F: FieldExt, L: Layouter<F>> {
    configured: MulConfigured,
    layouter: &'a mut L,
    marker: PhantomData<F>,
}

impl<F: FieldExt, L: Layouter<F>> Config for MulConfig<'_, F, L> {
    type Root = Self;
    type Configured = MulConfigured;
    type Loaded = ();
    type Field = F;
    type Layouter = L;

    fn get_root(&mut self) -> &mut Self::Root {
        self
    }

    fn configured(&self) -> &Self::Configured {
        &self.configured
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }

    fn load(&mut self) -> Result<(), halo2::plonk::Error> {
        // None of the instructions implemented by this chip have any fixed state.
        // But if we required e.g. a lookup table, this is where we would load it.
        Ok(())
    }

    fn layouter(&mut self) -> &mut Self::Layouter {
        self.layouter
    }

    fn push_namespace<NR, N>(&mut self, _name_fn: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        // TODO
    }

    /// Exits out of the existing namespace.
    ///
    /// Not intended for downstream consumption; use [`Layouter::namespace`] instead.
    fn pop_namespace(&mut self, _gadget_name: Option<String>) {
        // TODO
    }
}
// ANCHOR_END: mul-config

// ANCHOR: mul-configured
#[derive(Clone, Debug)]
struct MulConfigured {
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

impl<F: FieldExt, L: Layouter<F>> MulConfig<'_, F, L> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        perm: Permutation,
        advice: [Column<Advice>; 2],
        s_mul: Selector,
    ) -> MulConfigured {
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

        MulConfigured {
            perm,
            advice,
            s_mul,
        }
    }
}
// ANCHOR_END: mul-configured

// ANCHOR: mul-instructions-impl
impl<F: FieldExt, L: Layouter<F>> MulInstructions for MulConfig<'_, F, L> {
    type Num = Number<F>;

    fn mul(&mut self, a: Self::Num, b: Self::Num) -> Result<Self::Num, Error> {
        let configured = self.configured().clone();
        let mut out = None;
        self.layouter().assign_region(
            || "mul",
            |mut region: Region<'_, Self>| {
                // We only want to use a single multiplication gate in this region,
                // so we enable it at region offset 0; this means it will constrain
                // cells at offsets 0 and 1.
                configured.s_mul.enable(&mut region, 0)?;

                // The inputs we've been given could be located anywhere in the circuit,
                // but we can only rely on relative offsets inside this region. So we
                // assign new cells inside the region and constrain them to have the
                // same values as the inputs.
                let lhs = region.assign_advice(
                    || "lhs",
                    configured.advice[0],
                    0,
                    || a.value.ok_or(Error::SynthesisError),
                )?;
                let rhs = region.assign_advice(
                    || "rhs",
                    configured.advice[1],
                    0,
                    || b.value.ok_or(Error::SynthesisError),
                )?;
                region.constrain_equal(&configured.perm, a.cell, lhs)?;
                region.constrain_equal(&configured.perm, b.cell, rhs)?;

                // Now we can assign the multiplication result into the output position.
                let value = a.value.and_then(|a| b.value.map(|b| a * b));
                let cell = region.assign_advice(
                    || "lhs * rhs",
                    configured.advice[0],
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
// ANCHOR_END: mul-instructions-impl

// ANCHOR: circuit
/// The full circuit implementation.
///
/// In this struct we store the private input variables. We use `Option<F>` because
/// they won't have any value during key generation. During proving, if any of these
/// were `None` we would get an error.
struct MyCircuit<F: FieldExt> {
    a: Option<F>,
    b: Option<F>,
}

impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
    // Since we are using a single chip for everything, we can just reuse its configured.
    type Configured = FieldConfigured;

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Configured {
        // We create the two advice columns that FieldChip uses for I/O.
        let advice = [meta.advice_column(), meta.advice_column()];

        // We also need an instance column to store public inputs.
        let instance = meta.instance_column();

        FieldConfig::<F, ()>::configure(meta, advice, instance)
    }

    fn synthesize(
        &self,
        cs: &mut impl Assignment<F>,
        configured: Self::Configured,
    ) -> Result<(), Error> {
        let mut config = FieldConfig {
            configured,
            layouter: &mut SingleConfigLayouter::new(cs),
            marker: PhantomData,
        };

        // Load our private values into the circuit.
        let a = config.load_private(self.a)?;
        let b = config.load_private(self.b)?;

        // We only have access to plain multiplication.
        // We could implement our circuit as:
        //     asq = a*a
        //     bsq = b*b
        //     c   = asq*bsq
        //
        // but it's more efficient to implement it as:
        //     ab = a*b
        //     c  = ab + ab
        let ab = config.mul(a, b)?;
        let c = config.add(ab.clone(), ab)?;

        // Expose the result as a public input to the circuit.
        config.expose_public(c)
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
    let c = a * b + a * b;

    // Instantiate the circuit with the private inputs.
    let circuit = MyCircuit {
        a: Some(a),
        b: Some(b),
    };

    // Arrange the public input. We expose the multiplication result in row 6
    // of the instance column, so we position it there in our public inputs.
    let mut public_inputs = vec![Fp::zero(); 1 << k];
    public_inputs[6] = c;

    // Given the correct public input, our circuit will verify.
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));

    // If we try some other public input, the proof will fail!
    public_inputs[6] += Fp::one();
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert_eq!(
        prover.verify(),
        Err(VerifyFailure::Gate {
            gate_index: 2,
            gate_name: "public input",
            row: 6,
        })
    );
    // ANCHOR_END: test-circuit
}
