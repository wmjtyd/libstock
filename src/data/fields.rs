//! The prefered encoded data structures of `libstock`.

use std::{
    io::{BufReader, Read},
    str::FromStr,
    time::{SystemTime, SystemTimeError},
};

use crypto_market_type::MarketType;
use crypto_message::TradeSide;
use crypto_msg_type::MessageType;
use either::Either;

use super::{
    hex::{six_byte_hex_to_unix_ms, unix_ms_to_six_byte_hex},
    types::{
        bit_deserialize_message_type, bit_deserialize_trade_side, bit_serialize_message_type,
        bit_serialize_trade_side, DataTypesError, Exchange, InfoType, MARKET_TYPE_BIT, PERIOD,
        SYMBOL_PAIR,
    },
};

pub trait ReadExt: Read {
    /// Read data to a fixed array.
    fn read_exact_array<const LEN: usize>(&mut self) -> StructureResult<[u8; LEN]> {
        let mut payload = [0u8; LEN];
        self.read_exact(&mut payload)
            .map_err(|e| StructureError::ReadFromReaderFailed(e, LEN))?;

        Ok(payload)
    }
}

impl<R: Read> ReadExt for BufReader<R> {}

/// The timestamp of exchange (6 byte).
pub struct ExchangeTimestampRepr(pub i64);

impl ExchangeTimestampRepr {
    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Ok(Self::from_bytes(&reader.read_exact_array()?))
    }

    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        Self(six_byte_hex_to_unix_ms(bytes) as i64)
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        unix_ms_to_six_byte_hex(self.0 as u64)
    }
}

/// The timestamp when a message such as order and trade received.
pub struct ReceivedTimestampRepr(pub u64);

impl ReceivedTimestampRepr {
    /// Create a new `ReceivedTimestamp` from the current time.
    pub fn try_new_from_now() -> StructureResult<Self> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        debug_assert!(u64::try_from(now.as_millis()).is_ok());
        let now_sec = now.as_millis() as u64;

        Ok(Self(now_sec))
    }

    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Ok(Self::from_bytes(&reader.read_exact_array()?))
    }

    pub fn from_bytes(bytes: &[u8; 6]) -> Self {
        Self(six_byte_hex_to_unix_ms(bytes))
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        unix_ms_to_six_byte_hex(self.0)
    }
}

/// The exchange type of a message.
pub struct ExchangeTypeRepr(pub Exchange);

impl ExchangeTypeRepr {
    pub fn try_from_str(str: &str) -> StructureResult<Self> {
        let exchange = Exchange::from_str(str)
            .map_err(|_| StructureError::UnimplementedExchange(Either::Left(str.to_string())))?;
        Ok(Self(exchange))
    }

    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Self::try_from_bytes(&reader.read_exact_array()?)
    }

    pub fn try_from_bytes(bytes: &[u8; 1]) -> StructureResult<Self> {
        let bit = bytes[0] as usize;
        let name = Exchange::from_repr(bit)
            .ok_or(StructureError::UnimplementedExchange(either::Right(bit)))?;

        Ok(Self(name))
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [self.0 as u8]
    }
}

/// The market type of a message.
pub struct MarketTypeRepr(pub MarketType);

impl MarketTypeRepr {
    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Ok(Self::from_bytes(&reader.read_exact_array()?))
    }

    pub fn from_bytes(bytes: &[u8; 1]) -> Self {
        let bit = bytes[0];

        let name = MARKET_TYPE_BIT
            .get_by_right(&bit)
            .unwrap_or(&MarketType::Unknown);

        Self(*name)
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        let bit = *MARKET_TYPE_BIT.get_by_left(&self.0).unwrap_or(&0);

        [bit]
    }
}

/// The type of a message.
pub struct MessageTypeRepr(pub MessageType);

impl MessageTypeRepr {
    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Ok(Self::from_bytes(&reader.read_exact_array()?))
    }

    pub fn from_bytes(bytes: &[u8; 1]) -> Self {
        Self(bit_deserialize_message_type(bytes[0]))
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [bit_serialize_message_type(self.0)]
    }
}

/// The representation of [`TradeSide`].
pub struct TradeSideRepr(pub TradeSide);

impl TradeSideRepr {
    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Self::try_from_bytes(&reader.read_exact_array()?)
    }

    pub fn try_from_bytes(bytes: &[u8; 1]) -> StructureResult<Self> {
        Ok(Self(bit_deserialize_trade_side(bytes[0])?))
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [bit_serialize_trade_side(self.0)]
    }
}

/// Exchange-specific trading symbol or id, recognized by RESTful API.
pub type Symbol = u16;
/// Unified pair, base/quote, e.g., `BTC/USDT`.
pub type Pair<'a> = &'a str;

/// The symbol of a message.
pub struct SymbolPairRepr<'a>(pub Symbol, pub Pair<'a>);

impl<'a> SymbolPairRepr<'a> {
    pub fn from_pair(pair: &'a str) -> Self {
        let symbol = *SYMBOL_PAIR.get_by_right(&pair).unwrap_or(&0);

        Self(symbol, pair)
    }

    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Ok(Self::from_bytes(&reader.read_exact_array()?))
    }

    pub fn from_bytes(bytes: &[u8; 2]) -> Self {
        let symbol: Symbol = u16::from_be_bytes(*bytes);
        let pair: Pair = SYMBOL_PAIR.get_by_left(&symbol).unwrap_or(&"UNKNOWN");

        Self(symbol, pair)
    }

    pub fn to_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

/// The info type (`asks` or `bids`) of a message.
pub struct InfoTypeRepr(pub InfoType);

impl InfoTypeRepr {
    pub fn try_from_str(str: &str) -> StructureResult<Self> {
        let exchange = InfoType::from_str(str)
            .map_err(|_| StructureError::UnimplementedInfoType(Either::Left(str.to_string())))?;
        Ok(Self(exchange))
    }

    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Self::try_from_bytes(&reader.read_exact_array()?)
    }

    pub fn try_from_bytes(bytes: &[u8; 1]) -> StructureResult<Self> {
        let bit = bytes[0] as usize;
        let name = InfoType::from_repr(bit)
            .ok_or(StructureError::UnimplementedInfoType(Either::Right(bit)))?;

        Ok(Self(name))
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [self.0 as u8]
    }
}

/// The period of a message.
pub struct PeriodRepr<'a>(pub &'a str);

impl<'a> PeriodRepr<'a> {
    pub fn try_from_reader(reader: &mut impl ReadExt) -> StructureResult<Self> {
        Self::try_from_bytes(&reader.read_exact_array()?)
    }

    pub fn try_from_bytes(bytes: &[u8; 1]) -> StructureResult<Self> {
        let bit = bytes[0] as u8;

        let name = PERIOD
            .get_by_right(&bit)
            .ok_or(StructureError::UnimplementedPeriod(Either::Right(bit)))?;

        Ok(Self(*name))
    }

    pub fn try_to_bytes(&self) -> StructureResult<[u8; 1]> {
        let bit = *PERIOD
            .get_by_left(&self.0)
            .ok_or_else(|| StructureError::UnimplementedPeriod(Either::Left(self.0.to_string())))?;

        Ok([bit])
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StructureError {
    #[error("data/types error: {0}")]
    DataTypesError(#[from] DataTypesError),

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
}

pub type StructureResult<T> = Result<T, StructureError>;

#[cfg(test)]
mod tests {
    use crate::data::{
        fields::InfoTypeRepr,
        types::{Exchange, InfoType},
    };

    use super::ExchangeTypeRepr;

    #[test]
    fn test_exchange_expr_from_str() {
        assert_eq!(
            ExchangeTypeRepr::try_from_str("crypto").unwrap().0,
            Exchange::Crypto
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_str("ftx").unwrap().0,
            Exchange::Ftx
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_str("binance").unwrap().0,
            Exchange::Binance
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_str("huobi").unwrap().0,
            Exchange::Huobi
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_str("kucoin").unwrap().0,
            Exchange::Kucoin
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_str("okx").unwrap().0,
            Exchange::Okx
        );
    }

    #[test]
    fn test_exchange_expr_from_byte() {
        assert_eq!(
            ExchangeTypeRepr::try_from_bytes(&[1]).unwrap().0,
            Exchange::Crypto
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_bytes(&[2]).unwrap().0,
            Exchange::Ftx
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_bytes(&[3]).unwrap().0,
            Exchange::Binance
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_bytes(&[8]).unwrap().0,
            Exchange::Huobi
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_bytes(&[10]).unwrap().0,
            Exchange::Kucoin
        );
        assert_eq!(
            ExchangeTypeRepr::try_from_bytes(&[11]).unwrap().0,
            Exchange::Okx
        );
    }

    #[test]
    fn test_info_type_from_str() {
        assert_eq!(
            InfoTypeRepr::try_from_str("asks").unwrap().0,
            InfoType::Asks
        );
        assert_eq!(
            InfoTypeRepr::try_from_str("bids").unwrap().0,
            InfoType::Bids
        );
    }

    #[test]
    fn test_info_expr_from_byte() {
        assert_eq!(
            InfoTypeRepr::try_from_bytes(&[1]).unwrap().0,
            InfoType::Asks
        );
        assert_eq!(
            InfoTypeRepr::try_from_bytes(&[2]).unwrap().0,
            InfoType::Bids
        );
    }
}
