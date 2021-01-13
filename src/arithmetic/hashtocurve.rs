//! This module implements "simplified SWU" hashing to short Weierstrass curves
//! with a = 0.

use byteorder::{BigEndian, WriteBytesExt};
use core::fmt::Debug;
use core::marker::PhantomData;
use subtle::ConstantTimeEq;

use super::{Curve, CurveAffine, FieldExt};

/// A method of hashing to an elliptic curve.
/// (If no isogeny is required, then C and I should be the same.)
///
/// This is intended to conform to the work-in-progress Internet Draft
/// [IRTF-CFRG-Hash-to-Curve](https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-10.html).
pub trait HashToCurve<F: FieldExt, I: CurveAffine<Base = F>, C: CurveAffine<Base = F>> {
    /// The MAP_ID of this method as specified in
    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-10.html#name-suite-id-naming-conventions-2>.
    fn map_id(&self) -> &str;

    /// A non-uniform map from a field element to the isogenous curve.
    fn map_to_curve(&self, u: &C::Base) -> I::Projective;

    /// The isogeny map from curve I to curve C.
    /// (If no isogeny is required, this should be the identity function.)
    fn iso_map(&self, p: &I::Projective) -> C::Projective;

    /// The random oracle map.
    fn field_elements_to_curve(&self, u0: &C::Base, u1: &C::Base) -> C::Projective;

    /// The full hash from an input message to a curve point.
    ///
    /// `domain_prefix` should identify the application protocol, usage
    /// within that protocol, and version, e.g. "z.cash:Orchard-V1".
    /// Other fields required to conform to [IRTF-CFRG-Hash-to-Curve]
    /// will be added automatically. There may be a length limitation on
    /// `domain_prefix`.
    ///
    /// For example, the resulting full domain separation tag for the
    /// Pallas curve using `Shake128` and the simplified SWU map might be
    /// b"z.cash:Orchard-V1-pallas_XOF:SHAKE128_SSWU_RO_".
    fn hash_to_curve(
        &self,
        domain_prefix: &str,
        hasher: impl MessageHasher<F> + 'static,
    ) -> Box<dyn Fn(&[u8]) -> C::Projective + '_> {
        let domain_separation_tag = format!(
            "{}-{}_{}_{}_RO_",
            domain_prefix,
            C::CURVE_ID,
            hasher.hash_name(),
            self.map_id()
        );

        Box::new(move |message| {
            let us = hasher.hash_to_field(message, domain_separation_tag.as_bytes(), 2);
            self.field_elements_to_curve(&us[0], &us[1])
        })
    }

    /// A non-uniform hash from an input message to a curve point.
    /// This is *not* suitable for applications requiring a random oracle.
    /// Use `hash_to_curve` instead unless you are really sure that a
    /// non-uniform map is sufficient.
    ///
    /// `domain_prefix` is as described for `hash_to_curve`.
    ///
    /// For example, the resulting full domain separation tag for the
    /// Pallas curve using `Shake128` and the simplified SWU map might be
    /// b"z.cash:Orchard-V1-pallas_XOF:SHAKE128_SSWU_NU_".
    fn encode_to_curve(
        &self,
        domain_prefix: &str,
        hasher: impl MessageHasher<F> + 'static,
    ) -> Box<dyn Fn(&[u8]) -> C::Projective + '_> {
        let domain_separation_tag = format!(
            "{}-{}_{}_{}_NU_",
            domain_prefix,
            C::CURVE_ID,
            hasher.hash_name(),
            self.map_id()
        );

        Box::new(move |message| {
            let us = hasher.hash_to_field(message, domain_separation_tag.as_bytes(), 1);
            let r = self.map_to_curve(&us[0]);
            self.iso_map(&r)
        })
    }
}

/// Method of hashing a message and domain_separation_tag to field elements.
pub trait MessageHasher<F: FieldExt> {
    /// The HASH_NAME of this message hasher as specified in
    /// <https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-10.html#name-suite-id-naming-conventions-2>.
    fn hash_name(&self) -> &str;

    /// Hash the given message and domain separation tag to give `count`
    /// field elements.
    fn hash_to_field(&self, message: &[u8], domain_separation_tag: &[u8], count: usize) -> Vec<F>;
}

/// A MessageHasher for SHAKE128
/// [FIPS202](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf).
/// It does not support domain separation tags longer than 128 bytes.
#[derive(Debug, Default)]
pub struct Shake128<F: FieldExt> {
    marker: PhantomData<F>,
}

impl<F: FieldExt> MessageHasher<F> for Shake128<F> {
    fn hash_name(&self) -> &str {
        "XOF:SHAKE128"
    }

    fn hash_to_field(&self, message: &[u8], domain_separation_tag: &[u8], count: usize) -> Vec<F> {
        use sha3::digest::{ExtendableOutput, Update};
        assert!(domain_separation_tag.len() < 256);

        // Assume that the field size is 32 bytes and k is 256, where k is defined in
        // <https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-10.html#name-security-considerations-3>.
        const CHUNKLEN: usize = 64;

        let outlen = count * CHUNKLEN;
        let mut outlen_enc = vec![];
        outlen_enc.write_u32::<BigEndian>(outlen as u32).unwrap();

        let mut xof = sha3::Shake128::default();
        xof.update(message);
        xof.update(outlen_enc);
        xof.update([domain_separation_tag.len() as u8]);
        xof.update(domain_separation_tag);

        xof.finalize_boxed(outlen)
            .chunks(CHUNKLEN)
            .map(|big| {
                let mut little = [0u8; CHUNKLEN];
                little.copy_from_slice(big);
                little.reverse();
                F::from_bytes_wide(&little)
            })
            .collect()
    }
}

/// A MessageHasher for BLAKE2b.
#[derive(Debug, Default)]
pub struct Blake2bXof<F: FieldExt> {
    marker: PhantomData<F>,
}

impl<F: FieldExt> MessageHasher<F> for Blake2bXof<F> {
    fn hash_name(&self) -> &str {
        "XOF:BLAKE2b"
    }

    #[allow(unused_variables)]
    fn hash_to_field(&self, message: &[u8], domain_separation_tag: &[u8], count: usize) -> Vec<F> {
        todo!()
    }
}

/// The simplified SWU hash-to-curve method, using an isogenous curve
/// y^2 = x^3 + a*x + b. This currently only supports prime-order curves.
#[derive(Debug)]
pub struct SimplifiedSWUWithDegree3Isogeny<
    F: FieldExt,
    I: CurveAffine<Base = F>,
    C: CurveAffine<Base = F>,
> {
    /// `Z` parameter (ξ in [WB2019]).
    pub z: F,

    /// Precomputed -b/a for the isogenous curve.
    pub minus_b_over_a: F,

    /// Precomputed b/Za for the isogenous curve.
    pub b_over_za: F,

    /// Precomputed sqrt(Z / ROOT_OF_UNITY).
    pub theta: F,

    /// Constants for the isogeny.
    pub isogeny_constants: [F; 13],

    marker_curve: PhantomData<C>,
    marker_iso: PhantomData<I>,
}

impl<F: FieldExt, I: CurveAffine<Base = F>, C: CurveAffine<Base = F>>
    SimplifiedSWUWithDegree3Isogeny<F, I, C>
{
    /// Create a SimplifiedSWUWithDegree3Isogeny method for the given parameters.
    ///
    /// # Panics
    /// Panics if z is square.
    pub fn new(z: &F, isogeny_constants: &[F; 13]) -> Self {
        let a = I::a();
        let b = I::b();

        SimplifiedSWUWithDegree3Isogeny {
            z: *z,
            minus_b_over_a: (-b) * &(a.invert().unwrap()),
            b_over_za: b * &((*z * a).invert().unwrap()),
            theta: (F::ROOT_OF_UNITY.invert().unwrap() * z).sqrt().unwrap(),
            isogeny_constants: *isogeny_constants,
            marker_curve: PhantomData,
            marker_iso: PhantomData,
        }
    }
}

impl<F: FieldExt, I: CurveAffine<Base = F>, C: CurveAffine<Base = F>> HashToCurve<F, I, C>
    for SimplifiedSWUWithDegree3Isogeny<F, I, C>
{
    fn map_id(&self) -> &str {
        "SSWU"
    }

    fn map_to_curve(&self, u: &F) -> I::Projective {
        // 1. tv1 = inv0(Z^2 * u^4 + Z * u^2)
        // 2. x1 = (-B / A) * (1 + tv1)
        // 3. If tv1 == 0, set x1 = B / (Z * A)
        // 4. gx1 = x1^3 + A * x1 + B
        //
        // We use the "Avoiding inversions" optimization in [WB2019, section 4.2]
        // (not to be confused with section 4.3):
        //
        //   here       [WB2019]
        //   -------    ---------------------------------
        //   Z          ξ
        //   u          t
        //   Z * u^2    ξ * t^2 (called u, confusingly)
        //   x1         X_0(t)
        //   x2         X_1(t)
        //   gx1        g(X_0(t))
        //   gx2        g(X_1(t))
        //
        // Using the "here" names:
        //    x1 = num_x1/div      = [B*(Z^2 * u^4 + Z * u^2 + 1)] / [-A*(Z^2 * u^4 + Z * u^2]
        //   gx1 = num_gx1/div_gx1 = [num_x1^3 + A * num_x1 * div^2 + B * div^3] / div^3

        let a = I::a();
        let b = I::b();
        let z_u2 = self.z * u.square();
        let ta = z_u2.square() + z_u2;
        let num_x1 = b * (ta + F::one());
        let div = -a * ta;
        let num2_x1 = num_x1.square();
        let div2 = div.square();
        let div3 = div2 * div;
        let ta_is_zero = ta.ct_is_zero();
        let num_gx1 = F::conditional_select(
            &((num2_x1 + a * div2) * num_x1 + b * div3),
            &self.b_over_za,
            ta_is_zero,
        );
        let div_gx1 = F::conditional_select(&div3, &F::one(), ta_is_zero);

        // 5. x2 = Z * u^2 * x1
        let num_x2 = z_u2 * num_x1; // same div

        // 6. gx2 = x2^3 + A * x2 + B  [optimized out; see below]
        // 7. If is_square(gx1), set x = x1 and y = sqrt(gx1)
        // 8. Else set x = x2 and y = sqrt(gx2)
        let (gx1_square, y1) = F::sqrt_ratio(&num_gx1, &div_gx1);

        // This magic also comes from a generalization of [WB2019, section 4.2].
        //
        // The Sarkar square root algorithm with input s gives us a square root of
        // ROOT_OF_UNITY * s for free when s is not square, where h is a fixed nonsquare.
        // We know that Z / ROOT_OF_UNITY is a square since both Z and ROOT_OF_UNITY are
        // nonsquares. Precompute theta as a square root of Z / ROOT_OF_UNITY.
        //
        // We have gx2 = g(Z * u^2 * x1) = Z^3 * u^6 * gx1
        //                               = (Z * u^3)^2 * (Z/h * h * gx1)
        //                               = (Z * theta * u^3)^2 * (h * gx1)
        //
        // When gx1 is not square, y1 is a square root of h * gx1, and so Z * theta * u^3 * y1
        // is a square root of gx2. Note that we don't actually need to compute gx2.

        let y2 = self.theta * z_u2 * u * y1;
        let num_x = F::conditional_select(&num_x2, &num_x1, gx1_square);
        let y = F::conditional_select(&y2, &y1, gx1_square);

        // 9. If sgn0(u) != sgn0(y), set y = -y
        let y = F::conditional_select(
            &(-y),
            &y,
            (u.get_lower_32() % 2).ct_eq(&(y.get_lower_32() % 2)),
        );

        I::Projective::new_jacobian(num_x * div, y * div3, div).unwrap()
    }

    /// Implements a degree 3 isogeny map.
    fn iso_map(&self, p: &I::Projective) -> C::Projective {
        // The input and output are in Jacobian coordinates, using the method
        // in "Avoiding inversions" [WB2019, section 4.3].

        let iso = self.isogeny_constants;
        let (x, y, z) = p.jacobian_coordinates();

        let z2 = z.square();
        let z3 = z2 * z;
        let z4 = z2.square();
        let z6 = z3.square();

        let num_x = ((iso[0] * x + iso[1] * z2) * x + iso[2] * z4) * x + iso[3] * z6;
        let div_x = (z2 * x + iso[4] * z4) * x + iso[5] * z6;

        let num_y = (((iso[6] * x + iso[7] * z2) * x + iso[8] * z4) * x + iso[9] * z6) * y;
        let div_y = (((x + iso[10] * z2) * x + iso[11] * z4) * x + iso[12] * z6) * z3;

        let zo = div_x * div_y;
        let xo = num_x * div_y * zo;
        let yo = num_y * div_x * zo.square();

        C::Projective::new_jacobian(xo, yo, zo).unwrap()
    }

    fn field_elements_to_curve(&self, u0: &C::Base, u1: &C::Base) -> C::Projective {
        let q0 = self.map_to_curve(u0);
        let q1 = self.map_to_curve(u1);
        let r: I::Projective = q0 + &q1;
        assert!(bool::from(r.is_on_curve()));
        // here is where we would scale by the cofactor if we supported nonprime-order curves
        self.iso_map(&r)
    }
}
