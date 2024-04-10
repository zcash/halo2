//! This module provides "ZK Acceleration Layer" traits
//! to abstract away the execution engine for performance-critical primitives.
//!
//! Terminology
//! -----------
//!
//! We use the name Backend+Engine for concrete implementations of ZalEngine.
//! For example H2cEngine for pure Halo2curves implementation.
//!
//! Alternative names considered were Executor or Driver however
//! - executor is already used in Rust (and the name is long)
//! - driver will be confusing as we work quite low-level with GPUs and FPGAs.
//!
//! Unfortunately the "Engine" name is used in bn256 for pairings.
//! Fortunately a ZalEngine is only used in the prover (at least for now)
//! while "pairing engine" is only used in the verifier
//!
//! Initialization design space
//! ---------------------------
//!
//! It is recommended that ZAL backends provide:
//! - an initialization function:
//!   - either "fn new() -> ZalEngine" for simple libraries
//!   - or a builder pattern for complex initializations
//! - a shutdown function or document when it is not needed (when it's a global threadpool like Rayon for example).
//!
//! Backends might want to add as an option:
//! - The number of threads (CPU)
//! - The device(s) to run on (multi-sockets machines, multi-GPUs machines, ...)
//! - The curve (JIT-compiled backend)
//!
//! Descriptors
//! ---------------------------
//!
//! Descriptors enable providers to configure opaque details on data
//! when doing repeated computations with the same input(s).
//! For example:
//! - Pointer(s) caching to limit data movement between CPU and GPU, FPGAs
//! - Length of data
//! - data in layout:
//!    - canonical or Montgomery fields, unsaturated representation, endianness
//!    - jacobian or projective coordinates or maybe even Twisted Edwards for faster elliptic curve additions,
//!    - FFT: canonical or bit-reversed permuted
//! - data out layout
//! - Device(s) ID
//!
//! For resources that need special cleanup like GPU memory, a custom `Drop` is required.
//!
//! Note that resources can also be stored in the engine in a hashmap
//! and an integer ID or a pointer can be opaquely given as a descriptor.

// The ZK Accel Layer API
// ---------------------------------------------------
pub mod traits {
    use halo2curves::CurveAffine;

    pub trait MsmAccel<C: CurveAffine> {
        fn msm(&self, coeffs: &[C::Scalar], base: &[C]) -> C::Curve;

        // Caching API
        // -------------------------------------------------
        // From here we propose an extended API
        // that allows reusing coeffs and/or the base points
        //
        // This is inspired by CuDNN API (Nvidia GPU)
        // and oneDNN API (CPU, OpenCL) https://docs.nvidia.com/deeplearning/cudnn/api/index.html#cudnn-ops-infer-so-opaque
        // usage of descriptors
        //
        // https://github.com/oneapi-src/oneDNN/blob/master/doc/programming_model/basic_concepts.md
        //
        // Descriptors are opaque pointers that hold the input in a format suitable for the accelerator engine.
        // They may be:
        // - Input moved on accelerator device (only once for repeated calls)
        // - Endianess conversion
        // - Converting from Montgomery to Canonical form
        // - Input changed from Projective to Jacobian coordinates or even to a Twisted Edwards curve.
        // - other form of expensive preprocessing
        type CoeffsDescriptor<'c>;
        type BaseDescriptor<'b>;

        fn get_coeffs_descriptor<'c>(&self, coeffs: &'c [C::Scalar]) -> Self::CoeffsDescriptor<'c>;
        fn get_base_descriptor<'b>(&self, base: &'b [C]) -> Self::BaseDescriptor<'b>;

        fn msm_with_cached_scalars(
            &self,
            coeffs: &Self::CoeffsDescriptor<'_>,
            base: &[C],
        ) -> C::Curve;

        fn msm_with_cached_base(
            &self,
            coeffs: &[C::Scalar],
            base: &Self::BaseDescriptor<'_>,
        ) -> C::Curve;

        fn msm_with_cached_inputs(
            &self,
            coeffs: &Self::CoeffsDescriptor<'_>,
            base: &Self::BaseDescriptor<'_>,
        ) -> C::Curve;
        // Execute MSM according to descriptors
        // Unsure of naming, msm_with_cached_inputs, msm_apply, msm_cached, msm_with_descriptors, ...
    }
}

// ZAL using Halo2curves as a backend
// ---------------------------------------------------

pub mod impls {
    use std::marker::PhantomData;

    use crate::zal::traits::MsmAccel;
    use halo2curves::msm::best_multiexp;
    use halo2curves::CurveAffine;

    // Halo2curve Backend
    // ---------------------------------------------------
    #[derive(Default)]
    pub struct H2cEngine;

    pub struct H2cMsmCoeffsDesc<'c, C: CurveAffine> {
        raw: &'c [C::Scalar],
    }

    pub struct H2cMsmBaseDesc<'b, C: CurveAffine> {
        raw: &'b [C],
    }

    impl H2cEngine {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl<C: CurveAffine> MsmAccel<C> for H2cEngine {
        fn msm(&self, coeffs: &[C::Scalar], bases: &[C]) -> C::Curve {
            best_multiexp(coeffs, bases)
        }

        // Caching API
        // -------------------------------------------------

        type CoeffsDescriptor<'c> = H2cMsmCoeffsDesc<'c, C>;
        type BaseDescriptor<'b> = H2cMsmBaseDesc<'b, C>;

        fn get_coeffs_descriptor<'c>(&self, coeffs: &'c [C::Scalar]) -> Self::CoeffsDescriptor<'c> {
            // Do expensive device/library specific preprocessing here
            Self::CoeffsDescriptor { raw: coeffs }
        }
        fn get_base_descriptor<'b>(&self, base: &'b [C]) -> Self::BaseDescriptor<'b> {
            Self::BaseDescriptor { raw: base }
        }

        fn msm_with_cached_scalars(
            &self,
            coeffs: &Self::CoeffsDescriptor<'_>,
            base: &[C],
        ) -> C::Curve {
            best_multiexp(coeffs.raw, base)
        }

        fn msm_with_cached_base(
            &self,
            coeffs: &[C::Scalar],
            base: &Self::BaseDescriptor<'_>,
        ) -> C::Curve {
            best_multiexp(coeffs, base.raw)
        }

        fn msm_with_cached_inputs(
            &self,
            coeffs: &Self::CoeffsDescriptor<'_>,
            base: &Self::BaseDescriptor<'_>,
        ) -> C::Curve {
            best_multiexp(coeffs.raw, base.raw)
        }
    }

    // Backend-agnostic engine objects
    // ---------------------------------------------------
    #[derive(Debug)]
    pub struct PlonkEngine<C: CurveAffine, MsmEngine: MsmAccel<C>> {
        pub msm_backend: MsmEngine,
        _marker: PhantomData<C>, // compiler complains about unused C otherwise
    }

    #[derive(Default)]
    pub struct PlonkEngineConfig<C, M> {
        curve: PhantomData<C>,
        msm_backend: M,
    }

    #[derive(Default)]
    pub struct NoCurve;

    #[derive(Default)]
    pub struct HasCurve<C: CurveAffine>(PhantomData<C>);

    #[derive(Default)]
    pub struct NoMsmEngine;

    pub struct HasMsmEngine<C: CurveAffine, M: MsmAccel<C>>(M, PhantomData<C>);

    impl PlonkEngineConfig<NoCurve, NoMsmEngine> {
        pub fn new() -> PlonkEngineConfig<NoCurve, NoMsmEngine> {
            Default::default()
        }

        pub fn set_curve<C: CurveAffine>(self) -> PlonkEngineConfig<HasCurve<C>, NoMsmEngine> {
            Default::default()
        }

        pub fn build_default<C: CurveAffine>() -> PlonkEngine<C, H2cEngine> {
            PlonkEngine {
                msm_backend: H2cEngine::new(),
                _marker: Default::default(),
            }
        }
    }

    impl<C: CurveAffine, M> PlonkEngineConfig<HasCurve<C>, M> {
        pub fn set_msm<MsmEngine: MsmAccel<C>>(
            self,
            engine: MsmEngine,
        ) -> PlonkEngineConfig<HasCurve<C>, HasMsmEngine<C, MsmEngine>> {
            // Copy all other parameters
            let Self { curve, .. } = self;
            // Return with modified MSM engine
            PlonkEngineConfig {
                curve,
                msm_backend: HasMsmEngine(engine, Default::default()),
            }
        }
    }

    impl<C: CurveAffine, M: MsmAccel<C>> PlonkEngineConfig<HasCurve<C>, HasMsmEngine<C, M>> {
        pub fn build(self) -> PlonkEngine<C, M> {
            PlonkEngine {
                msm_backend: self.msm_backend.0,
                _marker: Default::default(),
            }
        }
    }
}

// Testing
// ---------------------------------------------------

#[cfg(test)]
mod test {
    use crate::zal::impls::{H2cEngine, PlonkEngineConfig};
    use crate::zal::traits::MsmAccel;
    use halo2curves::bn256::G1Affine;
    use halo2curves::msm::best_multiexp;
    use halo2curves::CurveAffine;

    use ark_std::{end_timer, start_timer};
    use ff::Field;
    use group::{Curve, Group};
    use rand_core::OsRng;

    fn run_msm_zal_default<C: CurveAffine>(min_k: usize, max_k: usize) {
        let points = (0..1 << max_k)
            .map(|_| C::Curve::random(OsRng))
            .collect::<Vec<_>>();
        let mut affine_points = vec![C::identity(); 1 << max_k];
        C::Curve::batch_normalize(&points[..], &mut affine_points[..]);
        let points = affine_points;

        let scalars = (0..1 << max_k)
            .map(|_| C::Scalar::random(OsRng))
            .collect::<Vec<_>>();

        for k in min_k..=max_k {
            let points = &points[..1 << k];
            let scalars = &scalars[..1 << k];

            let t0 = start_timer!(|| format!("freestanding msm k={}", k));
            let e0 = best_multiexp(scalars, points);
            end_timer!(t0);

            let engine = PlonkEngineConfig::build_default::<G1Affine>();
            let t1 = start_timer!(|| format!("H2cEngine msm k={}", k));
            let e1 = engine.msm_backend.msm(scalars, points);
            end_timer!(t1);

            assert_eq!(e0, e1);

            // Caching API
            // -----------
            let t2 = start_timer!(|| format!("H2cEngine msm cached base k={}", k));
            let base_descriptor = engine.msm_backend.get_base_descriptor(points);
            let e2 = engine
                .msm_backend
                .msm_with_cached_base(scalars, &base_descriptor);
            end_timer!(t2);

            assert_eq!(e0, e2)
        }
    }

    fn run_msm_zal_custom<C: CurveAffine>(min_k: usize, max_k: usize) {
        let points = (0..1 << max_k)
            .map(|_| C::Curve::random(OsRng))
            .collect::<Vec<_>>();
        let mut affine_points = vec![C::identity(); 1 << max_k];
        C::Curve::batch_normalize(&points[..], &mut affine_points[..]);
        let points = affine_points;

        let scalars = (0..1 << max_k)
            .map(|_| C::Scalar::random(OsRng))
            .collect::<Vec<_>>();

        for k in min_k..=max_k {
            let points = &points[..1 << k];
            let scalars = &scalars[..1 << k];

            let t0 = start_timer!(|| format!("freestanding msm k={}", k));
            let e0 = best_multiexp(scalars, points);
            end_timer!(t0);

            let engine = PlonkEngineConfig::new()
                .set_curve::<G1Affine>()
                .set_msm(H2cEngine::new())
                .build();
            let t1 = start_timer!(|| format!("H2cEngine msm k={}", k));
            let e1 = engine.msm_backend.msm(scalars, points);
            end_timer!(t1);

            assert_eq!(e0, e1);

            // Caching API
            // -----------
            let t2 = start_timer!(|| format!("H2cEngine msm cached base k={}", k));
            let base_descriptor = engine.msm_backend.get_base_descriptor(points);
            let e2 = engine
                .msm_backend
                .msm_with_cached_base(scalars, &base_descriptor);
            end_timer!(t2);

            assert_eq!(e0, e2)
        }
    }

    #[test]
    fn test_msm_zal() {
        run_msm_zal_default::<G1Affine>(3, 14);
        run_msm_zal_custom::<G1Affine>(3, 14);
    }
}
