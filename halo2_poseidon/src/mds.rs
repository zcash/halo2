use alloc::vec::Vec;

use ff::{Field, FromUniformBytes};

use super::{grain::Grain, Mds, State};

/// Parameters for a single width-3 Poseidon partial round in equivalent sparse form.
///
/// Following Appendix B of the Poseidon paper (<https://eprint.iacr.org/2019/458.pdf>),
/// non-zero round constants on words `1..3` are propagated forward into subsequent
/// rounds so only `state[0]` receives a round constant `rc`, and the dense `3x3` MDS
/// matrix is factored across partial rounds into a sparse step
///
/// ```text
/// [ m00   w_hat[0]   w_hat[1] ]
/// [ v[0]     1          0     ]
/// [ v[1]     0          1     ]
/// ```
///
/// requiring 5 field multiplications instead of 9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SparsePartialRound<F> {
    pub(crate) rc: F,
    pub(crate) m00: F,
    pub(crate) w_hat: [F; 2],
    pub(crate) v: [F; 2],
}

/// Precomputed constants for a width-3 Poseidon permutation with equivalent sparse
/// partial rounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrecomputedT3<F, const HALF_FULL_ROUNDS: usize, const PARTIAL_ROUNDS: usize> {
    pub(crate) mds: Mds<F, 3>,
    pub(crate) pre_sparse_mds: Mds<F, 3>,
    pub(crate) initial_round_constants: [[F; 3]; HALF_FULL_ROUNDS],
    pub(crate) partial_rounds: [SparsePartialRound<F>; PARTIAL_ROUNDS],
    pub(crate) terminal_round_constants: [[F; 3]; HALF_FULL_ROUNDS],
}

impl<F: Field, const HALF_FULL_ROUNDS: usize, const PARTIAL_ROUNDS: usize>
    PrecomputedT3<F, HALF_FULL_ROUNDS, PARTIAL_ROUNDS>
{
    #[inline]
    pub(crate) fn permute<Sbox: Fn(F) -> F>(
        &self,
        mut state: State<F, 3>,
        sbox: Sbox,
    ) -> State<F, 3> {
        assert!(HALF_FULL_ROUNDS > 0);

        let apply_mds = |m: &Mds<F, 3>, s: State<F, 3>| -> State<F, 3> {
            [
                m[0][0] * s[0] + m[0][1] * s[1] + m[0][2] * s[2],
                m[1][0] * s[0] + m[1][1] * s[1] + m[1][2] * s[2],
                m[2][0] * s[0] + m[2][1] * s[1] + m[2][2] * s[2],
            ]
        };

        for rcs in &self.initial_round_constants[..HALF_FULL_ROUNDS - 1] {
            state = [
                sbox(state[0] + rcs[0]),
                sbox(state[1] + rcs[1]),
                sbox(state[2] + rcs[2]),
            ];
            state = apply_mds(&self.mds, state);
        }

        let last_initial = &self.initial_round_constants[HALF_FULL_ROUNDS - 1];
        state = [
            sbox(state[0] + last_initial[0]),
            sbox(state[1] + last_initial[1]),
            sbox(state[2] + last_initial[2]),
        ];
        state = apply_mds(&self.pre_sparse_mds, state);

        for round in &self.partial_rounds {
            let s0 = sbox(state[0] + round.rc);
            let s1 = state[1];
            let s2 = state[2];
            state = [
                round.m00 * s0 + round.w_hat[0] * s1 + round.w_hat[1] * s2,
                round.v[0] * s0 + s1,
                round.v[1] * s0 + s2,
            ];
        }

        for rcs in &self.terminal_round_constants {
            state = [
                sbox(state[0] + rcs[0]),
                sbox(state[1] + rcs[1]),
                sbox(state[2] + rcs[2]),
            ];
            state = apply_mds(&self.mds, state);
        }

        state
    }
}

#[cfg(test)]
pub(crate) fn derive_precomputed_t3<
    F: Field,
    const HALF_FULL_ROUNDS: usize,
    const PARTIAL_ROUNDS: usize,
>(
    mds: &Mds<F, 3>,
    round_constants: &[[F; 3]],
) -> PrecomputedT3<F, HALF_FULL_ROUNDS, PARTIAL_ROUNDS> {
    assert_eq!(round_constants.len(), 2 * HALF_FULL_ROUNDS + PARTIAL_ROUNDS);

    let mat_vec = |m: &Mds<F, 3>, x: [F; 3]| -> [F; 3] {
        [
            m[0][0] * x[0] + m[0][1] * x[1] + m[0][2] * x[2],
            m[1][0] * x[0] + m[1][1] * x[1] + m[1][2] * x[2],
            m[2][0] * x[0] + m[2][1] * x[1] + m[2][2] * x[2],
        ]
    };
    let mat_mul = |a: &Mds<F, 3>, b: &Mds<F, 3>| -> Mds<F, 3> {
        core::array::from_fn(|i| {
            core::array::from_fn(|j| a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j])
        })
    };

    // Propagate words 1..3 of each partial-round constant vector forward through `mds`.
    let mut rc_work = round_constants.to_vec();
    let mut sparse_rc = [F::ZERO; PARTIAL_ROUNDS];
    for (idx, r) in (HALF_FULL_ROUNDS..HALF_FULL_ROUNDS + PARTIAL_ROUNDS).enumerate() {
        sparse_rc[idx] = rc_work[r][0];
        let pushed = mat_vec(mds, [F::ZERO, rc_work[r][1], rc_work[r][2]]);
        for (word, p) in rc_work[r + 1].iter_mut().zip(pushed) {
            *word += p;
        }
    }

    // Factor `acc = sparse_r * p_r` from the last partial round down to the first,
    // absorbing `p_r` onto the left of the preceding round's linear layer.
    let mut partial_rounds = [SparsePartialRound {
        rc: F::ZERO,
        m00: F::ZERO,
        w_hat: [F::ZERO; 2],
        v: [F::ZERO; 2],
    }; PARTIAL_ROUNDS];
    let mut acc = *mds;
    for idx in (0..PARTIAL_ROUNDS).rev() {
        let m00 = acc[0][0];
        let w = [acc[0][1], acc[0][2]];
        let v = [acc[1][0], acc[2][0]];
        let h00 = acc[1][1];
        let h01 = acc[1][2];
        let h10 = acc[2][1];
        let h11 = acc[2][2];

        let inv_det = (h00 * h11 - h01 * h10)
            .invert()
            .expect("2x2 MDS submatrix power is invertible");
        let w_hat = [
            (w[0] * h11 - w[1] * h10) * inv_det,
            (w[1] * h00 - w[0] * h01) * inv_det,
        ];

        partial_rounds[idx] = SparsePartialRound {
            rc: sparse_rc[idx],
            m00,
            w_hat,
            v,
        };

        let p_mat = [
            [F::ONE, F::ZERO, F::ZERO],
            [F::ZERO, h00, h01],
            [F::ZERO, h10, h11],
        ];
        acc = mat_mul(&p_mat, mds);
    }

    let mut initial_round_constants = [[F::ZERO; 3]; HALF_FULL_ROUNDS];
    initial_round_constants.copy_from_slice(&rc_work[..HALF_FULL_ROUNDS]);

    let mut terminal_round_constants = [[F::ZERO; 3]; HALF_FULL_ROUNDS];
    terminal_round_constants.copy_from_slice(
        &rc_work[HALF_FULL_ROUNDS + PARTIAL_ROUNDS..2 * HALF_FULL_ROUNDS + PARTIAL_ROUNDS],
    );

    PrecomputedT3 {
        mds: *mds,
        pre_sparse_mds: acc,
        initial_round_constants,
        partial_rounds,
        terminal_round_constants,
    }
}

pub(super) fn generate_mds<F: FromUniformBytes<64> + Ord, const T: usize>(
    grain: &mut Grain<F>,
    mut select: usize,
) -> (Mds<F, T>, Mds<F, T>) {
    let (xs, ys, mds) = loop {
        // Generate two [F; T] arrays of unique field elements.
        let (xs, ys) = loop {
            let mut vals: Vec<_> = (0..2 * T)
                .map(|_| grain.next_field_element_without_rejection())
                .collect();

            // Check that we have unique field elements.
            let mut unique = vals.clone();
            unique.sort_unstable();
            unique.dedup();
            if vals.len() == unique.len() {
                let rhs = vals.split_off(T);
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
        //     a_ij = 1/(x_i + y_j); x_i + y_j != 0
        //
        // It would be much easier to use the alternate definition:
        //     a_ij = 1/(x_i - y_j); x_i - y_j != 0
        //
        // These are clearly equivalent on `y <- -y`, but it is easier to work with the
        // negative formulation, because ensuring that xs ∪ ys is unique implies that
        // x_i - y_j != 0 by construction (whereas the positive case does not hold). It
        // also makes computation of the matrix inverse simpler below (the theorem used
        // was formulated for the negative definition).
        //
        // However, the Poseidon paper and reference impl use the positive formulation,
        // and we want to rely on the reference impl for MDS security, so we use the same
        // formulation.
        let mut mds = [[F::ZERO; T]; T];
        #[allow(clippy::needless_range_loop)]
        for i in 0..T {
            for j in 0..T {
                let sum = xs[i] + ys[j];
                // We leverage the secure MDS selection counter to also check this.
                assert!(!sum.is_zero_vartime());
                mds[i][j] = sum.invert().unwrap();
            }
        }

        break (xs, ys, mds);
    };

    // Compute the inverse. All square Cauchy matrices have a non-zero determinant and
    // thus are invertible. The inverse for a Cauchy matrix of the form:
    //
    //     a_ij = 1/(x_i - y_j); x_i - y_j != 0
    //
    // has elements b_ij given by:
    //
    //     b_ij = (x_j - y_i) A_j(y_i) B_i(x_j)    (Schechter 1959, Theorem 1)
    //
    // where A_i(x) and B_i(x) are the Lagrange polynomials for xs and ys respectively.
    //
    // We adapt this to the positive Cauchy formulation by negating ys.
    let mut mds_inv = [[F::ZERO; T]; T];
    let l = |xs: &[F], j, x: F| {
        let x_j = xs[j];
        xs.iter().enumerate().fold(F::ONE, |acc, (m, x_m)| {
            if m == j {
                acc
            } else {
                // We hard-code the type, to avoid spurious "cannot infer type" rustc errors.
                let denominator: F = x_j - x_m;

                // We can invert freely; by construction, the elements of xs are distinct.
                let denominator_inverted: F = denominator.invert().unwrap();

                acc * (x - x_m) * denominator_inverted
            }
        })
    };
    let neg_ys: Vec<_> = ys.iter().map(|y| -*y).collect();
    for i in 0..T {
        for j in 0..T {
            mds_inv[i][j] = (xs[j] - neg_ys[i]) * l(&xs, j, neg_ys[i]) * l(&neg_ys, i, xs[j]);
        }
    }

    (mds, mds_inv)
}

#[cfg(test)]
mod tests {
    use group::ff::Field;
    use pasta_curves::Fp;

    use super::{generate_mds, Grain};

    #[test]
    fn poseidon_mds() {
        const T: usize = 3;
        let mut grain = Grain::new(super::super::grain::SboxType::Pow, T as u16, 8, 56);
        let (mds, mds_inv) = generate_mds::<Fp, T>(&mut grain, 0);

        // Verify that MDS * MDS^-1 = I.
        #[allow(clippy::needless_range_loop)]
        for i in 0..T {
            for j in 0..T {
                let expected = if i == j { Fp::ONE } else { Fp::ZERO };
                assert_eq!(
                    (0..T).fold(Fp::ZERO, |acc, k| acc + (mds[i][k] * mds_inv[k][j])),
                    expected
                );
            }
        }
    }
}
