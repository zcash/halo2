use halo2_proofs::{arithmetic::FieldExt, plonk::Expression};

pub struct Gate<F: FieldExt>(pub Expression<F>);

impl<F: FieldExt> Gate<F> {
    fn identity() -> Expression<F> {
        Expression::Constant(F::one())
    }

    //If number is in range one of the multiplication expressions will be zero 
    pub fn range_check(value: Expression<F>, lower_range: u64, upper_range: u64) -> Expression<F> {
        let mut expr = Self::identity();
        for i in lower_range..(upper_range + 1) {
            expr = expr * (Self::identity() * (-F::one()) * F::from(i) + value.clone())
        }
        expr
    }

    pub fn decompose_32_to_8(
        q_decompose: Expression<F>,
        x: Expression<F>,
        x0: Expression<F>,
        x1: Expression<F>,
        x2: Expression<F>,
        x3: Expression<F>
    ) -> impl Iterator<Item = (&'static str, Expression<F>)> {

        let decomposition_check = x0.clone()
        + x1.clone() * F::from(1 << 8)
        + x2.clone() * F::from(1 << 16)
        + x3.clone() * F::from(1 << 24)
        + x * (-F::one());

        let range_check_x0 = Self::range_check(x0, 0, 255);
        let range_check_x1 = Self::range_check(x1, 0, 255);
        let range_check_x2 = Self::range_check(x2, 0, 255);
        let range_check_x3 = Self::range_check(x3, 0, 255);

        std::iter::empty()
        .chain(Some(("decomposition_check", decomposition_check)))
        .chain(Some(("range_check_x0", range_check_x0)))
        .chain(Some(("range_check_x1", range_check_x1)))
        .chain(Some(("range_check_x2", range_check_x2)))
        .chain(Some(("range_check_x3", range_check_x3)))
        .map(move |(name, poly)| (name, q_decompose.clone() * poly))
    }
}