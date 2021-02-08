use std::convert::TryInto;
use std::marker::PhantomData;

use super::{PoseidonInstructions, PoseidonSpec};
use crate::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, Permutation, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

const ARITY: usize = 3;

fn mds_array<F: FieldExt>(matrix: Vec<Vec<F>>) -> [[F; ARITY]; ARITY] {
    assert_eq!(matrix.len(), ARITY);
    let mut arr = matrix.into_iter().map(|v| v.try_into().unwrap());
    [
        arr.next().unwrap(),
        arr.next().unwrap(),
        arr.next().unwrap(),
    ]
}

/// Configuration for an [`Arity3Chip`].
#[derive(Clone, Debug)]
pub struct Arity3Config<F: FieldExt> {
    state: [Column<Advice>; ARITY],
    state_permutation: Permutation,
    partial_sbox: Column<Advice>,
    rc_a: [Column<Fixed>; ARITY],
    rc_b: [Column<Fixed>; ARITY],
    s_full: Column<Fixed>,
    s_partial: Column<Fixed>,
    s_final: Column<Fixed>,

    half_full_rounds: usize,
    half_partial_rounds: usize,
    alpha: [u64; 4],
    round_constants: Vec<[F; ARITY]>,
    m_reg: [[F; ARITY]; ARITY],
    m_inv: [[F; ARITY]; ARITY],
}

/// A Poseidon chip with an arity of 3, suitable for a 2:1 reduction.
#[derive(Debug)]
pub struct Arity3Chip<F: FieldExt> {
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Arity3Chip<F> {
    /// Configures this chip for use in a circuit.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        spec: impl PoseidonSpec<F>,
        state: [Column<Advice>; ARITY],
    ) -> Arity3Config<F> {
        // Generate constants for the Poseidon permutation.
        // This gadget requires R_F and R_P to be even.
        assert!(spec.full_rounds() & 1 == 0);
        assert!(spec.partial_rounds() & 1 == 0);
        let half_full_rounds = spec.full_rounds() / 2;
        let half_partial_rounds = spec.partial_rounds() / 2;
        let (round_constants, m_reg, m_inv) = {
            let (rc, m_reg, m_inv) = spec.constants();
            (
                rc.into_iter().map(|v| v.try_into().unwrap()).collect(),
                mds_array(m_reg),
                mds_array(m_inv),
            )
        };

        let state_permutation = Permutation::new(meta, &state);

        let partial_sbox = meta.advice_column();

        let rc_a = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        let rc_b = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];

        let s_full = meta.fixed_column();
        let s_partial = meta.fixed_column();
        let s_final = meta.fixed_column();

        let alpha = [5, 0, 0, 0];
        let pow_5 = |v: Expression<F>| {
            let v2 = v.clone() * v.clone();
            v2.clone() * v2 * v
        };

        let full_round = |meta: &mut ConstraintSystem<F>, next_idx: usize| {
            let cur_0 = meta.query_advice(state[0], Rotation::cur());
            let cur_1 = meta.query_advice(state[1], Rotation::cur());
            let cur_2 = meta.query_advice(state[2], Rotation::cur());
            let next_cell = meta.query_advice(state[next_idx], Rotation::next());

            let rc_0 = meta.query_fixed(rc_a[0], Rotation::cur());
            let rc_1 = meta.query_fixed(rc_a[1], Rotation::cur());
            let rc_2 = meta.query_fixed(rc_a[2], Rotation::cur());

            let s_full = meta.query_fixed(s_full, Rotation::cur());

            s_full
                * (pow_5(cur_0 + rc_0) * m_reg[next_idx][0]
                    + pow_5(cur_1 + rc_1) * m_reg[next_idx][1]
                    + pow_5(cur_2 + rc_2) * m_reg[next_idx][2]
                    - next_cell)
        };
        meta.create_gate("full round state_0", |meta| full_round(meta, 0));
        meta.create_gate("full round state_1", |meta| full_round(meta, 1));
        meta.create_gate("full round state_2", |meta| full_round(meta, 2));

        meta.create_gate("partial round state_0 part_a", |meta| {
            let cur_0 = meta.query_advice(state[0], Rotation::cur());
            let mid_0 = meta.query_advice(partial_sbox, Rotation::cur());
            let rc_a0 = meta.query_fixed(rc_a[0], Rotation::cur());
            let s_partial = meta.query_fixed(s_partial, Rotation::cur());

            s_partial * (pow_5(cur_0 + rc_a0) - mid_0)
        });

        meta.create_gate("partial round state_0 part_b", |meta| {
            let mid_0 = meta.query_advice(partial_sbox, Rotation::cur());
            let cur_1 = meta.query_advice(state[1], Rotation::cur());
            let cur_2 = meta.query_advice(state[2], Rotation::cur());
            let next_0 = meta.query_advice(state[0], Rotation::next());
            let next_1 = meta.query_advice(state[1], Rotation::next());
            let next_2 = meta.query_advice(state[2], Rotation::next());

            let rc_a1 = meta.query_fixed(rc_a[1], Rotation::cur());
            let rc_a2 = meta.query_fixed(rc_a[2], Rotation::cur());
            let rc_b0 = meta.query_fixed(rc_b[0], Rotation::cur());

            let s_partial = meta.query_fixed(s_partial, Rotation::cur());

            s_partial
                * (pow_5(
                    mid_0 * m_reg[0][0]
                        + (cur_1 + rc_a1) * m_reg[0][1]
                        + (cur_2 + rc_a2) * m_reg[0][2]
                        + rc_b0,
                ) - (next_0 * m_inv[0][0] + next_1 * m_inv[0][1] + next_2 * m_inv[0][2]))
        });

        let partial_round_linear = |meta: &mut ConstraintSystem<F>, idx: usize| {
            let mid_0 = meta.query_advice(partial_sbox, Rotation::cur());
            let cur_1 = meta.query_advice(state[1], Rotation::cur());
            let cur_2 = meta.query_advice(state[2], Rotation::cur());
            let next_0 = meta.query_advice(state[0], Rotation::next());
            let next_1 = meta.query_advice(state[1], Rotation::next());
            let next_2 = meta.query_advice(state[2], Rotation::next());

            let rc_a1 = meta.query_fixed(rc_a[1], Rotation::cur());
            let rc_a2 = meta.query_fixed(rc_a[2], Rotation::cur());
            let rc_b = meta.query_fixed(rc_b[idx], Rotation::cur());

            let s_partial = meta.query_fixed(s_partial, Rotation::cur());

            s_partial
                * (mid_0 * m_reg[idx][0]
                    + (cur_1 + rc_a1) * m_reg[idx][1]
                    + (cur_2 + rc_a2) * m_reg[idx][2]
                    + rc_b
                    - (next_0 * m_inv[idx][0] + next_1 * m_inv[idx][1] + next_2 * m_inv[idx][2]))
        };
        meta.create_gate("partial round state_1", |meta| {
            partial_round_linear(meta, 1)
        });
        meta.create_gate("partial round state_2", |meta| {
            partial_round_linear(meta, 2)
        });

        let final_full_round = |meta: &mut ConstraintSystem<F>, idx: usize| {
            let cur = meta.query_advice(state[idx], Rotation::cur());
            let next = meta.query_advice(state[idx], Rotation::next());
            let rc = meta.query_fixed(rc_a[idx], Rotation::cur());
            let s_final = meta.query_fixed(s_final, Rotation::cur());
            s_final * (pow_5(cur + rc) - next)
        };
        meta.create_gate("final full round state_0", |meta| final_full_round(meta, 0));
        meta.create_gate("final full round state_1", |meta| final_full_round(meta, 1));
        meta.create_gate("final full round state_2", |meta| final_full_round(meta, 2));

        Arity3Config {
            state,
            state_permutation,
            partial_sbox,
            rc_a,
            rc_b,
            s_full,
            s_partial,
            s_final,
            half_full_rounds,
            half_partial_rounds,
            alpha,
            round_constants,
            m_reg,
            m_inv,
        }
    }
}

impl<F: FieldExt> Chip for Arity3Chip<F> {
    type Field = F;
    type Config = Arity3Config<F>;

    fn load(_: &mut impl Layouter<Self>) -> Result<(), Error> {
        // We have no tables to load.
        Ok(())
    }
}

impl<F: FieldExt> PoseidonInstructions for Arity3Chip<F> {
    type State = Arity3State<F>;

    fn permute(
        layouter: &mut impl Layouter<Self>,
        initial_state: &Self::State,
    ) -> Result<Self::State, Error> {
        let config = layouter.config().clone();

        layouter.assign_region(
            || "permute state",
            |mut region| {
                // Load the initial state into this region.
                let state = Arity3State::load(&mut region, &config, initial_state)?;

                let state = (0..config.half_full_rounds).fold(Ok(state), |res, r| {
                    res.and_then(|state| state.full_round(&mut region, &config, r, r))
                })?;

                let state = (0..config.half_partial_rounds).fold(Ok(state), |res, r| {
                    res.and_then(|state| {
                        state.partial_round(
                            &mut region,
                            &config,
                            config.half_full_rounds + 2 * r,
                            config.half_full_rounds + r,
                        )
                    })
                })?;

                (0..config.half_full_rounds).fold(Ok(state), |res, r| {
                    res.and_then(|state| {
                        if r < config.half_full_rounds - 1 {
                            state.full_round(
                                &mut region,
                                &config,
                                config.half_full_rounds + 2 * config.half_partial_rounds + r,
                                config.half_full_rounds + config.half_partial_rounds + r,
                            )
                        } else {
                            state.final_round(
                                &mut region,
                                &config,
                                config.half_full_rounds + 2 * config.half_partial_rounds + r,
                                config.half_full_rounds + config.half_partial_rounds + r,
                            )
                        }
                    })
                })
            },
        )
    }
}

#[derive(Debug)]
struct StateWord<F: FieldExt> {
    var: Cell,
    value: Option<F>,
}

#[derive(Debug)]
pub struct Arity3State<F: FieldExt>([StateWord<F>; ARITY]);

impl<F: FieldExt> Arity3State<F> {
    fn full_round(
        self,
        region: &mut Region<Arity3Chip<F>>,
        config: &Arity3Config<F>,
        round: usize,
        offset: usize,
    ) -> Result<Self, Error> {
        Self::round(region, config, round, offset, config.s_full, |_| {
            let q_0 = self.0[0]
                .value
                .map(|v| v + config.round_constants[round][0]);
            let q_1 = self.0[1]
                .value
                .map(|v| v + config.round_constants[round][1]);
            let q_2 = self.0[2]
                .value
                .map(|v| v + config.round_constants[round][2]);

            let r_0 = q_0.map(|v| v.pow(&config.alpha));
            let r_1 = q_1.map(|v| v.pow(&config.alpha));
            let r_2 = q_2.map(|v| v.pow(&config.alpha));

            let m = &config.m_reg;
            let r = r_0.and_then(|r_0| r_1.and_then(|r_1| r_2.map(|r_2| [r_0, r_1, r_2])));

            Ok((
                round + 1,
                [
                    r.map(|r| m[0][0] * r[0] + m[0][1] * r[1] + m[0][2] * r[2]),
                    r.map(|r| m[1][0] * r[0] + m[1][1] * r[1] + m[1][2] * r[2]),
                    r.map(|r| m[2][0] * r[0] + m[2][1] * r[1] + m[2][2] * r[2]),
                ],
            ))
        })
    }

    fn partial_round(
        self,
        region: &mut Region<Arity3Chip<F>>,
        config: &Arity3Config<F>,
        round: usize,
        offset: usize,
    ) -> Result<Self, Error> {
        Self::round(region, config, round, offset, config.s_partial, |region| {
            let m = &config.m_reg;

            let p = self.0[0].value.and_then(|p_0| {
                self.0[1]
                    .value
                    .and_then(|p_1| self.0[2].value.map(|p_2| [p_0, p_1, p_2]))
            });

            let r = p.map(|p| {
                [
                    (p[0] + config.round_constants[round][0]).pow(&config.alpha),
                    p[1] + config.round_constants[round][1],
                    p[2] + config.round_constants[round][2],
                ]
            });

            region.assign_advice(
                || format!("round_{} partial_sbox", round),
                config.partial_sbox,
                offset,
                || r.map(|r| r[0]).ok_or(Error::SynthesisError),
            )?;

            let p_mid = r.map(|r| {
                [
                    m[0][0] * r[0] + m[0][1] * r[1] + m[0][2] * r[2],
                    m[1][0] * r[0] + m[1][1] * r[1] + m[1][2] * r[2],
                    m[2][0] * r[0] + m[2][1] * r[1] + m[2][2] * r[2],
                ]
            });

            // Load the second round constants.
            let mut load_round_constant = |i: usize| {
                region.assign_fixed(
                    || format!("round_{} rc_{}", round + 1, i),
                    config.rc_b[i],
                    offset,
                    || Ok(config.round_constants[round + 1][i]),
                )
            };
            for i in 0..ARITY {
                load_round_constant(i)?;
            }

            let r_mid = p_mid.map(|p| {
                [
                    (p[0] + config.round_constants[round + 1][0]).pow(&config.alpha),
                    p[1] + config.round_constants[round + 1][1],
                    p[2] + config.round_constants[round + 1][2],
                ]
            });

            Ok((
                round + 2,
                [
                    r_mid.map(|r| m[0][0] * r[0] + m[0][1] * r[1] + m[0][2] * r[2]),
                    r_mid.map(|r| m[1][0] * r[0] + m[1][1] * r[1] + m[1][2] * r[2]),
                    r_mid.map(|r| m[2][0] * r[0] + m[2][1] * r[1] + m[2][2] * r[2]),
                ],
            ))
        })
    }

    fn final_round(
        self,
        region: &mut Region<Arity3Chip<F>>,
        config: &Arity3Config<F>,
        round: usize,
        offset: usize,
    ) -> Result<Self, Error> {
        Self::round(region, config, round, offset, config.s_final, |_| {
            let mut new_state = self
                .0
                .iter()
                .zip(config.round_constants[round].iter())
                .map(|(word, rc)| word.value.map(|v| (v + rc).pow(&config.alpha)));

            Ok((
                round + 1,
                [
                    new_state.next().unwrap(),
                    new_state.next().unwrap(),
                    new_state.next().unwrap(),
                ],
            ))
        })
    }

    fn load(
        region: &mut Region<Arity3Chip<F>>,
        config: &Arity3Config<F>,
        initial_state: &Self,
    ) -> Result<Self, Error> {
        let mut load_state_word = |i: usize| {
            let value = initial_state.0[i].value;
            let var = region.assign_advice(
                || format!("load state_{}", i),
                config.state[i],
                0,
                || value.ok_or(Error::SynthesisError),
            )?;
            region.constrain_equal(&config.state_permutation, initial_state.0[i].var, var)?;
            Ok(StateWord { var, value })
        };

        Ok(Arity3State([
            load_state_word(0)?,
            load_state_word(1)?,
            load_state_word(2)?,
        ]))
    }

    fn round(
        region: &mut Region<Arity3Chip<F>>,
        config: &Arity3Config<F>,
        round: usize,
        offset: usize,
        round_gate: Column<Fixed>,
        round_fn: impl FnOnce(&mut Region<Arity3Chip<F>>) -> Result<(usize, [Option<F>; ARITY]), Error>,
    ) -> Result<Self, Error> {
        // Enable the required gate.
        region.assign_fixed(
            || format!("round_{} gate", round),
            round_gate,
            offset,
            || Ok(F::one()),
        )?;

        // Load the round constants.
        let mut load_round_constant = |i: usize| {
            region.assign_fixed(
                || format!("round_{} rc_{}", round, i),
                config.rc_a[i],
                offset,
                || Ok(config.round_constants[round][i]),
            )
        };
        for i in 0..ARITY {
            load_round_constant(i)?;
        }

        // Compute the next round's state.
        let (next_round, next_state) = round_fn(region)?;

        let mut next_state_word = |i: usize| {
            let value = next_state[i];
            let var = region.assign_advice(
                || format!("round_{} state_{}", next_round, i),
                config.state[i],
                offset + 1,
                || value.ok_or(Error::SynthesisError),
            )?;
            Ok(StateWord { var, value })
        };

        Ok(Arity3State([
            next_state_word(0)?,
            next_state_word(1)?,
            next_state_word(2)?,
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::{Arity3Chip, Arity3Config, Arity3State, PoseidonInstructions, StateWord};
    use crate::{
        circuit::{layouter, Layouter},
        dev::MockProver,
        gadget::poseidon::LsbPoseidon,
        pasta::Fp,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };

    struct MyCircuit {}

    impl Circuit<Fp> for MyCircuit {
        type Config = Arity3Config<Fp>;

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Arity3Config<Fp> {
            let spec = LsbPoseidon::with_pow_sbox(3, 8, 56);

            let state = [
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            ];

            Arity3Chip::configure(meta, spec, state)
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<Fp>,
            config: Arity3Config<Fp>,
        ) -> Result<(), Error> {
            let mut layouter = layouter::SingleChip::new(cs, config.clone());

            let initial_state = layouter.assign_region(
                || "prepare initial state",
                |mut region| {
                    let mut state_word = |i: usize| {
                        let value = Some(Fp::from(i as u64));
                        let var = region.assign_advice(
                            || format!("load state_{}", i),
                            config.state[i],
                            0,
                            || value.ok_or(Error::SynthesisError),
                        )?;
                        Ok(StateWord { var, value })
                    };

                    Ok(Arity3State([
                        state_word(0)?,
                        state_word(1)?,
                        state_word(2)?,
                    ]))
                },
            )?;

            Arity3Chip::permute(&mut layouter, &initial_state).map(|_| ())
        }
    }

    #[test]
    fn poseidon() {
        let k = 6;
        let circuit = MyCircuit {};
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_poseidon_chip() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("poseidon-chip-layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Poseidon Chip Layout", ("sans-serif", 60))
            .unwrap();

        let circuit = MyCircuit {};
        crate::dev::circuit_layout(&circuit, &root).unwrap();
    }
}
