//! The bbo-related operations.

use std::io::{BufRead, BufReader};

use crypto_msg_parser::BboMsg;
use rust_decimal::prelude::ToPrimitive;

use super::{
    fields::{
        ExchangeTimestampRepr, ExchangeTypeRepr, MarketTypeRepr, MessageTypeRepr, ReadExt,
        ReceivedTimestampRepr, StructureError, SymbolPairRepr,
    },
    hex::{HexDataError, NumToBytesExt},
};

/// Encode a [`BboMsg`] to bytes.
pub fn encode_bbo(bbo: &BboMsg) -> BboResult<Vec<u8>> {
    // This data should have 41 bytes.
    let mut bytes = Vec::<u8>::with_capacity(41);

    // 1. 交易所时间戳: 8 字节
    bytes.extend_from_slice(&ExchangeTimestampRepr(bbo.timestamp).to_bytes());

    // 2. 收到时间戳: 8 字节
    bytes.extend_from_slice(&ReceivedTimestampRepr::try_new_from_now()?.to_bytes());

    // 3. EXCHANGE: 1 字节
    bytes.extend_from_slice(&ExchangeTypeRepr::try_from_str(&bbo.exchange)?.to_bytes());

    // 4. MARKET_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MarketTypeRepr(bbo.market_type).to_bytes());

    // 5. MESSAGE_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MessageTypeRepr(bbo.msg_type).to_bytes());

    // 6. SYMBOL: 2 字节信息标识
    bytes.extend_from_slice(&SymbolPairRepr::from_pair(&bbo.pair).to_bytes());

    // 7. asks price(5)、quant(5)
    bytes.extend_from_slice(&u32::encode_bytes(&bbo.ask_price.to_string())?);
    bytes.extend_from_slice(&u32::encode_bytes(&bbo.ask_quantity_base.to_string())?);

    // 8. bids price(5)、quant(5)
    bytes.extend_from_slice(&u32::encode_bytes(&bbo.bid_price.to_string())?);
    bytes.extend_from_slice(&u32::encode_bytes(&bbo.bid_quantity_base.to_string())?);

    Ok(bytes)
}

/// Decode the specified bytes to a [`BboMsg`].
pub fn decode_bbo(payload: &[u8]) -> BboResult<BboMsg> {
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

    // 7. asks price(5)、quant(5)
    let ask_price = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| BboError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    let ask_quantity_base = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| BboError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    // 8. bids price(5)、quant(5)
    let bid_price = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| BboError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    let bid_quantity_base = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| BboError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    Ok(BboMsg {
        exchange: exchange_type.to_string(),
        market_type,
        msg_type,
        pair: pair.to_string(),
        symbol: symbol.to_string(),
        timestamp: exchange_timestamp,
        ask_price,
        ask_quantity_base,
        ask_quantity_quote: 0.0,
        ask_quantity_contract: None,
        bid_price,
        bid_quantity_base,
        bid_quantity_quote: 0.0,
        bid_quantity_contract: None,
        id: None,
        json: String::new(),
    })
}

#[derive(thiserror::Error, Debug)]
pub enum BboError {
    #[error("data/hex error: {0}")]
    HexDataError(#[from] HexDataError),

    #[error("structure error: {0}")]
    StructureError(#[from] StructureError),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),
}

pub type BboResult<T> = Result<T, BboError>;
