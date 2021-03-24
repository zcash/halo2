//! Sinsemilla generators

/// Number of bits of each message piece in $\mathsf{SinsemillaHashToPoint}$
pub const K: usize = 10;

/// The largest integer such that $2^c \leq (r_P - 1) / 2$, where $r_P$ is the order
/// of Pallas.
pub const C: usize = 253;

// Sinsemilla Q generators

/// SWU hash-to-curve personalization for Sinsemilla $Q$ generators.
pub const Q_PERSONALIZATION: &str = "z.cash:SinsemillaQ";

// Sinsemilla S generators

/// SWU hash-to-curve personalization for Sinsemilla $S$ generators.
pub const S_PERSONALIZATION: &str = "z.cash:SinsemillaS";
