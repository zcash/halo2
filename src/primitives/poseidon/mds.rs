use halo2::arithmetic::FieldExt;

use super::grain::Grain;

pub(super) fn generate_mds<F: FieldExt>(
    grain: &mut Grain<F>,
    arity: usize,
    mut select: usize,
) -> (Vec<Vec<F>>, Vec<Vec<F>>) {
    let (xs, ys, mds) = loop {
        // Generate two [F; arity] arrays of unique field elements.
        let (xs, ys) = loop {
            let mut vals: Vec<_> = (0..2 * arity).map(|_| grain.next_field_element()).collect();

            // Check that we have unique field elements.
            let mut unique = vals.clone();
            unique.sort_unstable();
            unique.dedup();
            if vals.len() == unique.len() {
                let rhs = vals.split_off(arity);
                break (vals, rhs);
            }
        };

        // We need to ensure that the MDS is secure. Instead of checking the MDS against
        // the relevant algorithms directly, we witness a fixed number of MDS matrices
        // that we need to sample from the given Grain state before obtaining a secure
        // matrix. This can be determined out-of-band via the reference implementation in
        // Sage.
        if select != 0 {
            select -= 1;
            continue;
        }

        // Generate a Cauchy matrix, with elements a_ij in the form:
        //     a_ij = 1/(x_i - y_j); x_i - y_j != 0
        //
        // The Poseidon paper uses the alternate definition:
        //     a_ij = 1/(x_i + y_j); x_i + y_j != 0
        //
        // These are clearly equivalent on `y <= -y`, but it is easier to work with the
        // negative formulation, because ensuring that xs ∪ ys is unique implies that
        // x_i - y_j != 0 by construction (whereas the positive case does not hold). It
        // also makes computation of the matrix inverse simpler below (the theorem used
        // was formulated for the negative definition).
        let mut mds = vec![vec![F::zero(); arity]; arity];
        for i in 0..arity {
            for j in 0..arity {
                mds[i][j] = (xs[i] - ys[j]).invert().unwrap();
            }
        }

        break (xs, ys, mds);
    };

    // Compute the inverse. All square Cauchy matrices have a non-zero determinant and
    // thus are invertible. The inverse has elements b_ij given by:
    //
    //     b_ij = (x_j - y_i) A_j(y_i) B_i(x_j)    (Schechter 1959, Theorem 1)
    //
    // where A_i(x) and B_i(x) are the Lagrange polynomials for xs and ys respectively.
    let mut mds_inv = vec![vec![F::zero(); arity]; arity];
    let l = |xs: &[F], j, x: F| {
        let x_j = xs[j];
        xs.iter().enumerate().fold(F::one(), |acc, (m, x_m)| {
            if m == j {
                acc
            } else {
                // We can invert freely; by construction, the elements of xs are distinct.
                acc * (x - x_m) * (x_j - x_m).invert().unwrap()
            }
        })
    };
    for i in 0..arity {
        for j in 0..arity {
            mds_inv[i][j] = (xs[j] - ys[i]) * l(&xs, j, ys[i]) * l(&ys, i, xs[j]);
        }
    }

    (mds, mds_inv)
}

#[cfg(test)]
mod tests {
    use pasta_curves::Fp;

    use super::{generate_mds, Grain};

    #[test]
    fn poseidon_mds() {
        let arity = 3;
        let mut grain = Grain::new(super::super::grain::SboxType::Pow, arity as u16, 8, 56);
        let (mds, mds_inv) = generate_mds::<Fp>(&mut grain, arity, 0);

        // Verify that MDS * MDS^-1 = I.
        for i in 0..arity {
            for j in 0..arity {
                let expected = if i == j { Fp::one() } else { Fp::zero() };
                assert_eq!(
                    (0..arity).fold(Fp::zero(), |acc, k| acc + (mds[i][k] * mds_inv[k][j])),
                    expected
                );
            }
        }
    }
}
