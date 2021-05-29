use halo2::{arithmetic::FieldExt, plonk::Expression};

fn ones<F: FieldExt>() -> Expression<F> {
    Expression::Constant(F::one())
}

// Helper gates
fn lagrange_interpolate<F: FieldExt>(
    var: Expression<F>,
    points: Vec<u16>,
    evals: Vec<u32>,
) -> (F, Expression<F>) {
    assert_eq!(points.len(), evals.len());
    let deg = points.len();

    fn factorial<F: FieldExt>(n: u64) -> u64 {
        if n < 2 {
            1
        } else {
            n * factorial::<F>(n - 1)
        }
    }

    // Scale the whole expression by factor to avoid divisions
    let factor = factorial::<F>((deg - 1) as u64);

    let numerator = |var: Expression<F>, eval: u32, idx: u64| {
        let mut expr = ones();
        for i in 0..deg {
            let i = i as u64;
            if i != idx {
                expr = expr * (ones() * (-F::one()) * F::from_u64(i) + var.clone());
            }
        }
        expr * F::from_u64(eval.into())
    };
    let denominator = |idx: i32| {
        let mut denom: i32 = 1;
        for i in 0..deg {
            let i = i as i32;
            if i != idx {
                denom *= idx - i
            }
        }
        if denom < 0 {
            -F::one() * F::from_u64(factor / (-denom as u64))
        } else {
            F::from_u64(factor / (denom as u64))
        }
    };

    let mut expr = ones() * F::zero();
    for ((idx, _), eval) in points.iter().enumerate().zip(evals.iter()) {
        expr = expr + numerator(var.clone(), *eval, idx as u64) * denominator(idx as i32)
    }

    (F::from_u64(factor), expr)
}

pub fn range_check<F: FieldExt>(
    value: Expression<F>,
    lower_range: u64,
    upper_range: u64,
) -> Expression<F> {
    let mut expr = ones();
    for i in lower_range..(upper_range + 1) {
        expr = expr * (ones() * (-F::one()) * F::from_u64(i) + value.clone())
    }
    expr
}

// 2-bit range check
pub fn two_bit_range_check<F: FieldExt>(value: Expression<F>) -> Expression<F> {
    range_check(value, 0, (1 << 2) - 1)
}

// 2-bit spread interpolation
pub fn two_bit_spread<F: FieldExt>(dense: Expression<F>, spread: Expression<F>) -> Expression<F> {
    let (factor, lagrange_poly) = lagrange_interpolate(
        dense,
        vec![0b00, 0b01, 0b10, 0b11],
        vec![0b0000, 0b0001, 0b0100, 0b0101],
    );

    lagrange_poly + (spread * factor * (-F::one()))
}

// 3-bit range check
pub fn three_bit_range_check<F: FieldExt>(value: Expression<F>) -> Expression<F> {
    range_check(value, 0, (1 << 3) - 1)
}

// 3-bit spread
pub fn three_bit_spread<F: FieldExt>(dense: Expression<F>, spread: Expression<F>) -> Expression<F> {
    let (factor, lagrange_poly) = lagrange_interpolate(
        dense,
        vec![0b000, 0b001, 0b010, 0b011, 0b100, 0b101, 0b110, 0b111],
        vec![
            0b000000, 0b000001, 0b000100, 0b000101, 0b010000, 0b010001, 0b010100, 0b010101,
        ],
    );

    lagrange_poly + (spread * factor * (-F::one()))
}

/// Spread and range check on 2-bit word
pub fn two_bit_spread_and_range<F: FieldExt>(
    dense: Expression<F>,
    spread: Expression<F>,
) -> Vec<Expression<F>> {
    vec![
        two_bit_range_check(dense.clone()),
        two_bit_spread(dense, spread),
    ]
}

/// Spread and range check on 3-bit word
pub fn three_bit_spread_and_range<F: FieldExt>(
    dense: Expression<F>,
    spread: Expression<F>,
) -> Vec<Expression<F>> {
    vec![
        three_bit_range_check(dense.clone()),
        three_bit_spread(dense, spread),
    ]
}
