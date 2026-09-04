//! Capture and serialization support for reproducible circuit fixtures.

use std::{format, string::String, vec, vec::Vec};

use ff::{Field, PrimeField};

use crate::{
    circuit::Value,
    plonk::{
        Advice, Any, Assigned, Assignment, Column, ConstraintSystem, Error, Fixed, Instance,
        Selector, SelectorAssignmentData,
    },
};

/// The Lean module environment used for generated fixture definitions.
#[derive(Clone, Copy, Debug)]
pub struct LeanEnvironment<'a> {
    /// Module imported by the generated fixture.
    pub fixture_import: &'a str,
    /// Namespace containing the generated definition.
    pub fixture_namespace: &'a str,
    /// Namespace opened within `fixture_namespace`.
    pub open_namespace: &'a str,
}

/// An [`Assignment`] that records the keygen-visible circuit layout without witnesses.
#[derive(Debug)]
pub struct CircuitFixtureRecorder<F: Field> {
    in_region: bool,
    regions: Vec<(String, Option<usize>)>,
    copies: Vec<(Column<Any>, usize, Column<Any>, usize)>,
    constants: Vec<(F, usize, usize)>,
    fixed: Vec<(usize, usize, F)>,
    fills: Vec<(usize, usize, F)>,
    selectors: Vec<(usize, usize)>,
}

impl<F: Field> Default for CircuitFixtureRecorder<F> {
    fn default() -> Self {
        Self {
            in_region: false,
            regions: Vec::new(),
            copies: Vec::new(),
            constants: Vec::new(),
            fixed: Vec::new(),
            fills: Vec::new(),
            selectors: Vec::new(),
        }
    }
}

impl<F: Field> CircuitFixtureRecorder<F> {
    fn touch(&mut self, row: usize) {
        if self.in_region {
            let minimum = &mut self.regions.last_mut().expect("entered region").1;
            *minimum = Some(minimum.map_or(row, |current| current.min(row)));
        }
    }

    fn selector_data(&self, meta: &ConstraintSystem<F>, n: usize) -> SelectorAssignmentData<F> {
        let mut activations = vec![vec![false; n]; meta.lean_dump_num_selectors()];
        for &(selector, row) in &self.selectors {
            activations[selector][row] = true;
        }
        meta.lean_dump_selector_assignments(activations)
    }
}

impl<F: Field> Assignment<F> for CircuitFixtureRecorder<F> {
    fn enter_region<NR, N>(&mut self, name: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
        self.in_region = true;
        self.regions.push((name().into(), None));
    }

    fn exit_region(&mut self) {
        self.in_region = false;
    }

    fn enable_selector<A, AR>(&mut self, _: A, selector: &Selector, row: usize) -> Result<(), Error>
    where
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.touch(row);
        self.selectors.push((selector.0, row));
        Ok(())
    }

    fn query_instance(&self, _: Column<Instance>, _: usize) -> Result<Value<F>, Error> {
        Ok(Value::unknown())
    }

    fn assign_advice<V, VR, A, AR>(
        &mut self,
        _: A,
        _: Column<Advice>,
        row: usize,
        _: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.touch(row);
        Ok(())
    }

    fn assign_fixed<V, VR, A, AR>(
        &mut self,
        _: A,
        column: Column<Fixed>,
        row: usize,
        value: V,
    ) -> Result<(), Error>
    where
        V: FnOnce() -> Value<VR>,
        VR: Into<Assigned<F>>,
        A: FnOnce() -> AR,
        AR: Into<String>,
    {
        self.touch(row);
        let mut assigned = F::ZERO;
        value().map(|value| assigned = Into::<Assigned<F>>::into(value).evaluate());
        if !self.in_region {
            self.constants.push((assigned, column.index(), row));
        }
        self.fixed.push((column.index(), row, assigned));
        Ok(())
    }

    fn copy(
        &mut self,
        left_column: Column<Any>,
        left_row: usize,
        right_column: Column<Any>,
        right_row: usize,
    ) -> Result<(), Error> {
        self.copies
            .push((left_column, left_row, right_column, right_row));
        Ok(())
    }

    fn fill_from_row(
        &mut self,
        column: Column<Fixed>,
        row: usize,
        value: Value<Assigned<F>>,
    ) -> Result<(), Error> {
        let mut assigned = F::ZERO;
        value.map(|value| assigned = value.evaluate());
        self.fills.push((column.index(), row, assigned));
        Ok(())
    }

    fn push_namespace<NR, N>(&mut self, _: N)
    where
        NR: Into<String>,
        N: FnOnce() -> NR,
    {
    }

    fn pop_namespace(&mut self, _: Option<String>) {}
}

impl<F: PrimeField> CircuitFixtureRecorder<F> {
    /// Render the selector-compressed constraint system in Ironwood's compact JSON schema.
    pub fn render_cs_json(&self, meta: &ConstraintSystem<F>, n: usize) -> String {
        let mut activations = vec![vec![false; n]; meta.lean_dump_num_selectors()];
        for &(selector, row) in &self.selectors {
            activations[selector][row] = true;
        }
        meta.lean_dump_compressed(activations)
            .lean_dump_cs_fixture_json(&field_decimal)
    }

    /// Render a complete keygen layout in Ironwood's compact JSON schema.
    ///
    /// `fill_column_order` preserves the order in which expanded fixed-column fills appear
    /// in historical fixtures when that order differs from assignment order.
    pub fn render_layout_json(
        &self,
        meta: &ConstraintSystem<F>,
        k: u32,
        fill_column_order: &[usize],
    ) -> String {
        let n = 1usize << k;
        let permutation_columns = meta.lean_dump_permutation_columns();
        let permutation_index = |column: &Column<Any>| {
            permutation_columns
                .iter()
                .position(|candidate| candidate == column)
                .expect("copied column must participate in the permutation")
        };

        let constants = self
            .constants
            .iter()
            .map(|(value, column, row)| format!("[{},{},{}]", field_decimal(value), column, row))
            .collect::<Vec<_>>()
            .join(",");
        let copies = self
            .copies
            .iter()
            .map(|(left_column, left_row, right_column, right_row)| {
                format!(
                    "[{},{},{},{}]",
                    permutation_index(left_column),
                    left_row,
                    permutation_index(right_column),
                    right_row
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let mut fixed = self.fixed.clone();
        let usable_rows = n - (meta.lean_dump_blinding_factors() + 1);
        let mut fills = self.fills.clone();
        fills.sort_by_key(|(column, _, _)| {
            fill_column_order
                .iter()
                .position(|candidate| candidate == column)
                .expect("every filled column must have a serialization position")
        });
        for (column, first_row, value) in &fills {
            for row in *first_row..usable_rows {
                fixed.push((*column, row, *value));
            }
        }
        let (fixed_before, packed, _) = self.selector_data(meta, n);
        for (packed_index, values) in packed.iter().enumerate() {
            for (row, value) in values.iter().enumerate() {
                if *value != F::ZERO {
                    fixed.push((fixed_before + packed_index, row, *value));
                }
            }
        }
        let fixed = fixed
            .iter()
            .map(|(column, row, value)| format!("[{},{},{}]", column, row, field_decimal(value)))
            .collect::<Vec<_>>()
            .join(",");

        let columns = permutation_columns
            .iter()
            .map(|column| {
                let kind = match column.column_type() {
                    Any::Advice => "advice",
                    Any::Fixed => "fixed",
                    Any::Instance => "instance",
                };
                format!("[\"{}\",{}]", kind, column.index())
            })
            .collect::<Vec<_>>()
            .join(",");
        let regions = self
            .regions
            .iter()
            .enumerate()
            .map(|(index, (name, row))| {
                // The Action circuit contains an empty region. Its historical JSON fixture
                // represents the absent start row with zero, so preserve that sentinel.
                format!("[{},{},{}]", index, json_string(name), row.unwrap_or(0))
            })
            .collect::<Vec<_>>()
            .join(",");

        let mut assembly = SigmaAssembly::new(permutation_columns.len(), n);
        for (left_column, left_row, right_column, right_row) in &self.copies {
            assembly.copy(
                permutation_index(left_column),
                *left_row,
                permutation_index(right_column),
                *right_row,
            );
        }
        let mut sigma = Vec::new();
        for column in 0..permutation_columns.len() {
            for row in 0..n {
                let mapped = assembly.mapping[column][row];
                if mapped != (column, row) {
                    sigma.push(format!("[{},{},{},{}]", column, row, mapped.0, mapped.1));
                }
            }
        }

        format!(
            "{{\"constants\":[{}],\"copyList\":[{}],\"fixed\":[{}],\"k\":{},\"n\":{},\"permColumns\":[{}],\"regions\":[{}],\"sigma\":[{}]}}",
            constants,
            copies,
            fixed,
            k,
            n,
            columns,
            regions,
            sigma.join(","),
        )
    }

    /// Render a selector-compression map in a caller-selected Lean environment.
    pub fn render_selector_map_lean(
        &self,
        meta: &ConstraintSystem<F>,
        n: usize,
        header: &str,
        environment: LeanEnvironment<'_>,
        fixture_name: &str,
        sort_assignments: bool,
    ) -> String {
        use std::fmt::Write as _;

        let (fixed_before, packed, mut assignments) = self.selector_data(meta, n);
        if sort_assignments {
            assignments.sort_by_key(|(selector, _, _)| *selector);
        }
        let entries = assignments
            .iter()
            .map(|(selector, packed_index, root)| {
                let len = assignments
                    .iter()
                    .filter(|(_, candidate, _)| candidate == packed_index)
                    .count();
                format!(
                    "({}, ⟨{}, {}, {}⟩)",
                    selector,
                    fixed_before + packed_index,
                    len,
                    root
                )
            })
            .collect::<Vec<_>>();

        let mut output = format!(
            "{}\nimport {}\n\nnamespace {}\n\nopen {}\n\ndef {} : SelCompressMap :=\n",
            header,
            environment.fixture_import,
            environment.fixture_namespace,
            environment.open_namespace,
            fixture_name
        );
        writeln!(output, "  {{ newFixedCols := {},", packed.len()).unwrap();
        if entries.is_empty() {
            writeln!(output, "    entries := [] }}").unwrap();
        } else {
            writeln!(output, "    entries := [{}] }}", entries.join(",\n      ")).unwrap();
        }
        write!(output, "\nend {}\n", environment.fixture_namespace).unwrap();
        output
    }

    /// Render the selector-compressed constraint system in a caller-selected Lean environment.
    pub fn render_cs_lean(
        &self,
        meta: &ConstraintSystem<F>,
        n: usize,
        fixture_name: &str,
        environment: LeanEnvironment<'_>,
        format_field: &dyn Fn(&F) -> String,
    ) -> String {
        let mut activations = vec![vec![false; n]; meta.lean_dump_num_selectors()];
        for &(selector, row) in &self.selectors {
            activations[selector][row] = true;
        }
        meta.lean_dump_compressed(activations)
            .lean_dump_cs_fixture_in(
                fixture_name,
                environment.fixture_import,
                environment.fixture_namespace,
                environment.open_namespace,
                format_field,
            )
    }

    /// Render a complete keygen layout in Ironwood's Lean fixture schema.
    pub fn render_layout_lean(
        &self,
        meta: &ConstraintSystem<F>,
        k: u32,
        header: &str,
        fixture_name: &str,
        environment: LeanEnvironment<'_>,
        long_data: bool,
    ) -> String {
        use std::fmt::Write as _;

        let n = 1usize << k;
        let permutation_columns = meta.lean_dump_permutation_columns();
        let permutation_index = |column: &Column<Any>| {
            permutation_columns
                .iter()
                .position(|candidate| candidate == column)
                .expect("copied column must participate in the permutation")
        };

        let regions = self
            .regions
            .iter()
            .enumerate()
            .map(|(index, (name, row))| {
                format!(
                    "⟨{}, {:?}, {}⟩",
                    index,
                    name,
                    row.expect("every recorded region must touch a row")
                )
            })
            .collect::<Vec<_>>();
        let columns = permutation_columns
            .iter()
            .map(|column| {
                let kind = match column.column_type() {
                    Any::Advice => "advice",
                    Any::Fixed => "fixed",
                    Any::Instance => "instance",
                };
                format!(".{} {}", kind, column.index())
            })
            .collect::<Vec<_>>();
        let copies = self
            .copies
            .iter()
            .map(|(left_column, left_row, right_column, right_row)| {
                format!(
                    "({}, {}, {}, {})",
                    permutation_index(left_column),
                    left_row,
                    permutation_index(right_column),
                    right_row
                )
            })
            .collect::<Vec<_>>();

        let mut assembly = SigmaAssembly::new(permutation_columns.len(), n);
        for (left_column, left_row, right_column, right_row) in &self.copies {
            assembly.copy(
                permutation_index(left_column),
                *left_row,
                permutation_index(right_column),
                *right_row,
            );
        }
        let mut sigma = Vec::new();
        for column in 0..permutation_columns.len() {
            for row in 0..n {
                let mapped = assembly.mapping[column][row];
                if mapped != (column, row) {
                    sigma.push(format!("({}, {}, {}, {})", column, row, mapped.0, mapped.1));
                }
            }
        }

        let constants = self
            .constants
            .iter()
            .map(|(value, column, row)| format!("({}, {}, {})", field_decimal(value), column, row))
            .collect::<Vec<_>>();
        let mut fixed = self.fixed.clone();
        let usable_rows = n - (meta.lean_dump_blinding_factors() + 1);
        for (column, first_row, value) in &self.fills {
            for row in *first_row..usable_rows {
                fixed.push((*column, row, *value));
            }
        }
        let (fixed_before, packed, _) = self.selector_data(meta, n);
        for (packed_index, values) in packed.iter().enumerate() {
            for (row, value) in values.iter().enumerate() {
                if *value != F::ZERO {
                    fixed.push((fixed_before + packed_index, row, *value));
                }
            }
        }
        fixed.sort_by_key(|(column, row, _)| (*column, *row));
        let fixed = fixed
            .iter()
            .map(|(column, row, value)| format!("({}, {}, {})", column, row, field_decimal(value)))
            .collect::<Vec<_>>();

        let mut output = format!(
            "{}\nnamespace {}\n\nopen {}\n\n",
            header, environment.fixture_namespace, environment.open_namespace
        );
        if long_data {
            output.push_str("-- `maxRecDepth` is raised only to elaborate the long flat data lists (the full lookup-table\n-- column contents can be thousands of rows). This is a data-literal elaboration depth, not a\n-- proof-search/heartbeat budget; the fixture is inert data.\nset_option maxRecDepth 100000 in\n");
        }
        writeln!(output, "def {} : LayoutFixture :=", fixture_name).unwrap();
        writeln!(output, "  {{ k := {},", k).unwrap();
        writeln!(output, "    n := {},", n).unwrap();
        writeln!(
            output,
            "    regions := {},",
            lean_list(&regions).replace("\n      ", " ")
        )
        .unwrap();
        writeln!(
            output,
            "    permColumns := {},",
            lean_list(&columns).replace("\n      ", " ")
        )
        .unwrap();
        writeln!(output, "    copyList := {},", lean_list(&copies)).unwrap();
        writeln!(output, "    sigma := {},", lean_list(&sigma)).unwrap();
        writeln!(
            output,
            "    constants := {},",
            lean_list(&constants).replace("\n      ", " ")
        )
        .unwrap();
        writeln!(output, "    fixed := {} }}", lean_list(&fixed)).unwrap();
        write!(output, "\nend {}\n", environment.fixture_namespace).unwrap();
        output
    }
}

fn lean_list(items: &[String]) -> String {
    if items.is_empty() {
        "[]".into()
    } else {
        format!("[{}]", items.join(",\n      "))
    }
}

fn field_decimal<F: PrimeField>(value: &F) -> String {
    const RADIX: u64 = 1_000_000_000;
    let repr = value.to_repr();
    let mut limbs = vec![0u32];
    for &byte in repr.as_ref().iter().rev() {
        let mut carry = byte as u64;
        for limb in &mut limbs {
            let next = (*limb as u64) * 256 + carry;
            *limb = (next % RADIX) as u32;
            carry = next / RADIX;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
    }
    let mut output = format!("{}", limbs.pop().unwrap());
    for limb in limbs.iter().rev() {
        output.push_str(&format!("{:09}", limb));
    }
    output
}

fn json_string(value: &str) -> String {
    use std::fmt::Write as _;

    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character <= '\u{1f}' => {
                write!(output, "\\u{:04x}", character as u32).unwrap();
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

#[derive(Debug)]
struct SigmaAssembly {
    mapping: Vec<Vec<(usize, usize)>>,
    auxiliary: Vec<Vec<(usize, usize)>>,
    sizes: Vec<Vec<usize>>,
}

impl SigmaAssembly {
    fn new(num_columns: usize, n: usize) -> Self {
        let identity = (0..num_columns)
            .map(|column| (0..n).map(|row| (column, row)).collect())
            .collect::<Vec<Vec<_>>>();
        Self {
            mapping: identity.clone(),
            auxiliary: identity,
            sizes: vec![vec![1; n]; num_columns],
        }
    }

    fn copy(&mut self, left_column: usize, left_row: usize, right_column: usize, right_row: usize) {
        let mut left_cycle = self.auxiliary[left_column][left_row];
        let mut right_cycle = self.auxiliary[right_column][right_row];
        if left_cycle == right_cycle {
            return;
        }
        if self.sizes[left_cycle.0][left_cycle.1] < self.sizes[right_cycle.0][right_cycle.1] {
            std::mem::swap(&mut left_cycle, &mut right_cycle);
        }
        self.sizes[left_cycle.0][left_cycle.1] += self.sizes[right_cycle.0][right_cycle.1];
        let mut position = right_cycle;
        loop {
            self.auxiliary[position.0][position.1] = left_cycle;
            position = self.mapping[position.0][position.1];
            if position == right_cycle {
                break;
            }
        }
        let left_mapping = self.mapping[left_column][left_row];
        self.mapping[left_column][left_row] = self.mapping[right_column][right_row];
        self.mapping[right_column][right_row] = left_mapping;
    }
}
