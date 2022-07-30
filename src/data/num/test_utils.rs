#![allow(clippy::identity_op)]
#![cfg(test)]

use super::Decimal;

pub const SIGN: u8 = 0x80;

// Meaningless behavior
// For readability of the code only
pub const NOT_SIGN: u8 = 0x00;

pub fn convert_to_10b_test(src: [u8; 5]) -> [u8; 10] {
    let num_part = &src[0..4];
    let scale_part = src[4];

    let mut dst = [0; 10];
    dst[5..9].copy_from_slice(num_part);
    dst[9] = scale_part;

    dst
}

pub fn get_basic_tests_set<const B: usize>() -> [(Decimal, Vec<u8>); 10] {
    use rust_decimal_macros::dec;
    let decider = |m: [u8; 5]| match B {
        5 => m.to_vec(),
        10 => convert_to_10b_test(m).to_vec(),
        _ => unreachable!(),
    };

    [
        (dec!(1280), decider([0, 0, 5, 0, 0 | NOT_SIGN])),
        (dec!(25600), decider([0, 0, 100, 0, 0 | NOT_SIGN])),
        (dec!(512000), decider([0, 7, 208, 0, 0 | NOT_SIGN])),
        (dec!(10240000), decider([0, 156, 64, 0, 0 | NOT_SIGN])),
        (dec!(-10240000), decider([0, 156, 64, 0, 0 | SIGN])),
        (dec!(512.000), decider([0, 7, 208, 0, 3 | NOT_SIGN])),
        (dec!(512.001), decider([0, 7, 208, 1, 3 | NOT_SIGN])),
        (dec!(512.016), decider([0, 7, 208, 16, 3 | NOT_SIGN])),
        (dec!(-10240000.1), decider([6, 26, 128, 1, 1 | SIGN])),
        (dec!(-10240000.12), decider([61, 9, 0, 12, 2 | SIGN])),
    ]
}
