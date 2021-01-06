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

// Rows needed for each gate
pub const DECOMPOSE_0_ROWS: usize = 2;
pub const DECOMPOSE_1_ROWS: usize = 2;
pub const DECOMPOSE_2_ROWS: usize = 3;
pub const DECOMPOSE_3_ROWS: usize = 2;
pub const SIGMA_0_V1_ROWS: usize = 4;
pub const SIGMA_0_V2_ROWS: usize = 4;
pub const SIGMA_1_V1_ROWS: usize = 4;
pub const SIGMA_1_V2_ROWS: usize = 4;

// Rows needed for each subregion
pub const SUBREGION_0_LEN: usize = 1; // W_0
pub const SUBREGION_0_ROWS: usize = SUBREGION_0_LEN * DECOMPOSE_0_ROWS;
pub const SUBREGION_1_WORD: usize = DECOMPOSE_1_ROWS + SIGMA_0_V1_ROWS;
pub const SUBREGION_1_LEN: usize = 13; // W_[1..14]
pub const SUBREGION_1_ROWS: usize = SUBREGION_1_LEN * SUBREGION_1_WORD;
pub const SUBREGION_2_WORD: usize = DECOMPOSE_2_ROWS + SIGMA_0_V2_ROWS + SIGMA_1_V2_ROWS;
pub const SUBREGION_2_LEN: usize = 35; // W_[14..49]
pub const SUBREGION_2_ROWS: usize = SUBREGION_2_LEN * SUBREGION_2_WORD;
pub const SUBREGION_3_WORD: usize = DECOMPOSE_3_ROWS + SIGMA_1_V1_ROWS;
pub const SUBREGION_3_LEN: usize = 13; // W[49..62]
pub const SUBREGION_3_ROWS: usize = SUBREGION_3_LEN * SUBREGION_3_WORD;
pub const SUBREGION_4_LEN: usize = 2; // W_[62..64]
pub const SUBREGION_4_ROWS: usize = SUBREGION_4_LEN * DECOMPOSE_0_ROWS;

/// Returns row number of a word
pub fn get_word_row(word_idx: usize) -> usize {
    assert!(word_idx <= 63);
    if word_idx == 0 {
        0
    } else if word_idx >= 1 && word_idx <= 13 {
        SUBREGION_0_ROWS + SUBREGION_1_WORD * (word_idx - 1) as usize
    } else if word_idx >= 14 && word_idx <= 48 {
        SUBREGION_0_ROWS + SUBREGION_1_ROWS + SUBREGION_2_WORD * (word_idx - 14) + 1 as usize
    } else if word_idx >= 49 && word_idx <= 61 {
        SUBREGION_0_ROWS
            + SUBREGION_1_ROWS
            + SUBREGION_2_ROWS
            + SUBREGION_3_WORD * (word_idx - 49) as usize
    } else {
        SUBREGION_0_ROWS
            + SUBREGION_1_ROWS
            + SUBREGION_2_ROWS
            + SUBREGION_3_ROWS
            + DECOMPOSE_0_ROWS * (word_idx - 62) as usize
    }
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

use super::{super::BLOCK_SIZE, BlockWord, ROUNDS};

/// Test vector: "abc"
pub fn get_msg_schedule_test_input() -> [BlockWord; BLOCK_SIZE] {
    [
        BlockWord::new(0b01100001011000100110001110000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000000000),
        BlockWord::new(0b00000000000000000000000000011000),
    ]
}

pub const MSG_SCHEDULE_TEST_OUTPUT: [u32; ROUNDS] = [
    0b01100001011000100110001110000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000000000,
    0b00000000000000000000000000011000,
    0b01100001011000100110001110000000,
    0b00000000000011110000000000000000,
    0b01111101101010000110010000000101,
    0b01100000000000000000001111000110,
    0b00111110100111010111101101111000,
    0b00000001100000111111110000000000,
    0b00010010110111001011111111011011,
    0b11100010111000101100001110001110,
    0b11001000001000010101110000011010,
    0b10110111001101100111100110100010,
    0b11100101101111000011100100001001,
    0b00110010011001100011110001011011,
    0b10011101001000001001110101100111,
    0b11101100100001110010011011001011,
    0b01110000001000010011100010100100,
    0b11010011101101111001011100111011,
    0b10010011111101011001100101111111,
    0b00111011011010001011101001110011,
    0b10101111111101001111111111000001,
    0b11110001000010100101110001100010,
    0b00001010100010110011100110010110,
    0b01110010101011111000001100001010,
    0b10010100000010011110001100111110,
    0b00100100011001000001010100100010,
    0b10011111010001111011111110010100,
    0b11110000101001100100111101011010,
    0b00111110001001000110101001111001,
    0b00100111001100110011101110100011,
    0b00001100010001110110001111110010,
    0b10000100000010101011111100100111,
    0b01111010001010010000110101011101,
    0b00000110010111000100001111011010,
    0b11111011001111101000100111001011,
    0b11001100011101100001011111011011,
    0b10111001111001100110110000110100,
    0b10101001100110010011011001100111,
    0b10000100101110101101111011011101,
    0b11000010000101000110001010111100,
    0b00010100100001110100011100101100,
    0b10110010000011110111101010011001,
    0b11101111010101111011100111001101,
    0b11101011111001101011001000111000,
    0b10011111111000110000100101011110,
    0b01111000101111001000110101001011,
    0b10100100001111111100111100010101,
    0b01100110100010110010111111111000,
    0b11101110101010111010001011001100,
    0b00010010101100011110110111101011,
];
