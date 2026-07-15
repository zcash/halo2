# Dynamic lookups
*Note: This is an unstable nightly feature and can be enabled with the flag `#[cfg(feature = unstable-dynamic-lookups)]`.*

The current `halo2` [lookup argument](../design/proving-system/lookup.md) only supports fixed tables, whose values are fixed as part of the constraint system definition. However, some use-cases require looking up witnessed values, which can change with each instance: for example, a set of preimages and hashes of the SHA256 hash function.

## `DynamicTable`
A `DynamicTable` is associated with:
   ​     |  description
--------|--------------------------------------------------------------------------------------------------------------------------------------
`index` | This table's index in the constraint system's list of dynamic tables
columns | A list of `Advice` and `Fixed` columns where this table's values can be assigned.
  rows  | The rows in this table's columns, where its values are assigned. Every table column must be assigned in the same rows.
 `tag`  | `index` + 1; the tag is appended to each lookup argument involving this table.

Consider a `DynamicTable` with `index = 1`, `tag = index + 1 = 2`. It is assigned on advice column `A1` and fixed column `F0`, on rows `0`, `2`, `3`. Note that the table need not occupy a contiguous area in the circuit:

| row | A0 | A1 | A2 | F0 | F1 |
|-----|----|----|----|----|----|
|  0  |    | ⬤ |    | ⬤ |    |
|  1  |    | `x`|    | `x`|    |
|  2  |    | ⬤ |    | ⬤ |    |
|  3  |    | ⬤ |    | ⬤ |    |

Now, we specify a lookup `[(q_lookup * A0, A1), (q_lookup * A2,F0)]`: i.e., on the rows where `q_lookup = 1` is enabled, we enforce that the values in `A0` (`A2`) appear somewhere in the table column `A1` (`F0`). If all inputs (◯, ◯) indeed map to a table entry (⬤, ⬤), the lookup argument passes.
| row | q_lookup | A0 | A1 | A2 | F0 | F1 |
|-----|----------|----|----|----|----|----|
|  0  |     1    |  ◯ | ⬤ |  ◯ | ⬤ |    |
|  1  |     1    |  ◯ | `x`|  ◯ | `x`|    |
|  2  |     0    |    | ⬤ |    | ⬤ |    |
|  3  |     0    |    | ⬤ |    | ⬤ |    |

## Table tag
A subtle problem arises in the example above: the cells marked `x` are in the table *columns*, but are not part of the table's *rows*, and therefore must not be considered valid lookup values. To enforce this, we:
- repurpose the fixed column `F1` to serve as the table's "tag column", and
- append the table's **tag** to the lookup argument: `[(q_lookup * A0, A1), (q_lookup * A2,F0), (q_lookup * 2, q_lookup * F1)]`.

In other words, we append a **tag** to get input entries of the form (◯, ◯, 2), in order to match the tagged table entries (⬤, ⬤, 2).
| row | q_lookup | A0 | A1 | A2 | F0 | F1 |
|-----|----------|----|----|----|----|----|
|  0  |     1    |  ◯ | ⬤ |  ◯ | ⬤ |  2 |
|  1  |     1    |  ◯ | `x`|  ◯ | `x`|  0 |
|  2  |     0    |    | ⬤ |    | ⬤ |  2 |
|  3  |     0    |    | ⬤ |    | ⬤ |  2 |

Notice that if we now attempt to lookup some illegal inputs `x` on row 1, the lookup argument will fail, since the input `(x, x, 2)` does not appear anywhere in the table:
```ignore
  [(q_lookup * A0, A1), (q_lookup * A2, F0), (q_lookup * 2, q_lookup * F1)]
= [(1 * x, A1), (1 * x, F0), (1 * 2, 1 * F1)]
= [(x, A1), (x, F0), (2, F1)]
```

| row | q_lookup | A0 | A1 | A2 | F0 | F1 |
|-----|----------|----|----|----|----|----|
|  0  |     1    |  ◯ | ⬤ |  ◯ | ⬤ |  2 |
|  1  |     1    | `x`| `x`| `x`| `x`|  0 |
|  2  |     0    |    | ⬤ |    | ⬤ |  2 |
|  3  |     0    |    | ⬤ |    | ⬤ |  2 |

Table tags also enable an important optimization: it allows unused rows in table columns to be recycled for other purposes, without affecting the lookup argument. By the same reasoning, this also allows dynamic tables to be "stacked" vertically by the layouter. This optimization may easily be applied to fixed lookups too.

Dynamic table tags are assigned to fixed columns in order to enforce their shape in the circuit: even though the values of a table can change across instances, its shape must stay the same. Under the hood, we use the [selector combining](../design/implementation/selector-combining.md) algorithm to combine table tags into fewer fixed columns without overlapping.

## Usage
### Creating dynamic tables
A `DynamicTable` is created using the `ConstraintSystem::create_dynamic_table()` method. The `Fixed` and `Advice` columns involved in the table are passed as arguments. (NB: `Instance` columns are not yet supported.)

```ignore
pub fn create_dynamic_table(
    &mut self,
    name: impl Into<String>,
    fixed_columns: &[Column<Fixed>],
    advice_columns: &[Column<Advice>],
) -> DynamicTable;
```

### Configuring dynamic lookups
A `DynamicTable` can be used in a lookup argument configuration using the `ConstraintSystem::lookup_dynamic()` method. This works similarly to `ConstraintSystem::lookup()`; however, it additionally:
- enforces that the table columns involved in the argument are indeed part of the provided `DynamicTable`; and
- adds the table tag to the lookup argument, as described in [[Table tag]](#table-tag).

### Assigning table values
Table values are assigned within regions, using the method `DynamicTable::add_row(&self, region: Region, offset: usize)`. As usual, the absolute rows involved in the dynamic table are only known after the floor planner rearranges the regions. (See [[Chips]](../concepts/chips.md) for an explanation of floor planners and regions.)

### Assigning input values
Input values are assigned within regions, and are usually identified by enabling the appropriate `q_lookup` selector at the desired offset. This is the same behavior as in assigning inputs to fixed tables.