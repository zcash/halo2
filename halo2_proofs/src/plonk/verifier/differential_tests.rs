use crate::pasta::Fp;
use rand::rngs::SysRng;
use rand_core::UnwrapErr;
fn check<F: ff::PrimeField>() {
    use crate::{
        plonk::{AdviceQuery, Expression, FixedQuery, InstanceQuery, VerifierExpressions},
        poly::Rotation,
    };

    let fixed = Expression::Fixed(FixedQuery {
        index: 0,
        column_index: 0,
        rotation: Rotation::cur(),
    });
    let advice = Expression::Advice(AdviceQuery {
        index: 0,
        column_index: 0,
        rotation: Rotation::next(),
    });
    let instance = Expression::Instance(InstanceQuery {
        index: 0,
        column_index: 0,
        rotation: Rotation::prev(),
    });
    let shared = fixed.clone() * (Expression::Constant(F::ONE) - fixed);
    let expressions = [
        shared.clone() * (advice.clone() + instance.clone()),
        shared.clone() * (advice - instance),
        shared,
    ];
    let plan = VerifierExpressions::compile(&expressions);
    let mut rng = UnwrapErr(SysRng);
    for count in 0..=7 {
        let fixed = [F::random(&mut rng)];
        let advice = (0..count)
            .map(|_| vec![F::random(&mut rng)])
            .collect::<Vec<_>>();
        let instance = (0..count)
            .map(|_| vec![F::random(&mut rng)])
            .collect::<Vec<_>>();
        let expected = advice
            .iter()
            .zip(&instance)
            .map(|(advice, instance)| {
                expressions
                    .iter()
                    .map(|expression| {
                        expression.evaluate(
                            &|scalar| scalar,
                            &|_| panic!("test circuit has no virtual selectors"),
                            &|query| fixed[query.index],
                            &|query| advice[query.index],
                            &|query| instance[query.index],
                            &|a| -a,
                            &|a, b| a + b,
                            &|a, b| a * b,
                            &|a, scalar| a * scalar,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            plan.evaluate_all(&plan.common_values(&fixed), &fixed, &advice, &instance),
            expected
        );
    }
}

#[test]
fn compiled_gates_match_recursive_evaluation_for_random_instances() {
    check::<Fp>();
    check::<crate::pasta::Fq>();
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(3_000))]
    #[test]
    fn compiled_gates_match_original_random_expression_trees(
        recipe in proptest::collection::vec((0u8..8, 0usize..32, 0usize..32, proptest::prelude::any::<u64>()), 0..12),
        fixed in proptest::collection::vec(proptest::prelude::any::<u64>(), 5),
        evaluations in proptest::collection::vec((
            proptest::collection::vec(proptest::prelude::any::<u64>(), 5),
            proptest::collection::vec(proptest::prelude::any::<u64>(), 5),
        ), 0..6),
    ) {
        use crate::{
            plonk::{AdviceQuery, Expression, FixedQuery, InstanceQuery, VerifierExpressions},
            poly::Rotation,
        };
        use ff::Field;
        let mut expressions = vec![Expression::Constant(Fp::ONE)];
        for (kind, a, b, constant) in recipe {
            let a = expressions[a % expressions.len()].clone();
            let b = expressions[b % expressions.len()].clone();
            let index = usize::try_from(constant % 5).expect("indices below five fit usize");
            expressions.push(match kind {
                0 => -a,
                1 => a + b,
                2 => a * b,
                3 => a * Fp::from(constant),
                4 => Expression::Fixed(FixedQuery {
                    index, column_index: index, rotation: Rotation::cur(),
                }),
                5 => Expression::Advice(AdviceQuery {
                    index, column_index: index, rotation: Rotation::next(),
                }),
                6 => Expression::Instance(InstanceQuery {
                    index, column_index: index, rotation: Rotation::prev(),
                }),
                _ => Expression::Constant(Fp::from(constant)),
            });
        }
        let fixed = fixed.into_iter().map(Fp::from).collect::<Vec<_>>();
        let (advice, instance): (Vec<_>, Vec<_>) = evaluations.into_iter().map(|(a, i)| {
            (a.into_iter().map(Fp::from).collect::<Vec<_>>(),
             i.into_iter().map(Fp::from).collect::<Vec<_>>())
        }).unzip();
        let plan = VerifierExpressions::compile(&expressions);
        let expected = advice.iter().zip(&instance).map(|(a, i)| {
            expressions.iter().map(|expression| expression.evaluate(
                &|scalar| scalar,
                &|_| panic!("test circuit has no virtual selectors"),
                &|query| fixed[query.index],
                &|query| a[query.index],
                &|query| i[query.index],
                &|value| -value,
                &|left, right| left + right,
                &|left, right| left * right,
                &|value, scalar| value * scalar,
            )).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        proptest::prop_assert_eq!(
            plan.evaluate_all(&plan.common_values(&fixed), &fixed, &advice, &instance),
            expected,
        );
    }
}
