use super::Params;
use crate::arithmetic::{best_multiexp_public, CurveAffine};
use ff::{Field, PrimeField};
use group::Group;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy)]
struct CanonicalFieldKey<F: PrimeField> {
    value: F,
    repr: F::Repr,
}

impl<F: PrimeField> CanonicalFieldKey<F> {
    fn new(value: F) -> Self {
        Self {
            value,
            repr: value.to_repr(),
        }
    }
}

impl<F: PrimeField> fmt::Debug for CanonicalFieldKey<F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(formatter)
    }
}

impl<F: PrimeField> PartialEq for CanonicalFieldKey<F> {
    fn eq(&self, other: &Self) -> bool {
        self.repr.as_ref() == other.repr.as_ref()
    }
}

impl<F: PrimeField> Eq for CanonicalFieldKey<F> {}

impl<F: PrimeField> PartialOrd for CanonicalFieldKey<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: PrimeField> Ord for CanonicalFieldKey<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Any canonical byte ordering groups equal coordinates. Reversing
        // bytes also preserves Pasta's existing little-endian field order.
        self.repr
            .as_ref()
            .iter()
            .rev()
            .cmp(other.repr.as_ref().iter().rev())
    }
}

type ArbitraryTerm<C> = (
    CanonicalFieldKey<<C as CurveAffine>::Base>,
    (<C as CurveAffine>::ScalarExt, <C as CurveAffine>::Base),
);
type OtherTerms<C> = BTreeMap<
    CanonicalFieldKey<<C as CurveAffine>::Base>,
    (<C as CurveAffine>::ScalarExt, <C as CurveAffine>::Base),
>;

fn canonicalize_other<C: CurveAffine>(terms: &mut Vec<ArbitraryTerm<C>>) {
    // Stable sorting preserves the first encountered orientation. Coalesce
    // in place, negating scalars when equal x coordinates have opposite y.
    terms.sort_by_key(|a| a.0);
    terms.dedup_by(|(x, (scalar, y)), (our_x, (our_scalar, our_y))| {
        if x != our_x {
            return false;
        }
        if y == our_y {
            *our_scalar += *scalar;
        } else {
            assert!(*our_y == -*y);
            *our_scalar -= *scalar;
        }
        true
    });
}

/// A multiscalar multiplication in the polynomial commitment scheme
#[derive(Debug, Clone)]
pub struct MSM<'a, C: CurveAffine> {
    pub(crate) params: &'a Params<C>,
    g_scalars: Option<Vec<C::Scalar>>,
    w_scalar: Option<C::Scalar>,
    u_scalar: Option<C::Scalar>,
    // Cached x-coordinate -> (scalar, y-coordinate).
    other: OtherTerms<C>,
    // Batch reductions buffer arbitrary terms for one final sort/coalesce.
    batched_other: Vec<ArbitraryTerm<C>>,
}

impl<'a, C: CurveAffine> MSM<'a, C> {
    /// Create a new, empty MSM using the provided parameters.
    pub fn new(params: &'a Params<C>) -> Self {
        let g_scalars = None;
        let w_scalar = None;
        let u_scalar = None;
        let other = BTreeMap::new();

        MSM {
            params,
            g_scalars,
            w_scalar,
            u_scalar,
            other,
            batched_other: Vec::new(),
        }
    }

    /// Add another multiexp into this one
    pub fn add_msm(&mut self, other: &Self) {
        for (x, (scalar, y)) in other
            .other
            .iter()
            .chain(other.batched_other.iter().map(|(x, v)| (x, v)))
        {
            self.add_other_key(*x, *scalar, *y);
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

    fn add_other_key(&mut self, x: CanonicalFieldKey<C::Base>, scalar: C::Scalar, y: C::Base) {
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

    /// Moves an MSM into the batch without repeatedly coalescing arbitrary terms.
    #[cfg(any(test, feature = "batch"))]
    pub(crate) fn add_msm_batch(&mut self, other: Self) {
        self.batched_other
            .reserve(other.other.len() + other.batched_other.len());
        self.batched_other.extend(other.other);
        self.batched_other.extend(other.batched_other);
        if let Some(scalars) = other.g_scalars {
            self.add_owned_g_scalars(scalars);
        }
        if let Some(scalar) = other.w_scalar {
            self.add_to_w_scalar(scalar);
        }
        if let Some(scalar) = other.u_scalar {
            self.add_to_u_scalar(scalar);
        }
    }

    /// Add arbitrary term (the scalar and the point)
    pub fn append_term(&mut self, scalar: C::Scalar, point: C) {
        if !bool::from(point.is_identity()) {
            let xy = point.coordinates().unwrap();
            self.add_other_key(CanonicalFieldKey::new(*xy.x()), scalar, *xy.y());
        }
    }

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

    /// Retains the owned coefficient allocation when there are no existing
    /// generator terms. The length must be exactly `params.n`.
    pub(super) fn add_owned_g_scalars(&mut self, scalars: Vec<C::Scalar>) {
        assert_eq!(scalars.len(), self.params.n as usize);
        if let Some(g_scalars) = &mut self.g_scalars {
            for (g_scalar, scalar) in g_scalars.iter_mut().zip(scalars) {
                *g_scalar += scalar;
            }
        } else {
            self.g_scalars = Some(scalars);
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

    /// Scale all scalars in the MSM by some scaling factor
    pub fn scale(&mut self, factor: C::Scalar) {
        if let Some(g_scalars) = &mut self.g_scalars {
            for g_scalar in g_scalars {
                *g_scalar *= &factor;
            }
        }

        for other in self.other.values_mut() {
            other.0 *= factor;
        }
        for (_, (scalar, _)) in &mut self.batched_other {
            *scalar *= factor;
        }

        self.w_scalar = self.w_scalar.map(|a| a * &factor);
        self.u_scalar = self.u_scalar.map(|a| a * &factor);
    }

    /// Perform multiexp and check that it results in zero
    pub fn eval(mut self) -> bool {
        if !self.batched_other.is_empty() {
            self.batched_other
                .extend(self.other.iter().map(|(x, v)| (*x, *v)));
            self.other.clear();
            canonicalize_other::<C>(&mut self.batched_other);
        }
        let len = self.g_scalars.as_deref().map(<[_]>::len).unwrap_or(0)
            + self.w_scalar.map(|_| 1).unwrap_or(0)
            + self.u_scalar.map(|_| 1).unwrap_or(0)
            + self.other.len()
            + self.batched_other.len();
        let mut scalars: Vec<C::Scalar> = Vec::with_capacity(len);
        let mut bases: Vec<C> = Vec::with_capacity(len);

        for (x, (scalar, y)) in self
            .other
            .iter()
            .chain(self.batched_other.iter().map(|(x, v)| (x, v)))
        {
            scalars.push(*scalar);
            bases.push(C::from_xy(x.value, *y).unwrap());
        }

        if let Some(w_scalar) = self.w_scalar {
            scalars.push(w_scalar);
            bases.push(self.params.w);
        }

        if let Some(u_scalar) = self.u_scalar {
            scalars.push(u_scalar);
            bases.push(self.params.u);
        }

        if let Some(g_scalars) = self.g_scalars {
            scalars.extend(g_scalars);
            bases.extend(self.params.g.iter());
        }

        assert_eq!(scalars.len(), len);

        bool::from(best_multiexp_public(&scalars, &bases).is_identity())
    }

    /// Exposes the same terms evaluated by [`Self::eval`], with buffered batch
    /// terms coalesced identically. Copies are confined to this dev-only path.
    #[cfg(feature = "unstable-verifier-fingerprint")]
    #[allow(clippy::type_complexity)]
    pub(crate) fn fingerprint_terms(
        &self,
    ) -> (
        Option<Vec<C::Scalar>>,
        Option<C::Scalar>,
        Option<C::Scalar>,
        Vec<(C::Scalar, C::Base, C::Base)>,
    ) {
        let mut other = self.batched_other.clone();
        other.extend(self.other.iter().map(|(x, v)| (*x, *v)));
        canonicalize_other::<C>(&mut other);
        let other = other
            .into_iter()
            .map(|(x, (scalar, y))| (scalar, x.value, y))
            .collect();
        (self.g_scalars.clone(), self.w_scalar, self.u_scalar, other)
    }
}

#[cfg(test)]
mod scalar_tests;

#[cfg(test)]
mod tests {
    use crate::poly::commitment::{Params, MSM};
    use pasta_curves::{
        arithmetic::{CurveAffine, CurveExt},
        EpAffine, Fp, Fq,
    };

    #[test]
    fn msm_arithmetic() {
        let base = EpAffine::from_xy(-Fp::one(), Fp::from(2)).unwrap();
        let base_viol = (base + base).to_affine_vartime();

        let params = Params::new(4);
        let mut a: MSM<EpAffine> = MSM::new(&params);
        a.append_term(Fq::one(), base);
        // a = [1] P
        assert!(!a.clone().eval());
        a.append_term(Fq::one(), base);
        // a = [1+1] P
        assert!(!a.clone().eval());
        a.append_term(-Fq::one(), base_viol);
        // a = [1+1] P + [-1] 2P
        assert!(a.clone().eval());
        let b = a.clone();

        // Append a point that is the negation of an existing one.
        a.append_term(Fq::from(4), -base);
        // a = [1+1-4] P + [-1] 2P
        assert!(!a.clone().eval());
        a.append_term(Fq::from(2), base_viol);
        // a = [1+1-4] P + [-1+2] 2P
        assert!(a.clone().eval());

        // Add two MSMs with common bases.
        a.scale(Fq::from(3));
        a.add_msm(&b);
        // a = [3*(1+1)+(1+1-4)] P + [3*(-1)+(-1+2)] 2P
        assert!(a.clone().eval());

        let mut c: MSM<EpAffine> = MSM::new(&params);
        c.append_term(Fq::from(2), base);
        c.append_term(Fq::one(), -base_viol);
        // c = [2] P + [1] (-2P)
        assert!(c.clone().eval());
        // Add two MSMs with bases that differ only in sign.
        a.add_msm(&c);
        assert!(a.eval());
    }
}
