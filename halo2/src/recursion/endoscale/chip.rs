use super::{
    primitive::{endoscale_pair, endoscale_scalar, i2lebsp},
    EndoscaleInstructions,
};
use ff::{Field, PrimeFieldBits};
use group::Curve;
use halo2_gadgets::{
    ecc::chip::{double_and_add, witness_point, NonIdentityEccPoint},
    utilities::{
        bool_check,
        boolean::Bit,
        decompose_running_sum::be,
        lookup_range_check::{self, LookupRangeCheckConfig},
        UtilitiesInstructions,
    },
};
use halo2_proofs::{
    arithmetic::CurveAffine,
    circuit::{AssignedCell, Layouter, Region},
    plonk::{
        Advice, Column, ConstraintSystem, Error, Expression, Instance, Selector, TableColumn,
        VirtualCells,
    },
    poly::Rotation,
};
use pasta_curves::arithmetic::FieldExt;
use std::marker::PhantomData;

/// Configuration for endoscalar table.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TableConfig<F: FieldExt, const K: usize> {
    pub(in crate::recursion) bits: TableColumn,
    pub(in crate::recursion) endoscalar: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: FieldExt, const K: usize> TableConfig<F, K> {
    #[allow(dead_code)]
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        TableConfig {
            bits: meta.lookup_table_column(),
            endoscalar: meta.lookup_table_column(),
            _marker: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "endoscalar_map",
            |mut table| {
                for index in 0..(1 << K) {
                    table.assign_cell(|| "bits", self.bits, index, || Ok(F::from(index as u64)))?;
                    table.assign_cell(
                        || "endoscalar",
                        self.endoscalar,
                        index,
                        || {
                            Ok(endoscale_scalar(
                                Some(F::zero()),
                                &i2lebsp::<K>(index as u64),
                            ))
                        },
                    )?;
                }
                Ok(())
            },
        )
    }
}

/// Columns used in processing endoscalars.
#[derive(Clone, Debug)]
pub struct EndoscaleConfig<C: CurveAffine, const K: usize>
where
    C::Base: PrimeFieldBits,
{
    // Selector enabling a lookup in the (bitstring, endoscalar) table.
    q_lookup: Selector,
    // Selector handling lookups of partial chunks.
    q_partial_lookup: Selector,
    // Selector for Alg 2 endoscaling.
    q_endoscale_scalar: Selector,
    // Selector checking that partial chunks are correctly shifted.
    q_partial_chunk_shift: Selector,
    // Public inputs are provided as endoscalars. Each endoscalar corresponds
    // to a K-bit chunk.
    endoscalars: Column<Instance>,
    // An additional advice column where endoscalar values are copied and used
    // in the lookup argument.
    endoscalars_copy: Column<Advice>,
    // Accumulator used in committing to a word by the endoscaling algorithm.
    // (x, y) coordinates
    acc: (Column<Advice>, Column<Advice>),
    // Point that is added to the accumulator.
    point: (Column<Advice>, Column<Advice>),
    // Fixed that is used in endoscaling.
    base: (Column<Advice>, Column<Advice>),
    // Configuration for running sum decomposition into pairs of bits.
    running_sum_pairs: be::Config<C::Base, 2>,
    // Configuration for running sum decomposition into K-bit chunks.
    running_sum_chunks: be::Config<C::Base, K>,
    // Bits used in endoscaling. These are in (b_0, b_1) pairs.
    pair: (Column<Advice>, Column<Advice>),
    // Table mapping words to their corresponding endoscalars.
    pub(in crate::recursion) table: TableConfig<C::Base, K>,
    // Config used in double-and-add on the accumulator.
    dbl_and_add_config: double_and_add::Config<C>,
    // Config used in witnessing accumulator points.
    acc_point_config: witness_point::Config<C>,
    // Config used in witnessing endoscaled points.
    endo_point_config: witness_point::Config<C>,
    // Config used to range-check words not greater than K bits.
    lookup_range_check_config: lookup_range_check::LookupRangeCheckConfig<C::Base, K>,
}

impl<C: CurveAffine, const K: usize> UtilitiesInstructions<C::Base> for EndoscaleConfig<C, K>
where
    C::Base: PrimeFieldBits,
{
    type Var = AssignedCell<C::Base, C::Base>;
}

impl<C: CurveAffine, const K: usize> EndoscaleConfig<C, K>
where
    C::Base: PrimeFieldBits,
{
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        endoscalars: Column<Instance>,
        endoscalars_copy: Column<Advice>,
        acc: (Column<Advice>, Column<Advice>),
        point: (Column<Advice>, Column<Advice>),
        base: (Column<Advice>, Column<Advice>),
        running_sum: Column<Advice>,
        pair: (Column<Advice>, Column<Advice>),
        table: TableConfig<C::Base, K>,
    ) -> Self {
        let dbl_and_add_config = double_and_add::Config::configure(
            meta,
            point.0,
            point.1,
            endoscalars_copy,
            acc.0,
            acc.1,
        );
        let acc_point_config = witness_point::Config::configure(meta, acc.0, acc.1);
        let endo_point_config = witness_point::Config::configure(meta, point.0, point.1);
        let running_sum_pairs = be::Config::<C::Base, 2>::configure(meta, running_sum);
        let running_sum_chunks = be::Config::<C::Base, K>::configure(meta, running_sum);
        let lookup_range_check_config =
            LookupRangeCheckConfig::configure(meta, running_sum, table.bits);

        let config = Self {
            q_lookup: meta.complex_selector(),
            q_partial_lookup: meta.complex_selector(),
            q_endoscale_scalar: meta.selector(),
            q_partial_chunk_shift: meta.selector(),
            endoscalars,
            endoscalars_copy,
            acc,
            point,
            base,
            running_sum_pairs,
            running_sum_chunks,
            pair,
            table,
            dbl_and_add_config,
            acc_point_config,
            endo_point_config,
            lookup_range_check_config,
        };

        meta.enable_equality(config.endoscalars);
        meta.enable_equality(config.endoscalars_copy);
        meta.enable_equality(acc.0);
        meta.enable_equality(base.0);
        meta.enable_equality(base.1);

        /*
            The accumulator is initialised to [2](φ(P) + P) = (init_x, init_y).

            | b_0 | b_1 |   endo_x  |   endo_y   | acc_x  | acc_y  | P_x | P_y | <- column names
            --------------------------------------------------------------------
            | b_0 | b_1 | endo(P)_x |  endo(P)_y | init_x | init_y | P_x | P_y |

            (0, 0) -> (P_x, -P_y)
            (0, 1) -> (ζ * P_x, -P_y)
            (1, 0) -> (P_x, P_y)
            (1, 1) -> (ζ * P_x, P_y)
        */
        meta.create_gate("Endoscale base", |meta| {
            let q_endoscale_base = meta.query_selector(config.running_sum_pairs.q_range_check());

            // Pair of bits from the decomposition.
            let b_0 = meta.query_advice(config.pair.0, Rotation::cur());
            let b_1 = meta.query_advice(config.pair.1, Rotation::cur());

            // Boolean-constrain b_0, b_1
            let b_0_check = bool_check(b_0.clone());
            let b_1_check = bool_check(b_1.clone());

            // Check that `b_0, b_1` are consistent with the running sum decomposition.
            let decomposition_check = {
                let word = b_0.clone() + Expression::Constant(C::Base::from(2)) * b_1.clone();
                let expected_word = config.running_sum_pairs.window_expr()(meta);

                word - expected_word
            };

            // If the first bit is not set, check that endo_y = -P_y
            let y_check = {
                let endo_y = meta.query_advice(config.point.1, Rotation::cur());
                let p_y = meta.query_advice(config.base.1, Rotation::cur());
                let not_b0 = Expression::Constant(C::Base::one()) - b_0;
                not_b0 * (endo_y + p_y)
            };
            // If the second bit is set, check that endo_x = ζ * P_x
            let x_check = {
                let endo_x = meta.query_advice(config.point.0, Rotation::cur());
                let p_x = meta.query_advice(config.base.0, Rotation::cur());
                let zeta = Expression::Constant(C::Base::ZETA);
                b_1 * (endo_x - zeta * p_x)
            };

            std::array::IntoIter::new([
                ("b_0_check", b_0_check),
                ("b_1_check", b_1_check),
                ("decomposition_check", decomposition_check),
                ("x_check", x_check),
                ("y_check", y_check),
            ])
            .map(move |(name, poly)| (name, q_endoscale_base.clone() * poly))
        });

        let endoscale_scalar_acc_check = |meta: &mut VirtualCells<C::Base>| {
            let endo = meta.query_advice(config.endoscalars_copy, Rotation::cur());
            let acc = meta.query_advice(config.acc.0, Rotation::cur());
            let next_acc = meta.query_advice(config.acc.0, Rotation::next());

            // Check that next_acc = acc + endo * 2^{K/2}
            let expected_next_acc = acc + (endo * C::Base::from(1 << (K / 2)));
            next_acc - expected_next_acc
        };

        meta.create_gate("Endoscale scalar with lookup", |meta| {
            let q_endoscale_scalar = meta.query_selector(config.q_endoscale_scalar);
            let acc_check = endoscale_scalar_acc_check(meta);

            vec![q_endoscale_scalar * acc_check]
        });

        // Endoscalar lookup for `K`-bit chunk.
        meta.lookup(|meta| {
            let q_lookup = meta.query_selector(config.q_lookup);
            let neg_q_lookup = Expression::Constant(C::Base::one()) - q_lookup.clone();
            let word = config.running_sum_chunks.window_expr()(meta);
            let endo = meta.query_advice(config.endoscalars_copy, Rotation::cur());
            let default_endo = {
                let val = endoscale_scalar(Some(C::Base::zero()), &[false; K]);
                Expression::Constant(val)
            };

            vec![
                (q_lookup.clone() * word, table.bits),
                (
                    q_lookup * endo + neg_q_lookup * default_endo,
                    table.endoscalar,
                ),
            ]
        });

        // Endoscalar lookup for partial chunk (less than `K` bits).
        meta.lookup(|meta| {
            let q_partial_lookup = meta.query_selector(config.q_partial_lookup);
            let neg_q_partial_lookup =
                Expression::Constant(C::Base::one()) - q_partial_lookup.clone();
            let word = meta.query_advice(config.running_sum_chunks.z(), Rotation::cur());
            let endo = meta.query_advice(config.point.0, Rotation::cur());
            let default_endo = {
                let val = endoscale_scalar(Some(C::Base::zero()), &[false; K]);
                Expression::Constant(val)
            };

            vec![
                (q_partial_lookup.clone() * word, table.bits),
                (
                    q_partial_lookup * endo + neg_q_partial_lookup * default_endo,
                    table.endoscalar,
                ),
            ]
        });

        meta.create_gate("Endoscale scalar of partial chunk", |meta| {
            let q_partial_chunk_shift = meta.query_selector(config.q_partial_chunk_shift);

            let acc_check = endoscale_scalar_acc_check(meta);
            let shift_check = {
                let padded_endoscalar = meta.query_advice(config.point.0, Rotation::cur());
                let shift = meta.query_advice(config.point.1, Rotation::cur());
                let shifted_endoscalar =
                    meta.query_advice(config.endoscalars_copy, Rotation::cur());
                (padded_endoscalar - shift) - shifted_endoscalar
            };

            std::array::IntoIter::new([("acc_check", acc_check), ("shift_check", shift_check)])
                .map(move |(name, poly)| (name, q_partial_chunk_shift.clone() * poly))
        });

        config
    }
}

impl<C: CurveAffine, const K: usize> EndoscaleInstructions<C> for EndoscaleConfig<C, K>
where
    C::Base: PrimeFieldBits,
{
    type Bitstring = AssignedCell<C::Base, C::Base>;
    const MAX_BITSTRING_LENGTH: usize = 248;

    fn witness_bitstring(_bits: &[bool]) -> Vec<Self::Bitstring> {
        todo!()
    }

    #[allow(clippy::type_complexity)]
    fn endoscale_fixed_base<
        L: Layouter<C::Base>,
        const NUM_BITS: usize,
        const NUM_WINDOWS: usize,
    >(
        &self,
        mut layouter: L,
        base: C,
        bitstring: &Self::Bitstring,
    ) -> Result<
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        Error,
    > {
        // NUM_BITS must be an even number not greater than MAX_BITSTRING_LENGTH.
        assert!(NUM_BITS <= Self::MAX_BITSTRING_LENGTH);
        assert_eq!(NUM_BITS % 2, 0);

        layouter.assign_region(
            || "Commit to bitstring (fixed base)",
            |mut region| {
                let offset = 0;
                // The accumulator is initialised to [2](φ(P) + P) = (init_x, init_y).
                let acc = {
                    let acc = base.to_curve() + base * C::Scalar::ZETA;
                    self.acc_point_config.point_non_id_from_constant(
                        acc.to_affine(),
                        offset,
                        &mut region,
                    )?
                };

                self.endoscale_base_inner::<NUM_BITS, NUM_WINDOWS>(
                    &mut region,
                    offset,
                    acc,
                    Base::Fixed(base),
                    bitstring,
                )
            },
        )
    }

    #[allow(clippy::type_complexity)]
    fn endoscale_var_base<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        mut layouter: L,
        base: (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),

        bitstring: &Self::Bitstring,
    ) -> Result<
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        Error,
    > {
        // NUM_BITS must be an even number not greater than MAX_BITSTRING_LENGTH.
        assert!(NUM_BITS <= Self::MAX_BITSTRING_LENGTH);
        assert_eq!(NUM_BITS % 2, 0);

        layouter.assign_region(
            || "Commit to bitstring (fixed base)",
            |mut region| {
                let offset = 0;
                // The accumulator is initialised to [2](φ(P) + P) = (init_x, init_y).
                let acc = {
                    let base_val = base
                        .0
                        .value()
                        .zip(base.1.value())
                        .map(|(&x, &y)| C::from_xy(x, y).unwrap());
                    let acc = base_val.map(|base| base.to_curve() + base * C::Scalar::ZETA);
                    let acc = self.acc_point_config.point_non_id(
                        acc.map(|a| a.to_affine()),
                        offset,
                        &mut region,
                    )?;
                    // Copy-constrain witnessed (x, y) to original base.
                    region.constrain_equal(acc.x().cell(), base.0.cell())?;
                    region.constrain_equal(acc.y().cell(), base.1.cell())?;

                    acc
                };

                self.endoscale_base_inner::<NUM_BITS, NUM_WINDOWS>(
                    &mut region,
                    offset,
                    acc,
                    Base::Variable(base.clone()),
                    bitstring,
                )
            },
        )
    }

    fn endoscale_scalar<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        mut layouter: L,
        bitstring: &Self::Bitstring,
    ) -> Result<AssignedCell<C::Base, C::Base>, Error> {
        // NUM_BITS must be an even number not greater than MAX_BITSTRING_LENGTH.
        assert!(NUM_BITS <= Self::MAX_BITSTRING_LENGTH);
        assert_eq!(NUM_BITS % 2, 0);

        // The bitstring will be broken into K-bit chunks with a k_prime-bit
        // partial chunk.
        let k_prime = NUM_BITS % K;

        let (acc, bitstring) = layouter.assign_region(
            || "Endoscale scalar using bitstring (lookup optimisation)",
            |mut region| {
                let offset = 0;

                // The endoscalar is initialised to 2 * (ζ + 1).
                let mut acc = {
                    let init = (C::Base::ZETA + C::Base::one()).double();
                    region.assign_advice_from_constant(
                        || "initialise acc",
                        self.acc.0,
                        offset,
                        init,
                    )?
                };

                // Decompose the bitstring into `K`-bit chunks using a running sum.
                let bitstring = self
                    .running_sum_chunks
                    .copy_decompose::<NUM_BITS, NUM_WINDOWS>(
                        &mut region,
                        offset,
                        bitstring,
                        false,
                    )?;

                // For each chunk, lookup the (chunk, endoscalar) pair and add
                // it to the accumulator.
                for (idx, chunk) in bitstring.windows().iter().enumerate() {
                    self.q_lookup.enable(&mut region, offset + idx)?;
                    self.q_endoscale_scalar.enable(&mut region, offset + idx)?;

                    let endoscalar =
                        chunk.map(|c| endoscale_scalar(Some(C::Base::zero()), &c.bits()));
                    // Witness endoscalar.
                    region.assign_advice(
                        || format!("Endoscalar for chunk {}", NUM_WINDOWS - 1 - idx),
                        self.endoscalars_copy,
                        offset + idx,
                        || endoscalar.ok_or(Error::Synthesis),
                    )?;

                    // Bitshift the endoscalar by {K / 2} and add to accumulator.
                    let acc_val = acc
                        .value()
                        .zip(endoscalar)
                        .map(|(&acc, endo)| acc + endo * C::Base::from(1 << (K / 2)));
                    acc = region.assign_advice(
                        || format!("Endoscalar for chunk {}", NUM_WINDOWS - 1 - idx),
                        self.acc.0,
                        offset + idx + 1,
                        || acc_val.ok_or(Error::Synthesis),
                    )?;
                }

                Ok((acc, bitstring))
            },
        )?;

        // Handle the partial chunk.
        let acc = if k_prime == 0 {
            acc
        } else {
            let padded_endoscalar = self.partial_chunk_lookup(
                layouter.namespace(|| "partial chunk lookup"),
                bitstring.z(NUM_WINDOWS),
                k_prime,
            )?;

            layouter.assign_region(
                || "Add partial endo to acc",
                |mut region| {
                    let offset = 0;
                    self.q_partial_chunk_shift.enable(&mut region, offset)?;

                    // Copy padded_endoscalar to the correct offset.
                    padded_endoscalar.copy_advice(
                        || "padded endoscalar",
                        &mut region,
                        self.point.0,
                        offset,
                    )?;

                    // Assign 2^{K'/2} - 2^{K/2} from a fixed column.
                    let two_pow_k_prime_div2 = C::Base::from(1 << (k_prime / 2));
                    let two_pow_k_div2 = C::Base::from(1 << (K / 2));
                    let shift = two_pow_k_prime_div2 - two_pow_k_div2;
                    region.assign_advice_from_constant(
                        || "k_prime",
                        self.point.1,
                        offset,
                        shift,
                    )?;

                    // Subtract 2^{K'/2} - 2^{K/2} from the padded_endoscalar to
                    // recover the endoscalar corresponding to the unpadded
                    // partial chunk.
                    let shifted_endoscalar = padded_endoscalar.value().map(|&v| v - shift);
                    region.assign_advice(
                        || "shifted endoscalar",
                        self.endoscalars_copy,
                        offset,
                        || shifted_endoscalar.ok_or(Error::Synthesis),
                    )?;

                    // Copy the current accumulator to the correct offset.
                    acc.copy_advice(|| "current acc", &mut region, self.acc.0, offset)?;

                    // Bitshift the endoscalar by {K / 2} and add to accumulator.
                    let acc_val = acc
                        .value()
                        .zip(shifted_endoscalar)
                        .map(|(&acc, endo)| acc + endo * two_pow_k_div2);
                    println!("acc_val: {:?}", acc_val);

                    region.assign_advice(
                        || "Endoscalar for partial chunk",
                        self.acc.0,
                        offset + 1,
                        || acc_val.ok_or(Error::Synthesis),
                    )
                },
            )?
        };

        Ok(acc)
    }

    fn recover_bitstring<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        mut layouter: L,
        bitstring: &Self::Bitstring,
        pub_input_rows: [usize; NUM_WINDOWS],
    ) -> Result<(), Error> {
        // NUM_BITS must be an even number not greater than MAX_BITSTRING_LENGTH.
        assert!(NUM_BITS <= Self::MAX_BITSTRING_LENGTH);
        assert_eq!(NUM_BITS % 2, 0);

        // The bitstring will be broken into K-bit chunks with a k_prime-bit
        // partial chunk.
        let k_prime = NUM_BITS % K;

        let bitstring = layouter.assign_region(
            || "Recover bitstring from endoscalars",
            |mut region| {
                let offset = 0;

                // Decompose the bitstring into `K`-bit chunks using a running sum.
                // This internally enables the `q_range_check` selector, which is
                // used in the "Endoscale scalar with lookup" gate.
                let bitstring = self
                    .running_sum_chunks
                    .copy_decompose::<NUM_BITS, NUM_WINDOWS>(
                        &mut region,
                        offset,
                        bitstring,
                        false,
                    )?;

                // For each chunk, lookup the (chunk, endoscalar) pair.
                for (idx, (chunk, pub_input_row)) in bitstring
                    .windows()
                    .iter()
                    .zip(pub_input_rows.iter())
                    .enumerate()
                {
                    self.q_lookup.enable(&mut region, offset + idx)?;

                    let _computed_endoscalar =
                        chunk.map(|c| endoscale_scalar(Some(C::Base::zero()), &c.bits()));
                    // Copy endoscalar from given row on instance column
                    let _copied_endoscalar = region.assign_advice_from_instance(
                        || format!("Endoscalar at row {:?}", pub_input_row),
                        self.endoscalars,
                        *pub_input_row,
                        self.endoscalars_copy,
                        offset + idx,
                    )?;

                    #[cfg(test)]
                    {
                        if let Some(&copied) = _copied_endoscalar.value() {
                            if let Some(computed) = _computed_endoscalar {
                                assert_eq!(copied, computed);
                            }
                        }
                    }
                }

                Ok(bitstring)
            },
        )?;

        // Handle the partial chunk.
        if k_prime > 0 {
            self.partial_chunk_lookup(
                layouter.namespace(|| "partial chunk lookup"),
                bitstring.z(NUM_WINDOWS),
                k_prime,
            )?;
        }

        Ok(())
    }
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
enum Base<C: CurveAffine> {
    Fixed(C),
    Variable(
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
    ),
}

impl<C: CurveAffine, const K: usize> EndoscaleConfig<C, K>
where
    C::Base: PrimeFieldBits,
{
    #[allow(clippy::type_complexity)]
    fn endoscale_base_inner<const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        mut acc: NonIdentityEccPoint<C>,
        base: Base<C>,
        bitstring: &AssignedCell<C::Base, C::Base>,
    ) -> Result<
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        Error,
    > {
        // Decompose the bitstring into 2-bit windows using a running sum.
        // This internally enables the `q_range_check` selector, which is
        // used in the "Endoscale base" gate.
        let bitstring = self
            .running_sum_pairs
            .copy_decompose::<NUM_BITS, NUM_WINDOWS>(region, offset, bitstring, true)?;

        for (pair_idx, pair) in bitstring
            .windows()
            .iter()
            .map(|w| w.map(|w| w.bits()))
            .enumerate()
        {
            // Assign base
            match base {
                Base::Fixed(base) => {
                    // Assign base_x
                    region.assign_advice_from_constant(
                        || "base_x",
                        self.base.0,
                        offset + pair_idx,
                        *base.coordinates().unwrap().x(),
                    )?;

                    // Assign base_y
                    region.assign_advice_from_constant(
                        || "base_y",
                        self.base.1,
                        offset + pair_idx,
                        *base.coordinates().unwrap().y(),
                    )?;
                }
                Base::Variable((ref x, ref y)) => {
                    x.copy_advice(|| "base_x", region, self.base.0, offset + pair_idx)?;
                    y.copy_advice(|| "base_y", region, self.base.1, offset + pair_idx)?;
                }
            }

            // Assign b_0
            let b_0: Option<Bit> = pair.map(|pair| pair[0].into());
            region.assign_advice(
                || format!("pair_idx: {}, b_0", pair_idx),
                self.pair.0,
                offset + pair_idx,
                || b_0.ok_or(Error::Synthesis),
            )?;

            // Assign b_1
            let b_1: Option<Bit> = pair.map(|pair| pair[1].into());
            region.assign_advice(
                || format!("pair_idx: {}, b_1", pair_idx),
                self.pair.1,
                offset + pair_idx,
                || b_1.ok_or(Error::Synthesis),
            )?;

            // Assign endoscaled point
            let endo = {
                let base = base.clone();
                let base = match base {
                    Base::Fixed(base) => Some(base),
                    Base::Variable((x, y)) => x
                        .value()
                        .zip(y.value())
                        .map(|(&x, &y)| C::from_xy(x, y).unwrap()),
                };
                pair.zip(base)
                    .map(|(pair, base)| endoscale_pair::<C>(pair, base).unwrap())
            };
            let endo = self
                .endo_point_config
                .point_non_id(endo, offset + pair_idx, region)?;

            // Add endo to acc.
            acc = self
                .dbl_and_add_config
                .assign_region(&endo, &acc, offset + pair_idx, region)?;
        }

        Ok((acc.x(), acc.y()))
    }

    fn partial_chunk_lookup<L: Layouter<C::Base>>(
        &self,
        mut layouter: L,
        partial_chunk: &AssignedCell<C::Base, C::Base>,
        partial_chunk_num_bits: usize,
    ) -> Result<AssignedCell<C::Base, C::Base>, Error> {
        // `z_w` of the running sum is the value of the partial chunk.
        // Range-constrain the partial chunk to `k_prime` bits.
        self.lookup_range_check_config.copy_short_check(
            layouter.namespace(|| {
                format!(
                    "Check that partial_chunk is {} bits",
                    partial_chunk_num_bits
                )
            }),
            &partial_chunk,
            partial_chunk_num_bits,
        )?;

        layouter.assign_region(
            || "Endoscale partial chunk",
            |mut region| {
                let offset = 0;
                self.q_partial_lookup.enable(&mut region, offset)?;

                let partial_chunk = partial_chunk.copy_advice(
                    || "partial chunk",
                    &mut region,
                    self.running_sum_chunks.z(),
                    offset,
                )?;

                // Pad the partial chunk to `K` bits and lookup the corresponding
                // padded_endoscalar.
                let padded_endoscalar = {
                    let padded_bits = partial_chunk.value().map(|v| {
                        v.to_le_bits()
                            .iter()
                            .by_val()
                            .take(partial_chunk_num_bits)
                            .chain(std::iter::repeat(false))
                            .take(K)
                            .collect::<Vec<_>>()
                    });
                    let padded_endoscalar =
                        padded_bits.map(|b| endoscale_scalar(Some(C::Base::zero()), &b));
                    region.assign_advice(
                        || "padded endoscalar",
                        self.point.0,
                        offset,
                        || padded_endoscalar.ok_or(Error::Synthesis),
                    )?
                };

                Ok(padded_endoscalar)
            },
        )
    }
}
