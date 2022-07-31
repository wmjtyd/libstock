//! The methods for operating with data structures.
//!
//! This module includes some methods for converting numbers,
//! see the module [`num`].

pub mod num;
pub mod serializer;

#[cfg(feature = "crypto")]
pub mod orderbook;

#[cfg(feature = "crypto")]
pub mod fields;

#[cfg(feature = "crypto")]
pub mod order;

#[cfg(feature = "crypto")]
pub mod trade;

#[cfg(feature = "crypto")]
pub mod bbo;

#[cfg(feature = "crypto")]
pub mod kline;

#[cfg(feature = "crypto")]
pub mod funding_rate;

#[cfg(feature = "compat-v0_3")]
/// The 0.3-compatible `hex` module.
/// 
/// Note that:
/// 
/// - It will **panic** for parsing error due to the
///   large change of API; therefore, you should
///   migrate and replace it as soon as possible.
/// - The performance of this module will be **degraded**
///   due to the meaningless convert.
pub mod hex {
    // allow using deprecated code to stop the annoying deprecated warnings.
    #[allow(deprecated)]

    use rust_decimal::Decimal;

    use super::num::{self, Encoder, Decoder};

    #[deprecated = "Use `data::num::six_byte_hex_to_unix_ms` instead!"]
    pub fn six_byte_hex_to_unix_ms(encoded_timestamp: &[u8; 6]) -> u64 {
        num::six_byte_hex_to_unix_ms(encoded_timestamp)
    }

    #[deprecated = "Use `data::num::unix_ms_to_six_byte_hex` instead!"]
    pub fn unix_ms_to_six_byte_hex(timestamp: u64) -> [u8; 6] {
        num::unix_ms_to_six_byte_hex(timestamp)
    }

    #[deprecated = "Migrate to `data::num::NumError`!"]
    pub type NumError = num::NumError;

    #[deprecated = "Replace this to `Result<T, data::num::NumError>`!"]
    pub type HexDataResult<T> = Result<T, NumError>;
    
    #[deprecated = "Replace this to `data::num::{Encoder, Decoder}`. See CHANGELOG."]
    pub trait NumToBytesExt<const LEN: usize> {
        /// Encode a number string to [`u8`] bytes.
        #[deprecated = "Replace this to new Encoder and Decoder trait. See CHANGELOG."]
        fn encode_bytes(value: &str) -> Result<[u8; LEN], NumError>;
        
        /// Decode the specified [`u8`] bytes to a [`Decimal`].
        #[deprecated = "Replace this to new Encoder and Decoder trait. See CHANGELOG."]
        fn decode_bytes(value: &[u8; LEN]) -> Decimal;
        
        /// Encode a number string to [`i8`] bytes safely.
        #[deprecated = "Replace this to new Encoder and Decoder trait. See CHANGELOG."]
        fn encode_i8_bytes(value: &str) -> Result<[i8; LEN], NumError> {
            let encoded_u8 = Self::encode_bytes(value)?;
            
            Ok(encoded_u8.map(|v| v as i8))
        }
        
        /// Decode a number string to [`i8`] bytes safely.
        #[deprecated = "Replace this to new Encoder and Decoder trait. See CHANGELOG."]
        fn decode_i8_bytes(value: &[i8; LEN]) -> Decimal {
            let encoded_u8 = value.map(|v| v as u8);
    
            Self::decode_bytes(&encoded_u8)
        }
    }

    impl NumToBytesExt<5> for u32 {
        fn encode_bytes(value: &str) -> Result<[u8; 5], NumError> {
            let d = Decimal::from_str_exact(value).expect("deprecated: from_str_exact bug!");
            d.encode()
        }

        fn decode_bytes(value: &[u8; 5]) -> Decimal {
            Decoder::decode(value).unwrap()
        }
    }

    impl NumToBytesExt<10> for u64 {
        fn encode_bytes(value: &str) -> Result<[u8; 10], NumError> {
            let d = Decimal::from_str_exact(value).expect("deprecated: from_str_exact bug!");
            d.encode()
        }

        fn decode_bytes(value: &[u8; 10]) -> Decimal {
            Decoder::decode(value).unwrap()
        }
    }

    impl NumToBytesExt<5> for i32 {
        fn encode_bytes(value: &str) -> Result<[u8; 5], NumError> {
            u32::encode_bytes(value)
        }

        fn decode_bytes(value: &[u8; 5]) -> Decimal {
            u32::decode_bytes(value)
        }
    }

    impl NumToBytesExt<10> for i64 {
        fn encode_bytes(value: &str) -> Result<[u8; 10], NumError> {
            u64::encode_bytes(value)
        }

        fn decode_bytes(value: &[u8; 10]) -> Decimal {
            u64::decode_bytes(value)
        }
    }
}
