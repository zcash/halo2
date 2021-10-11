//! Public parameters for processing public inputs.

use super::pedersen::pedersen_hash;
use group::{Curve, Group};
use pasta_curves::arithmetic::CurveAffine;
use std::convert::TryInto;

/// TODO: Figure out how many generators are needed during keygen.
/// NUM_BITS / 4
#[derive(Clone, Debug)]
pub struct Params<C: CurveAffine, const N: usize> {
    pub(super) generators: [C; N],
    pub(super) pedersen_windows: [[C; 4]; N],
}

impl<C: CurveAffine, const N: usize> Params<C, N> {
    pub(super) fn init() -> Self {
        // TODO: use hash-to-curve
        let generators_projective: Vec<_> = {
            use rand::rngs::OsRng;
            let rng = OsRng;

            (0..N).map(|_| C::CurveExt::random(rng)).collect()
        };

        let mut generators = vec![C::identity(); generators_projective.len()];
        C::Curve::batch_normalize(&generators_projective, &mut generators);

        let pedersen_windows: Vec<[C; 4]> = generators_projective
            .iter()
            .map(|&g| {
                let window_projective = [g, g.double(), g + g.double(), g.double().double()];
                let mut window = vec![C::identity(); window_projective.len()];
                C::Curve::batch_normalize(&window_projective, &mut window);
                window.try_into().unwrap()
            })
            .collect();

        Self {
            generators: generators.try_into().unwrap(),
            pedersen_windows: pedersen_windows.try_into().unwrap(),
        }
    }

    pub(super) fn commit(&self, bits: &[bool]) -> C {
        pedersen_hash(
            bits,
            &self
                .generators
                .iter()
                .map(|g| g.to_curve())
                .collect::<Vec<_>>(),
        )
        .to_affine()
    }
}
