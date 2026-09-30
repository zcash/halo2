use super::Expression;
use ff::Field;

/// Describes a selector's activation pattern and degree budget.
///
/// Only selectors with `max_degree > 0` are eligible for combining.
/// A `max_degree` of 0 indicates either a complex (non-simple) selector
/// or a simple selector that does not appear in any gate; such selectors
/// are allocated their own dedicated fixed column without combining.
#[derive(Debug, Clone)]
pub struct SelectorDescription {
    /// The selector that this description references, by index.
    pub selector: usize,

    /// The vector of booleans defining which rows are active for this selector.
    pub activations: Vec<bool>,

    /// The maximum degree of a gate involving this selector, including the
    /// virtual selector itself. This means this will be at least 1 for any
    /// expression containing a simple selector, even if that selector is not
    /// multiplied by anything.
    pub max_degree: usize,
}

/// This describes the assigned combination of a particular selector as well as
/// the expression it should be substituted with.
#[derive(Debug, Clone)]
pub struct SelectorAssignment<F> {
    /// The selector that this structure references, by index.
    pub selector: usize,

    /// The combination this selector was assigned to
    pub combination_index: usize,

    /// The expression we wish to substitute with
    pub expression: Expression<F>,
}

/// Combines compatible simple selectors into shared fixed columns.
///
/// # Background
///
/// Every simple selector `s_i` appears in gate expressions only in the
/// factored form `s_i * t_{i,j}` where `t_{i,j}` contains no simple
/// selectors (this invariant is enforced by the `Expression` arithmetic
/// operators).  The optimization replaces each virtual boolean selector
/// with an expression over a shared fixed column `q` that evaluates to
/// nonzero exactly on the rows where that selector is active.
///
/// # Combining strategy
///
/// Multiple selectors that are never simultaneously active (i.e., their
/// activation patterns are disjoint) can share the same fixed column.
/// If selectors `s_1, ..., s_m` are combined into one column `q`, the
/// column is assigned distinct nonzero "roots" `r_1, ..., r_m` on each
/// selector's active rows (and 0 elsewhere).  Each selector is then
/// replaced by the expression:
///
/// ```text
///   q * prod_{k=1..m, k != i} (r_k - q)
/// ```
///
/// This expression is nonzero iff `q = r_i` (i.e., selector `i` is active),
/// and zero when `q = 0` (no selector active) or `q = r_k` for `k != i`.
/// The substitution expression has degree `m` (one factor of `q` plus
/// `m - 1` factors from the product).
///
/// # Degree budget
///
/// For a gate `s_i * t_{i,j} = 0` of total degree `d`, the selector
/// `s_i` contributes degree 1.  After substitution, the selector is
/// replaced by an expression of degree `m`, so the gate becomes degree
/// `(d - 1) + m`.  We require `(d - 1) + m <= max_degree`, i.e.,
/// `m <= max_degree - (d - 1)`.  This is enforced by the combination
/// loop below.
///
/// # Arguments
///
/// * `selectors` - a vector of `SelectorDescription`s describing each
///   selector's activation pattern and degree budget.
/// * `max_degree` - the maximum allowed degree of any gate expression.
/// * `allocate_fixed_column` - a closure that allocates a new fixed column
///   and returns the `Expression` for querying it at `Rotation::cur()`.
///
/// # Returns
///
/// A pair of:
/// * `Vec<Vec<F>>` - the assignment of each new fixed column (one per
///   combination).
/// * `Vec<SelectorAssignment<F>>` - the expression substitutions for
///   each selector.
///
/// This function is completely deterministic.
pub fn process<F: Field, E>(
    mut selectors: Vec<SelectorDescription>,
    max_degree: usize,
    mut allocate_fixed_column: E,
) -> (Vec<Vec<F>>, Vec<SelectorAssignment<F>>)
where
    E: FnMut() -> Expression<F>,
{
    if selectors.is_empty() {
        // There is nothing to optimize.
        return (vec![], vec![]);
    }

    // The length of all provided selectors must be the same.
    let n = selectors[0].activations.len();
    assert!(selectors.iter().all(|a| a.activations.len() == n));

    let mut combination_assignments = vec![];
    let mut selector_assignments = vec![];

    // Selectors with max_degree 0 are either complex (non-simple) selectors
    // or simple selectors not used in any gate.  In either case they cannot
    // participate in combining: complex selectors may appear in sums (not
    // just as a multiplicative factor), and unused selectors have no gate
    // to benefit from.  Each gets its own fixed column.
    selectors.retain(|selector| {
        if selector.max_degree == 0 {
            // This is a complex selector, or a selector that does not appear in any
            // gate constraint.
            let expression = allocate_fixed_column();

            let combination_assignment = selector
                .activations
                .iter()
                .map(|b| if *b { F::ONE } else { F::ZERO })
                .collect::<Vec<_>>();
            let combination_index = combination_assignments.len();
            combination_assignments.push(combination_assignment);
            selector_assignments.push(SelectorAssignment {
                selector: selector.selector,
                combination_index,
                expression,
            });

            false
        } else {
            true
        }
    });

    // All remaining selectors are simple (max_degree > 0).  We try to combine
    // them into shared fixed columns.  Two selectors can share a column only
    // if their activations are disjoint (never both enabled on the same row).
    //
    // We compute a lower-triangular exclusion matrix: entry (j, k) is true
    // when selectors j and k conflict (both active on some row).  The matrix
    // is symmetric with false diagonal, so the lower triangle suffices.
    let mut exclusion_matrix = (0..selectors.len())
        .map(|i| vec![false; i])
        .collect::<Vec<_>>();

    for (i, rows) in selectors
        .iter()
        .map(|selector| &selector.activations)
        .enumerate()
    {
        // Loop over the selectors previous to this one
        for (j, other_selector) in selectors.iter().enumerate().take(i) {
            // Look at what selectors are active at the same row
            if rows
                .iter()
                .zip(other_selector.activations.iter())
                .any(|(l, r)| l & r)
            {
                // Mark them as incompatible
                exclusion_matrix[i][j] = true;
            }
        }
    }

    // Simple selectors that we've added to combinations already.
    let mut added = vec![false; selectors.len()];

    for (i, selector) in selectors.iter().enumerate() {
        if added[i] {
            continue;
        }
        added[i] = true;
        assert!(selector.max_degree <= max_degree);
        // This is used to keep track of the largest degree gate involved in the
        // combination so far. We subtract by one to omit the virtual selector
        // which will be substituted by the caller with the expression we give
        // them.
        let mut d = selector.max_degree - 1;
        let mut combination = vec![selector];
        let mut combination_added = vec![i];

        // Try to find other selectors that can join this one.
        'try_selectors: for (j, selector) in selectors.iter().enumerate().skip(i + 1) {
            if d + combination.len() == max_degree {
                // Short circuit; nothing can be added to this
                // combination.
                break 'try_selectors;
            }

            // Skip selectors that have been added to previous combinations
            if added[j] {
                continue 'try_selectors;
            }

            // Is this selector excluded from co-existing in the same
            // combination with any of the other selectors so far?
            for &i in combination_added.iter() {
                if exclusion_matrix[j][i] {
                    continue 'try_selectors;
                }
            }

            // Can the new selector join the combination? Reminder: we use
            // selector.max_degree - 1 to omit the influence of the virtual
            // selector on the degree, as it will be substituted.
            let new_d = std::cmp::max(d, selector.max_degree - 1);
            if new_d + combination.len() + 1 > max_degree {
                // Guess not.
                continue 'try_selectors;
            }

            d = new_d;
            combination.push(selector);
            combination_added.push(j);
            added[j] = true;
        }

        // Now, compute the selector and combination assignments.
        let mut combination_assignment = vec![F::ZERO; n];
        let combination_len = combination.len();
        let combination_index = combination_assignments.len();
        let query = allocate_fixed_column();

        let mut assigned_root = F::ONE;
        selector_assignments.extend(combination.into_iter().map(|selector| {
            // Compute the expression for substitution. This produces an expression of the
            // form
            //     q * Prod[i = 1..=combination_len, i != assigned_root](i - q)
            //
            // which is non-zero only on rows where `combination_assignment` is set to
            // `assigned_root`. In particular, rows set to 0 correspond to all selectors
            // being disabled.
            let mut expression = query.clone();
            let mut root = F::ONE;
            for _ in 0..combination_len {
                if root != assigned_root {
                    expression = expression * (Expression::Constant(root) - query.clone());
                }
                root += F::ONE;
            }

            // Update the combination assignment
            for (combination, selector) in combination_assignment
                .iter_mut()
                .zip(selector.activations.iter())
            {
                // This will not overwrite another selector's activations because
                // we have ensured that selectors are disjoint.
                if *selector {
                    *combination = assigned_root;
                }
            }

            assigned_root += F::ONE;

            SelectorAssignment {
                selector: selector.selector,
                combination_index,
                expression,
            }
        }));
        combination_assignments.push(combination_assignment);
    }

    (combination_assignments, selector_assignments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{plonk::FixedQuery, poly::Rotation};
    use pasta_curves::Fp;
    use proptest::collection::{vec, SizeRange};
    use proptest::prelude::*;

    prop_compose! {
        fn arb_selector(assignment_size: usize, max_degree: usize)
                       (degree in 0..max_degree,
                        assignment in vec(any::<bool>(), assignment_size))
                       -> (usize, Vec<bool>) {
            (degree, assignment)
        }
    }

    prop_compose! {
        fn arb_selector_list(assignment_size: usize, max_degree: usize, num_selectors: impl Into<SizeRange>)
                            (list in vec(arb_selector(assignment_size, max_degree), num_selectors))
                            -> Vec<SelectorDescription>
        {
            list.into_iter().enumerate().map(|(i, (max_degree, activations))| {
                SelectorDescription {
                    selector: i,
                    activations,
                    max_degree,
                }
            }).collect()
        }
    }

    prop_compose! {
        fn arb_instance(max_assignment_size: usize,
                        max_degree: usize,
                        max_selectors: usize)
                       (assignment_size in 1..max_assignment_size,
                        degree in 1..max_degree,
                        num_selectors in 1..max_selectors)
                       (list in arb_selector_list(assignment_size, degree, num_selectors),
                        degree in Just(degree))
                       -> (Vec<SelectorDescription>, usize)
        {
            (list, degree)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn test_selector_combination((selectors, max_degree) in arb_instance(10, 10, 15)) {
            let mut query = 0;
            let (combination_assignments, selector_assignments) =
                process::<Fp, _>(selectors.clone(), max_degree, || {
                    let tmp = Expression::Fixed(FixedQuery {
                        index: query,
                        column_index: query,
                        rotation: Rotation::cur(),
                    });
                    query += 1;
                    tmp
                });

            {
                let mut selectors_seen = vec![];
                assert_eq!(selectors.len(), selector_assignments.len());
                for selector in &selector_assignments {
                    // Every selector should be assigned to a combination
                    assert!(selector.combination_index < combination_assignments.len());
                    assert!(!selectors_seen.contains(&selector.selector));
                    selectors_seen.push(selector.selector);
                }
            }

            // Test that, for each selector, the provided expression
            //  1. evaluates to zero on rows where the selector's activation is off
            //  2. evaluates to nonzero on rows where the selector's activation is on
            //  3. is of degree d such that d + (selector.max_degree - 1) <= max_degree
            //     OR selector.max_degree is zero
            for selector in selector_assignments {
                assert_eq!(
                    selectors[selector.selector].activations.len(),
                    combination_assignments[selector.combination_index].len()
                );
                for (&activation, &assignment) in selectors[selector.selector]
                    .activations
                    .iter()
                    .zip(combination_assignments[selector.combination_index].iter())
                {
                    let eval = selector.expression.evaluate(
                        &|c| c,
                        &|_| panic!("should not occur in returned expressions"),
                        &|query| {
                            // Should be the correct combination in the expression
                            assert_eq!(selector.combination_index, query.index);
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
                        assert!(!eval.is_zero_vartime());
                    } else {
                        assert!(eval.is_zero_vartime());
                    }
                }

                let expr_degree = selector.expression.degree();
                assert!(expr_degree <= max_degree);
                if selectors[selector.selector].max_degree > 0 {
                    assert!(
                        (selectors[selector.selector].max_degree - 1) + expr_degree <= max_degree
                    );
                }
            }
        }
    }
}
