use super::super::SinsemillaInstructions;
use super::{get_s_by_idx, CellValue, EccPoint, SinsemillaChip, Var};
use crate::{
    circuit::gadget::utilities::copy,
    primitives::sinsemilla::{self, lebs2ip_k, INV_TWO_POW_K},
};
use halo2::{
    circuit::{Chip, Region},
    plonk::Error,
};

use ff::{Field, PrimeFieldBits};
use group::Curve;
use pasta_curves::{
    arithmetic::{CurveAffine, FieldExt},
    pallas,
};

use std::ops::Deref;

impl SinsemillaChip {
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    pub(super) fn hash_message(
        &self,
        region: &mut Region<'_, pallas::Base>,
        Q: pallas::Affine,
        message: &<Self as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::Message,
    ) -> Result<(EccPoint, Vec<Vec<CellValue<pallas::Base>>>), Error> {
        let config = self.config().clone();
        let mut offset = 0;

        // Get the `x`- and `y`-coordinates of the starting `Q` base.
        let x_q = *Q.coordinates().unwrap().x();
        let y_q = *Q.coordinates().unwrap().y();

        // Initialize the accumulator to `Q`.
        let (mut x_a, mut y_a): (X<pallas::Base>, Y<pallas::Base>) = {
            // Constrain the initial x_q to equal the x-coordinate of the domain's `Q`.
            let fixed_x_q =
                region.assign_fixed(|| "fixed x_q", config.constants, offset, || Ok(x_q))?;
            let x_q_cell = region.assign_advice(|| "x_q", config.x_a, offset, || Ok(x_q))?;
            region.constrain_equal(&config.perm, fixed_x_q, x_q_cell)?;

            // This cell gets copied into itself by the first call to `hash_piece` below.
            let x_a = CellValue::new(x_q_cell, Some(x_q));

            // Constrain the initial x_a, lambda_1, lambda_2, x_p using the fixed y_q
            // initializer.
            region.assign_fixed(|| "fixed y_q", config.fixed_y_q, offset, || Ok(y_q))?;

            let y_a = Some(y_q);

            (x_a.into(), y_a.into())
        };

        let mut zs_sum: Vec<Vec<CellValue<pallas::Base>>> = Vec::new();

        // Hash each piece in the message except the final piece.
        for piece in message[0..(message.len() - 1)].iter() {
            // The value of the accumulator after this piece is processed.
            let (x, y, zs, _) = self.hash_piece(region, offset, piece, x_a, y_a)?;

            // Since each message word takes one row to process, we increase
            // the offset by `piece.num_words` on each iteration.
            offset += piece.num_words();

            // Update the accumulator to the latest value.
            x_a = x;
            y_a = y;
            zs_sum.push(zs);
        }

        // Hash the final message piece.
        let y_a = {
            let piece = &message[message.len() - 1];
            // The value of the accumulator after this piece is processed.
            let (x, y, mut zs, z_n) = self.hash_piece(region, offset, piece, x_a, y_a)?;

            // Since each message word takes one row to process, we increase
            // the offset by `piece.num_words` on each iteration.
            offset += piece.num_words();

            // Assign the final z_n
            let z_n = {
                let cell = region.assign_advice(
                    || "z_n",
                    config.bits,
                    offset,
                    || z_n.ok_or(Error::SynthesisError),
                )?;
                CellValue::new(cell, z_n)
            };

            // The last piece of a message will return the message's final `z_n`.
            zs.push(z_n);

            // Update the accumulator to the latest value.
            x_a = x;
            y_a = y;
            zs_sum.push(zs);

            // Assign and constrain the final `y_a`.
            region.assign_fixed(
                || "qs_2 = 2 on final row",
                config.q_sinsemilla2,
                offset - 1,
                || Ok(pallas::Base::from_u64(2)),
            )?;

            let y_a_cell = region.assign_advice(
                || "y_a",
                config.lambda_1,
                offset,
                || y_a.ok_or(Error::SynthesisError),
            )?;

            // Assign lambda_2 and x_p zero values since they are queried
            // in the gate.
            {
                region.assign_advice(
                    || "dummy lambda2",
                    config.lambda_2,
                    offset,
                    || Ok(pallas::Base::zero()),
                )?;
                region.assign_advice(
                    || "dummy x_p",
                    config.x_p,
                    offset,
                    || Ok(pallas::Base::zero()),
                )?;
            }

            CellValue::new(y_a_cell, y_a.0)
        };

        #[cfg(test)]
        #[allow(non_snake_case)]
        // Check equivalence to result from primitives::sinsemilla::hash_to_point
        {
            use crate::circuit::gadget::sinsemilla::message::MessagePiece;
            use crate::primitives::sinsemilla::{K, S_PERSONALIZATION};
            use group::prime::PrimeCurveAffine;
            use pasta_curves::arithmetic::CurveExt;

            let field_elems: Option<Vec<pallas::Base>> =
                message.iter().map(|piece| piece.field_elem()).collect();

            if field_elems.is_some() {
                // Get message as a bitstring.
                let bitstring: Vec<bool> = message
                    .iter()
                    .map(|piece: &MessagePiece<pallas::Base, K>| {
                        piece
                            .field_elem()
                            .unwrap()
                            .to_le_bits()
                            .into_iter()
                            .take(K * piece.num_words())
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect();

                let hasher_S = pallas::Point::hash_to_curve(S_PERSONALIZATION);
                let S = |chunk: &[bool]| hasher_S(&lebs2ip_k(chunk).to_le_bytes());

                let expected_point = bitstring
                    .chunks(K)
                    .fold(Q.to_curve(), |acc, chunk| (acc + S(chunk)) + acc);
                let actual_point =
                    pallas::Affine::from_xy(x_a.value().unwrap(), y_a.value().unwrap()).unwrap();
                assert_eq!(expected_point.to_affine(), actual_point);
            }
        }

        Ok((EccPoint::from_coordinates_unchecked(x_a.0, y_a), zs_sum))
    }

    #[allow(clippy::type_complexity)]
    // Hash a message piece containing `piece.length` number of `K`-bit words.
    fn hash_piece(
        &self,
        region: &mut Region<'_, pallas::Base>,
        offset: usize,
        piece: &<Self as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::MessagePiece,
        x_a: X<pallas::Base>,
        y_a: Y<pallas::Base>,
    ) -> Result<
        (
            X<pallas::Base>,
            Y<pallas::Base>,
            Vec<CellValue<pallas::Base>>,
            Option<pallas::Base>,
        ),
        Error,
    > {
        let config = self.config().clone();

        // Selector assignments
        {
            // Enable `q_sinsemilla1` selector on every row.
            for row in 0..piece.num_words() {
                config.q_sinsemilla1.enable(region, offset + row)?;
            }

            // Set `q_sinsemilla2` fixed column to 1 on every row but the last.
            for row in 0..(piece.num_words() - 1) {
                region.assign_fixed(
                    || "q_s2 = 1",
                    config.q_sinsemilla2,
                    offset + row,
                    || Ok(pallas::Base::one()),
                )?;
            }

            // Set `q_sinsemilla2` fixed column to 0 on the last row.
            region.assign_fixed(
                || "q_s2 = 1",
                config.q_sinsemilla2,
                offset + piece.num_words() - 1,
                || Ok(pallas::Base::zero()),
            )?;
        }

        // Message piece as K * piece.length bitstring
        let bitstring: Option<Vec<bool>> = piece.field_elem().map(|value| {
            value
                .to_le_bits()
                .into_iter()
                .take(sinsemilla::K * piece.num_words())
                .collect()
        });

        let words: Option<Vec<u32>> = bitstring.map(|bitstring| {
            bitstring
                .chunks_exact(sinsemilla::K)
                .map(|word| lebs2ip_k(word))
                .collect()
        });

        // Get (x_p, y_p) for each word. We precompute this here so that we can use `batch_normalize()`.
        let generators_projective: Option<Vec<pallas::Point>> = words
            .clone()
            .map(|words| words.iter().map(|word| get_s_by_idx(*word)).collect());
        let generators: Option<Vec<(pallas::Base, pallas::Base)>> =
            generators_projective.map(|generators_projective| {
                let mut generators = vec![pallas::Affine::default(); generators_projective.len()];
                pallas::Point::batch_normalize(&generators_projective, &mut generators);
                generators
                    .iter()
                    .map(|gen| {
                        let point = gen.coordinates().unwrap();
                        (*point.x(), *point.y())
                    })
                    .collect()
            });

        // Convert `words` from `Option<Vec<u32>>` to `Vec<Option<u32>>`
        let words: Vec<Option<u32>> = if let Some(words) = words {
            words.into_iter().map(Some).collect()
        } else {
            vec![None; piece.num_words()]
        };

        // Decompose message into `K`-bit pieces with a running sum `z`.
        let (zs, z_n) = {
            let mut zs = Vec::with_capacity(piece.num_words() + 1);

            // Copy message and initialize running sum `z` to decompose message in-circuit
            let cell = region.assign_advice(
                || "z_0 (copy of message)",
                config.bits,
                offset,
                || piece.field_elem().ok_or(Error::SynthesisError),
            )?;
            region.constrain_equal(&config.perm, piece.cell(), cell)?;
            zs.push(CellValue::new(cell, piece.field_elem()));

            // Assign cumulative sum such that
            //          z_i = 2^K * z_{i + 1} + m_{i + 1}
            // => z_{i + 1} = (z_i - m_{i + 1}) / 2^K
            //
            // For a message m = m_1 + 2^K m_2 + ... + 2^{K(n-1)} m_n}, initialize z_0 = m.
            // We end up with z_n = 0.
            let mut z = piece.field_elem();
            let inv_2_k = pallas::Base::from_bytes(&INV_TWO_POW_K).unwrap();

            // We do not assign the final z_n.
            for (idx, word) in words[0..(words.len() - 1)].iter().enumerate() {
                // z_{i + 1} = (z_i - m_{i + 1}) / 2^K
                z = z
                    .zip(*word)
                    .map(|(z, word)| (z - pallas::Base::from_u64(word as u64)) * inv_2_k);
                let cell = region.assign_advice(
                    || format!("z_{:?}", idx + 1),
                    config.bits,
                    offset + idx + 1,
                    || z.ok_or(Error::SynthesisError),
                )?;
                zs.push(CellValue::new(cell, z))
            }

            let z_n = {
                let word = words[words.len() - 1];
                z.zip(word)
                    .map(|(z, word)| (z - pallas::Base::from_u64(word as u64)) * inv_2_k)
            };

            (zs, z_n)
        };

        // Copy in the accumulator x-coordinate
        let mut x_a: X<pallas::Base> = copy(
            region,
            || "Initialize accumulator x-coordinate",
            config.x_a,
            offset,
            &x_a.0,
            &config.perm,
        )?
        .into();

        let mut y_a = y_a;

        let generators: Vec<Option<(pallas::Base, pallas::Base)>> =
            if let Some(generators) = generators {
                generators.into_iter().map(Some).collect()
            } else {
                vec![None; piece.num_words()]
            };

        for (row, gen) in generators.iter().enumerate() {
            let x_p = gen.map(|gen| gen.0);
            let y_p = gen.map(|gen| gen.1);

            // Assign `x_p`
            region.assign_advice(
                || "x_p",
                config.x_p,
                offset + row,
                || x_p.ok_or(Error::SynthesisError),
            )?;

            // Compute and assign `lambda_1`
            let lambda_1 = {
                let lambda_1 = x_a
                    .value()
                    .zip(y_a.0)
                    .zip(x_p)
                    .zip(y_p)
                    .map(|(((x_a, y_a), x_p), y_p)| (y_a - y_p) * (x_a - x_p).invert().unwrap());

                // Assign lambda_1
                region.assign_advice(
                    || "lambda_1",
                    config.lambda_1,
                    offset + row,
                    || lambda_1.ok_or(Error::SynthesisError),
                )?;

                lambda_1
            };

            // Compute `x_r`
            let x_r = lambda_1
                .zip(x_a.value())
                .zip(x_p)
                .map(|((lambda_1, x_a), x_p)| lambda_1.square() - x_a - x_p);

            // Compute and assign `lambda_2`
            let lambda_2 = {
                let lambda_2 = x_a.value().zip(y_a.0).zip(x_r).zip(lambda_1).map(
                    |(((x_a, y_a), x_r), lambda_1)| {
                        pallas::Base::from_u64(2) * y_a * (x_a - x_r).invert().unwrap() - lambda_1
                    },
                );

                region.assign_advice(
                    || "lambda_2",
                    config.lambda_2,
                    offset + row,
                    || lambda_2.ok_or(Error::SynthesisError),
                )?;

                lambda_2
            };

            // Compute and assign `x_a` for the next row.
            let x_a_new: X<pallas::Base> = {
                let x_a_new = lambda_2
                    .zip(x_a.value())
                    .zip(x_r)
                    .map(|((lambda_2, x_a), x_r)| lambda_2 * lambda_2 - x_a - x_r);

                let x_a_cell = region.assign_advice(
                    || "x_a",
                    config.x_a,
                    offset + row + 1,
                    || x_a_new.ok_or(Error::SynthesisError),
                )?;

                CellValue::new(x_a_cell, x_a_new).into()
            };

            // Compute y_a for the next row.
            let y_a_new: Y<pallas::Base> = lambda_2
                .zip(x_a.value())
                .zip(x_a_new.value())
                .zip(y_a.0)
                .map(|(((lambda_2, x_a), x_a_new), y_a)| lambda_2 * (x_a - x_a_new) - y_a)
                .into();

            // Update the mutable `x_a`, `y_a` variables.
            x_a = x_a_new;
            y_a = y_a_new;
        }

        Ok((x_a, y_a, zs, z_n))
    }
}

// The x-coordinate of the accumulator in a Sinsemilla hash instance.
struct X<F: FieldExt>(CellValue<F>);

impl<F: FieldExt> From<CellValue<F>> for X<F> {
    fn from(cell_value: CellValue<F>) -> Self {
        X(cell_value)
    }
}

impl<F: FieldExt> Deref for X<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &CellValue<F> {
        &self.0
    }
}

// The y-coordinate of the accumulator in a Sinsemilla hash instance.
// This is never actually witnessed until the last round, since it
// can be derived from other variables. Thus it only exists as a field
// element, not a `CellValue`.
struct Y<F: FieldExt>(Option<F>);

impl<F: FieldExt> From<Option<F>> for Y<F> {
    fn from(value: Option<F>) -> Self {
        Y(value)
    }
}

impl<F: FieldExt> Deref for Y<F> {
    type Target = Option<F>;

    fn deref(&self) -> &Option<F> {
        &self.0
    }
}
