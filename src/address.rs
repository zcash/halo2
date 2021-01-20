use crate::{keys::Diversifier, Chain};

/// A shielded payment address.
#[derive(Debug)]
pub struct Address<C: Chain> {
    chain: C,
    d: Diversifier<C>,
    pk_d: (),
}
