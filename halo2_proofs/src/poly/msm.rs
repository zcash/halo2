use crate::arithmetic::{best_multiexp, parallelize, CurveAffine, Engine};
use group::Curve;

/// A multiscalar multiplication in the polynomial commitment scheme
#[derive(Debug, Clone)]
pub struct MSM<C: CurveAffine> {
    scalars: Vec<C::Scalar>,
    bases: Vec<C>,
}

impl<C: CurveAffine> Default for MSM<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, C: CurveAffine> MSM<C> {
    /// Create a new, empty MSM using the provided parameters.
    pub fn new() -> Self {
        MSM {
            scalars: vec![],
            bases: vec![],
        }
    }

    /// Add another multiexp into this one
    pub fn add_msm(&mut self, other: &Self) {
        self.scalars.extend(other.scalars.iter());
        self.bases.extend(other.bases.iter());
    }

    /// Add arbitrary term (the scalar and the point)
    pub fn append_term(&mut self, scalar: C::Scalar, point: C) {
        self.scalars.push(scalar);
        self.bases.push(point);
    }

    /// Scale all scalars in the MSM by some scaling factor
    pub fn scale(&mut self, factor: C::Scalar) {
        if !self.scalars.is_empty() {
            parallelize(&mut self.scalars, |scalars, _| {
                for other_scalar in scalars {
                    *other_scalar *= &factor;
                }
            })
        }
    }

    /// Prepares all scalars in the MSM to linear combination
    pub fn combine_with_base(&mut self, base: C::Scalar) {
        use ff::Field;
        let mut acc = C::Scalar::one();
        if !self.scalars.is_empty() {
            for scalar in self.scalars.iter_mut().rev() {
                *scalar *= &acc;
                acc *= base;
            }
        }
    }

    /// Perform multiexp and check that it results in zero
    pub fn eval(&self) -> C {
        best_multiexp(&self.scalars, &self.bases).into()
    }

    /// Check if eval is equal to identity
    pub fn check(self) -> bool {
        bool::from(self.eval().is_identity())
    }
}

/// A guard returned by the verifier
#[derive(Debug, Default)]
pub struct PairMSM<C: CurveAffine> {
    left: MSM<C>,
    right: MSM<C>,
}

impl<C: CurveAffine> PairMSM<C> {
    /// Create a new, with prepared two channel MSM
    pub fn with(left: MSM<C>, right: MSM<C>) -> Self {
        Self { left, right }
    }

    /// Perform multiexp on both channels
    pub fn eval(&self) -> (C, C) {
        (self.left.eval(), self.right.eval())
    }

    /// Scale all scalars in the MSM by some scaling factor
    pub fn scale(&mut self, e: C::Scalar) {
        self.left.scale(e);
        self.right.scale(e);
    }

    /// Add another multiexp into this one
    pub fn add_msm(&mut self, other: Self) {
        self.left.add_msm(&other.left);
        self.right.add_msm(&other.right);
    }
}

#[cfg(feature = "shplonk")]
#[derive(Debug, Clone)]
pub struct ProjectiveMSM<E: Engine> {
    scalars: Vec<E::Scalar>,
    bases: Vec<E::G1>,
}

#[cfg(feature = "shplonk")]
impl<'a, E: Engine> ProjectiveMSM<E> {
    /// Create a new, empty MSM using the provided parameters.
    pub fn new() -> Self {
        ProjectiveMSM {
            scalars: vec![],
            bases: vec![],
        }
    }

    /// Add arbitrary term (the scalar and the point)
    pub fn append_term(&mut self, scalar: E::Scalar, point: E::G1) {
        self.scalars.push(scalar);
        self.bases.push(point);
    }

    /// Scale all scalars in the MSM by some scaling factor
    pub fn scale(&mut self, factor: E::Scalar) {
        if !self.scalars.is_empty() {
            parallelize(&mut self.scalars, |scalars, _| {
                for other_scalar in scalars {
                    *other_scalar *= &factor;
                }
            })
        }
    }

    /// Prepares all scalars in the MSM to linear combination
    pub fn combine_with_base(&mut self, base: E::Scalar) {
        use ff::Field;
        let mut acc = E::Scalar::one();
        if !self.scalars.is_empty() {
            for scalar in self.scalars.iter_mut().rev() {
                *scalar *= &acc;
                acc *= base;
            }
        }
    }
}

/// A projective point collector
#[cfg(feature = "shplonk")]
#[derive(Debug, Clone)]
pub struct PreMSM<E: Engine> {
    projectives_msms: Vec<ProjectiveMSM<E>>,
}

#[cfg(feature = "shplonk")]
impl<'a, E: Engine> PreMSM<E> {
    pub fn new() -> Self {
        PreMSM {
            projectives_msms: vec![],
        }
    }

    pub fn normalize(self) -> MSM<E::G1Affine> {
        use group::prime::PrimeCurveAffine;

        let bases: Vec<E::G1> = self
            .projectives_msms
            .iter()
            .map(|msm| msm.bases.clone())
            .collect::<Vec<Vec<E::G1>>>()
            .into_iter()
            .flatten()
            .collect();

        let scalars: Vec<E::Scalar> = self
            .projectives_msms
            .iter()
            .map(|msm| msm.scalars.clone())
            .collect::<Vec<Vec<E::Scalar>>>()
            .into_iter()
            .flatten()
            .collect();

        let mut affine_bases = vec![E::G1Affine::identity(); bases.len()];
        E::G1::batch_normalize(&bases[..], &mut affine_bases);
        MSM {
            scalars,
            bases: affine_bases,
        }
    }

    pub fn add_msm(&mut self, other: ProjectiveMSM<E>) {
        self.projectives_msms.push(other);
    }

    /// Prepares all scalars in the MSM to linear combination
    pub fn combine_with_base(&mut self, base: E::Scalar) {
        use ff::Field;
        let mut acc = E::Scalar::one();
        if !self.projectives_msms.is_empty() {
            for msm in self.projectives_msms.iter_mut().rev() {
                msm.scale(acc);
                acc *= base;
            }
        }
    }
}
