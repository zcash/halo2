//! Metadata about circuits.

use super::metadata::Column as ColumnMetadata;
use crate::plonk::{self, Any};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
};
/// Metadata about a column within a circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Column {
    /// The type of the column.
    pub(super) column_type: Any,
    /// The index of the column.
    pub(super) index: usize,
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Column('{:?}', {})", self.column_type, self.index)
    }
}

impl From<(Any, usize)> for Column {
    fn from((column_type, index): (Any, usize)) -> Self {
        Column { column_type, index }
    }
}

impl From<plonk::Column<Any>> for Column {
    fn from(column: plonk::Column<Any>) -> Self {
        Column {
            column_type: *column.column_type(),
            index: column.index(),
        }
    }
}

/// A helper structure that allows to print a Column with it's annotation as a single structure.
#[derive(Debug, Clone)]
struct DebugColumn {
    /// The type of the column.
    column_type: Any,
    /// The index of the column.
    index: usize,
    /// Annotation of the column
    annotation: String,
}

impl From<(Column, Option<&HashMap<Column, String>>)> for DebugColumn {
    fn from(info: (Column, Option<&HashMap<Column, String>>)) -> Self {
        DebugColumn {
            column_type: info.0.column_type,
            index: info.0.index,
            annotation: info
                .1
                .and_then(|map| map.get(&info.0).cloned())
                .unwrap_or_default(),
        }
    }
}

impl fmt::Display for DebugColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Column('{:?}', {} - {})",
            self.column_type, self.index, self.annotation
        )
    }
}

/// A "virtual cell" is a PLONK cell that has been queried at a particular relative offset
/// within a custom gate.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualCell {
    name: &'static str,
    pub(super) column: Column,
    pub(super) rotation: i32,
}

impl From<(Column, i32)> for VirtualCell {
    fn from((column, rotation): (Column, i32)) -> Self {
        VirtualCell {
            name: "",
            column,
            rotation,
        }
    }
}

impl From<(&'static str, Column, i32)> for VirtualCell {
    fn from((name, column, rotation): (&'static str, Column, i32)) -> Self {
        VirtualCell {
            name,
            column,
            rotation,
        }
    }
}

impl From<plonk::VirtualCell> for VirtualCell {
    fn from(c: plonk::VirtualCell) -> Self {
        VirtualCell {
            name: "",
            column: c.column.into(),
            rotation: c.rotation.0,
        }
    }
}

impl fmt::Display for VirtualCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.column, self.rotation)?;
        if !self.name.is_empty() {
            write!(f, "({})", self.name)?;
        }
        Ok(())
    }
}

/// Metadata about a configured gate within a circuit.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Gate {
    /// The index of the active gate. These indices are assigned in the order in which
    /// `ConstraintSystem::create_gate` is called during `Circuit::configure`.
    pub(super) index: usize,
    /// The name of the active gate. These are specified by the gate creator (such as
    /// a chip implementation), and is not enforced to be unique.
    pub(super) name: &'static str,
}

impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gate {} ('{}')", self.index, self.name)
    }
}

impl From<(usize, &'static str)> for Gate {
    fn from((index, name): (usize, &'static str)) -> Self {
        Gate { index, name }
    }
}

/// Metadata about a configured constraint within a circuit.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Constraint {
    /// The gate containing the constraint.
    pub(super) gate: Gate,
    /// The index of the polynomial constraint within the gate. These indices correspond
    /// to the order in which the constraints are returned from the closure passed to
    /// `ConstraintSystem::create_gate` during `Circuit::configure`.
    pub(super) index: usize,
    /// The name of the constraint. This is specified by the gate creator (such as a chip
    /// implementation), and is not enforced to be unique.
    pub(super) name: &'static str,
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Constraint {}{} in gate {} ('{}')",
            self.index,
            if self.name.is_empty() {
                String::new()
            } else {
                format!(" ('{}')", self.name)
            },
            self.gate.index,
            self.gate.name,
        )
    }
}

impl From<(Gate, usize, &'static str)> for Constraint {
    fn from((gate, index, name): (Gate, usize, &'static str)) -> Self {
        Constraint { gate, index, name }
    }
}

/// Metadata about an assigned region within a circuit.
#[derive(Clone)]
pub struct Region {
    /// The index of the region. These indices are assigned in the order in which
    /// `Layouter::assign_region` is called during `Circuit::synthesize`.
    pub(super) index: usize,
    /// The name of the region. This is specified by the region creator (such as a chip
    /// implementation), and is not enforced to be unique.
    pub(super) name: String,
    /// A reference to the annotations of the Columns that exist within this `Region`.
    pub(super) column_annotations: Option<HashMap<ColumnMetadata, String>>,
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.name == other.name
    }
}

impl Eq for Region {}

impl Debug for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Region {} ('{}')", self.index, self.name)
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Region {} ('{}')", self.index, self.name)
    }
}

impl From<(usize, String)> for Region {
    fn from((index, name): (usize, String)) -> Self {
        Region {
            index,
            name,
            column_annotations: None,
        }
    }
}

impl From<(usize, &str)> for Region {
    fn from((index, name): (usize, &str)) -> Self {
        Region {
            index,
            name: name.to_owned(),
            column_annotations: None,
        }
    }
}

impl From<(usize, String, HashMap<ColumnMetadata, String>)> for Region {
    fn from((index, name, annotations): (usize, String, HashMap<ColumnMetadata, String>)) -> Self {
        Region {
            index,
            name,
            column_annotations: Some(annotations),
        }
    }
}

impl From<(usize, &str, HashMap<ColumnMetadata, String>)> for Region {
    fn from((index, name, annotations): (usize, &str, HashMap<ColumnMetadata, String>)) -> Self {
        Region {
            index,
            name: name.to_owned(),
            column_annotations: Some(annotations),
        }
    }
}
