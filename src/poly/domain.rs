//! Contains utilities for performing polynomial arithmetic over an evaluation
//! domain that is of a suitable size for the application.

use crate::arithmetic::{best_fft, parallelize, BatchInvert, Field, Group};

use super::{Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial};
use std::marker::PhantomData;

/// Describes a relative location in the evaluation domain; applying a rotation
/// by i will rotate the vector in the evaluation domain by i.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Rotation(pub i32);

impl Default for Rotation {
    fn default() -> Rotation {
        Rotation(0)
    }
}

/// This structure contains precomputed constants and other details needed for
/// performing operations on an evaluation domain of size $2^k$ and an extended
/// domain of size $2^{k} * j$ with $j \neq 0$.
#[derive(Debug)]
pub struct EvaluationDomain<G: Group> {
    n: u64,
    k: u32,
    extended_k: u32,
    omega: G::Scalar,
    omega_inv: G::Scalar,
    extended_omega: G::Scalar,
    extended_omega_inv: G::Scalar,
    g_coset: G::Scalar,
    g_coset_inv: G::Scalar,
    quotient_poly_degree: u64,
    ifft_divisor: G::Scalar,
    extended_ifft_divisor: G::Scalar,
    t_evaluations: Vec<G::Scalar>,
    barycentric_weight: G::Scalar,
}

impl<G: Group> EvaluationDomain<G> {
    /// This constructs a new evaluation domain object based on the provided
    /// values $j, k$.
    pub fn new(j: u32, k: u32) -> Self {
        // quotient_poly_degree * params.n - 1 is the degree of the quotient polynomial
        let quotient_poly_degree = (j - 1) as u64;

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
        let mut extended_omega_inv = extended_omega; // Inversion computed later

        let mut omega = extended_omega;
        for _ in k..extended_k {
            omega = omega.square();
        }
        let omega = omega; // 2^{k}'th root of unity
        let mut omega_inv = omega; // Inversion computed later

        // We use zeta here because we know it generates a coset, and it's available
        // already.
        let g_coset = G::Scalar::ZETA;
        let g_coset_inv = g_coset.square();

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
            // We invert in a batch, below.
        }

        let mut ifft_divisor = G::Scalar::from_u64(1 << k); // Inversion computed later
        let mut extended_ifft_divisor = G::Scalar::from_u64(1 << extended_k); // Inversion computed later

        // The barycentric weight of 1 over the evaluation domain
        // 1 / \prod_{i != 0} (1 - omega^i)
        let mut barycentric_weight = G::Scalar::from(n); // Inversion computed later

        // Compute batch inversion
        t_evaluations
            .iter_mut()
            .chain(Some(&mut ifft_divisor))
            .chain(Some(&mut extended_ifft_divisor))
            .chain(Some(&mut barycentric_weight))
            .chain(Some(&mut extended_omega_inv))
            .chain(Some(&mut omega_inv))
            .batch_invert();

        EvaluationDomain {
            n,
            k,
            extended_k,
            omega,
            omega_inv,
            extended_omega,
            extended_omega_inv,
            g_coset,
            g_coset_inv,
            quotient_poly_degree,
            ifft_divisor,
            extended_ifft_divisor,
            t_evaluations,
            barycentric_weight,
        }
    }

    /// Obtains a polynomial in Lagrange form when given a vector of Lagrange
    /// coefficients of size `n`; panics if the provided vector is the wrong
    /// length.
    pub fn lagrange_from_vec(&self, values: Vec<G>) -> Polynomial<G, LagrangeCoeff> {
        assert_eq!(values.len(), self.n as usize);

        Polynomial {
            values,
            _marker: PhantomData,
        }
    }

    /// Obtains a polynomial in coefficient form when given a vector of
    /// coefficients of size `n`; panics if the provided vector is the wrong
    /// length.
    pub fn coeff_from_vec(&self, values: Vec<G>) -> Polynomial<G, Coeff> {
        assert_eq!(values.len(), self.n as usize);

        Polynomial {
            values,
            _marker: PhantomData,
        }
    }

    /// Returns an empty (zero) polynomial in the coefficient basis
    pub fn empty_coeff(&self) -> Polynomial<G, Coeff> {
        Polynomial {
            values: vec![G::group_zero(); self.n as usize],
            _marker: PhantomData,
        }
    }

    /// Returns an empty (zero) polynomial in the Lagrange coefficient basis
    pub fn empty_lagrange(&self) -> Polynomial<G, LagrangeCoeff> {
        Polynomial {
            values: vec![G::group_zero(); self.n as usize],
            _marker: PhantomData,
        }
    }

    /// Returns an empty (zero) polynomial in the extended Lagrange coefficient
    /// basis
    pub fn empty_extended(&self) -> Polynomial<G, ExtendedLagrangeCoeff> {
        Polynomial {
            values: vec![G::group_zero(); self.extended_len()],
            _marker: PhantomData,
        }
    }

    /// This takes us from an n-length vector into the coefficient form.
    ///
    /// This function will panic if the provided vector is not the correct
    /// length.
    pub fn lagrange_to_coeff(&self, mut a: Polynomial<G, LagrangeCoeff>) -> Polynomial<G, Coeff> {
        assert_eq!(a.values.len(), 1 << self.k);

        // Perform inverse FFT to obtain the polynomial in coefficient form
        Self::ifft(&mut a.values, self.omega_inv, self.k, self.ifft_divisor);

        Polynomial {
            values: a.values,
            _marker: PhantomData,
        }
    }

    /// This takes us from an n-length coefficient vector into the extended
    /// evaluation domain, rotating by `rotation` if desired.
    pub fn coeff_to_extended(
        &self,
        mut a: Polynomial<G, Coeff>,
        rotation: Rotation,
    ) -> Polynomial<G, ExtendedLagrangeCoeff> {
        assert_eq!(a.values.len(), 1 << self.k);

        assert!(rotation.0 != i32::MIN);
        if rotation.0 == 0 {
            // In this special case, the powers of zeta repeat so we do not need
            // to compute them.
            Self::distribute_powers_zeta(&mut a.values, self.g_coset);
        } else {
            let mut g = G::Scalar::ZETA;
            if rotation.0 > 0 {
                g *= &self.omega.pow_vartime(&[rotation.0 as u64, 0, 0, 0]);
            } else {
                g *= &self
                    .omega_inv
                    .pow_vartime(&[rotation.0.abs() as u64, 0, 0, 0]);
            }
            Self::distribute_powers(&mut a.values, g);
        }
        a.values.resize(self.extended_len(), G::group_zero());
        best_fft(&mut a.values, self.extended_omega, self.extended_k);

        Polynomial {
            values: a.values,
            _marker: PhantomData,
        }
    }

    /// This takes us from the extended evaluation domain and gets us the
    /// quotient polynomial coefficients.
    ///
    /// This function will panic if the provided vector is not the correct
    /// length.
    // TODO/FIXME: caller should be responsible for truncating
    pub fn extended_to_coeff(&self, mut a: Polynomial<G, ExtendedLagrangeCoeff>) -> Vec<G> {
        assert_eq!(a.values.len(), self.extended_len());

        // Inverse FFT
        Self::ifft(
            &mut a.values,
            self.extended_omega_inv,
            self.extended_k,
            self.extended_ifft_divisor,
        );

        // Distribute powers to move from coset; opposite from the
        // transformation we performed earlier.
        Self::distribute_powers(&mut a.values, self.g_coset_inv);

        // Truncate it to match the size of the quotient polynomial; the
        // evaluation domain might be slightly larger than necessary because
        // it always lies on a power-of-two boundary.
        a.values
            .truncate((&self.n * self.quotient_poly_degree) as usize);

        a.values
    }

    /// This divides the polynomial (in the extended domain) by the vanishing
    /// polynomial of the $2^k$ size domain.
    pub fn divide_by_vanishing_poly(
        &self,
        mut a: Polynomial<G, ExtendedLagrangeCoeff>,
    ) -> Polynomial<G, ExtendedLagrangeCoeff> {
        assert_eq!(a.values.len(), self.extended_len());

        // Divide to obtain the quotient polynomial in the coset evaluation
        // domain.
        parallelize(&mut a.values, |h, mut index| {
            for h in h {
                h.group_scale(&self.t_evaluations[index % self.t_evaluations.len()]);
                index += 1;
            }
        });

        Polynomial {
            values: a.values,
            _marker: PhantomData,
        }
    }

    fn distribute_powers_zeta(mut a: &mut [G], g: G::Scalar) {
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

    fn distribute_powers(mut a: &mut [G], g: G::Scalar) {
        parallelize(&mut a, |a, index| {
            let mut cur = g.pow_vartime(&[index as u64, 0, 0, 0]);
            for a in a {
                a.group_scale(&cur);
                cur *= &g;
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

    /// Get the size of the extended domain
    pub fn extended_len(&self) -> usize {
        1 << self.extended_k
    }

    /// Get $\omega$, the generator of the $2^k$ order multiplicative subgroup.
    pub fn get_omega(&self) -> G::Scalar {
        self.omega
    }

    /// Get $\omega^{-1}$, the inverse of the generator of the $2^k$ order
    /// multiplicative subgroup.
    pub fn get_omega_inv(&self) -> G::Scalar {
        self.omega_inv
    }

    /// Get the generator of the extended domain's multiplicative subgroup.
    pub fn get_extended_omega(&self) -> G::Scalar {
        self.extended_omega
    }

    /// Multiplies a value by some power of $\omega$, essentially rotating over
    /// the domain.
    pub fn rotate_omega(&self, value: G::Scalar, rotation: Rotation) -> G::Scalar {
        let mut point = value;
        if rotation.0 >= 0 {
            point *= &self.get_omega().pow(&[rotation.0 as u64, 0, 0, 0]);
        } else {
            point *= &self
                .get_omega_inv()
                .pow(&[rotation.0.abs() as u64, 0, 0, 0]);
        }
        point
    }

    /// Gets the barycentric weight of $1$ over the $2^k$ size domain.
    pub fn get_barycentric_weight(&self) -> G::Scalar {
        self.barycentric_weight
    }
}
