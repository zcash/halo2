// This example creates a single-chip circuit that proves knowledge of two field elements `a` and
// `b` such that `a^2 + b^2 == PI^2` (for public input `PI`), i.e. proves knowledge of a Pythagorean
// triple.
//
// The arithmetization for this computation is:
//
// (0) allocate private inputs: a, b
// (1) allocate public input: c = PI
// (2) multiply: a * a = a^2
// (3) multiply: b * b = b^2
// (4) multiply: c * c = c^2
// (5) add: a^2 + b^2 = c^2
//
// The constraint system has 3 advice columns `l` (left), `r` (right), and `o` (output), one
// instance column `pub_col` (contains the public inputs), and 3 selectors (fixed columns) `s_add`
// (addition gate), `s_mul` (multiplication gate), and `s_pub` (public input gate).
//
// |-----|-------|-------|-------|---------|-------|-------|-------|
// | row | l_col | r_col | o_col | pub_col | s_add | s_mul | s_pub |
// |-----|-------|-------|-------|---------|-------|-------|-------|
// |  0  |   a   |   b   |       |   0     |   0   |   0   |   0   |
// |  1  |   c   |       |       |   PI    |   0   |   0   |   1   |
// |  2  |   a   |   a   |  aa   |   0     |   0   |   1   |   0   |
// |  3  |   b   |   b   |  bb   |   0     |   0   |   1   |   0   |
// |  4  |   c   |   c   |  cc   |   0     |   0   |   1   |   0   |
// |  5  |   aa  |   bb  |  cc   |   0     |   1   |   0   |   0   |
// |-----|-------|-------|-------|---------|-------|-------|-------|
//
// Any advice value that appears in multiple rows has the consistency of its value enforced across
// rows via permutation argument, e.g. row #0 `a` == row #2 `a` is enforced within in the
// permutation argument.

use halo2::{
    circuit::{layouter::SingleChipLayouter, Cell, Chip, Layouter},
    dev::MockProver,
    pasta::Fp,
    plonk::{
        Advice, Any, Assignment, Circuit, Column, ConstraintSystem, Error, Permutation, Selector,
    },
    poly::Rotation,
};

// A value that has been allocated in the constraint system.
struct Alloc {
    cell: Cell,
    // Must be `Option` because parameter generation will not assign values within the constraint
    // system.
    value: Option<Fp>,
}

struct MyChip {
    config: MyChipConfig,
}

#[derive(Clone, Debug)]
struct MyChipConfig {
    l_col: Column<Advice>,
    r_col: Column<Advice>,
    o_col: Column<Advice>,
    perm: Permutation,
    s_add: Selector,
    s_mul: Selector,
    s_pub: Selector,
}

impl Chip<Fp> for MyChip {
    type Config = MyChipConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl MyChip {
    fn new(config: <Self as Chip<Fp>>::Config) -> Self {
        MyChip { config }
    }

    // Creates the columns and gates (constraint polynomials) required by this chip and stores
    // references to the columns in the chip config structure.
    fn configure(cs: &mut ConstraintSystem<Fp>) -> <Self as Chip<Fp>>::Config {
        let l_col = cs.advice_column();
        let r_col = cs.advice_column();
        let o_col = cs.advice_column();

        // We won't store a reference to the public input column in the config structure because the
        // column's values will be provided by the verifier, i.e. the chip will never assign values
        // into `pub_col`; the selector is used only to defining gates.
        let pub_col = cs.instance_column();

        let perm = {
            // Convert advice columns into an "any" columns.
            let cols: [Column<Any>; 3] = [l_col.into(), r_col.into(), o_col.into()];
            Permutation::new(cs, &cols)
        };

        let s_add = cs.selector();
        let s_mul = cs.selector();
        let s_pub = cs.selector();

        // Define the addition gate.
        //
        // | l_col | r_col | o_col | s_add |
        // |-------|-------|-------|-------|
        // |   l   |   r   |   o   | s_add |
        //
        // Constraint: s_add*l + s_add*r = s_add*o
        cs.create_gate("add", |cs| {
            let l = cs.query_advice(l_col, Rotation::cur());
            let r = cs.query_advice(r_col, Rotation::cur());
            let o = cs.query_advice(o_col, Rotation::cur());
            let s_add = cs.query_selector(s_add, Rotation::cur());
            s_add * (l + r - o)
        });

        // Define the multiplication gate.
        //
        // | l_col | r_col | o_col | s_mul |
        // |-------|-------|-------|-------|
        // |   l   |   r   |   o   | s_mul |
        //
        // Constraint: s_mul*l*r = s_mul*o
        cs.create_gate("mul", |cs| {
            let l = cs.query_advice(l_col, Rotation::cur());
            let r = cs.query_advice(r_col, Rotation::cur());
            let o = cs.query_advice(o_col, Rotation::cur());
            let s_mul = cs.query_selector(s_mul, Rotation::cur());
            s_mul * (l * r - o)
        });

        // Define the public input gate.
        //
        // | l_col | pub_col | s_pub |
        // |-------|---------|-------|
        // |   l   |    pi   | s_pub |
        //
        // Constraint: s_pub*l = s_pub*pi
        cs.create_gate("public input", |cs| {
            let l = cs.query_advice(l_col, Rotation::cur());
            let pi = cs.query_instance(pub_col, Rotation::cur());
            let s_pub = cs.query_selector(s_pub, Rotation::cur());
            s_pub * (l - pi)
        });

        MyChipConfig {
            l_col,
            r_col,
            o_col,
            perm,
            s_add,
            s_mul,
            s_pub,
        }
    }

    // In the next available row, writes `a` into the row's left cell and `b` into the row's right
    // cell.
    fn alloc_private_inputs(
        &self,
        layouter: &mut impl Layouter<Fp>,
        a: Option<Fp>,
        b: Option<Fp>,
    ) -> Result<(Alloc, Alloc), Error> {
        layouter.assign_region(
            || "load private inputs",
            |mut region| {
                let row_offset = 0;
                let a_cell = region.assign_advice(
                    || "private input 'a'",
                    self.config.l_col,
                    row_offset,
                    || a.ok_or(Error::SynthesisError),
                )?;
                let b_cell = region.assign_advice(
                    || "private input 'b'",
                    self.config.r_col,
                    row_offset,
                    || b.ok_or(Error::SynthesisError),
                )?;
                // Note that no arithmetic is performed here, all we are doing is allocating the
                // initial private wire values (i.e. private values which are not the output of any
                // gate), thus there is no selector enabled in this row.
                let a_alloc = Alloc {
                    cell: a_cell,
                    value: a,
                };
                let b_alloc = Alloc {
                    cell: b_cell,
                    value: b,
                };
                Ok((a_alloc, b_alloc))
            },
        )
    }

    // Writes `c` into the next available row's left column cell; enabling the `s_pub` selector
    // enforces that `c = PI` for a PI provided by the verifier via the instance column `pub_col`.
    fn alloc_public_input(
        &self,
        layouter: &mut impl Layouter<Fp>,
        c: Option<Fp>,
    ) -> Result<Alloc, Error> {
        layouter.assign_region(
            || "expose public input",
            |mut region| {
                let row_offset = 0;
                self.config.s_pub.enable(&mut region, row_offset)?;
                let l_cell = region.assign_advice(
                    || "public input advice",
                    self.config.l_col,
                    row_offset,
                    || c.ok_or(Error::SynthesisError),
                )?;
                let c_alloc = Alloc {
                    cell: l_cell,
                    value: c,
                };
                Ok(c_alloc)
            },
        )
    }

    // In the next available row, copies a previously allocated value `prev_alloc` into the row's left
    // and right cells, then writes the product of the left and right cells into the row's output
    // cell; enabling `s_mul` in the row enforces that the left, right, and output cells satisfy the
    // multiplication constraint: `l * r = o`.
    fn square(&self, layouter: &mut impl Layouter<Fp>, prev_alloc: Alloc) -> Result<Alloc, Error> {
        let squared_value = prev_alloc.value.map(|x| x * x);
        layouter.assign_region(
            || "square",
            |mut region| {
                let row_offset = 0;
                self.config.s_mul.enable(&mut region, row_offset)?;

                let l_cell = region.assign_advice(
                    || "l",
                    self.config.l_col,
                    row_offset,
                    || prev_alloc.value.ok_or(Error::SynthesisError),
                )?;
                let r_cell = region.assign_advice(
                    || "r",
                    self.config.r_col,
                    row_offset,
                    || prev_alloc.value.ok_or(Error::SynthesisError),
                )?;

                region.constrain_equal(&self.config.perm, prev_alloc.cell, l_cell)?;
                region.constrain_equal(&self.config.perm, prev_alloc.cell, r_cell)?;

                let o_cell = region.assign_advice(
                    || "l * r",
                    self.config.o_col,
                    row_offset,
                    || squared_value.ok_or(Error::SynthesisError),
                )?;
                let squared_alloc = Alloc {
                    cell: o_cell,
                    value: squared_value,
                };
                Ok(squared_alloc)
            },
        )
    }

    // In the next available row, copies the previously allocated values `l_prev_alloc`, `r_prev_alloc`,
    // and `o_prev_alloc` into the row's left, right, and output cells respectively. Enabling the
    // `s_add` selector enforces that the values written in the row satisfy the addition constraint
    // `l + r = o`.
    //
    // This function is called `constrained_add` because the output of `l + r` is provided by the
    // function caller as a previously allocated value.
    fn constrained_add(
        &self,
        layouter: &mut impl Layouter<Fp>,
        l_in_alloc: Alloc,
        r_in_alloc: Alloc,
        o_in_alloc: Alloc,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "constrained add",
            |mut region| {
                let row_offset = 0;
                self.config.s_add.enable(&mut region, row_offset)?;

                let l_cell = region.assign_advice(
                    || "l",
                    self.config.l_col,
                    row_offset,
                    || l_in_alloc.value.ok_or(Error::SynthesisError),
                )?;
                let r_cell = region.assign_advice(
                    || "r",
                    self.config.r_col,
                    row_offset,
                    || r_in_alloc.value.ok_or(Error::SynthesisError),
                )?;
                let o_cell = region.assign_advice(
                    || "o",
                    self.config.o_col,
                    row_offset,
                    || o_in_alloc.value.ok_or(Error::SynthesisError),
                )?;

                region.constrain_equal(&self.config.perm, l_in_alloc.cell, l_cell)?;
                region.constrain_equal(&self.config.perm, r_in_alloc.cell, r_cell)?;
                region.constrain_equal(&self.config.perm, o_in_alloc.cell, o_cell)?;

                Ok(())
            },
        )
    }
}

#[derive(Clone)]
struct MyCircuit {
    // Private inputs.
    a: Option<Fp>,
    b: Option<Fp>,
    // Public inputs (from prover).
    c: Option<Fp>,
}

impl Circuit<Fp> for MyCircuit {
    // Our circuit uses one chip, thus we can reuse the chip's config as the circuit's config.
    type Config = MyChipConfig;

    fn configure(cs: &mut ConstraintSystem<Fp>) -> Self::Config {
        MyChip::configure(cs)
    }

    fn synthesize(&self, cs: &mut impl Assignment<Fp>, config: Self::Config) -> Result<(), Error> {
        let mut layouter = SingleChipLayouter::new(cs)?;
        let chip = MyChip::new(config);
        let (a_alloc, b_alloc) = chip.alloc_private_inputs(&mut layouter, self.a, self.b)?;
        let c_alloc = chip.alloc_public_input(&mut layouter, self.c)?;
        let a_sq_alloc = chip.square(&mut layouter, a_alloc)?;
        let b_sq_alloc = chip.square(&mut layouter, b_alloc)?;
        let c_sq_alloc = chip.square(&mut layouter, c_alloc)?;
        chip.constrained_add(&mut layouter, a_sq_alloc, b_sq_alloc, c_sq_alloc)
    }
}

fn main() {
    // The number of rows utilized in the constraint system matrix.
    const N_ROWS_USED: u32 = 6;

    // The row index of the public input.
    const PUB_INPUT_ROW_INDEX: usize = 1;

    // The circuit's public input `c` where `a^2 + b^2 = c^2` for private inputs `a` and `b`.
    const PUB_INPUT: u64 = 545;

    // The verifier creates the public inputs column (instance column). The total number of
    // rows `n_rows` in our constraint system cannot exceed 2^k, i.e.
    // `n_rows = 2^(floor(log2(N_ROWS_USED)))`.
    let k = (N_ROWS_USED as f32).log2().ceil() as u32;
    let n_rows = 1 << k;
    let mut pub_inputs = vec![Fp::zero(); n_rows];
    pub_inputs[PUB_INPUT_ROW_INDEX] = Fp::from(PUB_INPUT);

    // The prover creates a circuit containing the public and private inputs.
    let circuit = MyCircuit {
        a: Some(Fp::from(33)),
        b: Some(Fp::from(544)),
        c: Some(Fp::from(PUB_INPUT)),
    };

    // Assert that the constraint system is satisfied.
    let prover = MockProver::run(k, &circuit, vec![pub_inputs.clone()]).unwrap();
    assert!(prover.verify().is_ok());

    // Assert that changing the public inputs results in the constraint system becoming unsatisfied.
    let mut bad_pub_inputs = pub_inputs.clone();
    bad_pub_inputs[PUB_INPUT_ROW_INDEX] = Fp::from(PUB_INPUT + 1);
    let prover = MockProver::run(k, &circuit, vec![bad_pub_inputs]).unwrap();
    assert!(prover.verify().is_err());

    // Assert that changing a private input results in the constraint system becoming unsatisfied.
    let mut bad_circuit = circuit.clone();
    bad_circuit.b = Some(Fp::from(5));
    let prover = MockProver::run(k, &bad_circuit, vec![pub_inputs]).unwrap();
    assert!(prover.verify().is_err());
}
