//! The trade-related operations.

use std::io::{BufReader, BufRead};

use crypto_msg_parser::TradeMsg;
use rust_decimal::prelude::ToPrimitive;

use super::{
    hex::{decode_bytes_to_num, encode_num_to_bytes, HexDataError},
    fields::{ExchangeTimestampRepr, ReceivedTimestampRepr, ExchangeTypeRepr, MarketTypeRepr, MessageTypeRepr, SymbolPairRepr, TradeSideRepr, StructureError, ReadExt},
};

/// Encode a [`TradeMsg`] to bytes.
pub fn encode_trade(orderbook: &TradeMsg) -> TradeResult<Vec<u8>> {    // Preallocate 21 bytes.
    let mut orderbook_bytes = Vec::<u8>::with_capacity(21);

    // 1. 交易所时间戳: 8 字节
    orderbook_bytes.extend_from_slice(&ExchangeTimestampRepr(orderbook.timestamp).to_bytes());

    // 2. 收到时间戳: 8 字节
    orderbook_bytes.extend_from_slice(&ReceivedTimestampRepr::try_new_from_now()?.to_bytes());

    // 3. EXCHANGE: 1 字节
    orderbook_bytes.extend_from_slice(&ExchangeTypeRepr::try_from_str(&orderbook.exchange)?.to_bytes());

    // 4. MARKET_TYPE: 1 字节信息标识
    orderbook_bytes.extend_from_slice(&MarketTypeRepr(orderbook.market_type).to_bytes());

    // 5. MESSAGE_TYPE: 1 字节信息标识
    orderbook_bytes.extend_from_slice(&MessageTypeRepr(orderbook.msg_type).to_bytes());

    // 6. SYMBOL: 2 字节信息标识
    orderbook_bytes.extend_from_slice(&SymbolPairRepr::from_pair(&orderbook.pair).to_bytes());

    // 7. TradeSide: 1 字节信息标识
    orderbook_bytes.extend_from_slice(&TradeSideRepr(orderbook.side).to_bytes());

    // 7#. data(price(5)、quant(5))
    orderbook_bytes.extend_from_slice(&encode_num_to_bytes(&orderbook.price.to_string())?);
    orderbook_bytes.extend_from_slice(&encode_num_to_bytes(&orderbook.quantity_base.to_string())?);

    Ok(orderbook_bytes)
}

/// Decode the specified bytes to a [`TradeMsg`].
pub fn decode_trade(payload: &[u8]) -> TradeResult<TradeMsg> {
    let mut reader = BufReader::new(payload);

    // 1. 交易所时间戳: 8 字节时间戳
    let exchange_timestamp = ExchangeTimestampRepr::try_from_reader(&mut reader)?.0;

    // 2. 收到时间戳: 8 字节时间戳 (NOT USED)
    reader.consume(8);
    // let received_timestamp = ReceivedTimestampRepr::try_from_reader(&mut reader)?;

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
        decode_bytes_to_num(&raw_bytes).to_f64().ok_or_else(|| {
            TradeError::DecimalConvertF64Failed(raw_bytes.to_vec())
        })?
    };

    let quantity_base = {
        let raw_bytes = reader.read_exact_array()?;
        decode_bytes_to_num(&raw_bytes).to_f64().ok_or_else(|| {
            TradeError::DecimalConvertF64Failed(raw_bytes.to_vec())
        })?
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
