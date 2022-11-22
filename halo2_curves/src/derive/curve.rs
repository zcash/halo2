use crate::secp256k1::Secp256k1;

#[macro_export]
macro_rules! batch_add {
    () => {
        fn batch_add<const COMPLETE: bool, const LOAD_POINTS: bool>(
            points: &mut [Self],
            output_indices: &[u32],
            num_points: usize,
            offset: usize,
            bases: &[Self],
            base_positions: &[u32],
        ) {
            // assert!(Self::constant_a().is_zero());

            let get_point = |point_data: u32| -> Self {
                let negate = point_data & 0x80000000 != 0;
                let base_idx = (point_data & 0x7FFFFFFF) as usize;
                if negate {
                    bases[base_idx].neg()
                } else {
                    bases[base_idx]
                }
            };

            // Affine addition formula (P != Q):
            // - lambda = (y_2 - y_1) / (x_2 - x_1)
            // - x_3 = lambda^2 - (x_2 + x_1)
            // - y_3 = lambda * (x_1 - x_3) - y_1

            // Batch invert accumulator
            let mut acc = Self::Base::one();

            for i in (0..num_points).step_by(2) {
                // Where that result of the point addition will be stored
                let out_idx = output_indices[i >> 1] as usize - offset;

                #[cfg(all(feature = "prefetch", target_arch = "x86_64"))]
                if i < num_points - 2 {
                    if LOAD_POINTS {
                        crate::prefetch::<Self>(bases, base_positions[i + 2] as usize);
                        crate::prefetch::<Self>(bases, base_positions[i + 3] as usize);
                    }
                    crate::prefetch::<Self>(points, output_indices[(i >> 1) + 1] as usize - offset);
                }
                if LOAD_POINTS {
                    points[i] = get_point(base_positions[i]);
                    points[i + 1] = get_point(base_positions[i + 1]);
                }

                if COMPLETE {
                    // Nothing to do here if one of the points is zero
                    if (points[i].is_identity() | points[i + 1].is_identity()).into() {
                        continue;
                    }

                    if points[i].x == points[i + 1].x {
                        if points[i].y == points[i + 1].y {
                            // Point doubling (P == Q)
                            // - s = (3 * x^2) / (2 * y)
                            // - x_2 = s^2 - (2 * x)
                            // - y_2 = s * (x - x_2) - y

                            // (2 * x)
                            points[out_idx].x = points[i].x + points[i].x;
                            // x^2
                            let xx = points[i].x.square();
                            // (2 * y)
                            points[i + 1].x = points[i].y + points[i].y;
                            // (3 * x^2) * acc
                            points[i + 1].y = (xx + xx + xx) * acc;
                            // acc * (2 * y)
                            acc *= points[i + 1].x;
                            continue;
                        } else {
                            // Zero
                            points[i] = Self::identity();
                            points[i + 1] = Self::identity();
                            continue;
                        }
                    }
                }

                // (x_2 + x_1)
                points[out_idx].x = points[i].x + points[i + 1].x;
                // (x_2 - x_1)
                points[i + 1].x -= points[i].x;
                // (y2 - y1) * acc
                points[i + 1].y = (points[i + 1].y - points[i].y) * acc;
                // acc * (x_2 - x_1)
                acc *= points[i + 1].x;
            }

            // Batch invert
            if COMPLETE {
                if (!acc.is_zero()).into() {
                    acc = acc.invert().unwrap();
                }
            } else {
                acc = acc.invert().unwrap();
            }

            for i in (0..num_points).step_by(2).rev() {
                // Where that result of the point addition will be stored
                let out_idx = output_indices[i >> 1] as usize - offset;

                #[cfg(all(feature = "prefetch", target_arch = "x86_64"))]
                if i > 0 {
                    crate::prefetch::<Self>(points, output_indices[(i >> 1) - 1] as usize - offset);
                }

                if COMPLETE {
                    // points[i] is zero so the sum is points[i + 1]
                    if points[i].is_identity().into() {
                        points[out_idx] = points[i + 1];
                        continue;
                    }
                    // points[i + 1] is zero so the sum is points[i]
                    if points[i + 1].is_identity().into() {
                        points[out_idx] = points[i];
                        continue;
                    }
                }

                // lambda
                points[i + 1].y *= acc;
                // acc * (x_2 - x_1)
                acc *= points[i + 1].x;
                // x_3 = lambda^2 - (x_2 + x_1)
                points[out_idx].x = points[i + 1].y.square() - points[out_idx].x;
                // y_3 = lambda * (x_1 - x_3) - y_1
                points[out_idx].y =
                    points[i + 1].y * (points[i].x - points[out_idx].x) - points[i].y;
            }
        }
    };
}

#[macro_export]
macro_rules! new_curve_impl {
    (($($privacy:tt)*),
    $name:ident,
    $name_affine:ident,
    $name_compressed:ident,
    $base:ident,
    $scalar:ident,
    $generator:expr,
    $constant_b:expr,
    $curve_id:literal,
    ) => {

        #[derive(Copy, Clone, Debug)]
        $($privacy)* struct $name {
            pub x: $base,
            pub y: $base,
            pub z: $base,
        }

        #[derive(Copy, Clone)]
        $($privacy)* struct $name_affine {
            pub x: $base,
            pub y: $base,
        }

        #[derive(Copy, Clone)]
        $($privacy)* struct $name_compressed([u8; $base::size()]);


        impl $name {
            pub fn generator() -> Self {
                let generator = $name_affine::generator();
                Self {
                    x: generator.x,
                    y: generator.y,
                    z: $base::one(),
                }
            }

            const fn curve_constant_b() -> $base {
                $name_affine::curve_constant_b()
            }
        }

        impl $name_affine {
            pub fn generator() -> Self {
                Self {
                    x: $generator.0,
                    y: $generator.1,
                }
            }

            const fn curve_constant_b() -> $base {
                $constant_b
            }

            pub fn random(mut rng: impl RngCore) -> Self {
                loop {
                    let x = $base::random(&mut rng);
                    let ysign = (rng.next_u32() % 2) as u8;

                    let x3 = x.square() * x;
                    let y = (x3 + $name::curve_constant_b()).sqrt();
                    if let Some(y) = Option::<$base>::from(y) {
                        let sign = y.to_bytes()[0] & 1;
                        let y = if ysign ^ sign == 0 { y } else { -y };

                        let p = $name_affine {
                            x,
                            y,
                        };


                        use crate::group::cofactor::CofactorGroup;
                        let p = p.to_curve();
                        return p.clear_cofactor().to_affine()
                    }
                }
            }
        }

        // Compressed

        impl std::fmt::Debug for $name_compressed {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0[..].fmt(f)
            }
        }

        impl Default for $name_compressed {
            fn default() -> Self {
                $name_compressed([0; $base::size()])
            }
        }

        impl AsRef<[u8]> for $name_compressed {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl AsMut<[u8]> for $name_compressed {
            fn as_mut(&mut self) -> &mut [u8] {
                &mut self.0
            }
        }


        // Jacobian implementations

        impl<'a> From<&'a $name_affine> for $name {
            fn from(p: &'a $name_affine) -> $name {
                p.to_curve()
            }
        }

        impl From<$name_affine> for $name {
            fn from(p: $name_affine) -> $name {
                p.to_curve()
            }
        }

        impl Default for $name {
            fn default() -> $name {
                $name::identity()
            }
        }

        impl subtle::ConstantTimeEq for $name {
            fn ct_eq(&self, other: &Self) -> Choice {
                // Is (xz^2, yz^3, z) equal to (x'z'^2, yz'^3, z') when converted to affine?

                let z = other.z.square();
                let x1 = self.x * z;
                let z = z * other.z;
                let y1 = self.y * z;
                let z = self.z.square();
                let x2 = other.x * z;
                let z = z * self.z;
                let y2 = other.y * z;

                let self_is_zero = self.is_identity();
                let other_is_zero = other.is_identity();

                (self_is_zero & other_is_zero) // Both point at infinity
                            | ((!self_is_zero) & (!other_is_zero) & x1.ct_eq(&x2) & y1.ct_eq(&y2))
                // Neither point at infinity, coordinates are the same
            }

        }

        impl subtle::ConditionallySelectable for $name {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                $name {
                    x: $base::conditional_select(&a.x, &b.x, choice),
                    y: $base::conditional_select(&a.y, &b.y, choice),
                    z: $base::conditional_select(&a.z, &b.z, choice),
                }
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.ct_eq(other).into()
            }
        }

        impl cmp::Eq for $name {}

        impl CurveExt for $name {

            type ScalarExt = $scalar;
            type Base = $base;
            type AffineExt = $name_affine;

            const CURVE_ID: &'static str = $curve_id;

            fn endo(&self) -> Self {
                self.endomorphism_base()
            }

            fn jacobian_coordinates(&self) -> ($base, $base, $base) {
               (self.x, self.y, self.z)
            }


            fn hash_to_curve<'a>(domain_prefix: &'a str) -> Box<dyn Fn(&[u8]) -> Self + 'a> {
                use super::hashtocurve;

                Box::new(move |message| {
                    let mut us = [$base::zero(); 2];
                    hashtocurve::hash_to_field($name::CURVE_ID, domain_prefix, message, &mut us);
                    let q0 = hashtocurve::map_to_curve_simple_swu::<$base, $name, $iso>(
                        &us[0],
                        $name::THETA,
                        $name::Z,
                    );
                    let q1 = hashtocurve::map_to_curve_simple_swu::<$base, $name, $iso>(
                        &us[1],
                        $name::THETA,
                        $name::Z,
                    );
                    let r = q0 + &q1;
                    debug_assert!(bool::from(r.is_on_curve()));
                    hashtocurve::iso_map::<$base, $name, $iso>(&r, &$name::ISOGENY_CONSTANTS)
                })
            }

            fn is_on_curve(&self) -> Choice {

                let z2 = self.z.square();
                let z4 = z2.square();
                let z6 = z4 * z2;
                (self.y.square() - self.x.square() * self.x)
                    .ct_eq(&(z6 * $name::curve_constant_b()))
                    | self.z.is_zero()
            }

            fn b() -> Self::Base {
                $name::curve_constant_b()
            }

            fn a() -> Self::Base {
                Self::Base::zero()
            }

            fn new_jacobian(x: Self::Base, y: Self::Base, z: Self::Base) -> CtOption<Self> {
                let p = $name { x, y, z };
                CtOption::new(p, p.is_on_curve())
            }
        }

        impl group::Curve for $name {

            type AffineRepr = $name_affine;

            fn batch_normalize(p: &[Self], q: &mut [Self::AffineRepr]) {
                assert_eq!(p.len(), q.len());

                let mut acc = $base::one();
                for (p, q) in p.iter().zip(q.iter_mut()) {
                    // We use the `x` field of $name_affine to store the product
                    // of previous z-coordinates seen.
                    q.x = acc;

                    // We will end up skipping all identities in p
                    acc = $base::conditional_select(&(acc * p.z), &acc, p.is_identity());
                }

                // This is the inverse, as all z-coordinates are nonzero and the ones
                // that are not are skipped.
                acc = acc.invert().unwrap();

                for (p, q) in p.iter().rev().zip(q.iter_mut().rev()) {
                    let skip = p.is_identity();

                    // Compute tmp = 1/z
                    let tmp = q.x * acc;

                    // Cancel out z-coordinate in denominator of `acc`
                    acc = $base::conditional_select(&(acc * p.z), &acc, skip);

                    // Set the coordinates to the correct value
                    let tmp2 = tmp.square();
                    let tmp3 = tmp2 * tmp;

                    q.x = p.x * tmp2;
                    q.y = p.y * tmp3;

                    *q = $name_affine::conditional_select(&q, &$name_affine::identity(), skip);
                }
            }

            fn to_affine(&self) -> Self::AffineRepr {
                let zinv = self.z.invert().unwrap_or($base::zero());
                let zinv2 = zinv.square();
                let x = self.x * zinv2;
                let zinv3 = zinv2 * zinv;
                let y = self.y * zinv3;

                let tmp = $name_affine {
                    x,
                    y,
                };

                $name_affine::conditional_select(&tmp, &$name_affine::identity(), zinv.is_zero())
            }
        }

        impl group::Group for $name {
            type Scalar = $scalar;

            fn random(mut rng: impl RngCore) -> Self {
                $name_affine::random(&mut rng).to_curve()
            }

            fn double(&self) -> Self {
                let a = self.x.square();
                let b = self.y.square();
                let c = b.square();
                let d = self.x + b;
                let d = d.square();
                let d = d - a - c;
                let d = d + d;
                let e = a + a + a;
                let f = e.square();
                let z3 = self.z * self.y;
                let z3 = z3 + z3;
                let x3 = f - (d + d);
                let c = c + c;
                let c = c + c;
                let c = c + c;
                let y3 = e * (d - x3) - c;

                let tmp = $name {
                    x: x3,
                    y: y3,
                    z: z3,
                };

                $name::conditional_select(&tmp, &$name::identity(), self.is_identity())
            }

            fn generator() -> Self {
                $name::generator()
            }

            fn identity() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                    z: $base::zero(),
                }
            }

            fn is_identity(&self) -> Choice {
                self.z.is_zero()
            }
        }

        impl GroupEncoding for $name {
            type Repr = $name_compressed;

            fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
                $name_affine::from_bytes(bytes).map(Self::from)
            }

            fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
                $name_affine::from_bytes(bytes).map(Self::from)
            }

            fn to_bytes(&self) -> Self::Repr {
                $name_affine::from(self).to_bytes()
            }
        }


        impl group::prime::PrimeGroup for $name {}

        impl group::prime::PrimeCurve for $name {
            type Affine = $name_affine;
        }

        impl group::cofactor::CofactorCurve for $name {
            type Affine = $name_affine;
        }

        impl Group for $name {
            type Scalar = $scalar;

            fn group_zero() -> Self {
                Self::identity()
            }
            fn group_add(&mut self, rhs: &Self) {
                *self += *rhs;
            }
            fn group_sub(&mut self, rhs: &Self) {
                *self -= *rhs;
            }
            fn group_scale(&mut self, by: &Self::Scalar) {
                *self *= *by;
            }
        }

        // Affine implementations

        impl std::fmt::Debug for $name_affine {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                if self.is_identity().into() {
                    write!(f, "Infinity")
                } else {
                    write!(f, "({:?}, {:?})", self.x, self.y)
                }
            }
        }

        impl<'a> From<&'a $name> for $name_affine {
            fn from(p: &'a $name) -> $name_affine {
                p.to_affine()
            }
        }

        impl From<$name> for $name_affine {
            fn from(p: $name) -> $name_affine {
                p.to_affine()
            }
        }

        impl Default for $name_affine {
            fn default() -> $name_affine {
                $name_affine::identity()
            }
        }

        impl subtle::ConstantTimeEq for $name_affine {
            fn ct_eq(&self, other: &Self) -> Choice {
                let z1 = self.is_identity();
                let z2 = other.is_identity();

                (z1 & z2) | ((!z1) & (!z2) & (self.x.ct_eq(&other.x)) & (self.y.ct_eq(&other.y)))
            }
        }

        impl subtle::ConditionallySelectable for $name_affine {
            fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                $name_affine {
                    x: $base::conditional_select(&a.x, &b.x, choice),
                    y: $base::conditional_select(&a.y, &b.y, choice),
                }
            }
        }

        impl PartialEq for $name_affine {
            fn eq(&self, other: &Self) -> bool {
                self.ct_eq(other).into()
            }
        }

        impl cmp::Eq for $name_affine {}

        impl group::GroupEncoding for $name_affine {
            type Repr = $name_compressed;

            fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
                let bytes = &bytes.0;
                let mut tmp = *bytes;
                let ysign = Choice::from(tmp[$base::size() - 1] >> 7);
                tmp[$base::size() - 1] &= 0b0111_1111;

                $base::from_bytes(&tmp).and_then(|x| {
                    CtOption::new(Self::identity(), x.is_zero() & (!ysign)).or_else(|| {
                        let x3 = x.square() * x;
                        (x3 + $name::curve_constant_b()).sqrt().and_then(|y| {
                            let sign = Choice::from(y.to_bytes()[0] & 1);

                            let y = $base::conditional_select(&y, &-y, ysign ^ sign);

                            CtOption::new(
                                $name_affine {
                                    x,
                                    y,
                                },
                                Choice::from(1u8),
                            )
                        })
                    })
                })
            }

            fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
                Self::from_bytes(bytes)
            }

            fn to_bytes(&self) -> Self::Repr {
                if bool::from(self.is_identity()) {
                    $name_compressed::default()
                } else {
                    let (x, y) = (self.x, self.y);
                    let sign = (y.to_bytes()[0] & 1) << 7;
                    let mut xbytes = x.to_bytes();
                    xbytes[$base::size() - 1] |= sign;
                    $name_compressed(xbytes)
                }
            }
        }

        impl group::prime::PrimeCurveAffine for $name_affine {
            type Curve = $name;
            type Scalar = $scalar;


            fn generator() -> Self {
                $name_affine::generator()
            }

            fn identity() -> Self {
                Self {
                    x: $base::zero(),
                    y: $base::zero(),
                }
            }

            fn is_identity(&self) -> Choice {
                self.x.is_zero() & self.y.is_zero()
            }

            fn to_curve(&self) -> Self::Curve {
                $name {
                    x: self.x,
                    y: self.y,
                    z: $base::conditional_select(&$base::one(), &$base::zero(), self.is_identity()),
                }
            }
        }

        impl group::cofactor::CofactorCurveAffine for $name_affine {
            type Curve = $name;
            type Scalar = $scalar;

            fn identity() -> Self {
                <Self as group::prime::PrimeCurveAffine>::identity()
            }

            fn generator() -> Self {
                <Self as group::prime::PrimeCurveAffine>::generator()
            }

            fn is_identity(&self) -> Choice {
                <Self as group::prime::PrimeCurveAffine>::is_identity(self)
            }

            fn to_curve(&self) -> Self::Curve {
                <Self as group::prime::PrimeCurveAffine>::to_curve(self)
            }
        }


        impl CurveAffine for $name_affine {
            type ScalarExt = $scalar;
            type Base = $base;
            type CurveExt = $name;

            fn is_on_curve(&self) -> Choice {
                // y^2 - x^3 - ax ?= b
                (self.y.square() - self.x.square() * self.x).ct_eq(&$name::curve_constant_b())
                    | self.is_identity()
            }

            fn coordinates(&self) -> CtOption<Coordinates<Self>> {
                Coordinates::from_xy( self.x, self.y )
            }

            fn from_xy(x: Self::Base, y: Self::Base) -> CtOption<Self> {
                let p = $name_affine {
                    x, y
                };
                CtOption::new(p, p.is_on_curve())
            }

            fn a() -> Self::Base {
                Self::Base::zero()
            }

            fn b() -> Self::Base {
                $name::curve_constant_b()
            }
        }


        impl_binops_additive!($name, $name);
        impl_binops_additive!($name, $name_affine);
        impl_binops_additive_specify_output!($name_affine, $name_affine, $name);
        impl_binops_additive_specify_output!($name_affine, $name, $name);
        impl_binops_multiplicative!($name, $scalar);
        impl_binops_multiplicative_mixed!($name_affine, $scalar, $name);

        impl<'a> Neg for &'a $name {
            type Output = $name;

            fn neg(self) -> $name {
                $name {
                    x: self.x,
                    y: -self.y,
                    z: self.z,
                }
            }
        }

        impl Neg for $name {
            type Output = $name;

            fn neg(self) -> $name {
                -&self
            }
        }

        impl<T> Sum<T> for $name
        where
            T: core::borrow::Borrow<$name>,
        {
            fn sum<I>(iter: I) -> Self
            where
                I: Iterator<Item = T>,
            {
                iter.fold(Self::identity(), |acc, item| acc + item.borrow())
            }
        }

        impl<'a, 'b> Add<&'a $name> for &'b $name {
            type Output = $name;

            fn add(self, rhs: &'a $name) -> $name {
                if bool::from(self.is_identity()) {
                    *rhs
                } else if bool::from(rhs.is_identity()) {
                    *self
                } else {
                    let z1z1 = self.z.square();
                    let z2z2 = rhs.z.square();
                    let u1 = self.x * z2z2;
                    let u2 = rhs.x * z1z1;
                    let s1 = self.y * z2z2 * rhs.z;
                    let s2 = rhs.y * z1z1 * self.z;

                    if u1 == u2 {
                        if s1 == s2 {
                            self.double()
                        } else {
                            $name::identity()
                        }
                    } else {
                        let h = u2 - u1;
                        let i = (h + h).square();
                        let j = h * i;
                        let r = s2 - s1;
                        let r = r + r;
                        let v = u1 * i;
                        let x3 = r.square() - j - v - v;
                        let s1 = s1 * j;
                        let s1 = s1 + s1;
                        let y3 = r * (v - x3) - s1;
                        let z3 = (self.z + rhs.z).square() - z1z1 - z2z2;
                        let z3 = z3 * h;

                        $name {
                            x: x3, y: y3, z: z3
                        }
                    }
                }
            }
        }

        impl<'a, 'b> Add<&'a $name_affine> for &'b $name {
            type Output = $name;

            fn add(self, rhs: &'a $name_affine) -> $name {
                if bool::from(self.is_identity()) {
                    rhs.to_curve()
                } else if bool::from(rhs.is_identity()) {
                    *self
                } else {
                    let z1z1 = self.z.square();
                    let u2 = rhs.x * z1z1;
                    let s2 = rhs.y * z1z1 * self.z;

                    if self.x == u2 {
                        if self.y == s2 {
                            self.double()
                        } else {
                            $name::identity()
                        }
                    } else {
                        let h = u2 - self.x;
                        let hh = h.square();
                        let i = hh + hh;
                        let i = i + i;
                        let j = h * i;
                        let r = s2 - self.y;
                        let r = r + r;
                        let v = self.x * i;
                        let x3 = r.square() - j - v - v;
                        let j = self.y * j;
                        let j = j + j;
                        let y3 = r * (v - x3) - j;
                        let z3 = (self.z + h).square() - z1z1 - hh;

                        $name {
                            x: x3, y: y3, z: z3
                        }
                    }
                }
            }
        }

        impl<'a, 'b> Sub<&'a $name> for &'b $name {
            type Output = $name;

            fn sub(self, other: &'a $name) -> $name {
                self + (-other)
            }
        }

        impl<'a, 'b> Sub<&'a $name_affine> for &'b $name {
            type Output = $name;

            fn sub(self, other: &'a $name_affine) -> $name {
                self + (-other)
            }
        }



        #[allow(clippy::suspicious_arithmetic_impl)]
        impl<'a, 'b> Mul<&'b $scalar> for &'a $name {
            type Output = $name;

            // This is a simple double-and-add implementation of point
            // multiplication, moving from most significant to least
            // significant bit of the scalar.

            fn mul(self, other: &'b $scalar) -> Self::Output {
                let mut acc = $name::identity();
                for bit in other
                    .to_repr()
                    .iter()
                    .rev()
                    .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
                {
                    acc = acc.double();
                    acc = $name::conditional_select(&acc, &(acc + self), bit);
                }

                acc
            }
        }

        impl<'a> Neg for &'a $name_affine {
            type Output = $name_affine;

            fn neg(self) -> $name_affine {
                $name_affine {
                    x: self.x,
                    y: -self.y,
                }
            }
        }

        impl Neg for $name_affine {
            type Output = $name_affine;

            fn neg(self) -> $name_affine {
                -&self
            }
        }

        impl<'a, 'b> Add<&'a $name> for &'b $name_affine {
            type Output = $name;

            fn add(self, rhs: &'a $name) -> $name {
                rhs + self
            }
        }

        impl<'a, 'b> Add<&'a $name_affine> for &'b $name_affine {
            type Output = $name;

            fn add(self, rhs: &'a $name_affine) -> $name {
                if bool::from(self.is_identity()) {
                    rhs.to_curve()
                } else if bool::from(rhs.is_identity()) {
                    self.to_curve()
                } else {
                    if self.x == rhs.x {
                        if self.y == rhs.y {
                            self.to_curve().double()
                        } else {
                            $name::identity()
                        }
                    } else {
                        let h = rhs.x - self.x;
                        let hh = h.square();
                        let i = hh + hh;
                        let i = i + i;
                        let j = h * i;
                        let r = rhs.y - self.y;
                        let r = r + r;
                        let v = self.x * i;
                        let x3 = r.square() - j - v - v;
                        let j = self.y * j;
                        let j = j + j;
                        let y3 = r * (v - x3) - j;
                        let z3 = h + h;

                        $name {
                            x: x3, y: y3, z: z3
                        }
                    }
                }
            }
        }

        impl<'a, 'b> Sub<&'a $name_affine> for &'b $name_affine {
            type Output = $name;

            fn sub(self, other: &'a $name_affine) -> $name {
                self + (-other)
            }
        }

        impl<'a, 'b> Sub<&'a $name> for &'b $name_affine {
            type Output = $name;

            fn sub(self, other: &'a $name) -> $name {
                self + (-other)
            }
        }

        #[allow(clippy::suspicious_arithmetic_impl)]
        impl<'a, 'b> Mul<&'b $scalar> for &'a $name_affine {
            type Output = $name;

            fn mul(self, other: &'b $scalar) -> Self::Output {
                let mut acc = $name::identity();

                // This is a simple double-and-add implementation of point
                // multiplication, moving from most significant to least
                // significant bit of the scalar.

                for bit in other
                    .to_repr()
                    .iter()
                    .rev()
                    .flat_map(|byte| (0..8).rev().map(move |i| Choice::from((byte >> i) & 1u8)))
                {
                    acc = acc.double();
                    acc = $name::conditional_select(&acc, &(acc + self), bit);
                }

                acc
            }
        }
    };
}
