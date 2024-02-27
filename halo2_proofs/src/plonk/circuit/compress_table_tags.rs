use super::{compress_selectors::exclusion_matrix, Expression};

use ff::Field;

/// This describes a table and where it is activated.
#[derive(Debug, Clone)]
pub struct TableDescription {
    /// The table that this description references, by index.
    pub index: usize,

    /// The vector of booleans defining which rows are active for this table.
    pub activations: Vec<bool>,
}

/// This describes the assigned combination of a particular table as well as
/// the expression it should be substituted with.
#[derive(Debug, Clone)]
pub struct TableAssignment<F> {
    /// The table that this structure references, by index.
    pub index: usize,

    /// The combination this table was assigned to
    pub combination_index: usize,

    /// The expression we wish to substitute with
    pub expression: Expression<F>,
}

/// This function takes a vector that defines each table as well as a closure
/// used to allocate new fixed columns, and returns the assignment of each
/// combination as well as details about each table assignment.
///
/// This function takes
/// * `tables`, a vector of `TableDescription`s that describe each table
/// * `allocate_fixed_columns`, a closure that constructs a new fixed column and
///   queries it at Rotation::cur(), returning the expression
///
/// and returns `Vec<Vec<F>>` containing the assignment of each new fixed column
/// (which each correspond to a combination) as well as a vector of
/// `TableAssignment` that the caller can use to perform the necessary
/// substitutions to the constraint system.
///
/// This function is completely deterministic.
pub fn process<F: Field + From<u64>, E>(
    tables: Vec<TableDescription>,
    mut allocate_fixed_column: E,
) -> (Vec<Vec<F>>, Vec<TableAssignment<F>>)
where
    E: FnMut() -> Expression<F>,
{
    if tables.is_empty() {
        // There is nothing to optimize.
        return (vec![], vec![]);
    }

    // The length of all provided table tags must be the same.
    let n = tables[0].activations.len();
    assert!(tables.iter().all(|a| a.activations.len() == n));

    let mut combination_assignments = vec![];
    let mut table_tag_assignments = vec![];

    let exclusion_matrix = exclusion_matrix(&tables, |table| table.activations.iter().cloned());

    // Virtual tag columns that we've added to combinations already.
    let mut added = vec![false; tables.len()];

    for (i, table) in tables.iter().enumerate() {
        if added[i] {
            continue;
        }
        added[i] = true;
        let mut combination: Vec<_> = vec![table];
        let mut combination_added = vec![i];

        // Try to find other virtual tag columns that can join this one.
        'try_columns: for (j, table) in tables.iter().enumerate().skip(i + 1) {
            // Skip columns that have been added to previous combinations
            if added[j] {
                continue 'try_columns;
            }

            // Is this virtual tag column excluded from co-existing in the same
            // combination with any of the other virtual tag column so far?
            for i in combination_added.iter() {
                if exclusion_matrix[j][*i] {
                    continue 'try_columns;
                }
            }

            combination.push(table);
            combination_added.push(j);
            added[j] = true;
        }

        let mut combination_assignment = vec![F::ZERO; n];
        let combination_index = combination_assignments.len();
        let query = allocate_fixed_column();

        table_tag_assignments.extend(combination.into_iter().map(|table| {
            // Update the combination assignment
            for (combination, active) in combination_assignment
                .iter_mut()
                .zip(table.activations.iter())
            {
                // This will not overwrite another table tag's activations because
                // we have ensured that table tags are disjoint.
                if *active {
                    *combination = F::from(table.index as u64 + 1);
                }
            }

            TableAssignment {
                index: table.index,
                combination_index,
                expression: query.clone(),
            }
        }));

        combination_assignments.push(combination_assignment);
    }

    (combination_assignments, table_tag_assignments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{plonk::FixedQuery, poly::Rotation};
    use pasta_curves::Fp;
    use proptest::collection::{vec, SizeRange};
    use proptest::prelude::*;

    prop_compose! {
        fn arb_table(assignment_size: usize)
                        (assignment in vec(any::<bool>(), assignment_size))
                        -> Vec<bool> {
            assignment
        }
    }

    prop_compose! {
        fn arb_table_list(assignment_size: usize, num_tables: impl Into<SizeRange>)
                            (list in vec(arb_table(assignment_size), num_tables))
                            -> Vec<TableDescription>
        {
            list.into_iter().enumerate().map(|(i, activations)| {
                TableDescription {
                    index: i,
                    activations,
                }
            }).collect()
        }
    }

    prop_compose! {
        fn arb_instance(max_assignment_size: usize,
                        max_tables: usize)
                       (assignment_size in 1..max_assignment_size,
                        num_tables in 1..max_tables)
                       (list in arb_table_list(assignment_size, num_tables))
                       -> Vec<TableDescription>
        {
            list
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn test_table_combination(tables in arb_instance(10, 15)) {
            let mut query = 0;
            let (combination_assignments, table_assignments) =
                process::<Fp, _>(tables.clone(), || {
                    let tmp = Expression::Fixed(FixedQuery {
                        index: query,
                        column_index: query,
                        rotation: Rotation::cur(),
                    });
                    query += 1;
                    tmp
                });

            {
                let mut tables_seen = vec![];
                assert_eq!(tables.len(), table_assignments.len());
                for table in &table_assignments {
                    // Every table should be assigned to a combination
                    assert!(table.combination_index < combination_assignments.len());
                    assert!(!tables_seen.contains(&table.index));
                    tables_seen.push(table.index);
                }
            }

            // Test that, for each table, the provided expression evaluates to
            // the table tag on rows where the table's activation is on
            for table in table_assignments {
                assert_eq!(
                    tables[table.index].activations.len(),
                    combination_assignments[table.combination_index].len()
                );
                for (&activation, &assignment) in tables[table.index]
                    .activations
                    .iter()
                    .zip(combination_assignments[table.combination_index].iter())
                {
                    let eval = table.expression.evaluate(
                        &|c| c,
                        &|_| panic!("should not occur in returned expressions"),
                        #[cfg(feature = "unstable-dynamic-lookups")]
                        &|_| panic!("should not occur in returned expressions"),
                        &|query| {
                            // Should be the correct combination in the expression
                            assert_eq!(table.combination_index, query.index);
                            assignment
                        },
                        &|_| panic!("should not occur in returned expressions"),
                        &|_| panic!("should not occur in returned expressions"),
                        &|a| -a,
                        &|a, b| a + b,
                        &|a, b| a * b,
                        &|a, f| a * f,
                    );

                    if activation {
                        assert_eq!(eval, Fp::from(table.index as u64 + 1));
                    }
                }
            }
        }
    }
}
