//! Pedersen hash used to encode public inputs.

use pasta_curves::arithmetic::CurveExt;
use std::convert::{TryFrom, TryInto};

fn mapping<C: CurveExt>(bits: [bool; 4], point: C) -> C {
    // 2-bit lookup
    let mut point = match [bits[0], bits[1]] {
        [false, false] => point,
        [true, false] => point.double(),
        [false, true] => point + point.double(),
        [true, true] => point.double().double(),
    };

    // Conditional negation
    if bits[2] {
        point = -point;
    }

    // Conditional endoscaling
    if bits[3] {
        point = point.endo();
    }

    point
}

fn usize_as_le_bits(int: usize) -> [bool; 4] {
    assert!(int < 16);
    [
        int & 0b0001 != 0,
        int & 0b0010 != 0,
        int & 0b0100 != 0,
        int & 0b1000 != 0,
    ]
}

fn le_bits_as_usize(bits: &[bool; 4]) -> usize {
    bits.iter()
        .rev()
        .fold(0, |acc, bit| 2 * acc + *bit as usize)
}

pub(crate) fn pedersen_hash<C: CurveExt>(bits: &[bool], generators: &[C]) -> C {
    assert!(bits.len() <= generators.len() * 4);
    let bits = chunks4(bits);

    let mut windows = [C::identity(); 16];

    for (index, generator) in bits.iter().map(le_bits_as_usize).zip(generators.iter()) {
        windows[index as usize] += *generator;
    }

    windows
        .iter()
        .enumerate()
        .fold(C::identity(), |acc, (i, window)| {
            acc + mapping(usize_as_le_bits(i), *window)
        })
}

pub(super) fn chunks4(bits: &[bool]) -> Vec<[bool; 4]> {
    let chunks = bits.chunks_exact(4);
    let remainder = chunks.remainder();

    let mut chunks: Vec<[bool; 4]> = chunks.map(|slice| slice.try_into().unwrap()).collect();

    if remainder.len() > 0 {
        let mut padded = [false; 4];
        padded.copy_from_slice(remainder);
        chunks.push(padded);
    }

    chunks
}

#[test]
fn test_pedersen_hash() {
    use group::Group;
    use pasta_curves::pallas;
    use std::convert::TryInto;

    let bits = [true, true, false, false, false, true, false, true];
    let generators = [
        pallas::Point::generator(),
        pallas::Point::generator().double(),
    ];

    let hash = pedersen_hash(&bits, &generators);
    let expected_hash = chunks4(&bits)
        .iter()
        .zip(generators.iter())
        .fold(pallas::Point::identity(), |acc, (bits, generator)| {
            acc + mapping(*bits, *generator)
        });

    assert_eq!(hash, expected_hash)
}
