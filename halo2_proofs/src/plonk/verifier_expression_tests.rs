use super::{AdviceQuery, Expression, FixedQuery, InstanceQuery, VerifierExpressions};
use crate::poly::Rotation;
use ff::PrimeField;

fn check<F: PrimeField>() {
    let fixed = Expression::Fixed(FixedQuery {
        index: 0,
        column_index: 0,
        rotation: Rotation::cur(),
    });
    let advice = Expression::Advice(AdviceQuery {
        index: 1,
        column_index: 0,
        rotation: Rotation::next(),
    });
    let instance = Expression::Instance(InstanceQuery {
        index: 2,
        column_index: 0,
        rotation: Rotation::prev(),
    });
    let common = fixed.clone() * (Expression::Constant(F::ONE) - fixed.clone());
    let dynamic = advice.clone() + instance.clone();
    let expressions = [
        common.clone() * dynamic.clone(),
        common.clone() * dynamic.clone(),
        -dynamic.clone(),
        dynamic.clone() * -F::from(7),
        Expression::Constant(F::ZERO),
        Expression::Constant(F::ONE),
        Expression::Constant(-F::ONE),
        common,
        dynamic * F::ZERO,
    ];
    let plan = VerifierExpressions::compile(&expressions);
    assert_eq!(plan.roots[0], plan.roots[1]);
    assert!(!plan.common.is_empty());
    assert!(!plan.per_proof.is_empty());

    for fixed_seed in 0_u64..8 {
        let fixed_evals = [F::from(fixed_seed).square()];
        let mut scratch = plan.common_values(&fixed_evals);
        for proof_seed in 0_u64..16 {
            let advice_evals = [F::ZERO, F::from(proof_seed + 3).square()];
            let instance_evals = [F::ZERO, F::ZERO, -F::from(proof_seed + 11).square()];
            let expected = expressions
                .iter()
                .map(|expression| {
                    expression.evaluate(
                        &|scalar| scalar,
                        &|_| panic!("no virtual selectors in the test expressions"),
                        &|query| fixed_evals[query.index],
                        &|query| advice_evals[query.index],
                        &|query| instance_evals[query.index],
                        &|a| -a,
                        &|a, b| a + b,
                        &|a, b| a * b,
                        &|a, scalar| a * scalar,
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                plan.evaluate(&mut scratch, &fixed_evals, &advice_evals, &instance_evals),
                expected,
            );
        }
    }

    let empty = VerifierExpressions::<F>::compile([]);
    assert_eq!(empty.payload_bytes(), 0);
    assert!(empty
        .evaluate(&mut empty.common_values(&[]), &[], &[], &[])
        .is_empty());
}

#[test]
fn compiled_gate_expressions_match_recursive_evaluation() {
    check::<crate::pasta::Fp>();
    check::<crate::pasta::Fq>();
}
