//! The utilities such as serialization and deserialization
//! for `crypto-market`'s types.

use bimap::BiMap;
use crypto_crawler::{MarketType, MessageType};
use crypto_msg_parser::TradeSide;

macro_rules! create_bimap {
    ($idn:ident { $lt:ty => $rt:ty, $($l:expr => $r:expr,)* }) => {
        pub static $idn: once_cell::sync::Lazy<BiMap<$lt, $rt>> = once_cell::sync::Lazy::new(|| {
            let mut map = BiMap::new();
            $(map.insert($l, $r);)*
            map
        });
    }
}

create_bimap!(EXCHANGE {
    &'static str => u8,
    "crypto" => 1,
    "ftx" => 2,
    "binance" => 3,
});

create_bimap!(SYMBLE {
    &'static str => u8,
    "BTC/USDT" => 1,
    "BTC/USD" => 2,
    "USDT/USD" => 3,
});

create_bimap!(INFOTYPE {
    &'static str => u8,
    "asks" => 1,
    "bids" => 2,
});

create_bimap!(MARKET_TYPE_BIT {
    MarketType => u8,
    MarketType::Spot => 1,
    MarketType::LinearFuture => 2,
    MarketType::InverseFuture => 3,
    MarketType::LinearSwap => 4,
    MarketType::InverseSwap => 5,
    MarketType::EuropeanOption => 6,
    MarketType::QuantoFuture => 7,
    MarketType::QuantoSwap => 8,
    // Default: MarketType::Unknown => 0,
});

/// Serialize [`MessageType`] to 1 bit identifier.
pub fn bit_serialize_message_type(mt: MessageType) -> u8 {
    match mt {
        MessageType::Trade => 1,
        MessageType::BBO => 2,
        MessageType::L2TopK => 3,
        MessageType::L2Snapshot => 4,
        MessageType::L2Event => 5,
        MessageType::L3Snapshot => 6,
        MessageType::L3Event => 7,
        MessageType::Ticker => 8,
        MessageType::Candlestick => 9,
        MessageType::OpenInterest => 10,
        MessageType::FundingRate => 11,
        MessageType::LongShortRatio => 12,
        MessageType::TakerVolume => 13,
        _ => 0,
    }
}

/// Deserialize a 1 bit identifier to a [`MessageType`].
pub fn bit_deserialize_message_type(id: u8) -> MessageType {
    match id {
        1 => MessageType::Trade,
        2 => MessageType::BBO,
        3 => MessageType::L2TopK,
        4 => MessageType::L2Snapshot,
        5 => MessageType::L2Event,
        6 => MessageType::L3Snapshot,
        7 => MessageType::L3Event,
        8 => MessageType::Ticker,
        9 => MessageType::Candlestick,
        10 => MessageType::OpenInterest,
        11 => MessageType::FundingRate,
        _ => MessageType::Other,
    }
}

/// Serialize [`TradeSide`] to 1 bit identifier.
pub fn bit_serialize_trade_side(side: TradeSide) -> u8 {
    match side {
        TradeSide::Buy => 1,
        TradeSide::Sell => 2,
    }
}

/// Deserialize a 1 bit identifier to a [`TradeSide`].
pub fn bit_deserialize_trade_side(id: u8) -> DataTypesResult<TradeSide> {
    Ok(match id {
        1 => TradeSide::Buy,
        2 => TradeSide::Sell,
        _ => return Err(DataTypesError::UnexpectedTradeSide(id)),
    })
}

#[derive(thiserror::Error, Debug)]
pub enum DataTypesError {
    #[error("unexpected trade side ID: {0}")]
    UnexpectedTradeSide(u8),
}

pub type DataTypesResult<T> = Result<T, DataTypesError>;
