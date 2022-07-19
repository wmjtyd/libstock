//! The funding rate related operations.

use std::io::BufReader;

use crypto_message::FundingRateMsg;
use rust_decimal::prelude::ToPrimitive;

use super::{
    fields::{
        ExchangeTimestampRepr, ExchangeTypeRepr, MarketTypeRepr, MessageTypeRepr, ReadExt,
        ReceivedTimestampRepr, StructureError, SymbolPairRepr,
    },
    hex::{HexDataError, NumToBytesExt},
};

/// Encode a [`FundingRateMsg`] to bytes.
pub fn encode_funding_rate(funding_rate: &FundingRateMsg) -> FundingRateResult<Vec<u8>> {
    // This data should have 32 bytes.
    let mut bytes = Vec::<u8>::with_capacity(32);

    // 1. 交易所时间戳: 6 字节
    bytes.extend_from_slice(&ExchangeTimestampRepr(funding_rate.timestamp).to_bytes());

    // 2. 收到时间戳: 6 字节
    bytes.extend_from_slice(&ReceivedTimestampRepr::try_new_from_now()?.to_bytes());

    // 3. EXCHANGE: 1 字节
    bytes.extend_from_slice(&ExchangeTypeRepr::try_from_str(&funding_rate.exchange)?.to_bytes());

    // 4. MARKET_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MarketTypeRepr(funding_rate.market_type).to_bytes());

    // 5. MESSAGE_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MessageTypeRepr(funding_rate.msg_type).to_bytes());

    // 6. SYMBOL: 2 字节信息标识
    bytes.extend_from_slice(&SymbolPairRepr::from_pair(&funding_rate.pair).to_bytes());

    // 7. funding_rate: 10 bytes
    bytes.extend_from_slice(&u64::encode_bytes(&funding_rate.funding_rate.to_string())?);

    // 8. funding_time: 6 bytes
    bytes.extend_from_slice(&ExchangeTimestampRepr(funding_rate.funding_time).to_bytes());

    // 9. estimated_rate: 10 bytes
    bytes.extend_from_slice(&u64::encode_bytes(
        &funding_rate.estimated_rate.unwrap().to_string(),
    )?);

    Ok(bytes)
}

/// Decode the specified bytes to a [`FundingRateMsg`].
pub fn decode_funding_rate(payload: &[u8]) -> FundingRateResult<FundingRateMsg> {
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

    // 7. funding_rate: 10 bytes
    let funding_rate = {
        let raw_bytes = reader.read_exact_array()?;
        u64::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| FundingRateError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    // 8. funding_time: 6 bytes
    let funding_time = ExchangeTimestampRepr::try_from_reader(&mut reader)?.0;

    // 9. estimated_rate: 10 bytes
    let estimated_rate = {
        let raw_bytes = reader.read_exact_array()?;
        u64::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| FundingRateError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    Ok(FundingRateMsg {
        exchange: exchange_type.to_string(),
        market_type,
        msg_type,
        pair: pair.to_string(),
        symbol: symbol.to_string(),
        timestamp: exchange_timestamp,
        funding_rate,
        funding_time,
        estimated_rate: Some(estimated_rate),
        json: String::new(),
    })
}

#[derive(thiserror::Error, Debug)]
pub enum FundingRateError {
    #[error("data/hex error: {0}")]
    HexDataError(#[from] HexDataError),

    #[error("structure error: {0}")]
    StructureError(#[from] StructureError),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),
}

pub type FundingRateResult<T> = Result<T, FundingRateError>;
