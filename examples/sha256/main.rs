//! Gadget and chips for the [SHA-256] hash function.
//!
//! [SHA-256]: https://tools.ietf.org/html/rfc6234

use std::cmp::min;
use std::convert::TryInto;
use std::fmt;

use halo2::{circuit::Config, plonk::Error};

mod benches;
mod table16;

pub use table16::{BlockWord, Table16Config, Table16Configured};

/// The size of a SHA-256 block, in 32-bit words.
pub const BLOCK_SIZE: usize = 16;
/// The size of a SHA-256 digest, in 32-bit words.
const DIGEST_SIZE: usize = 8;

/// The set of circuit instructions required to use the [`Sha256`] gadget.
pub trait Sha256Instructions: Config {
    /// Variable representing the SHA-256 internal state.
    type State: Clone + fmt::Debug;
    /// Variable representing a 32-bit word of the input block to the SHA-256 compression
    /// function.
    type BlockWord: Copy + fmt::Debug;

    /// The zero BlockWord
    fn zero() -> Self::BlockWord;

    /// Places the SHA-256 IV in the circuit, returning the initial state variable.
    fn initialization_vector(&mut self) -> Result<Self::State, Error>;

    /// Creates an initial state from the output state of a previous block
    fn initialization(&mut self, init_state: &Self::State) -> Result<Self::State, Error>;

    /// Starting from the given initialized state, processes a block of input and returns the
    /// final state.
    fn compress(
        &mut self,
        initialized_state: &Self::State,
        input: [Self::BlockWord; BLOCK_SIZE],
    ) -> Result<Self::State, Error>;

    /// Converts the given state into a message digest.
    fn digest(&mut self, state: &Self::State) -> Result<[Self::BlockWord; DIGEST_SIZE], Error>;
}

/// The output of a SHA-256 circuit invocation.
#[derive(Debug)]
pub struct Sha256Digest<BlockWord>([BlockWord; DIGEST_SIZE]);

/// A gadget that constrains a SHA-256 invocation. It supports input at a granularity of
/// 32 bits.
#[derive(Debug)]
pub struct Sha256<C: Sha256Instructions> {
    state: C::State,
    cur_block: Vec<C::BlockWord>,
    length: usize,
}

impl<Sha256Config: Sha256Instructions> Sha256<Sha256Config> {
    /// Create a new hasher instance.
    pub fn new(config: &mut Sha256Config) -> Result<Self, Error> {
        let state = config.initialization_vector()?;
        Ok(Sha256 {
            // config,
            state,
            cur_block: Vec::with_capacity(BLOCK_SIZE),
            length: 0,
        })
    }

    /// Digest data, updating the internal state.
    pub fn update(
        &mut self,
        config: &mut Sha256Config,
        mut data: &[Sha256Config::BlockWord],
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
        self.state = config.compress(
            &self.state,
            self.cur_block[..]
                .try_into()
                .expect("cur_block.len() == BLOCK_SIZE"),
        )?;
        self.cur_block.clear();

        // Process any additional full blocks.
        let mut chunks_iter = data.chunks_exact(BLOCK_SIZE);
        for chunk in &mut chunks_iter {
            self.state = config.initialization(&self.state)?;
            self.state = config.compress(
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
    pub fn finalize(
        &mut self,
        config: &mut Sha256Config,
    ) -> Result<Sha256Digest<Sha256Config::BlockWord>, Error> {
        // Pad the remaining block
        if !self.cur_block.is_empty() {
            let padding = vec![Sha256Config::zero(); BLOCK_SIZE - self.cur_block.len()];
            self.cur_block.extend_from_slice(&padding);
            self.state = config.initialization(&self.state)?;
            self.state = config.compress(
                &self.state,
                self.cur_block[..]
                    .try_into()
                    .expect("cur_block.len() == BLOCK_SIZE"),
            )?;
        }
        config.digest(&self.state).map(Sha256Digest)
    }

    /// Convenience function to compute hash of the data. It will handle hasher creation,
    /// data feeding and finalization.
    pub fn digest(
        config: &mut Sha256Config,
        data: &[Sha256Config::BlockWord],
    ) -> Result<Sha256Digest<Sha256Config::BlockWord>, Error> {
        let mut hasher = Self::new(config)?;
        hasher.update(config, data)?;
        hasher.finalize(config)
    }
}

fn main() {}
