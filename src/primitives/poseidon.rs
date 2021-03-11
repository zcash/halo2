use std::iter;
use std::marker::PhantomData;

use halo2::arithmetic::FieldExt;

pub(crate) mod grain;
pub(crate) mod mds;

#[cfg(test)]
mod test_vectors;

use grain::SboxType;

/// A specification for a Poseidon permutation.
pub trait Spec<F: FieldExt> {
    /// The arity of this specification.
    fn arity(&self) -> usize;

    /// The number of full rounds for this specification.
    fn full_rounds(&self) -> usize;

    /// The number of partial rounds for this specification.
    fn partial_rounds(&self) -> usize;

    fn sbox(&self, val: F) -> F;

    /// Generates `(round_constants, mds, mds^-1)` corresponding to this specification.
    fn constants(&self) -> (Vec<Vec<F>>, Vec<Vec<F>>, Vec<Vec<F>>);
}

/// A generic Poseidon specification.
#[derive(Debug)]
pub struct Generic<F: FieldExt> {
    pow_sbox: u64,
    /// The arity of the Poseidon permutation.
    t: u16,
    /// The number of full rounds.
    r_f: u16,
    /// The number of partial rounds.
    r_p: u16,
    /// The index of the first secure MDS matrix that will be generated for the given
    /// parameters.
    secure_mds: usize,
    _field: PhantomData<F>,
}

impl<F: FieldExt> Generic<F> {
    /// Creates a new Poseidon specification for a field, using the `x^\alpha` S-box.
    pub fn with_pow_sbox(
        pow_sbox: u64,
        arity: usize,
        full_rounds: usize,
        partial_rounds: usize,
        secure_mds: usize,
    ) -> Self {
        Generic {
            pow_sbox,
            t: arity as u16,
            r_f: full_rounds as u16,
            r_p: partial_rounds as u16,
            secure_mds,
            _field: PhantomData::default(),
        }
    }
}

impl<F: FieldExt> Spec<F> for Generic<F> {
    fn arity(&self) -> usize {
        self.t as usize
    }

    fn full_rounds(&self) -> usize {
        self.r_f as usize
    }

    fn partial_rounds(&self) -> usize {
        self.r_p as usize
    }

    fn sbox(&self, val: F) -> F {
        val.pow_vartime(&[self.pow_sbox])
    }

    fn constants(&self) -> (Vec<Vec<F>>, Vec<Vec<F>>, Vec<Vec<F>>) {
        let mut grain = grain::Grain::new(SboxType::Pow, self.t, self.r_f, self.r_p);

        let round_constants = (0..(self.r_f + self.r_p))
            .map(|_| (0..self.t).map(|_| grain.next_field_element()).collect())
            .collect();

        let (mds, mds_inv) = mds::generate_mds(&mut grain, self.t as usize, self.secure_mds);

        (round_constants, mds, mds_inv)
    }
}

/// Runs the Poseidon permutation on the given state.
fn permute<F: FieldExt, S: Spec<F>>(
    state: &mut [F],
    spec: &S,
    mds: &[Vec<F>],
    round_constants: &[Vec<F>],
) {
    // TODO: Remove this when we can use const generics.
    assert!(state.len() == spec.arity());

    let r_f = spec.full_rounds() / 2;
    let r_p = spec.partial_rounds();

    let apply_mds = |state: &mut [F]| {
        let new_state: Vec<_> = mds
            .iter()
            .map(|mds_row| {
                mds_row
                    .iter()
                    .zip(state.iter())
                    .fold(F::zero(), |acc, (mds, word)| acc + *mds * *word)
            })
            .collect();
        for (word, new_word) in state.iter_mut().zip(new_state.into_iter()) {
            *word = new_word;
        }
    };

    let full_round = |state: &mut [F], rcs: &[F]| {
        for (word, rc) in state.iter_mut().zip(rcs.iter()) {
            *word = spec.sbox(*word + rc);
        }
        apply_mds(state);
    };

    let part_round = |state: &mut [F], rcs: &[F]| {
        for (word, rc) in state.iter_mut().zip(rcs.iter()) {
            *word += rc;
        }
        state[0] = spec.sbox(state[0]);
        apply_mds(state);
    };

    iter::empty()
        .chain(iter::repeat(&full_round as &dyn Fn(&mut [F], &[F])).take(r_f))
        .chain(iter::repeat(&part_round as &dyn Fn(&mut [F], &[F])).take(r_p))
        .chain(iter::repeat(&full_round as &dyn Fn(&mut [F], &[F])).take(r_f))
        .zip(round_constants.iter())
        .fold(state, |state, (round, rcs)| {
            round(state, rcs);
            state
        });
}

fn pad_and_add<F: FieldExt>(state: &mut [F], input: &[F]) {
    let padding = state.len() - input.len();
    // TODO: Decide on a padding strategy (currently padding with all-ones)
    for (word, val) in state
        .iter_mut()
        .zip(input.iter().chain(iter::repeat(&F::one()).take(padding)))
    {
        *word += val;
    }
}

enum SpongeState<F: FieldExt> {
    Absorbing(Vec<F>),
    Squeezing(Vec<F>),
}

/// A Poseidon duplex sponge.
pub struct Duplex<F: FieldExt, S: Spec<F>> {
    spec: S,
    sponge: Option<SpongeState<F>>,
    state: Vec<F>,
    rate: usize,
    mds_matrix: Vec<Vec<F>>,
    round_constants: Vec<Vec<F>>,
}

impl<F: FieldExt, S: Spec<F>> Duplex<F, S> {
    /// Constructs a new duplex sponge with the given rate.
    pub fn new(spec: S, rate: usize) -> Self {
        assert!(rate < spec.arity());

        let state = vec![F::zero(); spec.arity()];
        let (round_constants, mds_matrix, _) = spec.constants();

        Duplex {
            spec,
            sponge: Some(SpongeState::Absorbing(vec![])),
            state,
            rate,
            mds_matrix,
            round_constants,
        }
    }

    fn process(&mut self, input: &[F]) -> Vec<F> {
        pad_and_add(&mut self.state[..self.rate], input);

        permute(
            &mut self.state,
            &self.spec,
            &self.mds_matrix,
            &self.round_constants,
        );

        self.state[..self.rate].to_vec()
    }

    /// Absorbs an element into the sponge.
    pub fn absorb(&mut self, value: F) {
        match self.sponge.take().unwrap() {
            SpongeState::Absorbing(mut input) => {
                if input.len() < self.rate {
                    input.push(value);
                    self.sponge = Some(SpongeState::Absorbing(input));
                    return;
                }

                // We've already absorbed as many elements as we can
                let _ = self.process(&input);
                self.sponge = Some(SpongeState::Absorbing(vec![value]));
            }
            SpongeState::Squeezing(_) => {
                // Drop the remaining output elements
                self.sponge = Some(SpongeState::Absorbing(vec![value]));
            }
        }
    }

    /// Squeezes an element from the sponge.
    pub fn squeeze(&mut self) -> F {
        loop {
            match self.sponge.take().unwrap() {
                SpongeState::Absorbing(input) => {
                    self.sponge = Some(SpongeState::Squeezing(self.process(&input)));
                }
                SpongeState::Squeezing(mut output) => {
                    if !output.is_empty() {
                        let ret = output.remove(0);
                        self.sponge = Some(SpongeState::Squeezing(output));
                        return ret;
                    }

                    // We've already squeezed out all available elements
                    self.sponge = Some(SpongeState::Absorbing(vec![]));
                }
            }
        }
    }
}

/// A Poseidon hash function, built around a duplex sponge.
pub struct Hash<F: FieldExt, S: Spec<F>>(Duplex<F, S>);

impl<F: FieldExt, S: Spec<F>> Hash<F, S> {
    /// Initializes a new hasher.
    pub fn init(spec: S, rate: usize) -> Self {
        Hash(Duplex::new(spec, rate))
    }

    /// Updates the hasher with the given value.
    pub fn update(&mut self, value: F) {
        self.0.absorb(value);
    }

    /// Finalizes the hasher, returning its output.
    pub fn finalize(mut self) -> F {
        // TODO: Check which state element other implementations use.
        self.0.squeeze()
    }
}
