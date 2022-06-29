//! The trade-related operations.

use std::{time::{SystemTime, SystemTimeError}, sync::atomic::AtomicUsize};

use crypto_crawler::MarketType;
use crypto_msg_parser::TradeMsg;
use either::Either;
use rust_decimal::prelude::ToPrimitive;

use super::{hex::{long_to_hex, hex_to_byte, HexDataError, encode_num_to_bytes, decode_bytes_to_num}, types::{SYMBLE, bit_serialize_message_type, MARKET_TYPE_BIT, EXCHANGE, bit_serialize_trade_side, bit_deserialize_message_type, DataTypesError, bit_deserialize_trade_side}};

pub fn encode_trade(orderbook: &TradeMsg) -> TradeResult<Vec<u8>> {
    let mut orderbook_bytes = Vec::<u8>::new();

    // 1. 交易所时间戳: 6 or 8 字节时间戳
    {
        let exchange_timestamp = orderbook.timestamp;
        let exchange_timestamp_hex = long_to_hex(exchange_timestamp);
        let exchange_timestamp_hex_byte = hex_to_byte(&exchange_timestamp_hex)?;
        orderbook_bytes.extend_from_slice(&exchange_timestamp_hex_byte);
    }

    // 2. 收到时间戳: 6 or 8 字节时间戳
    {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?;
        let now_ms = now.as_millis();
        let received_timestamp_hex = long_to_hex(now_ms as i64);
        let received_timestamp_hex_byte = hex_to_byte(&received_timestamp_hex)?;
        orderbook_bytes.extend_from_slice(&received_timestamp_hex_byte);
    }

    // 3. EXCHANGE: 1 字节信息标识
    {
        let exchange_str = orderbook.exchange.as_str();
        let exchange_bit = EXCHANGE.get_by_left(exchange_str)
            .ok_or_else(|| TradeError::UnimplementedExchange(either::Left(exchange_str.to_string())))?;
        orderbook_bytes.push(*exchange_bit);
    }

    // 4. MARKET_TYPE: 1 字节信息标识
    {
        let market_type = MARKET_TYPE_BIT.get_by_left(&orderbook.market_type).unwrap_or(&0);
        orderbook_bytes.push(*market_type);
    }

    // 5. MESSAGE_TYPE: 1 字节信息标识
    {
        let message_type = bit_serialize_message_type(orderbook.msg_type);
        orderbook_bytes.push(message_type);
    }

    // 6. SYMBLE: 2 字节信息标识
    {
        let pair = SYMBLE.get_by_left(orderbook.pair.as_str()).unwrap();

        let pair_hex = long_to_hex(*pair as i64);
        let pair_hex = format!("{pair_hex:0>4}");
    
        let pair_hex_byte = hex_to_byte(&pair_hex)?;
        orderbook_bytes.extend_from_slice(&pair_hex_byte);
    }

    // 7. TradeSide: 1 字节信息标识
    {
        let side = bit_serialize_trade_side(orderbook.side);
        orderbook_bytes.push(side);
    }

    // 7#. data(price(5)、quant(5))
    {
        let price = orderbook.price;
        let quantity_base = orderbook.quantity_base;

        let price_bytes = encode_num_to_bytes(&price.to_string())?;
        let quantity_base_bytes = encode_num_to_bytes(&quantity_base.to_string())?;

        orderbook_bytes.extend_from_slice(&price_bytes);
        orderbook_bytes.extend_from_slice(&quantity_base_bytes);
    }

    Ok(orderbook_bytes)
}

pub fn decode_trade(payload: &[u8]) -> TradeResult<TradeMsg> {
    let data_byte_ptr = AtomicUsize::new(0);
    let getseek = |offset| {
        // 副作用: start 會進行 fetch_add!
        let start = data_byte_ptr.fetch_add(offset, std::sync::atomic::Ordering::SeqCst);
        let end = start + offset;
        
        &payload[start..end]
    };

    // 1. 交易所时间戳: 6 or 8 字节时间戳
    let exchange_timestamp = {
        let payload = getseek(6);
        let mut buf = [0u8; 16];
        buf[10..].copy_from_slice(payload);
        
        i128::from_be_bytes(buf)
    };

    // 2. 收到时间戳: 6 or 8 字节时间戳
    // -- 似乎暫時用不到？就略過了。
    getseek(6);
    // let received_timestamp = {
    //     let payload = getseek(6);
    //     let mut buf = [0u8; 16];
    //     buf[10..].copy_from_slice(payload);

    //     i128::from_be_bytes(buf)
    // };

    // 3. EXCHANGE: 1 字节信息标识
    let exchange_name = {
        let bit = getseek(1)[0];

        let name = EXCHANGE
            .get_by_right(&bit)
            .ok_or(TradeError::UnimplementedExchange(either::Right(bit)))?;
        
        *name
    };

    // 4. MARKET_TYPE: 1 字节信息标识
    let market_type_name = {
        let bit = getseek(1)[0];

        let name = MARKET_TYPE_BIT
            .get_by_right(&bit)
            .unwrap_or(&MarketType::Unknown);
        
        *name
    };

    // 5. MESSAGE_TYPE: 1 字节信息标识
    let message_type_name = {
        let bit = getseek(1)[0];

        bit_deserialize_message_type(bit)
    };
    

    // 6. SYMBLE: 2 字节信息标识
    let symble_pair = {
        let raw = getseek(2);
        
        let mut dst = [0u8; 2];
        dst.copy_from_slice(raw);

        let symbol = u16::from_be_bytes(dst) as u8;
        let pair = SYMBLE.get_by_right(&symbol).unwrap_or(&"UNKNOWN");

        *pair
    };

    // 7. TradeSide: 1 字节信息标识
    let trade_side = {
        let bit = getseek(1)[0];

        bit_deserialize_trade_side(bit)?
    };

    // 7#. data(price(5)、quant(5))
    let price = {
        let raw_bytes = getseek(5);
        decode_bytes_to_num(raw_bytes)
            .to_f64()
            .ok_or_else(|| TradeError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    let quantity_base = {
        let raw_bytes = getseek(5);
        decode_bytes_to_num(raw_bytes)
            .to_f64()
            .ok_or_else(|| TradeError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    let orderbook = TradeMsg {
        exchange: exchange_name.to_string(),
        market_type: market_type_name,
        msg_type: message_type_name,
        pair: symble_pair.to_string(),
        symbol: symble_pair.to_string(),
        timestamp: exchange_timestamp as i64,
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

    #[error("type error: {0}")]
    TypeError(#[from] DataTypesError),

    #[error("failed to get system time: {0}")]
    SystemTimeError(#[from] SystemTimeError),

    #[error("this exchange has not been implemented: {0:?}")]
    UnimplementedExchange(Either<String, u8>),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),

    #[error("unexpected data_type_flag: {0}")]
    UnexpectedDataTypeFlag(u8),
}

pub type TradeResult<T> = Result<T, TradeError>;
