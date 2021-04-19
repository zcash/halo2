use std::ops::Add;

use group::Curve;
use pasta_curves::{arithmetic::CurveAffine, pallas};
use subtle::{ConstantTimeEq, CtOption};

/// P ∪ {⊥}
///
/// Simulated incomplete addition built over complete addition.
#[derive(Clone, Copy, Debug)]
pub(super) struct IncompletePoint(CtOption<pallas::Point>);

impl From<pallas::Point> for IncompletePoint {
    fn from(p: pallas::Point) -> Self {
        IncompletePoint(CtOption::new(p, 1.into()))
    }
}

impl From<IncompletePoint> for CtOption<pallas::Point> {
    fn from(p: IncompletePoint) -> Self {
        p.0
    }
}

impl Add for IncompletePoint {
    type Output = IncompletePoint;

    fn add(self, rhs: Self) -> Self::Output {
        // ⊥ ⊹ ⊥ = ⊥
        // ⊥ ⊹ P = ⊥
        IncompletePoint(self.0.and_then(|p| {
            // P ⊹ ⊥ = ⊥
            rhs.0.and_then(|q| {
                // 0 ⊹ 0 = ⊥
                // 0 ⊹ P = ⊥
                p.to_affine().coordinates().and_then(|c_p| {
                    // P ⊹ 0 = ⊥
                    q.to_affine().coordinates().and_then(|c_q| {
                        // (x, y) ⊹ (x', y') = ⊥ if x == x'
                        // (x, y) ⊹ (x', y') = (x, y) + (x', y') if x != x'
                        CtOption::new(p + q, !c_p.x().ct_eq(c_q.x()))
                    })
                })
            })
        }))
    }
}

impl Add<pallas::Point> for IncompletePoint {
    type Output = IncompletePoint;

    fn add(self, rhs: pallas::Point) -> Self::Output {
        self + IncompletePoint(CtOption::new(rhs, 1.into()))
    }
}
