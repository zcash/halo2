use crate::arithmetic::{best_fft, parallelize, Field, Group};

/// This structure contains precomputed constants and other details needed for
/// performing operations on an evaluation domain of size $2^k$ in the context
/// of PLONK.
#[derive(Debug)]
pub struct EvaluationDomain<G: Group> {
    n: u64,
    k: u32,
    extended_k: u32,
    omega_inv: G::Scalar,
    extended_omega: G::Scalar,
    extended_omega_inv: G::Scalar,
    g_coset: G::Scalar,
    g_coset_inv: G::Scalar,
    quotient_poly_degree: u64,
    ifft_divisor: G::Scalar,
    extended_ifft_divisor: G::Scalar,
    t_evaluations: Vec<G::Scalar>,
}

impl<G: Group> EvaluationDomain<G> {
    /// This constructs a new evaluation domain object (containing precomputed
    /// constants) for operating on an evaluation domain of size $2^k$ and for
    /// some operations over an extended domain of size $2^{k + j}$ where $j$ is
    /// sufficiently large to describe the quotient polynomial depending on the
    /// maximum degree of all PLONK gates.
    pub fn new(gate_degree: u32, k: u32) -> Self {
        // quotient_poly_degree * params.n - 1 is the degree of the quotient polynomial
        let quotient_poly_degree = (gate_degree - 1) as u64;

        let n = 1u64 << k;

        // We need to work within an extended domain, not params.k but params.k + j
        // such that 2^(params.k + j) is sufficiently large to describe the quotient
        // polynomial.
        let mut extended_k = k;
        while (1 << extended_k) < (n * quotient_poly_degree) {
            extended_k += 1;
        }

        let mut extended_omega = G::Scalar::ROOT_OF_UNITY;
        for _ in extended_k..G::Scalar::S {
            extended_omega = extended_omega.square();
        }
        let extended_omega = extended_omega; // 2^{j+k}'th root of unity
        let extended_omega_inv = extended_omega.invert().unwrap();

        let mut omega = extended_omega;
        for _ in k..extended_k {
            omega = omega.square();
        }
        let omega = omega; // 2^{k}'th root of unity
        let omega_inv = omega.invert().unwrap();

        // We use zeta here because we know it generates a coset, and it's available
        // already.
        let g_coset = G::Scalar::ZETA;
        let g_coset_inv = g_coset.square();

        // TODO: merge these inversions together with t_evaluations batch inversion?
        let ifft_divisor = G::Scalar::from_u64(1 << k).invert().unwrap();
        let extended_ifft_divisor = G::Scalar::from_u64(1 << extended_k).invert().unwrap();

        let mut t_evaluations = Vec::with_capacity(1 << (extended_k - k));
        {
            // Compute the evaluations of t(X) in the coset evaluation domain.
            // We don't have to compute all of them, because it will repeat.
            let orig = G::Scalar::ZETA.pow_vartime(&[n as u64, 0, 0, 0]);
            let step = extended_omega.pow_vartime(&[n as u64, 0, 0, 0]);
            let mut cur = orig;
            loop {
                t_evaluations.push(cur);
                cur *= &step;
                if cur == orig {
                    break;
                }
            }
            assert_eq!(t_evaluations.len(), 1 << (extended_k - k));

            // Subtract 1 from each to give us t_evaluations[i] = t(zeta * extended_omega^i)
            for coeff in &mut t_evaluations {
                *coeff -= &G::Scalar::one();
            }

            // Invert, because we're dividing by this polynomial.
            G::Scalar::batch_invert(&mut t_evaluations);
        }

        EvaluationDomain {
            n,
            k,
            extended_k,
            omega_inv,
            extended_omega,
            extended_omega_inv,
            g_coset,
            g_coset_inv,
            quotient_poly_degree,
            ifft_divisor,
            extended_ifft_divisor,
            t_evaluations,
        }
    }

    /// This takes us from an n-length vector into the coset evaluation domain.
    /// Also returns the polynomial.
    ///
    /// This function will panic if the provided vector is not the correct
    /// length.
    pub fn obtain_coset(&self, mut a: Vec<G>) -> (Vec<G>, Vec<G>) {
        assert_eq!(a.len(), 1 << self.k);

        // Perform inverse FFT to obtain the polynomial in coefficient form
        Self::ifft(&mut a, self.omega_inv, self.k, self.ifft_divisor);

        // Keep this polynomial around; we'll need to evaluate it at arbitrary
        // points later.
        let old = a.clone();

        // Distributes powers so that an FFT will move us into the coset
        // evaluation domain.
        Self::distribute_powers(&mut a, self.g_coset);

        // Resize to account for the quotient polynomial's size
        a.resize(1 << self.extended_k, G::group_zero());

        // Move into coset evaluation domain
        best_fft(&mut a, self.extended_omega, self.extended_k);

        (a, old)
    }

    /// This takes us from the coset evaluation domain and gets us the quotient
    /// polynomial coefficients.
    ///
    /// This function will panic if the provided vector is not the correct
    /// length.
    pub fn from_coset(&self, mut a: Vec<G>) -> Vec<G> {
        assert_eq!(a.len(), 1 << self.extended_k);

        // Inverse FFT
        Self::ifft(
            &mut a,
            self.extended_omega_inv,
            self.extended_k,
            self.extended_ifft_divisor,
        );

        // Distribute powers to move from coset; opposite from the
        // transformation we performed earlier.
        Self::distribute_powers(&mut a, self.g_coset_inv);

        // Truncate it to match the size of the quotient polynomial; the
        // evaluation domain might be slightly larger than necessary because
        // it always lies on a power-of-two boundary.
        a.truncate((&self.n * self.quotient_poly_degree) as usize);

        a
    }

    /// This divides the polynomial (in the coset domain) by the vanishing
    /// polynomial.
    pub fn divide_by_vanishing_poly(&self, mut h_poly: Vec<G>) -> Vec<G> {
        assert_eq!(h_poly.len(), 1 << self.extended_k);

        // Divide to obtain the quotient polynomial in the coset evaluation
        // domain.
        parallelize(&mut h_poly, |h, mut index| {
            for h in h {
                h.group_scale(&self.t_evaluations[index % self.t_evaluations.len()]);
                index += 1;
            }
        });

        h_poly
    }

    fn distribute_powers(mut a: &mut [G], g: G::Scalar) {
        let coset_powers = [g, g.square()];
        parallelize(&mut a, |a, mut index| {
            for a in a {
                // Distribute powers to move into coset
                let i = index % (coset_powers.len() + 1);
                if i != 0 {
                    a.group_scale(&coset_powers[i - 1]);
                }
                index += 1;
            }
        });
    }

    fn ifft(a: &mut [G], omega_inv: G::Scalar, log_n: u32, divisor: G::Scalar) {
        best_fft(a, omega_inv, log_n);
        parallelize(a, |a, _| {
            for a in a {
                // Finish iFFT
                a.group_scale(&divisor);
            }
        });
    }
}
