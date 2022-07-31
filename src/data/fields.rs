//! The field (de)serialization module for libstock.

mod abstracts;
mod bimap;

pub mod decimal;
pub mod eod_flag;
pub mod exchange_type;
pub mod info_type;
pub mod market_type;
pub mod message_type;
pub mod period;
pub mod price_data;
pub mod symbol_pair;
pub mod timestamp;
pub mod trade_side;

use super::num::NumError;
use std::time::SystemTimeError;

pub use super::serializer::{FieldDeserializer, FieldSerializer};
pub use either::Either;

pub use decimal::DecimalField;
pub use eod_flag::EndOfDataFlag;
pub use exchange_type::ExchangeTypeField;
pub use info_type::InfoTypeField;
pub use market_type::MarketTypeField;
pub use message_type::MessageTypeField;
pub use period::PeriodField;
pub use price_data::PriceDataField;
pub use symbol_pair::SymbolPairField;
pub use timestamp::TimestampField;
pub use trade_side::TradeSideField;

pub use abstracts::{Field, Interopable};

#[derive(thiserror::Error, Debug)]
pub enum FieldError {
    #[error("unexpected trade side ID: {0}")]
    UnexpectedTradeSide(u8),

    #[error("number encode/decode error: {0}")]
    NumError(#[from] NumError),

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

    #[error("data ended too early (missing \\0 in the end)!")]
    DataEndedTooEarly,
}
pub type FieldResult<T> = Result<T, FieldError>;
