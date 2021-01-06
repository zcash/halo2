//! Gadget and chips for the [SHA-256] hash function.
//!
//! [SHA-256]: https://tools.ietf.org/html/rfc6234

use std::cmp::min;
use std::convert::TryInto;
use std::fmt;

use crate::{
    gadget::{Chip, Layouter},
    plonk::Error,
};

mod table16;

pub use table16::Table16Chip;

/// The size of a SHA-256 block, in 32-bit words.
const BLOCK_SIZE: usize = 16;
/// The size of a SHA-256 digest, in 32-bit words.
const DIGEST_SIZE: usize = 8;

/// The set of circuit instructions required to use the [`Sha256`] gadget.
pub trait Sha256Instructions: Chip {
    /// Variable representing the SHA-256 internal state.
    type State: Clone + fmt::Debug;
    /// Variable representing a 32-bit word of the input block to the SHA-256 compression
    /// function.
    type BlockWord: Copy + fmt::Debug;

    /// Places the SHA-256 IV in the circuit, returning the initial state variable.
    fn initialization_vector(layouter: &mut impl Layouter<Self>) -> Result<Self::State, Error>;

    /// Starting from the given intial state, processes a block of input and returns the
    /// final state.
    fn compress(
        layouter: &mut impl Layouter<Self>,
        initial_state: &Self::State,
        input: [Self::BlockWord; BLOCK_SIZE],
    ) -> Result<Self::State, Error>;

    /// Converts the given state into a message digest.
    fn digest(
        layouter: &mut impl Layouter<Self>,
        state: &Self::State,
    ) -> Result<[Self::BlockWord; DIGEST_SIZE], Error>;
}

/// The output of a SHA-256 circuit invocation.
#[derive(Debug)]
pub struct Sha256Digest {}

/// A gadget that constrains a SHA-256 invocation. It supports input at a granularity of
/// 32 bits.
#[derive(Debug)]
pub struct Sha256<CS: Sha256Instructions> {
    state: CS::State,
    cur_block: Vec<CS::BlockWord>,
    length: usize,
}

impl<Sha256Chip: Sha256Instructions> Sha256<Sha256Chip> {
    /// Create a new hasher instance.
    pub fn new(layouter: &mut impl Layouter<Sha256Chip>) -> Result<Self, Error> {
        Ok(Sha256 {
            state: Sha256Chip::initialization_vector(layouter)?,
            cur_block: Vec::with_capacity(BLOCK_SIZE),
            length: 0,
        })
    }

    /// Digest data, updating the internal state.
    pub fn update(
        &mut self,
        layouter: &mut impl Layouter<Sha256Chip>,
        mut data: &[Sha256Chip::BlockWord],
    ) -> Result<(), Error> {
        self.length += data.len() * 32;

        // Fill the current block, if possible.
        let remaining = BLOCK_SIZE - self.cur_block.len();
        let (l, r) = data.split_at(min(remaining, data.len()));
        self.cur_block.extend_from_slice(l);
        data = r;

        // If we still don't have a full block, we are done.
        if self.cur_block.len() < BLOCK_SIZE {
            return Ok(());
        }

        // Process the now-full current block.
        self.state = Sha256Chip::compress(
            layouter,
            &self.state,
            self.cur_block[..]
                .try_into()
                .expect("cur_block.len() == BLOCK_SIZE"),
        )?;
        self.cur_block.clear();

        // Process any additional full blocks.
        let mut chunks_iter = data.chunks_exact(BLOCK_SIZE);
        for chunk in &mut chunks_iter {
            self.state = Sha256Chip::compress(
                layouter,
                &self.state,
                chunk.try_into().expect("chunk.len() == BLOCK_SIZE"),
            )?;
        }

        // Cache the remaining partial block, if any.
        let rem = chunks_iter.remainder();
        self.cur_block.extend_from_slice(rem);

        Ok(())
    }

    /// Retrieve result and consume hasher instance.
    pub fn finalize(self, layouter: &mut impl Layouter<Sha256Chip>) -> Result<Sha256Digest, Error> {
        // Pad the remaining block
        // TODO: Can we simply clone variables in PLONK, or do we need copy constraints?
        todo!()
    }

    /// Convenience function to compute hash of the data. It will handle hasher creation,
    /// data feeding and finalization.
    pub fn digest(
        layouter: &mut impl Layouter<Sha256Chip>,
        data: &[Sha256Chip::BlockWord],
    ) -> Result<Sha256Digest, Error> {
        let mut hasher = Self::new(layouter)?;
        hasher.update(layouter, data)?;
        hasher.finalize(layouter)
    }
}
