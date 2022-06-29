//! Methods for operating with hexadecimal strings.

use std::{num::ParseIntError, iter};

pub fn long_to_hex(num: i64) -> String {
    let num_hex = format!("{:x}", num); // to hex
    // TODO: migrate to div_ceil
    let mut num_hex_len = num_hex.len() / 2;
    if num_hex_len * 2 < num_hex.len() {
        num_hex_len += 1;
    }
    let pad_len = num_hex_len * 2;
    let long_hex = format!("{num_hex:0>pad_len$}");
    long_hex
}

pub fn hex_to_byte(hex: &str) -> HexDataResult<Vec<u8>> {
    let hex = hex.replace(' ', "");
    let mut bytes = Vec::<u8>::new();

    if hex.len() % 2 == 1 {
        return Ok(bytes);
    }

    for i in 0..(hex.len() / 2) {
        let str = &hex[i * 2..i * 2 + 2];
        let byt =
            u8::from_str_radix(str, 16)
                .map_err(HexDataError::HexDecodeError)?;
        bytes.push(byt);
    }

    Ok(bytes)
}

pub fn encode_num_to_bytes(value: &str) -> HexDataResult<Vec<u8>> {
    const E: usize = 0;
    let mut result = Vec::<u8>::with_capacity(5);

    // if value.find("E-") != Some(0) {
    //     let split: Vec<&str> = value.split("E-").collect();
    //     let a = split[1];
    //     e = a.parse().unwrap();
    //     value = split[0].to_string();
    // }

    let float_indicator = match value.find('.') {
        Some(idx) => value.len() - idx - 1 + E,
        None => 0,
    } as u8;

    let value = value.replace('.', "");
    let hex_str = long_to_hex(value.parse().map_err(HexDataError::StrLongParseError)?);
    let hex_byte = hex_to_byte(&hex_str)?;

    //  Fill rule:
    //  0     1     2     3     4     5     6     7     8    9
    //  6     7     8     9    #FI
    //  5     -     -     -     -
    //  0     0     0     5    #FI

    result.extend(
        hex_byte
            .into_iter()
            // First, reverse the order:
            // Take the above as example:
            //   - [ 9  8  7  6  5  4  3  2  1  0 ]
            //   - [ 5 ]
            .rev()
            // Then, we chain a repeat '0' as the padding.
            // Take the above as example:
            //   - [ 9  8  7  6  5  4  3  2  1  0  0  ...]
            //   - [ 5  0  0  0  0  0  0  0  0  0  0  ...]
            .chain(iter::repeat(0))
            // We take only 4 elements.
            // Take the above as example:
            //   - [ 9  8  7  6 ]
            //   - [ 5  0  0  0 ]
            .take(4)
            // As `iter::repeat` did not implement `ExactSizeIterator`,
            // we return the intermediate result as a `Vec`.
            // Collect here and we'll reverse it later.
            .collect::<Vec<u8>>()
    );

    // Reverse again to get the positive sequence.
    // Take the above as example:
    //   - [ 6  7  8  9 ]
    //   - [ 0  0  0  5 ]
    result.reverse();
    // Push the float indicator (FI).
    result.push(float_indicator);

    if result.len() != 5 {
        panic!("code issue: result.len() != 5")
    } else {
        Ok(result)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HexDataError {
    #[error("unable to decode a hex string to bytes: {0}")]
    HexDecodeError(ParseIntError),

    #[error("unable to encode a string to i64: {0}")]
    StrLongParseError(<i64 as std::str::FromStr>::Err),
}

pub type HexDataResult<T> = Result<T, HexDataError>;

#[cfg(test)]
mod tests {
    use crate::data::hex::{hex_to_byte, HexDataError, encode_num_to_bytes};

    use super::long_to_hex;

    #[test]
    fn long_to_hex_test() {
        assert_eq!(long_to_hex(1280), "0500");
        assert_eq!(long_to_hex(25600), "6400");
        assert_eq!(long_to_hex(512000), "07d000");
        assert_eq!(long_to_hex(10240000), "9c4000");
    }

    #[test]
    fn hex_to_byte_test() {
        assert!(matches!(hex_to_byte("0500"), Ok(v) if v == [5, 0]));
        assert!(matches!(hex_to_byte("6400"), Ok(v) if v == [100, 0]));
        assert!(matches!(hex_to_byte("07d000"), Ok(v) if v == [7, 208, 0]));
        assert!(matches!(hex_to_byte("9c4000"), Ok(v) if v == [156, 64, 0]));

        assert!(matches!(hex_to_byte("9c400"), Ok(v) if v.is_empty()));
        assert!(matches!(hex_to_byte("9c4"), Ok(v) if v.is_empty()));
        assert!(matches!(hex_to_byte("9"), Ok(v) if v.is_empty()));

        // Failed tests:
        // assert!(matches!(hex_to_byte("$"), Err(HexDataError::HexDecodeError(_))));
        // assert!(matches!(hex_to_byte("你好"), Err(HexDataError::HexDecodeError(_))));
    }

    #[test]
    fn encode_num_to_bytes_test() {
        assert!(matches!(encode_num_to_bytes("1280"), Ok(v) if v == [0, 0, 5, 0, 0]));
        assert!(matches!(encode_num_to_bytes("25600"), Ok(v) if v == [0, 0, 100, 0, 0]));
        assert!(matches!(encode_num_to_bytes("512000"), Ok(v) if v == [0, 7, 208, 0, 0]));
        assert!(matches!(encode_num_to_bytes("10240000"), Ok(v) if v == [0, 156, 64, 0, 0]));
        assert!(matches!(encode_num_to_bytes("102400000000"), Ok(v) if v == [215, 132, 0, 0, 0]));

        assert!(matches!(encode_num_to_bytes("512.000"), Ok(v) if v == [0, 7, 208, 0, 3]));
        assert!(matches!(encode_num_to_bytes("512.001"), Ok(v) if v == [0, 7, 208, 1, 3]));
        assert!(matches!(encode_num_to_bytes("512.016"), Ok(v) if v == [0, 7, 208, 16, 3]));

        assert!(matches!(encode_num_to_bytes("Hello!"), Err(HexDataError::StrLongParseError(_))));
    }
}
