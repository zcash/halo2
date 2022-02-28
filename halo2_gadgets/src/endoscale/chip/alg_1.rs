use ff::PrimeFieldBits;
use halo2_proofs::{
    arithmetic::CurveAffine,
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
};

use crate::{
    ecc::chip::{add_incomplete, double, NonIdentityEccPoint},
    utilities::{
        decompose_running_sum::{RunningSum, RunningSumConfig},
        double_and_add::DoubleAndAdd,
    },
};

pub(super) type Bitstring<F, const K: usize> = RunningSum<F, K>;

/// Config used in Algorithm 1 (endoscaling with a base).
#[derive(Clone, Debug)]
pub(super) struct Alg1Config<C: CurveAffine>
where
    C::Base: PrimeFieldBits,
{
    // Selector for endoscaling checks.
    q_endoscale_base: Selector,
    // Selector for the initial check in double-and-add.
    q_double_and_add_init: Selector,
    // Selector for stead-state double-and-add.
    q_double_and_add: Selector,
    // Selector for the final check in double-and-add.
    q_double_and_add_final: Selector,
    // Configuration used for steady-state double-and-add.
    double_and_add: DoubleAndAdd<C>,
    // Incomplete point doubling config
    double: double::Config<C>,
    // Incomplete point addition config
    add_incomplete: add_incomplete::Config<C>,
    // Bases used in endoscaling.
    base: (Column<Advice>, Column<Advice>),
    // Bits used in endoscaling. These are in (b_0, b_1) pairs.
    pair: (Column<Advice>, Column<Advice>),
    // Configuration for running sum decomposition into pairs of bits.
    running_sum_pairs: RunningSumConfig<C::Base, 2>,
}

impl<C: CurveAffine> Alg1Config<C>
where
    C::Base: PrimeFieldBits,
{
    pub(super) fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        pair: (Column<Advice>, Column<Advice>),
        base: (Column<Advice>, Column<Advice>),
        (x_a, x_p, lambda_1, lambda_2): (
            Column<Advice>,
            Column<Advice>,
            Column<Advice>,
            Column<Advice>,
        ),
        running_sum_pairs: RunningSumConfig<C::Base, 2>,
    ) -> Self {
        meta.enable_equality(base.0);
        meta.enable_equality(base.1);

        let q_endoscale_base = meta.selector();

        // Initial double-and-add gate
        let q_double_and_add_init = meta.selector();
        // Steady-state double-and-add gate
        let q_double_and_add = meta.complex_selector();
        // Final double-and-add gate
        let q_double_and_add_final = meta.complex_selector();

        let double_and_add = DoubleAndAdd::configure(
            meta,
            x_a,
            x_p,
            lambda_1,
            lambda_2,
            &|meta| {
                let q_double_and_add = meta.query_selector(q_double_and_add);
                let q_double_and_add_final = meta.query_selector(q_double_and_add_final);
                q_double_and_add + q_double_and_add_final
            },
            &|meta| meta.query_selector(q_double_and_add),
        );

        let advices = double_and_add.advices();
        let add_incomplete =
            add_incomplete::Config::configure(meta, advices[2], advices[3], advices[0], advices[1]);
        let double =
            double::Config::configure(meta, advices[0], advices[1], advices[2], advices[3]);

        meta.enable_equality(add_incomplete.x_p);
        meta.enable_equality(add_incomplete.y_p);

        meta.create_gate("init double-and-add", |meta| {
            // TODO
            let selector = meta.query_selector(q_double_and_add_init);

            vec![selector]
        });

        meta.create_gate("final double-and-add", |meta| {
            // TODO
            let selector = meta.query_selector(q_double_and_add_final);

            vec![selector]
        });

        Self {
            q_endoscale_base,
            q_double_and_add_init,
            q_double_and_add,
            q_double_and_add_final,
            double_and_add,
            double,
            add_incomplete,
            base,
            pair,
            running_sum_pairs,
        }
    }

    pub(super) fn endoscale_fixed_base(
        &self,
        mut _layouter: &mut impl Layouter<C::Base>,
        _bitstring: &RunningSum<C::Base, 2>,
        _bases: &C,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        todo!()
    }

    pub(super) fn endoscale_var_base(
        &self,
        mut _layouter: &mut impl Layouter<C::Base>,
        _bitstring: &RunningSum<C::Base, 2>,
        _bases: &NonIdentityEccPoint<C>,
    ) -> Result<NonIdentityEccPoint<C>, Error> {
        todo!()
    }
}
