//! The kline-related operations.

use std::io::{BufRead, BufReader};

use crypto_msg_parser::KlineMsg;
use rust_decimal::prelude::ToPrimitive;

use super::{
    fields::{
        ExchangeTimestampRepr, ExchangeTypeRepr, MarketTypeRepr, MessageTypeRepr, PeriodRepr,
        ReadExt, ReceivedTimestampRepr, StructureError, SymbolPairRepr,
    },
    hex::{HexDataError, NumToBytesExt},
};

/// The size of the k-line indicators.
///
/// `[open, high, low, close, volume]`
const KLINE_INDICATOR_SIZE: [usize; 5] = [5, 5, 5, 5, 10];

/// Get the ordered fixed array with k-line indicators.
///
/// The indicators will be ordered in `[open, high, low, close, volume]`.
fn get_kline_indi_array(kline: &KlineMsg) -> [f64; 5] {
    [kline.open, kline.high, kline.low, kline.close, kline.volume]
}

/// Encode a [`KlineMsg`] to bytes.
pub fn encode_kline(kline: &KlineMsg) -> KlineResult<Vec<u8>> {
    // This data should have 47 bytes.
    let mut bytes = Vec::<u8>::with_capacity(47);

    // 1. 交易所时间戳: 8 字节
    bytes.extend_from_slice(&ExchangeTimestampRepr(kline.timestamp).to_bytes());

    // 2. 收到时间戳: 8 字节
    bytes.extend_from_slice(&ReceivedTimestampRepr::try_new_from_now()?.to_bytes());

    // 3. EXCHANGE: 1 字节
    bytes.extend_from_slice(&ExchangeTypeRepr::try_from_str(&kline.exchange)?.to_bytes());

    // 4. MARKET_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MarketTypeRepr(kline.market_type).to_bytes());

    // 5. MESSAGE_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MessageTypeRepr(kline.msg_type).to_bytes());

    // 6. SYMBOL: 2 字节信息标识
    bytes.extend_from_slice(&SymbolPairRepr::from_pair(&kline.pair).to_bytes());

    // 7. PERIOD: 1 字节信息标识
    bytes.extend_from_slice(&PeriodRepr(kline.period.as_str()).try_to_bytes()?);

    // 8. 五個指標 (open (5B), high (5B), low (5B), close (5B)、volume (10B))
    for (idx, price) in get_kline_indi_array(kline).iter().enumerate() {
        // FIXME: this code is too ugly!!
        macro_rules! create_extend_branch {
            ($ty:ty) => {
                bytes.extend_from_slice(&<$ty>::encode_bytes(&price.to_string())?)
            };
        }

        let size = KLINE_INDICATOR_SIZE[idx];

        match size {
            5 => create_extend_branch!(u32),
            10 => create_extend_branch!(u64),
            _ => unreachable!(),
        };
    }

    Ok(bytes)
}

/// Decode the specified bytes to a [`KlineMsg`].
pub fn decode_kline(payload: &[u8]) -> KlineResult<KlineMsg> {
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

    // 7. PERIOD: 1 字节信息标识
    let period = PeriodRepr::try_from_reader(&mut reader)?.0;

    // 8. 五個指標 (open (5B), high (5B), low (5B), close (5B)、volume (10B))
    let mut indicators = [0.0f64; 5];
    for (idx, size) in KLINE_INDICATOR_SIZE.iter().enumerate() {
        // FIXME: this code is too ugly!!
        macro_rules! get_indicators_branch {
            ($ty: ty) => {{
                let raw = reader.read_exact_array()?;
                let indicator = <$ty>::decode_bytes(&raw)
                    .to_f64()
                    .ok_or_else(|| KlineError::DecimalConvertF64Failed(raw.to_vec()))?;

                indicator
            }};
        }

        indicators[idx] = match *size {
            5 => get_indicators_branch!(u32),
            10 => get_indicators_branch!(u64),
            _ => unreachable!(),
        };
    }
    let [open, high, low, close, volume] = indicators;

    Ok(KlineMsg {
        exchange: exchange_type.to_string(),
        market_type,
        msg_type,
        pair: pair.to_string(),
        symbol: symbol.to_string(),
        timestamp: exchange_timestamp as i64,
        open,
        high,
        low,
        close,
        /// base volume
        volume,
        /// m, minute; H, hour; D, day; W, week; M, month; Y, year
        period: period.to_string(),
        /// quote volume
        quote_volume: None,
        json: String::new(),
    })
}

#[derive(thiserror::Error, Debug)]
pub enum KlineError {
    #[error("data/hex error: {0}")]
    HexDataError(#[from] HexDataError),

    #[error("structure error: {0}")]
    StructureError(#[from] StructureError),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),
}

pub type KlineResult<T> = Result<T, KlineError>;
