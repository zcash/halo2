//! This module contains an implementation of the polynomial commitment scheme
//! described in the [Halo][halo] paper.
//!
//! [halo]: https://eprint.iacr.org/2019/1021

use super::{Coeff, LagrangeCoeff, Polynomial};
use crate::arithmetic::{
    best_fft, best_multiexp, best_multiexp_public, parallelize, CurveAffine, CurveExt,
};
use crate::helpers::CurveRead;
#[cfg(feature = "batch")]
use crate::{
    GlvInstanceTable, INSTANCE_WINDOW_ENTRIES_PER_BASE, MAX_CACHED_INSTANCE_ROWS,
    PREPARED_INSTANCE_DENSE_ROWS, PREPARED_INSTANCE_OFFSETS, PREPARED_INSTANCE_ROWS,
};
#[cfg(feature = "batch")]
use std::{
    fmt,
    sync::{Arc, Mutex, OnceLock},
};

use ff::{Field, PrimeField};
use group::Group;
use std::ops::{Add, AddAssign, Mul, MulAssign};

mod msm;
mod prover;
#[cfg(all(feature = "batch", test))]
mod tests;
mod verifier;

pub use msm::MSM;
pub use prover::create_proof;
pub use verifier::{verify_proof, Accumulator, Guard};

use std::io;

/// These are the public parameters for the polynomial commitment scheme.
#[derive(Clone)]
#[cfg_attr(not(feature = "batch"), derive(Debug))]
pub struct Params<C: CurveAffine> {
    pub(crate) k: u32,
    pub(crate) n: u64,
    pub(crate) g: Vec<C>,
    pub(crate) g_lagrange: Vec<C>,
    pub(crate) w: C,
    pub(crate) u: C,
    #[cfg(feature = "batch")]
    instance_window_cache: InstanceWindowCache<C>,
    #[cfg(feature = "batch")]
    glv_instance_cache: Arc<OnceLock<Option<Arc<GlvInstanceTable<C>>>>>,
}

#[cfg(feature = "batch")]
#[derive(Clone)]
struct InstanceWindowCache<C>(Arc<Mutex<Option<Arc<Vec<C>>>>>);

#[cfg(feature = "batch")]
impl<C> Default for InstanceWindowCache<C> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

#[cfg(feature = "batch")]
impl<C> InstanceWindowCache<C> {
    fn get_or_grow(&self, base_count: usize, initialize: impl FnOnce() -> Vec<C>) -> Arc<Vec<C>> {
        let required_len = base_count
            .checked_mul(INSTANCE_WINDOW_ENTRIES_PER_BASE)
            .expect("instance window table length fits in usize");
        let mut table = self
            .0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(table) = table.as_ref().filter(|table| table.len() >= required_len) {
            return Arc::clone(table);
        }

        let initialized = Arc::new(initialize());
        assert_eq!(initialized.len(), required_len);
        *table = Some(Arc::clone(&initialized));
        initialized
    }
}

#[cfg(feature = "batch")]
impl<C: CurveAffine> fmt::Debug for Params<C> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Params")
            .field("k", &self.k)
            .field("n", &self.n)
            .field("g", &self.g)
            .field("g_lagrange", &self.g_lagrange)
            .field("w", &self.w)
            .field("u", &self.u)
            .finish()
    }
}

#[cfg(feature = "batch")]
impl<C: CurveAffine> Params<C> {
    pub(crate) fn instance_window_table(&self, base_count: usize) -> Arc<Vec<C>> {
        assert!(base_count <= MAX_CACHED_INSTANCE_ROWS);
        assert!(base_count <= self.g_lagrange.len());

        self.instance_window_cache.get_or_grow(base_count, || {
            let capacity = base_count
                .checked_mul(INSTANCE_WINDOW_ENTRIES_PER_BASE)
                .expect("instance window table length fits in usize");
            let mut projective = Vec::with_capacity(capacity);
            for base in &self.g_lagrange[..base_count] {
                let mut multiple = C::Curve::from(*base);
                for _ in 1..INSTANCE_WINDOW_ENTRIES_PER_BASE {
                    projective.push(multiple);
                    multiple += *base;
                }
                projective.push(multiple);
            }

            let mut affine = vec![C::identity(); projective.len()];
            C::Curve::batch_normalize_vartime(&projective, &mut affine);
            affine
        })
    }

    pub(crate) fn glv_instance_table(&self) -> Option<Arc<GlvInstanceTable<C>>> {
        self.glv_instance_cache
            .get_or_init(|| {
                if self.g_lagrange.len() < PREPARED_INSTANCE_ROWS {
                    return None;
                }
                let dense = C::CurveExt::try_prepare_fixed_base_sum(
                    &self.g_lagrange[..PREPARED_INSTANCE_DENSE_ROWS],
                )?;
                // Instance commitments use Blind::default(), whose scalar is one.
                let offsets = std::array::from_fn::<_, PREPARED_INSTANCE_OFFSETS, _>(|mask| {
                    self.g_lagrange[PREPARED_INSTANCE_DENSE_ROWS..PREPARED_INSTANCE_ROWS]
                        .iter()
                        .enumerate()
                        .filter(|(flag, _)| mask & (1 << flag) != 0)
                        .fold(C::Curve::from(self.w), |sum, (_, base)| sum + *base)
                });
                Some(Arc::new(GlvInstanceTable {
                    dense: core::panic::AssertUnwindSafe(dense),
                    offsets,
                }))
            })
            .clone()
    }
}

impl<C: CurveAffine> Params<C> {
    /// Initializes parameters for the curve, given a random oracle to draw
    /// points from.
    pub fn new(k: u32) -> Self {
        // This is usually a limitation on the curve, but we also want 32-bit
        // architectures to be supported.
        assert!(k < 32);

        // In src/arithmetic/fields.rs we ensure that usize is at least 32 bits.

        let n: u64 = 1 << k;

        let g_projective = {
            let mut g = Vec::with_capacity(n as usize);
            g.resize(n as usize, C::Curve::identity());

            parallelize(&mut g, move |g, start| {
                let hasher = C::CurveExt::hash_to_curve("Halo2-Parameters");

                for (i, g) in g.iter_mut().enumerate() {
                    let i = (i + start) as u32;

                    let mut message = [0u8; 5];
                    message[1..5].copy_from_slice(&i.to_le_bytes());

                    *g = hasher(&message);
                }
            });

            g
        };

        let g = {
            let mut g = vec![C::identity(); n as usize];
            parallelize(&mut g, |g, starts| {
                C::Curve::batch_normalize_vartime(&g_projective[starts..(starts + g.len())], g);
            });
            g
        };

        // Let's evaluate all of the Lagrange basis polynomials
        // using an inverse FFT.
        let mut alpha_inv = <<C as group::CurveAffine>::Curve as Group>::Scalar::ROOT_OF_UNITY_INV;
        for _ in k..C::Scalar::S {
            alpha_inv = alpha_inv.square();
        }
        let mut g_lagrange_projective = g_projective;
        best_fft(&mut g_lagrange_projective, alpha_inv, k);
        let minv = C::Scalar::TWO_INV.pow_vartime([k as u64]);
        parallelize(&mut g_lagrange_projective, |g, _| {
            for g in g.iter_mut() {
                *g *= minv;
            }
        });

        let g_lagrange = {
            let mut g_lagrange = vec![C::identity(); n as usize];
            parallelize(&mut g_lagrange, |g_lagrange, starts| {
                C::Curve::batch_normalize_vartime(
                    &g_lagrange_projective[starts..(starts + g_lagrange.len())],
                    g_lagrange,
                );
            });
            drop(g_lagrange_projective);
            g_lagrange
        };

        let hasher = C::CurveExt::hash_to_curve("Halo2-Parameters");
        let w = hasher(&[1]).to_affine_vartime();
        let u = hasher(&[2]).to_affine_vartime();

        Params {
            k,
            n,
            g,
            g_lagrange,
            w,
            u,
            #[cfg(feature = "batch")]
            instance_window_cache: InstanceWindowCache::default(),
            #[cfg(feature = "batch")]
            glv_instance_cache: Arc::new(OnceLock::new()),
        }
    }

    /// This computes a commitment to a polynomial described by the provided
    /// slice of coefficients. The commitment will be blinded by the blinding
    /// factor `r`.
    pub fn commit(&self, poly: &Polynomial<C::Scalar, Coeff>, r: Blind<C::Scalar>) -> C::Curve {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r.0);

        tmp_bases.extend(self.g.iter());
        tmp_bases.push(self.w);

        best_multiexp::<C>(&tmp_scalars, &tmp_bases)
    }

    /// This commits to a polynomial using its evaluations over the $2^k$ size
    /// evaluation domain. The commitment will be blinded by the blinding factor
    /// `r`.
    pub fn commit_lagrange(
        &self,
        poly: &Polynomial<C::Scalar, LagrangeCoeff>,
        r: Blind<C::Scalar>,
    ) -> C::Curve {
        let mut tmp_scalars = Vec::with_capacity(poly.len() + 1);
        let mut tmp_bases = Vec::with_capacity(poly.len() + 1);

        tmp_scalars.extend(poly.iter());
        tmp_scalars.push(r.0);

        tmp_bases.extend(self.g_lagrange.iter());
        tmp_bases.push(self.w);

        best_multiexp::<C>(&tmp_scalars, &tmp_bases)
    }

    /// This commits to an instance polynomial using its evaluations over the $2^k$ size
    /// evaluation domain. The commitment has no blinding factor, because instance
    /// polynomials are public.
    ///
    /// # Panics
    ///
    /// Panics if `instance.len() <= 1 << k`.
    pub(crate) fn commit_lagrange_instance(&self, instance: &[C::Scalar]) -> C::Curve {
        assert!(instance.len() <= self.n as usize);

        let mut tmp_scalars = Vec::with_capacity(instance.len() + 1);
        let mut tmp_bases = Vec::with_capacity(instance.len() + 1);

        tmp_scalars.extend(instance);
        tmp_scalars.push(Blind::default().0);

        tmp_bases.extend(&self.g_lagrange[..instance.len()]);
        tmp_bases.push(self.w);

        best_multiexp_public::<C>(&tmp_scalars, &tmp_bases)
    }

    /// Generates an empty multiscalar multiplication struct using the
    /// appropriate params.
    pub fn empty_msm(&self) -> MSM<'_, C> {
        MSM::new(self)
    }

    /// Getter for g generators
    pub fn get_g(&self) -> Vec<C> {
        self.g.clone()
    }

    /// Get the circuit size parameter k
    pub fn k(&self) -> u32 {
        self.k
    }

    /// Writes params to a buffer.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.k.to_le_bytes())?;
        for g_element in &self.g {
            writer.write_all(g_element.to_bytes().as_ref())?;
        }
        for g_lagrange_element in &self.g_lagrange {
            writer.write_all(g_lagrange_element.to_bytes().as_ref())?;
        }
        writer.write_all(self.w.to_bytes().as_ref())?;
        writer.write_all(self.u.to_bytes().as_ref())?;

        Ok(())
    }

    /// Reads params from a buffer.
    pub fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut k = [0u8; 4];
        reader.read_exact(&mut k[..])?;
        let k = u32::from_le_bytes(k);

        let n: u64 = 1 << k;

        let g: Vec<_> = (0..n).map(|_| C::read(reader)).collect::<Result<_, _>>()?;
        let g_lagrange: Vec<_> = (0..n).map(|_| C::read(reader)).collect::<Result<_, _>>()?;

        let w = C::read(reader)?;
        let u = C::read(reader)?;

        Ok(Params {
            k,
            n,
            g,
            g_lagrange,
            w,
            u,
            #[cfg(feature = "batch")]
            instance_window_cache: InstanceWindowCache::default(),
            #[cfg(feature = "batch")]
            glv_instance_cache: Arc::new(OnceLock::new()),
        })
    }
}

/// Wrapper type around a blinding factor.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Blind<F>(pub F);

impl<F: Field> Default for Blind<F> {
    fn default() -> Self {
        Blind(F::ONE)
    }
}

impl<F: Field> Add for Blind<F> {
    type Output = Self;

    fn add(self, rhs: Blind<F>) -> Self {
        Blind(self.0 + rhs.0)
    }
}

impl<F: Field> Mul for Blind<F> {
    type Output = Self;

    fn mul(self, rhs: Blind<F>) -> Self {
        Blind(self.0 * rhs.0)
    }
}

impl<F: Field> AddAssign for Blind<F> {
    fn add_assign(&mut self, rhs: Blind<F>) {
        self.0 += rhs.0;
    }
}

impl<F: Field> MulAssign for Blind<F> {
    fn mul_assign(&mut self, rhs: Blind<F>) {
        self.0 *= rhs.0;
    }
}

impl<F: Field> AddAssign<F> for Blind<F> {
    fn add_assign(&mut self, rhs: F) {
        self.0 += rhs;
    }
}

impl<F: Field> MulAssign<F> for Blind<F> {
    fn mul_assign(&mut self, rhs: F) {
        self.0 *= rhs;
    }
}

#[test]
fn test_commit_lagrange_epaffine() {
    const K: u32 = 6;

    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    use crate::pasta::{EpAffine, Fq};
    let params = Params::<EpAffine>::new(K);
    let domain = super::EvaluationDomain::new(1, K);

    let mut a = domain.empty_lagrange();

    for (i, a) in a.iter_mut().enumerate() {
        *a = Fq::from(i as u64);
    }

    let b = domain.lagrange_to_coeff(a.clone());

    let alpha = Blind(Fq::random(&mut UnwrapErr(SysRng)));

    assert_eq!(params.commit(&b, alpha), params.commit_lagrange(&a, alpha));
}

#[test]
fn test_commit_lagrange_eqaffine() {
    const K: u32 = 6;

    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    use crate::pasta::{EqAffine, Fp};
    let params = Params::<EqAffine>::new(K);
    let domain = super::EvaluationDomain::new(1, K);

    let mut a = domain.empty_lagrange();

    for (i, a) in a.iter_mut().enumerate() {
        *a = Fp::from(i as u64);
    }

    let b = domain.lagrange_to_coeff(a.clone());

    let alpha = Blind(Fp::random(&mut UnwrapErr(SysRng)));

    assert_eq!(params.commit(&b, alpha), params.commit_lagrange(&a, alpha));
}

#[test]
fn test_opening_proof() {
    const K: u32 = 6;

    use ff::Field;
    use rand::rngs::SysRng;
    use rand_core::UnwrapErr;

    use super::{
        commitment::{Blind, Params},
        EvaluationDomain,
    };
    use crate::arithmetic::eval_polynomial;
    use crate::pasta::{EpAffine, Fq};
    use crate::transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, Transcript, TranscriptRead, TranscriptWrite,
    };

    let mut rng = UnwrapErr(SysRng);

    let params = Params::<EpAffine>::new(K);
    let mut params_buffer = vec![];
    params.write(&mut params_buffer).unwrap();
    let params: Params<EpAffine> = Params::read::<_>(&mut &params_buffer[..]).unwrap();

    let domain = EvaluationDomain::new(1, K);

    let mut px = domain.empty_coeff();

    for (i, a) in px.iter_mut().enumerate() {
        *a = Fq::from(i as u64);
    }

    let blind = Blind(Fq::random(&mut rng));

    let p = params.commit(&px, blind).to_affine_vartime();

    let mut transcript = Blake2bWrite::<Vec<u8>, EpAffine, Challenge255<EpAffine>>::init(vec![]);
    transcript.write_point(p).unwrap();
    let x = transcript.squeeze_challenge_scalar::<()>();
    // Evaluate the polynomial
    let v = eval_polynomial(&px, *x);
    transcript.write_scalar(v).unwrap();

    let (proof, ch_prover) = {
        create_proof(&params, rng, &mut transcript, &px, blind, *x).unwrap();
        let ch_prover = transcript.squeeze_challenge();
        (transcript.finalize(), ch_prover)
    };

    // Verify the opening proof
    let mut transcript = Blake2bRead::<&[u8], EpAffine, Challenge255<EpAffine>>::init(&proof[..]);
    let p_prime = transcript.read_point().unwrap();
    assert_eq!(p, p_prime);
    let x_prime = transcript.squeeze_challenge_scalar::<()>();
    assert_eq!(*x, *x_prime);
    let v_prime = transcript.read_scalar().unwrap();
    assert_eq!(v, v_prime);

    let mut commitment_msm = params.empty_msm();
    commitment_msm.append_term(Field::ONE, p);
    let guard = verify_proof(&params, commitment_msm, &mut transcript, *x, v).unwrap();
    let ch_verifier = transcript.squeeze_challenge();
    assert_eq!(*ch_prover, *ch_verifier);

    // Test guard behavior prior to checking another proof
    {
        // Test use_challenges()
        let msm_challenges = guard.clone().use_challenges();
        assert!(msm_challenges.eval());

        // Test use_g()
        let g = guard.compute_g();
        let (msm_g, _accumulator) = guard.clone().use_g(g);
        assert!(msm_g.eval());
    }
}
