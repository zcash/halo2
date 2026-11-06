use super::Params;
use crate::arithmetic::{best_multiexp, CurveAffine};
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
        // Any byte ordering is sufficient for grouping equal coordinates.
        // Comparing from the final byte also preserves the existing order for
        // Pasta fields, whose canonical representations are little-endian.
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

/// A multiscalar multiplication in the polynomial commitment scheme
#[derive(Debug, Clone)]
pub struct MSM<'a, C: CurveAffine> {
    pub(crate) params: &'a Params<C>,
    g_scalars: Option<Vec<C::Scalar>>,
    w_scalar: Option<C::Scalar>,
    u_scalar: Option<C::Scalar>,
    // Cached x-coordinate -> (scalar, y-coordinate)
    other: OtherTerms<C>,
    // Batch reduction moves arbitrary terms here and canonicalizes them once
    // before the final multiscalar multiplication.
    batched_other: Vec<ArbitraryTerm<C>>,
}

fn canonicalize_other<C: CurveAffine>(mut terms: Vec<ArbitraryTerm<C>>) -> Vec<ArbitraryTerm<C>> {
    // The stable sort keeps the first-encountered orientation for each
    // x-coordinate. Which orientation that is depends on aggregation order
    // (Rayon's reduction shape is not fixed), but the choice cannot affect
    // the evaluated sum: `(scalar, P)` and `(-scalar, -P)` are the same term,
    // and the coalescing below negates the scalar exactly when the stored
    // orientation differs.
    terms.sort_by(|a, b| a.0.cmp(&b.0));

    let mut canonical = Vec::with_capacity(terms.len());
    for (x, (scalar, y)) in terms {
        if let Some((our_x, (our_scalar, our_y))) = canonical.last_mut() {
            if *our_x == x {
                if *our_y == y {
                    *our_scalar += scalar;
                } else {
                    assert!(*our_y == -y);
                    *our_scalar -= scalar;
                }
                continue;
            }
        }
        canonical.push((x, (scalar, y)));
    }

    canonical
}

impl<'a, C: CurveAffine> MSM<'a, C> {
    /// Create a new, empty MSM using the provided parameters.
    pub fn new(params: &'a Params<C>) -> Self {
        let g_scalars = None;
        let w_scalar = None;
        let u_scalar = None;
        let other = BTreeMap::new();
        let batched_other = vec![];

        MSM {
            params,
            g_scalars,
            w_scalar,
            u_scalar,
            other,
            batched_other,
        }
    }

    fn add_other(&mut self, x: C::Base, scalar: C::Scalar, y: C::Base) {
        self.add_other_key(CanonicalFieldKey::new(x), scalar, y);
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

    /// Add another multiexp into this one
    pub fn add_msm(&mut self, other: &Self) {
        for (x, (scalar, y)) in other.other.iter() {
            self.add_other_key(*x, *scalar, *y);
        }
        for (x, (scalar, y)) in other.batched_other.iter() {
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

    /// Adds an owned MSM while deferring arbitrary-point coalescing until the
    /// completed batch is evaluated.
    #[cfg(any(test, feature = "batch"))]
    pub(crate) fn add_msm_batch(&mut self, other: Self) {
        let Self {
            g_scalars,
            w_scalar,
            u_scalar,
            other,
            batched_other,
            ..
        } = other;

        self.batched_other
            .reserve(other.len() + batched_other.len());
        self.batched_other.extend(other);
        self.batched_other.extend(batched_other);

        if let Some(g_scalars) = g_scalars {
            if let Some(our_g_scalars) = &mut self.g_scalars {
                assert_eq!(our_g_scalars.len(), g_scalars.len());
                for (our_scalar, scalar) in our_g_scalars.iter_mut().zip(g_scalars) {
                    *our_scalar += scalar;
                }
            } else {
                self.g_scalars = Some(g_scalars);
            }
        }

        if let Some(w_scalar) = w_scalar {
            self.add_to_w_scalar(w_scalar);
        }
        if let Some(u_scalar) = u_scalar {
            self.add_to_u_scalar(u_scalar);
        }
    }

    /// Add arbitrary term (the scalar and the point)
    pub fn append_term(&mut self, scalar: C::Scalar, point: C) {
        if !bool::from(point.is_identity()) {
            let xy = point.coordinates().unwrap();
            let x = *xy.x();
            let y = *xy.y();
            self.add_other(x, scalar, y);
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

    /// Adds owned generator coefficients, retaining their allocation when
    /// this [`MSM`] has no existing generator terms.
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
        for (_, (scalar, _)) in self.batched_other.iter_mut() {
            *scalar *= factor;
        }

        self.w_scalar = self.w_scalar.map(|a| a * &factor);
        self.u_scalar = self.u_scalar.map(|a| a * &factor);
    }

    fn multiexp(self) -> C::Curve {
        let Self {
            params,
            g_scalars,
            w_scalar,
            u_scalar,
            other,
            mut batched_other,
        } = self;

        let other_len = if batched_other.is_empty() {
            other.len()
        } else {
            batched_other.extend(other.iter().map(|(x, values)| (*x, *values)));
            batched_other = canonicalize_other::<C>(batched_other);
            batched_other.len()
        };
        let len = g_scalars.as_deref().map(<[_]>::len).unwrap_or(0)
            + w_scalar.map(|_| 1).unwrap_or(0)
            + u_scalar.map(|_| 1).unwrap_or(0)
            + other_len;
        let mut scalars: Vec<C::Scalar> = Vec::with_capacity(len);
        let mut bases: Vec<C> = Vec::with_capacity(len);

        if batched_other.is_empty() {
            scalars.extend(other.values().map(|(scalar, _)| *scalar));
            bases.extend(
                other
                    .iter()
                    .map(|(x, (_, y))| C::from_xy(x.value, *y).unwrap()),
            );
        } else {
            scalars.extend(batched_other.iter().map(|(_, (scalar, _))| *scalar));
            bases.extend(
                batched_other
                    .iter()
                    .map(|(x, (_, y))| C::from_xy(x.value, *y).unwrap()),
            );
        }

        if let Some(w_scalar) = w_scalar {
            scalars.push(w_scalar);
            bases.push(params.w);
        }

        if let Some(u_scalar) = u_scalar {
            scalars.push(u_scalar);
            bases.push(params.u);
        }

        if let Some(g_scalars) = g_scalars {
            scalars.extend(g_scalars);
            bases.extend(params.g.iter());
        }

        assert_eq!(scalars.len(), len);

        best_multiexp(&scalars, &bases)
    }

    /// Perform multiexp and check that it results in zero
    pub fn eval(self) -> bool {
        bool::from(self.multiexp().is_identity())
    }

    /// The MSM's accumulated terms, exposed for capturing the verifier fingerprint without consuming
    /// it: the `g_scalars`, `w_scalar`, `u_scalar`, and the `other` terms as `(scalar, x, y)` for each
    /// accumulated commitment point. This uses the same canonicalization as
    /// [`eval`](Self::eval), so the exported fingerprint cannot drift from the
    /// evaluated view. The owned copies are confined to this dev-only path.
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
        let mut other = self
            .other
            .iter()
            .map(|(x, (scalar, y))| (*scalar, x.value, *y))
            .collect::<Vec<_>>();
        if !self.batched_other.is_empty() {
            let mut combined = self.batched_other.clone();
            combined.extend(self.other.iter().map(|(x, values)| (*x, *values)));
            other = canonicalize_other::<C>(combined)
                .into_iter()
                .map(|(x, (scalar, y))| (scalar, x.value, y))
                .collect();
        }
        (self.g_scalars.clone(), self.w_scalar, self.u_scalar, other)
    }
}

#[cfg(test)]
mod tests {
    use super::CanonicalFieldKey;
    use crate::poly::commitment::{Params, MSM};
    use pasta_curves::{
        arithmetic::{CurveAffine, CurveExt},
        EpAffine, EqAffine, Fp, Fq,
    };

    fn assert_cached_key_order<F: ff::PrimeField + Ord>(values: &[F]) {
        for left in values {
            for right in values {
                assert_eq!(
                    CanonicalFieldKey::new(*left).cmp(&CanonicalFieldKey::new(*right)),
                    left.cmp(right),
                );
            }
        }
    }

    #[test]
    fn cached_pasta_keys_preserve_field_order() {
        let pallas_params = Params::<EpAffine>::new(4);
        let pallas_xs = pallas_params
            .g
            .iter()
            .map(|point| *point.coordinates().unwrap().x())
            .chain([Fp::from(0), Fp::from(1)])
            .collect::<Vec<_>>();
        assert_cached_key_order(&pallas_xs);

        let vesta_params = Params::<EqAffine>::new(4);
        let vesta_xs = vesta_params
            .g
            .iter()
            .map(|point| *point.coordinates().unwrap().x())
            .chain([Fq::from(0), Fq::from(1)])
            .collect::<Vec<_>>();
        assert_cached_key_order(&vesta_xs);
    }

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

    #[test]
    fn batch_aggregation_matches_incremental_aggregation() {
        let params = Params::<EpAffine>::new(6);
        let mut components = Vec::with_capacity(64);

        for proof in 0..64 {
            let mut msm = MSM::new(&params);
            let g_scalars = (0..params.n)
                .map(|index| Fq::from(u64::from(index) + proof + 1))
                .collect::<Vec<_>>();
            msm.add_to_g_scalars(&g_scalars);
            msm.add_to_w_scalar(Fq::from(proof + 3));
            msm.add_to_u_scalar(Fq::from(proof + 5));

            // Exercise common points, proof-specific points, repeated points,
            // and opposite orientations of the same point.
            msm.append_term(Fq::from(proof + 7), params.g[1]);
            msm.append_term(Fq::from(proof + 11), params.g[2]);
            msm.append_term(Fq::from(proof + 13), -params.g[2]);
            msm.append_term(Fq::from(proof + 17), params.g[3 + proof as usize % 32]);
            if proof % 2 == 0 {
                msm.scale(Fq::from(19));
            }
            components.push(msm);
        }

        let mut incremental = MSM::new(&params);
        for component in &components {
            incremental.add_msm(component);
        }

        let mut batched = MSM::new(&params);
        for component in components.iter().cloned() {
            batched.add_msm_batch(component);
        }

        let expected = incremental.multiexp();
        assert_eq!(expected, batched.multiexp());

        // Match the tree-shaped reduction used by the multicore batch
        // verifier: aggregate proofs into worker-local MSMs, then aggregate
        // the worker results.
        let mut workers = (0..4).map(|_| MSM::new(&params)).collect::<Vec<_>>();
        for (index, component) in components.into_iter().enumerate() {
            workers[index % 4].add_msm_batch(component);
        }
        let mut reduced = MSM::new(&params);
        for worker in workers {
            reduced.add_msm_batch(worker);
        }

        assert_eq!(expected, reduced.multiexp());
    }

    /// Covers the interactions the batch-aggregation test does not reach:
    /// `add_msm` reading a batched source, an MSM evaluating with both the
    /// incremental map and the batched buffer populated, scaling after batch
    /// accumulation, and coalescing that cancels a point's scalar exactly.
    #[test]
    fn batched_msm_interoperates_with_incremental_paths() {
        let params = Params::<EpAffine>::new(4);

        let make_components = || {
            (0..6u64)
                .map(|index| {
                    let mut msm = MSM::new(&params);
                    let g_scalars = (0..params.n)
                        .map(|slot| Fq::from(slot + index + 1))
                        .collect::<Vec<_>>();
                    msm.add_to_g_scalars(&g_scalars);
                    msm.add_to_w_scalar(Fq::from(2 * index + 1));
                    msm.add_to_u_scalar(Fq::from(3 * index + 1));
                    msm.append_term(Fq::from(index + 2), params.g[1]);
                    // The same point in both orientations, with scalars that
                    // cancel exactly once coalesced (three of each).
                    if index % 2 == 0 {
                        msm.append_term(Fq::from(5), params.g[2]);
                    } else {
                        msm.append_term(Fq::from(5), -params.g[2]);
                    }
                    msm.append_term(Fq::from(7 + index), params.g[3 + index as usize]);
                    msm
                })
                .collect::<Vec<_>>()
        };

        fn incremental<'a>(
            params: &'a Params<EpAffine>,
            components: Vec<MSM<'a, EpAffine>>,
        ) -> MSM<'a, EpAffine> {
            let mut acc = MSM::new(params);
            for component in &components {
                acc.add_msm(component);
            }
            acc
        }
        fn batched<'a>(
            params: &'a Params<EpAffine>,
            components: Vec<MSM<'a, EpAffine>>,
        ) -> MSM<'a, EpAffine> {
            let mut acc = MSM::new(params);
            for component in components {
                acc.add_msm_batch(component);
            }
            acc
        }
        let expected = incremental(&params, make_components()).multiexp();

        // `add_msm` must drain a source whose terms live in the batched
        // buffer rather than the incremental map.
        let mut via_add_msm = MSM::new(&params);
        via_add_msm.add_msm(&batched(&params, make_components()));
        assert_eq!(expected, via_add_msm.multiexp());

        // Evaluation must merge both storages when terms were appended after
        // batch accumulation, including a fresh x and a new orientation of an
        // already-batched point.
        let mut mixed = batched(&params, make_components());
        mixed.append_term(Fq::from(23), params.g[4]);
        mixed.append_term(Fq::from(29), -params.g[3]);
        let mut mixed_reference = incremental(&params, make_components());
        mixed_reference.append_term(Fq::from(23), params.g[4]);
        mixed_reference.append_term(Fq::from(29), -params.g[3]);
        assert_eq!(mixed_reference.multiexp(), mixed.multiexp());

        // Scaling must reach the batched buffer.
        let mut scaled = batched(&params, make_components());
        scaled.scale(Fq::from(31));
        let mut scaled_reference = incremental(&params, make_components());
        scaled_reference.scale(Fq::from(31));
        assert_eq!(scaled_reference.multiexp(), scaled.multiexp());

        // The dev-only fingerprint must canonicalize the batched buffer to
        // the exact view the incremental accumulation exports; the insertion
        // order above pins the same first-encountered orientation for both.
        #[cfg(feature = "unstable-verifier-fingerprint")]
        assert_eq!(
            batched(&params, make_components()).fingerprint_terms(),
            incremental(&params, make_components()).fingerprint_terms(),
        );

        // A batch whose only terms cancel exactly still evaluates, and to
        // the identity.
        let mut first = MSM::new(&params);
        first.append_term(Fq::from(21), params.g[1]);
        let mut second = MSM::new(&params);
        second.append_term(Fq::from(21), -params.g[1]);
        let mut cancelled = MSM::new(&params);
        cancelled.add_msm_batch(first);
        cancelled.add_msm_batch(second);
        assert!(cancelled.eval());
    }
}
