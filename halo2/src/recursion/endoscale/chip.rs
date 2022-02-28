use super::{
    primitive::{endoscale_scalar, i2lebsp},
    EndoscaleInstructions,
};
use ff::PrimeFieldBits;
use halo2_gadgets::{
    ecc::chip::{double_and_add, witness_point},
    utilities::{decompose_running_sum::be, UtilitiesInstructions},
};
use halo2_proofs::{
    arithmetic::CurveAffine,
    circuit::{AssignedCell, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Instance, Selector, TableColumn},
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

        let config = Self {
            q_lookup: meta.complex_selector(),
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
        };

        meta.enable_equality(config.endoscalars);
        meta.enable_equality(config.endoscalars_copy);
        meta.enable_equality(base.0);
        meta.enable_equality(base.1);

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
        mut _layouter: L,
        _base: C,
        _bitstring: &Self::Bitstring,
    ) -> Result<
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        Error,
    > {
        todo!()
    }

    fn endoscale_var_base<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        mut _layouter: L,
        _base: (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),

        _bitstring: &Self::Bitstring,
    ) -> Result<
        (
            AssignedCell<C::Base, C::Base>,
            AssignedCell<C::Base, C::Base>,
        ),
        Error,
    > {
        todo!()
    }

    fn endoscale_scalar<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        mut _layouter: L,
        _bitstring: &Self::Bitstring,
    ) -> Result<AssignedCell<C::Base, C::Base>, Error> {
        todo!()
    }

    fn recover_bitstring<L: Layouter<C::Base>, const NUM_BITS: usize, const NUM_WINDOWS: usize>(
        &self,
        _layouter: L,
        _bitstring: &Self::Bitstring,
        _pub_input_rows: [usize; NUM_WINDOWS],
    ) -> Result<(), Error> {
        todo!()
    }
}
