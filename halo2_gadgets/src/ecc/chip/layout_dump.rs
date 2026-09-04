//! Exact exporters for Ironwood's isolated Add and Mul circuit fixtures.

use super::{add, mul, CircuitVersion, EccPoint, NonIdentityEccPoint};
use crate::utilities::lookup_range_check::{LookupRangeCheck, PallasLookupRangeCheckConfig};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    dev::circuit_fixture::{CircuitFixtureRecorder, LeanEnvironment},
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, FloorPlanner},
};
use pasta_curves::pallas;

const K: u32 = 11;
const FIXTURE_IMPORT: &str = "Zcash.Circuits.Fixtures.FixtureTypes";
const FIXTURE_NAMESPACE: &str = "Zcash.Circuits.Fixtures";
const OPEN_NAMESPACE: &str = "Fixtures";
const LEAN_ENVIRONMENT: LeanEnvironment<'static> = LeanEnvironment {
    fixture_import: FIXTURE_IMPORT,
    fixture_namespace: FIXTURE_NAMESPACE,
    open_namespace: OPEN_NAMESPACE,
};

struct MulDumpCircuit;

#[derive(Clone)]
struct MulDumpConfig {
    advices: [Column<Advice>; 10],
    range_check: PallasLookupRangeCheckConfig,
    mul: mul::Config<PallasLookupRangeCheckConfig>,
}

impl Circuit<pallas::Base> for MulDumpCircuit {
    type Config = MulDumpConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self
    }

    fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];
        let table_idx = meta.lookup_table_column();
        let constants = meta.fixed_column();
        meta.enable_constant(constants);
        let range_check = PallasLookupRangeCheckConfig::configure(meta, advices[9], table_idx);
        let add = add::Config::configure(
            meta, advices[0], advices[1], advices[2], advices[3], advices[4], advices[5],
            advices[6], advices[7], advices[8],
        );
        let mul = mul::Config::configure(meta, add, range_check, advices);
        Self::Config {
            advices,
            range_check,
            mul,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        config.range_check.load_range_check_table(&mut layouter)?;
        let (base_x, base_y) = layouter.assign_region(
            || "witness base",
            |mut region| {
                let x = region.assign_advice(
                    || "base_x",
                    config.advices[0],
                    0,
                    Value::<Assigned<pallas::Base>>::unknown,
                )?;
                let y = region.assign_advice(
                    || "base_y",
                    config.advices[1],
                    0,
                    Value::<Assigned<pallas::Base>>::unknown,
                )?;
                Ok((x, y))
            },
        )?;
        let alpha = layouter.assign_region(
            || "witness alpha",
            |mut region| {
                region.assign_advice(
                    || "alpha",
                    config.advices[0],
                    0,
                    Value::<pallas::Base>::unknown,
                )
            },
        )?;
        let base = NonIdentityEccPoint::from_coordinates_unchecked(base_x, base_y);
        config.mul.assign(
            layouter.namespace(|| "mul"),
            alpha,
            &base,
            CircuitVersion::AnchoredBase,
        )?;
        Ok(())
    }
}

struct AddDumpCircuit;

#[derive(Clone)]
struct AddDumpConfig {
    advices: [Column<Advice>; 9],
    add: add::Config,
}

impl Circuit<pallas::Base> for AddDumpCircuit {
    type Config = AddDumpConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self
    }

    fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];
        let add = add::Config::configure(
            meta, advices[0], advices[1], advices[2], advices[3], advices[4], advices[5],
            advices[6], advices[7], advices[8],
        );
        Self::Config { advices, add }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        let (px, py, qx, qy) = layouter.assign_region(
            || "witness pq",
            |mut region| {
                let px = region.assign_advice(
                    || "p_x",
                    config.advices[0],
                    0,
                    Value::<Assigned<pallas::Base>>::unknown,
                )?;
                let py = region.assign_advice(
                    || "p_y",
                    config.advices[1],
                    0,
                    Value::<Assigned<pallas::Base>>::unknown,
                )?;
                let qx = region.assign_advice(
                    || "q_x",
                    config.advices[2],
                    0,
                    Value::<Assigned<pallas::Base>>::unknown,
                )?;
                let qy = region.assign_advice(
                    || "q_y",
                    config.advices[3],
                    0,
                    Value::<Assigned<pallas::Base>>::unknown,
                )?;
                Ok((px, py, qx, qy))
            },
        )?;
        let p = EccPoint::from_coordinates_unchecked(px, py);
        let q = EccPoint::from_coordinates_unchecked(qx, qy);
        layouter.assign_region(
            || "complete point addition",
            |mut region| config.add.assign_region(&p, &q, 0, &mut region),
        )?;
        Ok(())
    }
}

fn capture<C>(
    circuit: &C,
) -> (
    ConstraintSystem<pallas::Base>,
    CircuitFixtureRecorder<pallas::Base>,
)
where
    C: Circuit<pallas::Base>,
{
    let mut meta = ConstraintSystem::default();
    let config = C::configure(&mut meta);
    let constants = meta.lean_dump_constants();
    let mut recorder = CircuitFixtureRecorder::default();
    C::FloorPlanner::synthesize(&mut recorder, circuit, config, constants)
        .expect("synthesis should succeed");
    (meta, recorder)
}

fn field_as_mk_fp(value: &pallas::Base) -> String {
    use group::ff::PrimeField;

    let repr = value.to_repr();
    let limb =
        |index: usize| u64::from_le_bytes(repr[index * 8..(index + 1) * 8].try_into().unwrap());
    format!("mkFp {} {} {} {}", limb(0), limb(1), limb(2), limb(3))
}

const ADD_LAYOUT_HEADER: &str = "-- Auto-generated by the sibling-checkout `ecc::chip::layout_dump::dump_layout_add`
-- (halo2_gadgets/src/ecc/chip/layout_dump.rs; Halo2-Clean VK matching). Do not edit by hand.
-- Regenerate:
--   cargo test --release -p halo2_gadgets --lib ecc::chip::layout_dump::dump_layout_add -- --nocapture
-- Minimal complete-addition harness (9 advices, add(a0..a8)): witness p/q (row 0),
-- one `add::Config::assign_region` (row 1). SimpleFloorPlanner keygen view, k=11,
-- Value::unknown().
import Zcash.Circuits.Fixtures.FixtureTypes
";

const MUL_LAYOUT_HEADER: &str = "-- Auto-generated by halo2 `dump_lean` (Halo2-Clean VK-matching, Phase 2 layout). Do not edit by hand.
-- Phase 2 layout of the isolated mul chain (`configure_mul` / `MulDumpCircuit`,
-- halo2_gadgets/src/ecc/chip/dump.rs). Produced by running `MulDumpCircuit::synthesize` —
-- one real variable-base scalar mul, Value::unknown() witnesses, k=11 — through the
-- `SimpleFloorPlanner` exactly as `keygen_vk` does, and reading:
--   regions    : floor-planner placements (index, name, start row = SimpleFloorPlanner
--                region_start = min row touched by the region window's OWN assigns/selector
--                enables — every region here assigns at region-local offset 0). Regions 3/6
--                regenerated 2026-07-17 by `ecc::chip::layout_dump` (sibling halo2 checkout,
--                halo2_gadgets/src/ecc/chip/layout_dump.rs): the original dump recorded 0/1
--                there (a min-touched attribution bug — out-of-window rows), contradicting
--                its own copyList (init-add copies at absolute rows 2/3) and single_pass.rs
--                placement (main uses advices 0/1, whose tails are 2 after the witness
--                regions). The layout_dump harness reproduces the original ordered copyList
--                byte-for-byte (56/56), pinning harness equivalence;
--   permColumns: cs.permutation.get_columns() in registration order;
--   copyList   : the ORDERED raw copy(leftCol,leftRow,rightCol,rightRow) calls (col indices
--                into permColumns), before the Assembly cycle-merge — the order-sensitive check;
--   sigma      : the keygen permutation Assembly mapping (cycle structure), sparse — only
--                cells where mapping[col][row] != (col,row), as (colIdx, row, colIdx', row');
--   constants  : floor-planner constants allocation in order (value, constantsColIdx, row) —
--                rows the planner picks for constrain_constant/assign_advice_from_constant;
--   fixed      : sparse fixed assignments (colIdx, row, value): the constants column, any
--                loaded lookup-table columns, and the post-compression packed-selector columns
--                (selector_polys from compress_selectors on the REAL activation table).
-- Values are decimal ℕ literals: canonical Pallas-base representatives.
-- Regenerate: cargo run -p halo2_gadgets --features dump-lean --bin dump_lean_mul -- <out_dir>
-- Regenerate (regions/copyList only, sibling checkout):
--   cargo test -p halo2_gadgets --lib ecc::chip::layout_dump -- --nocapture
import Zcash.Circuits.Fixtures.FixtureTypes
";

const ADD_SELMAP_HEADER: &str = "-- Auto-generated by the sibling-checkout `ecc::chip::layout_dump::dump_layout_add`.
-- Do not edit by hand. Regenerate:
--   cargo test --release -p halo2_gadgets --lib ecc::chip::layout_dump::dump_layout_add -- --nocapture";

const MUL_SELMAP_HEADER: &str =
    "-- Auto-generated by halo2 `dump_lean` (Halo2-Clean VK-matching). Do not edit by hand.
-- Selector-compression map (see `compress_selectors.rs`): per selector, the packed
-- fixed column, its combination length, and this selector's assigned root.";

/// Writes the six Halo2-owned fixtures in Ironwood's exact Lean environment and formatting.
///
/// Run with:
/// `IRONWOOD_FIXTURE_OUT=/path/to/Zcash/Circuits/Fixtures cargo test --release \
///   -p halo2_gadgets --lib ecc::chip::layout_dump::dump_ironwood_fixtures \
///   -- --ignored --nocapture`
#[test]
#[ignore = "writes generated Ironwood fixture files"]
fn dump_ironwood_fixtures() {
    use std::fs;
    use std::path::PathBuf;

    let output_dir = PathBuf::from(
        std::env::var_os("IRONWOOD_FIXTURE_OUT")
            .expect("set IRONWOOD_FIXTURE_OUT to the destination fixture directory"),
    );
    fs::create_dir_all(&output_dir).expect("create fixture output directory");

    let n = 1 << K;
    let (add_meta, add_recorder) = capture(&AddDumpCircuit);
    let (mul_meta, mul_recorder) = capture(&MulDumpCircuit);
    let fixtures = [
        (
            "AddLayout.lean",
            add_recorder.render_layout_lean(
                &add_meta,
                K,
                ADD_LAYOUT_HEADER,
                "addLayout",
                LEAN_ENVIRONMENT,
                false,
            ),
        ),
        (
            "AddPost.lean",
            add_recorder.render_cs_lean(&add_meta, n, "addPost", LEAN_ENVIRONMENT, &field_as_mk_fp),
        ),
        (
            "AddSelMap.lean",
            add_recorder.render_selector_map_lean(
                &add_meta,
                n,
                ADD_SELMAP_HEADER,
                LEAN_ENVIRONMENT,
                "addSelMap",
                true,
            ),
        ),
        (
            "MulLayout.lean",
            mul_recorder.render_layout_lean(
                &mul_meta,
                K,
                MUL_LAYOUT_HEADER,
                "mulLayout",
                LEAN_ENVIRONMENT,
                true,
            ),
        ),
        (
            "MulPost.lean",
            mul_recorder.render_cs_lean(&mul_meta, n, "mulPost", LEAN_ENVIRONMENT, &field_as_mk_fp),
        ),
        (
            "MulSelMap.lean",
            mul_recorder.render_selector_map_lean(
                &mul_meta,
                n,
                MUL_SELMAP_HEADER,
                LEAN_ENVIRONMENT,
                "mulSelMap",
                true,
            ),
        ),
    ];

    for (name, contents) in fixtures {
        let path = output_dir.join(name);
        fs::write(&path, contents).unwrap_or_else(|error| {
            panic!("write {}: {}", path.display(), error);
        });
        println!("wrote {}", path.display());
    }
}
