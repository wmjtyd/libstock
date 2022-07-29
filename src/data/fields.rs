//! The field (de)serialization module for libstock.

use std::{
    str::FromStr,
    time::{SystemTime, SystemTimeError},
};

use crypto_market_type::MarketType;
use crypto_message::TradeSide;
use crypto_msg_type::MessageType;
use either::Either;

use super::{
    hex::{six_byte_hex_to_unix_ms, unix_ms_to_six_byte_hex, HexDataError, NumToBytesExt},
    serializer::{FieldDeserializer, FieldSerializer},
    types::{
        bit_deserialize_message_type, bit_deserialize_trade_side, bit_serialize_message_type,
        bit_serialize_trade_side, DataTypesError, Exchange, InfoType, MARKET_TYPE_BIT, PERIOD,
        SYMBOL_PAIR,
    },
};

/// The timestamp of exchange (6 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExchangeTimestampField(pub u64);

impl FieldSerializer<6> for ExchangeTimestampField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 6], Self::Err> {
        Ok(unix_ms_to_six_byte_hex(self.0))
    }
}

impl FieldDeserializer<6> for ExchangeTimestampField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 6]) -> Result<Self, Self::Err> {
        Ok(Self(six_byte_hex_to_unix_ms(src)))
    }
}

/// The timestamp when a message such as order and trade received.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReceivedTimestampField(pub u64);

impl ReceivedTimestampField {
    /// Create a new `ReceivedTimestamp` from the current time.
    pub fn new_from_now() -> FieldResult<Self> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        debug_assert!(u64::try_from(now.as_millis()).is_ok());
        let now_sec = now.as_millis() as u64;

        Ok(Self(now_sec))
    }
}

impl FieldSerializer<6> for ReceivedTimestampField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 6], Self::Err> {
        Ok(unix_ms_to_six_byte_hex(self.0))
    }
}

impl FieldDeserializer<6> for ReceivedTimestampField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 6]) -> Result<Self, Self::Err> {
        Ok(Self(six_byte_hex_to_unix_ms(src)))
    }
}

/// The exchange type of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExchangeTypeField(pub Exchange);

impl ExchangeTypeField {
    pub fn try_from_str(str: &str) -> FieldResult<Self> {
        let exchange = Exchange::from_str(str)
            .map_err(|_| FieldError::UnimplementedExchange(Either::Left(str.to_string())))?;
        Ok(Self(exchange))
    }
}

impl FieldSerializer<1> for ExchangeTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([self.0 as u8])
    }
}

impl FieldDeserializer<1> for ExchangeTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0] as usize;
        let name = Exchange::from_repr(bit)
            .ok_or(FieldError::UnimplementedExchange(either::Right(bit)))?;

        Ok(Self(name))
    }
}

/// The market type of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketTypeField(pub MarketType);

impl FieldSerializer<1> for MarketTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        let bit = *MARKET_TYPE_BIT.get_by_left(&self.0).unwrap_or(&0);

        Ok([bit])
    }
}

impl FieldDeserializer<1> for MarketTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0];

        let name = MARKET_TYPE_BIT
            .get_by_right(&bit)
            .unwrap_or(&MarketType::Unknown);

        Ok(Self(*name))
    }
}

/// The type of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageTypeField(pub MessageType);

impl FieldSerializer<1> for MessageTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([bit_serialize_message_type(self.0)])
    }
}

impl FieldDeserializer<1> for MessageTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        Ok(Self(bit_deserialize_message_type(src[0])))
    }
}

/// The [`TradeSide`] of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeSideField(pub TradeSide);

impl FieldSerializer<1> for TradeSideField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([bit_serialize_trade_side(self.0)])
    }
}

impl FieldDeserializer<1> for TradeSideField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        Ok(Self(bit_deserialize_trade_side(src[0])?))
    }
}

/// Exchange-specific trading symbol or id, recognized by RESTful API.
pub type Symbol = u16;
/// Unified pair, base/quote, e.g., `BTC/USDT`.
pub type Pair = String;

/// The symbol of a message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolPairField(pub Symbol, pub Pair);

impl SymbolPairField {
    pub fn from_pair(pair: &str) -> Self {
        let symbol = *SYMBOL_PAIR.get_by_right(&pair).unwrap_or(&0);

        Self(symbol, pair.to_string())
    }
}

impl FieldSerializer<2> for SymbolPairField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 2], Self::Err> {
        Ok(self.0.to_be_bytes())
    }
}

impl FieldDeserializer<2> for SymbolPairField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 2]) -> Result<Self, Self::Err> {
        let symbol: Symbol = u16::from_be_bytes(*src);
        let pair: Pair = SYMBOL_PAIR
            .get_by_left(&symbol)
            .unwrap_or(&"UNKNOWN")
            .to_string();

        Ok(Self(symbol, pair))
    }
}

/// The info type (`asks` or `bids`) of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfoTypeField(pub InfoType);

impl InfoTypeField {
    pub fn try_from_str(str: &str) -> FieldResult<Self> {
        let exchange = InfoType::from_str(str)
            .map_err(|_| FieldError::UnimplementedInfoType(Either::Left(str.to_string())))?;
        Ok(Self(exchange))
    }
}

impl FieldSerializer<1> for InfoTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([self.0 as u8])
    }
}

impl FieldDeserializer<1> for InfoTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0] as usize;
        let name = InfoType::from_repr(bit)
            .ok_or(FieldError::UnimplementedInfoType(Either::Right(bit)))?;

        Ok(Self(name))
    }
}

/// The period of a message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeriodField(pub String);

impl FieldSerializer<1> for PeriodField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        let bit = *PERIOD
            .get_by_left(self.0.as_str())
            .ok_or_else(|| FieldError::UnimplementedPeriod(Either::Left(self.0.to_string())))?;

        Ok([bit])
    }
}

impl FieldDeserializer<1> for PeriodField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0] as u8;

        let name = PERIOD
            .get_by_right(&bit)
            .ok_or(FieldError::UnimplementedPeriod(Either::Right(bit)))?;

        Ok(Self(name.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PriceDataField {
    /// 價格
    ///
    /// NOTE: crypto-crawler 是用浮點數儲存價格的。
    /// 這可能造成非常嚴重的誤差（0.1+0.2=0.300000004），
    /// 因此是 Bug，遲早要改成 String。
    pub price: String,

    /// 基本量
    ///
    /// NOTE: crypto-crawler 是用浮點數儲存價格的。
    /// 這可能造成非常嚴重的誤差（0.1+0.2=0.300000004），
    /// 因此是 Bug，遲早要改成 String。
    pub quantity_base: String,
}

impl FieldSerializer<10> for PriceDataField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 10], Self::Err> {
        let mut bytes = [0; 10];

        bytes[..5].copy_from_slice(&u32::encode_bytes(&self.price)?);
        bytes[5..].copy_from_slice(&u32::encode_bytes(&self.quantity_base)?);

        Ok(bytes)
    }
}

impl FieldDeserializer<10> for PriceDataField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 10]) -> Result<Self, Self::Err> {
        let price = arrayref::array_ref![src, 0, 5];
        let quantity_base = arrayref::array_ref![src, 5, 5];

        Ok(Self {
            price: u32::decode_bytes(price).to_string(),
            quantity_base: u32::decode_bytes(quantity_base).to_string(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FieldError {
    #[error("data/types error: {0}")]
    DataTypesError(#[from] DataTypesError),

    #[error("hex encode/decode error: {0}")]
    HexDataError(#[from] HexDataError),

    #[error("failed to read {1} bytes from reader")]
    ReadFromReaderFailed(std::io::Error, usize),

    #[error("failed to get system time: {0}")]
    SystemTimeError(#[from] SystemTimeError),

    #[error("this exchange has not been implemented: {0:?}")]
    UnimplementedExchange(Either<String, usize>),

    #[error("this info type has not been implemented: {0:?}")]
    UnimplementedInfoType(Either<String, usize>),

    #[error("this period has not been implemented: {0:?}")]
    UnimplementedPeriod(Either<String, u8>),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),
}

pub type FieldResult<T> = Result<T, FieldError>;

#[cfg(test)]
mod tests {
    use crate::data::{
        fields::InfoTypeField,
        serializer::FieldDeserializer,
        types::{Exchange, InfoType},
    };

    use super::ExchangeTypeField;

    #[test]
    fn test_exchange_expr_try_from_str() {
        assert_eq!(
            ExchangeTypeField::try_from_str("crypto").unwrap().0,
            Exchange::Crypto
        );
        assert_eq!(ExchangeTypeField::try_from_str("ftx").unwrap().0, Exchange::Ftx);
        assert_eq!(
            ExchangeTypeField::try_from_str("binance").unwrap().0,
            Exchange::Binance
        );
        assert_eq!(
            ExchangeTypeField::try_from_str("huobi").unwrap().0,
            Exchange::Huobi
        );
        assert_eq!(
            ExchangeTypeField::try_from_str("kucoin").unwrap().0,
            Exchange::Kucoin
        );
        assert_eq!(ExchangeTypeField::try_from_str("okx").unwrap().0, Exchange::Okx);
    }

    #[test]
    fn test_exchange_expr_from_byte() {
        assert_eq!(
            ExchangeTypeField::deserialize(&[1]).unwrap().0,
            Exchange::Crypto
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[2]).unwrap().0,
            Exchange::Ftx
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[3]).unwrap().0,
            Exchange::Binance
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[8]).unwrap().0,
            Exchange::Huobi
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[10]).unwrap().0,
            Exchange::Kucoin
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[11]).unwrap().0,
            Exchange::Okx
        );
    }

    #[test]
    fn test_info_type_try_from_str() {
        assert_eq!(InfoTypeField::try_from_str("asks").unwrap().0, InfoType::Asks);
        assert_eq!(InfoTypeField::try_from_str("bids").unwrap().0, InfoType::Bids);
    }

    #[test]
    fn test_info_expr_from_byte() {
        assert_eq!(InfoTypeField::deserialize(&[1]).unwrap().0, InfoType::Asks);
        assert_eq!(InfoTypeField::deserialize(&[2]).unwrap().0, InfoType::Bids);
    }
}
