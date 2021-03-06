//! The trade-related operations.

use std::io::BufReader;

use crypto_message::TradeMsg;
use rust_decimal::prelude::ToPrimitive;

use super::{
    fields::{
        ExchangeTimestampRepr, ExchangeTypeRepr, MarketTypeRepr, MessageTypeRepr, ReadExt,
        ReceivedTimestampRepr, StructureError, SymbolPairRepr, TradeSideRepr,
    },
    hex::{HexDataError, NumToBytesExt},
};

/// Encode a [`TradeMsg`] to bytes.
pub fn encode_trade(trade: &TradeMsg) -> TradeResult<Vec<u8>> {
    // This data should have 32 bytes.
    let mut bytes = Vec::<u8>::with_capacity(32);

    // 1. 交易所时间戳: 6 字节
    bytes.extend_from_slice(&ExchangeTimestampRepr(trade.timestamp).to_bytes());

    // 2. 收到时间戳: 6 字节
    bytes.extend_from_slice(&ReceivedTimestampRepr::try_new_from_now()?.to_bytes());

    // 3. EXCHANGE: 1 字节
    bytes.extend_from_slice(&ExchangeTypeRepr::try_from_str(&trade.exchange)?.to_bytes());

    // 4. MARKET_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MarketTypeRepr(trade.market_type).to_bytes());

    // 5. MESSAGE_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MessageTypeRepr(trade.msg_type).to_bytes());

    // 6. SYMBOL: 2 字节信息标识
    bytes.extend_from_slice(&SymbolPairRepr::from_pair(&trade.pair).to_bytes());

    // 7. TradeSide: 1 字节信息标识
    bytes.extend_from_slice(&TradeSideRepr(trade.side).to_bytes());

    // 7#. data(price(5)、quant(5))
    bytes.extend_from_slice(&u32::encode_bytes(&trade.price.to_string())?);
    bytes.extend_from_slice(&u32::encode_bytes(&trade.quantity_base.to_string())?);

    Ok(bytes)
}

/// Decode the specified bytes to a [`TradeMsg`].
pub fn decode_trade(payload: &[u8]) -> TradeResult<TradeMsg> {
    let mut reader = BufReader::new(payload);

    // 1. 交易所时间戳: 6 字节时间戳
    let exchange_timestamp = ExchangeTimestampRepr::try_from_reader(&mut reader)?.0;

    // 2. 收到时间戳: 6 字节时间戳 (NOT USED)
    ReceivedTimestampRepr::try_from_reader(&mut reader)?;

    // 3. EXCHANGE: 1 字节信息标识
    let exchange_type = ExchangeTypeRepr::try_from_reader(&mut reader)?.0;

    // 4. MARKET_TYPE: 1 字节信息标识
    let market_type = MarketTypeRepr::try_from_reader(&mut reader)?.0;

    // 5. MESSAGE_TYPE: 1 字节信息标识
    let msg_type = MessageTypeRepr::try_from_reader(&mut reader)?.0;

    // 6. SYMBOL_PAIR: 2 字节信息标识
    let SymbolPairRepr(symbol, pair) = SymbolPairRepr::try_from_reader(&mut reader)?;

    // 7. TradeSide: 1 字节信息标识
    let trade_side = TradeSideRepr::try_from_reader(&mut reader)?.0;

    // 7#. data(price(5)、quant(5))
    let price = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| TradeError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    let quantity_base = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| TradeError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    let orderbook = TradeMsg {
        exchange: exchange_type.to_string(),
        market_type,
        msg_type,
        pair: pair.to_string(),
        symbol: symbol.to_string(),
        timestamp: exchange_timestamp,
        side: trade_side,
        price,
        quantity_base,
        quantity_quote: 0.0,
        quantity_contract: None,
        trade_id: String::new(),
        json: String::new(),
    };

    Ok(orderbook)
}

#[derive(thiserror::Error, Debug)]
pub enum TradeError {
    #[error("data/hex error: {0}")]
    HexDataError(#[from] HexDataError),

    #[error("structure error: {0}")]
    StructureError(#[from] StructureError),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),
}

pub type TradeResult<T> = Result<T, TradeError>;
