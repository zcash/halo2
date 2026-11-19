// This circuit implements a simple range check `a ∈ [RANGE_FIRST, RANGE_LAST]` for witness `a`.
//
// The prover allocates a single private value `a` in the advice column `a_col` and enables the
// selector `s_range`. The selector `s_range` toggles the "range" gate whose constraint polynomial
// is a polynomial of minimal degree having a root at each value in the range, i.e.
// `s_range * (a - RANGE_START)...(a - RANGE_LAST)` returns `0` when `s_range = 1` if `a` is a root
// (in the desired range).
//
// The constraint system matrix is:
//
//          Advice    Fixed
// |-----||--------|---------|
// | row || a_col  | s_range |
// |-----||--------|---------|
// |  0  ||   a    |    1    |
// |-----||--------|---------|

use halo2::{
    circuit::{layouter::SingleChipLayouter, Chip, Layouter},
    dev::{MockProver, VerifyFailure},
    pasta::Fp,
    plonk::{Advice, Assignment, Circuit, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

// The first and last values (inclusive) in the range.
const RANGE_FIRST: u64 = 1;
const RANGE_LAST: u64 = 5;

struct RangeChip {
    config: RangeChipConfig,
}

#[derive(Clone, Debug)]
struct RangeChipConfig {
    a_col: Column<Advice>,
    s_range: Selector,
}

impl Chip<Fp> for RangeChip {
    type Config = RangeChipConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl RangeChip {
    fn new(config: RangeChipConfig) -> Self {
        RangeChip { config }
    }

    fn configure(cs: &mut ConstraintSystem<Fp>) -> RangeChipConfig {
        let a_col = cs.advice_column();
        let s_range = cs.selector();

        // `s_range * (a - RANGE_FIRST)...(a - RANGE_LAST)`
        cs.create_gate("range check", |cs| {
            let a = cs.query_advice(a_col, Rotation::cur());
            let s_range = cs.query_selector(s_range, Rotation::cur());
            let mut poly = s_range;
            for i in RANGE_FIRST..=RANGE_LAST {
                let root = Expression::Constant(Fp::from(i));
                poly = poly * (a.clone() - root);
            }
            poly
        });

        RangeChipConfig { a_col, s_range }
    }

    fn alloc_and_range_check(
        &self,
        layouter: &mut impl Layouter<Fp>,
        a: Option<Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "load private inputs",
            |mut region| {
                let row_offset = 0;
                self.config.s_range.enable(&mut region, row_offset)?;
                region.assign_advice(
                    || "private input 'a'",
                    self.config.a_col,
                    row_offset,
                    || a.ok_or(Error::SynthesisError),
                )?;
                Ok(())
            },
        )
    }
}

// Allocates `a` and ensures that it is contained within the range `[RANGE_FIRST, RANGE_LAST]`.
#[derive(Clone)]
struct MyCircuit {
    // Private input.
    a: Option<Fp>,
}

impl Circuit<Fp> for MyCircuit {
    type Config = RangeChipConfig;

    fn configure(cs: &mut ConstraintSystem<Fp>) -> Self::Config {
        RangeChip::configure(cs)
    }

    fn synthesize(&self, cs: &mut impl Assignment<Fp>, config: Self::Config) -> Result<(), Error> {
        let mut layouter = SingleChipLayouter::new(cs)?;
        let chip = RangeChip::new(config);
        chip.alloc_and_range_check(&mut layouter, self.a)
    }
}

fn main() {
    // The number of rows utilized in the constraint system matrix.
    const N_ROWS_USED: u32 = 1;

    // `k` can be zero, which is the case when `N_ROWS_USED = 1`.
    let k = (N_ROWS_USED as f32).log2().ceil() as u32;
    // This circuit has no public inputs.
    let pub_inputs = vec![Fp::zero(); 1 << k];

    // Assert that the constraint system is satisfied when `a ∈ [RANGE_FIRST, RANGE_LAST]`.
    for a in RANGE_FIRST..=RANGE_LAST {
        let circuit = MyCircuit { a: Some(Fp::from(a)) };
        let prover = MockProver::run(k, &circuit, vec![pub_inputs.clone()])
            .expect("failed to synthesize circuit");
        assert!(prover.verify().is_ok());
    }

    // Assert that the constraint system is not satisfied when `a ∉ [RANGE_FIRST, RANGE_LAST]`.
    for bad_a in &[RANGE_FIRST - 1, RANGE_LAST + 1] {
        let bad_circuit = MyCircuit { a: Some(Fp::from(*bad_a)) };
        let prover = MockProver::run(k, &bad_circuit, vec![pub_inputs.clone()])
            .expect("failed to synthesize circuit");
        match prover.verify() {
            Err(errors) => {
                assert_eq!(errors.len(), 1, "expected one verification error, found: {:?}", errors);
                match &errors[0] {
                    VerifyFailure::Gate { .. } => {}
                    err => panic!("expected 'range check' gate failure, found: {:?}", err),
                }
            }
            _ => panic!("expected `prover.verify()` to return an error for `a = {}`", bad_a),
        };
    }
}
