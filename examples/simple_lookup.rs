// This example uses a lookup table to outsource the computation of XOR-ing two 2-bit unsigned
// integers. The circuit proves knowledge of two integers `a` and `b` such that `xor(a, b) = c`
// for public input `c`. All legal input-output pairs for the XOR function are written in a public
// lookup table using fixed columns.
//
// The constraint system matrix is:
//
// |-----||-------|-------|-------|-------|---------|-----------|-----------|-----------|
// | row || l_col | r_col | o_col | s_pub | pub_col | xor_l_col | xor_r_col | xor_o_col |
// |-----||-------|-------|-------|-------|---------|-----------|-----------|-----------|
// |  0  ||   a   |   b   |   c   |   1   |   PI    |     00    |     00    |     00    |
// |  1  ||       |       |       |   0   |         |     00    |     01    |     01    |
// |  2  ||       |       |       |   0   |         |     00    |     10    |     10    |
// |  3  ||       |       |       |   0   |         |     00    |     11    |     11    |
// |  4  ||       |       |       |   0   |         |     01    |     00    |     01    |
// |  5  ||       |       |       |   0   |         |     01    |     01    |     00    |
// |  6  ||       |       |       |   0   |         |     01    |     10    |     11    |
// |  7  ||       |       |       |   0   |         |     01    |     11    |     10    |
// |  8  ||       |       |       |   0   |         |     10    |     00    |     10    |
// |  9  ||       |       |       |   0   |         |     10    |     01    |     11    |
// |  10 ||       |       |       |   0   |         |     10    |     10    |     00    |
// |  11 ||       |       |       |   0   |         |     10    |     11    |     01    |
// |  12 ||       |       |       |   0   |         |     11    |     00    |     11    |
// |  13 ||       |       |       |   0   |         |     11    |     01    |     10    |
// |  14 ||       |       |       |   0   |         |     11    |     10    |     01    |
// |  15 ||       |       |       |   0   |         |     11    |     11    |     00    |
// |-----||-------|-------|-------|-------|---------|-----------|-----------|-----------|
//
// where row 0 is used to allocate the prover's private inputs (`a` and `b`) and public input (`c`).
// Row 0 is also used to allocate the verifier's public input (`PI`) where consistency between the
// prover's and verifier's public inputs `c = PI` is checked using a public-input gate (`s_pub`
// selector). Every legal input-output tuples for the 2-bit XOR function `xor(l, r) = o` is written
// into a row of the fixed columns `xor_r_col`, `xor_l_col`, and `xor_o_col`. Note that this circuit
// does not range check any of `a`, `b`, or `c` to be 2-bits, i.e. there are no polynomial
// constraints which explicity check `a, b, c ∈ {0, 1, 2, 3}`.

use halo2::{
    circuit::{layouter::SingleChipLayouter, Chip, Layouter},
    dev::{MockProver, VerifyFailure},
    pasta::Fp,
    plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Fixed, Selector},
    poly::Rotation,
};

const XOR_BITS: usize = 2;

struct XorChip {
    config: XorChipConfig,
}

#[derive(Clone, Debug)]
struct XorChipConfig {
    l_col: Column<Advice>,
    r_col: Column<Advice>,
    o_col: Column<Advice>,
    xor_l_col: Column<Fixed>,
    xor_r_col: Column<Fixed>,
    xor_o_col: Column<Fixed>,
    s_pub: Selector,
}

impl Chip<Fp> for XorChip {
    type Config = XorChipConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl XorChip {
    fn new(config: XorChipConfig) -> Self {
        XorChip { config }
    }

    fn configure(cs: &mut ConstraintSystem<Fp>) -> XorChipConfig {
        let l_col = cs.advice_column();
        let r_col = cs.advice_column();
        let o_col = cs.advice_column();
        let pub_col = cs.instance_column();
        let s_pub = cs.selector();

        cs.create_gate("public input", |cs| {
            let o = cs.query_advice(o_col, Rotation::cur());
            let pi = cs.query_instance(pub_col, Rotation::cur());
            let s_pub = cs.query_selector(s_pub, Rotation::cur());
            s_pub * (o - pi)
        });

        let xor_l_col = cs.fixed_column();
        let xor_r_col = cs.fixed_column();
        let xor_o_col = cs.fixed_column();

        let xor_lookup_query = [
            cs.query_any(l_col.into(), Rotation::cur()),
            cs.query_any(r_col.into(), Rotation::cur()),
            cs.query_any(o_col.into(), Rotation::cur()),
        ];
        let xor_table_query = [
            cs.query_any(xor_l_col.into(), Rotation::cur()),
            cs.query_any(xor_r_col.into(), Rotation::cur()),
            cs.query_any(xor_o_col.into(), Rotation::cur()),
        ];
        cs.lookup(&xor_lookup_query, &xor_table_query);

        XorChipConfig {
            l_col,
            r_col,
            o_col,
            xor_l_col,
            xor_r_col,
            xor_o_col,
            s_pub,
        }
    }

    // Allocates all legal input-output tuples for the XOR function in the first
    // `2^XOR_BITS * 2^XOR_BITS = 16` rows of the constraint system.
    fn alloc_table(&self, layouter: &mut impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_region(
            || "xor table",
            |mut region| {
                let mut row_offset = 0;
                for l in 0..1 << XOR_BITS {
                    for r in 0..1 << XOR_BITS {
                        region.assign_fixed(
                            || format!("xor_l_col row {}", row_offset),
                            self.config.xor_l_col,
                            row_offset,
                            || Ok(Fp::from(l)),
                        )?;
                        region.assign_fixed(
                            || format!("xor_r_col row {}", row_offset),
                            self.config.xor_r_col,
                            row_offset,
                            || Ok(Fp::from(r)),
                        )?;
                        region.assign_fixed(
                            || format!("xor_o_col row {}", row_offset),
                            self.config.xor_o_col,
                            row_offset,
                            || Ok(Fp::from(l ^ r)),
                        )?;
                        row_offset += 1;
                    }
                }
                Ok(())
            },
        )
    }

    // Allocates `a`, `b`, and `c` and enables `s_pub` in row #0, i.e. the first available row where
    // the `l_col`, `r_col`, and `o_col` cells have not been alloacted.
    fn alloc_private_and_public_inputs(
        &self,
        layouter: &mut impl Layouter<Fp>,
        a: Option<Fp>,
        b: Option<Fp>,
        c: Option<Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "private and public inputs",
            |mut region| {
                let row_offset = 0;
                self.config.s_pub.enable(&mut region, row_offset)?;
                region.assign_advice(
                    || "private input `a`",
                    self.config.l_col,
                    row_offset,
                    || a.ok_or(Error::SynthesisError),
                )?;
                region.assign_advice(
                    || "private input `b`",
                    self.config.r_col,
                    row_offset,
                    || b.ok_or(Error::SynthesisError),
                )?;
                region.assign_advice(
                    || "public input `c`",
                    self.config.o_col,
                    row_offset,
                    || c.ok_or(Error::SynthesisError),
                )?;
                Ok(())
            },
        )
    }
}

// Proves knowledge of `a` and `b` such that `xor(a, b) == c` for public input `c`.
#[derive(Clone)]
struct XorCircuit {
    // Private inputs.
    a: Option<Fp>,
    b: Option<Fp>,
    // Public input (from prover).
    c: Option<Fp>,
}

impl Circuit<Fp> for XorCircuit {
    type Config = XorChipConfig;

    fn configure(cs: &mut ConstraintSystem<Fp>) -> Self::Config {
        XorChip::configure(cs)
    }

    fn synthesize(&self, cs: &mut impl Assignment<Fp>, config: Self::Config) -> Result<(), Error> {
        let mut layouter = SingleChipLayouter::new(cs)?;
        let xor_chip = XorChip::new(config);
        xor_chip.alloc_table(&mut layouter)?;
        xor_chip.alloc_private_and_public_inputs(&mut layouter, self.a, self.b, self.c)
    }
}

fn main() {
    // The number of rows used in the constraint system matrix.
    const N_ROWS_USED: u32 = 16;

    // The row index for the public input.
    const PUB_INPUT_ROW: usize = 0;

    // The verifier's public input.
    const PUB_INPUT: u64 = 3;

    // The actual number of rows in the constraint system is `2^k` where `N_ROWS_USED <= 2^k`.
    let k = (N_ROWS_USED as f32).log2().ceil() as u32;
    let n_rows = 1 << k;
    let mut pub_inputs = vec![Fp::zero(); n_rows];
    pub_inputs[PUB_INPUT_ROW] = Fp::from(PUB_INPUT);

    // Assert that the lookup passes because `xor(2, 1) == PUB_INPUT`.
    let circuit = XorCircuit {
        a: Some(Fp::from(2)),
        b: Some(Fp::from(1)),
        c: Some(Fp::from(PUB_INPUT)),
    };
    let prover = MockProver::run(k, &circuit, vec![pub_inputs.clone()]).unwrap();
    assert!(prover.verify().is_ok());

    // Assert that the public input gate is unsatisfied when `c != PUB_INPUT` (but when the lookup
    // passes).
    let bad_circuit = XorCircuit {
        a: Some(Fp::from(2)),
        b: Some(Fp::from(0)),
        c: Some(Fp::from(2)),
    };
    let prover = MockProver::run(k, &bad_circuit, vec![pub_inputs.clone()]).unwrap();
    match prover.verify() {
        Err(errors) => {
            assert_eq!(errors.len(), 1, "expected one verification error, found: {:?}", errors);
            match &errors[0] {
                VerifyFailure::Gate { .. } => {}
                e => panic!("expected 'public input' gate error, found: {:?}", e),
            };
        }
        _ => panic!("expected `prover.verify()` to fail"),
    };

    // Assert that the lookup fails when `(a, b, c)` is not a row in the table; the lookup table is
    // for 2-bit XOR, using a 3-bit XOR input `a = 4` should result in a lookup failure.
    let mut bad_circuit = circuit.clone();
    bad_circuit.a = Some(Fp::from(4));
    let prover = MockProver::run(k, &bad_circuit, vec![pub_inputs]).unwrap();
    match prover.verify() {
        Err(errors) => {
            assert_eq!(errors.len(), 1, "expected one verification error");
            match &errors[0] {
                VerifyFailure::Lookup { .. } => {}
                e => panic!("expected lookup error, found: {:?}", e),
            };
        }
        _ => panic!("expected `prover.verify()` to fail"),
    };
}
