use super::Params;
use crate::arithmetic::{best_multiexp, Curve, CurveAffine};

/// A multiscalar multiplication in the polynomial commitment scheme
#[derive(Debug, Clone)]
pub struct MSM<'a, C: CurveAffine> {
    pub(crate) params: &'a Params<C>,
    g_scalars: Option<Vec<C::Scalar>>,
    h_scalar: Option<C::Scalar>,
    other_scalars: Vec<C::Scalar>,
    other_bases: Vec<C>,
}

impl<'a, C: CurveAffine> MSM<'a, C> {
    /// Create a new, empty MSM using the provided parameters.
    pub fn new(params: &'a Params<C>) -> Self {
        let g_scalars = None;
        let h_scalar = None;
        let other_scalars = vec![];
        let other_bases = vec![];

        MSM {
            params,
            g_scalars,
            h_scalar,
            other_scalars,
            other_bases,
        }
    }

    /// Add another multiexp into this one
    pub fn add_msm(&mut self, other: &Self) {
        self.other_scalars.extend(other.other_scalars.iter());
        self.other_bases.extend(other.other_bases.iter());

        if let Some(g_scalars) = &other.g_scalars {
            self.add_to_g(&g_scalars);
        }

        if let Some(h_scalar) = &other.h_scalar {
            self.add_to_h(*h_scalar);
        }
    }

    /// Add arbitrary term (the scalar and the point)
    pub fn add_term(&mut self, scalar: C::Scalar, point: C) {
        self.other_scalars.push(scalar);
        self.other_bases.push(point);
    }

    /// Add a vector of scalars to `g_scalars`. This function will panic if the
    /// caller provides a slice of scalars that is not of length `params.n`.
    // TODO: parallelize
    pub fn add_to_g(&mut self, scalars: &[C::Scalar]) {
        assert_eq!(scalars.len(), self.params.n as usize);
        if let Some(g_scalars) = &mut self.g_scalars {
            for (g_scalar, scalar) in g_scalars.iter_mut().zip(scalars.iter()) {
                *g_scalar += &scalar;
            }
        } else {
            self.g_scalars = Some(scalars.to_vec());
        }
    }

    /// Add term to h
    pub fn add_to_h(&mut self, scalar: C::Scalar) {
        self.h_scalar = self.h_scalar.map_or(Some(scalar), |a| Some(a + &scalar));
    }

    /// Scale all scalars in the MSM by some scaling factor
    // TODO: parallelize
    pub fn scale(&mut self, factor: C::Scalar) {
        if let Some(g_scalars) = &mut self.g_scalars {
            for g_scalar in g_scalars.iter_mut() {
                *g_scalar *= &factor;
            }
        }

        // TODO: parallelize
        for other_scalar in self.other_scalars.iter_mut() {
            *other_scalar *= &factor;
        }
        self.h_scalar = self.h_scalar.map(|a| a * &factor);
    }

    /// Perform multiexp and check that it results in zero
    pub fn eval(self) -> bool {
        let len = self.g_scalars.as_ref().map(|v| v.len()).unwrap_or(0)
            + self.h_scalar.map(|_| 1).unwrap_or(0)
            + self.other_scalars.len();
        let mut scalars: Vec<C::Scalar> = Vec::with_capacity(len);
        let mut bases: Vec<C> = Vec::with_capacity(len);

        scalars.extend(&self.other_scalars);
        bases.extend(&self.other_bases);

        if let Some(h_scalar) = self.h_scalar {
            scalars.push(h_scalar);
            bases.push(self.params.h);
        }

        if let Some(g_scalars) = &self.g_scalars {
            scalars.extend(g_scalars);
            bases.extend(self.params.g.iter());
        }

        assert_eq!(scalars.len(), len);

        bool::from(best_multiexp(&scalars, &bases).is_zero())
    }
}
