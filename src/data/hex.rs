//! Methods for operating with hexadecimal strings.

use rust_decimal::prelude::*;

/// Encode a number string to bytes.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::data::hex::{encode_num_to_bytes, HexDataError};
///
/// assert!(matches!(encode_num_to_bytes("1280"), Ok(v) if v == [0, 0, 5, 0, 0]));
/// assert!(matches!(encode_num_to_bytes("25600"), Ok(v) if v == [0, 0, 100, 0, 0]));
/// assert!(matches!(encode_num_to_bytes("512000"), Ok(v) if v == [0, 7, 208, 0, 0]));
/// assert!(matches!(encode_num_to_bytes("10240000"), Ok(v) if v == [0, 156, 64, 0, 0]));
///
/// assert!(matches!(encode_num_to_bytes("512.000"), Ok(v) if v == [0, 7, 208, 0, 3]));
/// assert!(matches!(encode_num_to_bytes("512.001"), Ok(v) if v == [0, 7, 208, 1, 3]));
/// assert!(matches!(encode_num_to_bytes("512.016"), Ok(v) if v == [0, 7, 208, 16, 3]));
///
/// assert!(matches!(
///     encode_num_to_bytes("Hello!"),
///     Err(HexDataError::StrU32ParseError(_))
/// ));
/// ```
pub fn encode_num_to_bytes(value: &str) -> HexDataResult<[u8; 5]> {
    const E: usize = 0;
    let mut result = Vec::<u8>::with_capacity(5);

    // if value.find("E-") != Some(0) {
    //     let split: Vec<&str> = value.split("E-").collect();
    //     let a = split[1];
    //     e = a.parse().unwrap();
    //     value = split[0].to_string();
    // }

    let scale_point_indicator = match value.find('.') {
        Some(idx) => value.len() - idx - 1 + E,
        None => 0,
    } as u8;

    let value = value.replace('.', "");
    result.extend_from_slice(
        &value
            .parse::<u32>()
            .map_err(HexDataError::StrU32ParseError)?
            .to_be_bytes(),
    );

    // Push the scale point indicator (SPI).
    result.push(scale_point_indicator);

    Ok(result.try_into().expect("len > 5"))
}

/// Decode the specified bytes to a [`Decimal`].
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::data::hex::decode_bytes_to_num;
///
/// assert_eq!(decode_bytes_to_num(&[0, 0, 5, 0, 0]).to_string(), "1280");
/// assert_eq!(decode_bytes_to_num(&[0, 0, 100, 0, 0]).to_string(), "25600");
/// assert_eq!(
///     decode_bytes_to_num(&[0, 7, 208, 0, 0]).to_string(),
///     "512000"
/// );
/// assert_eq!(
///     decode_bytes_to_num(&[0, 156, 64, 0, 0]).to_string(),
///     "10240000"
/// );
///
/// assert_eq!(
///     decode_bytes_to_num(&[0, 7, 208, 0, 3]).to_string(),
///     "512.000"
/// );
/// assert_eq!(
///     decode_bytes_to_num(&[0, 7, 208, 1, 3]).to_string(),
///     "512.001"
/// );
/// assert_eq!(
///     decode_bytes_to_num(&[0, 7, 208, 16, 3]).to_string(),
///     "512.016"
/// );
/// ```
pub fn decode_bytes_to_num(value: &[u8; 5]) -> Decimal {
    let num_part = i64::from_be_bytes({
        let raw = &value[0..4];
        let mut dst = [0u8; 8];

        dst[4..].copy_from_slice(raw);
        dst
    });

    let scale_part = u32::from_be_bytes({
        let raw = value[4];

        [0, 0, 0, raw]
    });

    Decimal::new(num_part, scale_part)
}

#[derive(thiserror::Error, Debug)]
pub enum HexDataError {
    #[error("unable to encode a string to u32: {0}")]
    StrU32ParseError(<usize as std::str::FromStr>::Err),
}

pub type HexDataResult<T> = Result<T, HexDataError>;
