use crate::arithmetic::{best_multiexp, CurveAffine};
use crate::poly::{commitment::MSM, ipa::commitment::ParamsVerifierIPA};
use ff::Field;
use group::Group;
use std::collections::BTreeMap;

/// A multiscalar multiplication in the polynomial commitment scheme
#[derive(Debug, Clone)]
pub struct MSMIPA<'params, C: CurveAffine> {
    pub(crate) params: &'params ParamsVerifierIPA<C>,
    g_scalars: Option<Vec<C::Scalar>>,
    w_scalar: Option<C::Scalar>,
    u_scalar: Option<C::Scalar>,
    // x-coordinate -> (scalar, y-coordinate)
    other: BTreeMap<C::Base, (C::Scalar, C::Base)>,
}

impl<'a, C: CurveAffine> MSMIPA<'a, C> {
    /// Given verifier parameters Creates an empty multi scalar engine
    pub fn new(params: &'a ParamsVerifierIPA<C>) -> Self {
        let g_scalars = None;
        let w_scalar = None;
        let u_scalar = None;
        let other = BTreeMap::new();

        Self {
            g_scalars,
            w_scalar,
            u_scalar,
            other,

            params,
        }
    }

    /// Add another multiexp into this one
    pub fn add_msm(&mut self, other: &Self) {
        for (x, (scalar, y)) in other.other.iter() {
            self.other
                .entry(*x)
                .and_modify(|(our_scalar, our_y)| {
                    if our_y == y {
                        *our_scalar += *scalar;
                    } else {
                        assert!(*our_y == -*y);
                        *our_scalar -= *scalar;
                    }
                })
                .or_insert((*scalar, *y));
        }

        if let Some(g_scalars) = &other.g_scalars {
            self.add_to_g_scalars(g_scalars);
        }

        if let Some(w_scalar) = &other.w_scalar {
            self.add_to_w_scalar(*w_scalar);
        }

        if let Some(u_scalar) = &other.u_scalar {
            self.add_to_u_scalar(*u_scalar);
        }
    }
}

impl<'a, C: CurveAffine> MSM<C> for MSMIPA<'a, C> {
    fn append_term(&mut self, scalar: C::Scalar, point: C::Curve) {
        if !bool::from(point.is_identity()) {
            use group::Curve;
            let point = point.to_affine();
            let xy = point.coordinates().unwrap();
            let x = *xy.x();
            let y = *xy.y();

            self.other
                .entry(x)
                .and_modify(|(our_scalar, our_y)| {
                    if *our_y == y {
                        *our_scalar += scalar;
                    } else {
                        assert!(*our_y == -y);
                        *our_scalar -= scalar;
                    }
                })
                .or_insert((scalar, y));
        }
    }

    /// Add another multiexp into this one
    fn add_msm(&mut self, other: &Self) {
        for (x, (scalar, y)) in other.other.iter() {
            self.other
                .entry(*x)
                .and_modify(|(our_scalar, our_y)| {
                    if our_y == y {
                        *our_scalar += *scalar;
                    } else {
                        assert!(*our_y == -*y);
                        *our_scalar -= *scalar;
                    }
                })
                .or_insert((*scalar, *y));
        }

        if let Some(g_scalars) = &other.g_scalars {
            self.add_to_g_scalars(g_scalars);
        }

        if let Some(w_scalar) = &other.w_scalar {
            self.add_to_w_scalar(*w_scalar);
        }

        if let Some(u_scalar) = &other.u_scalar {
            self.add_to_u_scalar(*u_scalar);
        }
    }

    fn scale(&mut self, factor: C::Scalar) {
        if let Some(g_scalars) = &mut self.g_scalars {
            for g_scalar in g_scalars {
                *g_scalar *= &factor;
            }
        }

        for other in self.other.values_mut() {
            other.0 *= factor;
        }

        self.w_scalar = self.w_scalar.map(|a| a * &factor);
        self.u_scalar = self.u_scalar.map(|a| a * &factor);
    }

    fn check(&self) -> bool {
        bool::from(self.eval().is_identity())
    }

    fn eval(&self) -> C::Curve {
        let len = self.g_scalars.as_ref().map(|v| v.len()).unwrap_or(0)
            + self.w_scalar.map(|_| 1).unwrap_or(0)
            + self.u_scalar.map(|_| 1).unwrap_or(0)
            + self.other.len();
        let mut scalars: Vec<C::Scalar> = Vec::with_capacity(len);
        let mut bases: Vec<C> = Vec::with_capacity(len);

        scalars.extend(self.other.values().map(|(scalar, _)| scalar));
        bases.extend(
            self.other
                .iter()
                .map(|(x, (_, y))| C::from_xy(*x, *y).unwrap()),
        );

        if let Some(w_scalar) = self.w_scalar {
            scalars.push(w_scalar);
            bases.push(self.params.w);
        }

        if let Some(u_scalar) = self.u_scalar {
            scalars.push(u_scalar);
            bases.push(self.params.u);
        }

        if let Some(g_scalars) = &self.g_scalars {
            scalars.extend(g_scalars);
            bases.extend(self.params.g.iter());
        }

        assert_eq!(scalars.len(), len);

        best_multiexp(&scalars, &bases)
    }

    fn bases(&self) -> Vec<C::CurveExt> {
        self.other
            .iter()
            .map(|(x, (_, y))| C::from_xy(*x, *y).unwrap().into())
            .collect()
    }

    fn scalars(&self) -> Vec<C::Scalar> {
        self.other.values().map(|(scalar, _)| *scalar).collect()
    }
}

impl<'a, C: CurveAffine> MSMIPA<'a, C> {
    /// Add a value to the first entry of `g_scalars`.
    pub fn add_constant_term(&mut self, constant: C::Scalar) {
        if let Some(g_scalars) = self.g_scalars.as_mut() {
            g_scalars[0] += &constant;
        } else {
            let mut g_scalars = vec![C::Scalar::ZERO; self.params.n as usize];
            g_scalars[0] += &constant;
            self.g_scalars = Some(g_scalars);
        }
    }

    /// Add a vector of scalars to `g_scalars`. This function will panic if the
    /// caller provides a slice of scalars that is not of length `params.n`.
    pub fn add_to_g_scalars(&mut self, scalars: &[C::Scalar]) {
        assert_eq!(scalars.len(), self.params.n as usize);
        if let Some(g_scalars) = &mut self.g_scalars {
            for (g_scalar, scalar) in g_scalars.iter_mut().zip(scalars.iter()) {
                *g_scalar += scalar;
            }
        } else {
            self.g_scalars = Some(scalars.to_vec());
        }
    }
    /// Add to `w_scalar`
    pub fn add_to_w_scalar(&mut self, scalar: C::Scalar) {
        self.w_scalar = self.w_scalar.map_or(Some(scalar), |a| Some(a + &scalar));
    }

    /// Add to `u_scalar`
    pub fn add_to_u_scalar(&mut self, scalar: C::Scalar) {
        self.u_scalar = self.u_scalar.map_or(Some(scalar), |a| Some(a + &scalar));
    }
}

#[cfg(test)]
mod tests {
    use crate::poly::{
        commitment::{ParamsProver, MSM},
        ipa::{commitment::ParamsIPA, msm::MSMIPA},
    };
    use halo2curves::{
        pasta::{Ep, EpAffine, Fp, Fq},
        CurveAffine,
    };

    #[test]
    fn msm_arithmetic() {
        let base: Ep = EpAffine::from_xy(-Fp::one(), Fp::from(2)).unwrap().into();
        let base_viol = base + base;

        let params = ParamsIPA::new(4);
        let mut a: MSMIPA<EpAffine> = MSMIPA::new(&params);
        a.append_term(Fq::one(), base);
        // a = [1] P
        assert!(!a.clone().check());
        a.append_term(Fq::one(), base);
        // a = [1+1] P
        assert!(!a.clone().check());
        a.append_term(-Fq::one(), base_viol);
        // a = [1+1] P + [-1] 2P
        assert!(a.clone().check());
        let b = a.clone();

        // Append a point that is the negation of an existing one.
        a.append_term(Fq::from(4), -base);
        // a = [1+1-4] P + [-1] 2P
        assert!(!a.clone().check());
        a.append_term(Fq::from(2), base_viol);
        // a = [1+1-4] P + [-1+2] 2P
        assert!(a.clone().check());

        // Add two MSMs with common bases.
        a.scale(Fq::from(3));
        a.add_msm(&b);
        // a = [3*(1+1)+(1+1-4)] P + [3*(-1)+(-1+2)] 2P
        assert!(a.clone().check());

        let mut c: MSMIPA<EpAffine> = MSMIPA::new(&params);
        c.append_term(Fq::from(2), base);
        c.append_term(Fq::one(), -base_viol);
        // c = [2] P + [1] (-2P)
        assert!(c.clone().check());
        // Add two MSMs with bases that differ only in sign.
        a.add_msm(&c);
        assert!(a.check());
    }
}
