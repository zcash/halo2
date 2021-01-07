pub const BITS_7: usize = 1 << 7;
pub const BITS_10: usize = 1 << 10;
pub const BITS_11: usize = 1 << 11;
pub const BITS_13: usize = 1 << 13;
pub const BITS_14: usize = 1 << 14;
pub const MASK_EVEN_32: u32 = 0x55555555;
pub const MASK_ODD_32: u32 = 0xAAAAAAAA;

/// Helper function that returns 32-bit spread version of 16-bit input.
pub fn interleave_u16_with_zeros(word: u16) -> u32 {
    let mut word: u32 = word.into();
    word = (word ^ (word << 8)) & 0x00ff00ff;
    word = (word ^ (word << 4)) & 0x0f0f0f0f;
    word = (word ^ (word << 2)) & 0x33333333;
    word = (word ^ (word << 1)) & 0x55555555;
    word
}

// Reverses interleaving function by removing interleaved zeros.
pub fn compress_u32(word: u32) -> u16 {
    let mut word = word;
    assert_eq!(word & MASK_EVEN_32, word);
    word = (word | (word >> 1)) & 0x33333333;
    word = (word | (word >> 2)) & 0x0f0f0f0f;
    word = (word | (word >> 4)) & 0x00ff00ff;
    word = (word | (word >> 8)) & 0x0000ffff;
    word as u16
}

// Chops a 32-bit word into pieces of given length. The lengths are specified
// starting from the little end.
pub fn chop_u32(word: u32, lengths: &[u8]) -> Vec<u32> {
    assert_eq!(lengths.iter().sum::<u8>(), 32 as u8);
    let mut pieces: Vec<u32> = Vec::with_capacity(lengths.len());
    for i in 0..lengths.len() {
        assert!(lengths[i] > 0);
        // lengths[i] bitstring of all 1's
        let mask: u32 = (1 << lengths[i]) as u32 - 1;
        // Shift mask by bits already shifted
        let offset: u8 = lengths[0..i].into_iter().sum();
        let mask: u32 = mask << offset;
        pieces.push((word & mask) >> offset as u32);
    }
    pieces
}

// Chops a 64-bit word into pieces of given length. The lengths are specified
// starting from the little end.
pub fn chop_u64(word: u64, lengths: &[u8]) -> Vec<u64> {
    assert_eq!(lengths.iter().sum::<u8>(), 64 as u8);
    let mut pieces: Vec<u64> = Vec::with_capacity(lengths.len());
    for i in 0..lengths.len() {
        assert!(lengths[i] > 0);
        // lengths[i] bitstring of all 1's
        let mask: u64 = ((1 as u64) << lengths[i]) - 1;
        // Shift mask by bits already shifted
        let offset: u8 = lengths[0..i].into_iter().sum();
        let mask: u64 = mask << offset;
        pieces.push((word & mask) >> offset as u64);
    }
    pieces
}

// Returns compressed even and odd bits of 32-bit word
pub fn get_even_and_odd_bits_u32(word: u32) -> (u16, u16) {
    let even = word & MASK_EVEN_32;
    let odd = (word & MASK_ODD_32) >> 1;
    (compress_u32(even), compress_u32(odd))
}

// Split 4-bit value into 2-bit lo and hi halves
pub fn bisect_four_bit(word: u32) -> (u32, u32) {
    assert!(word < 16); // 4-bit range-check
    let word_hi = (word & 0b1100) >> 2;
    let word_lo = word & 0b0011;
    (word_lo, word_hi)
}

// Helper function that returns tag of 16-bit input
pub fn get_tag(input: u16) -> u8 {
    let input = input as usize;
    if input < BITS_7 {
        0
    } else if input < BITS_10 {
        1
    } else if input < BITS_11 {
        2
    } else if input < BITS_13 {
        3
    } else if input < BITS_14 {
        4
    } else {
        5
    }
}
