//! The orderbook-related operations.
//! 
//! Note: we just copy it from `crypto-market`, and we may not test it well.

use std::{time::{SystemTime, SystemTimeError}, collections::HashMap, sync::atomic::{AtomicUsize, Ordering}};

use crypto_crawler::MarketType;
use crypto_msg_parser::{OrderBookMsg, Order};
use either::Either;
use rust_decimal::prelude::ToPrimitive;

use super::{order::{OrderType, get_orders}, hex::{long_to_hex, hex_to_byte, HexDataError, encode_num_to_bytes, decode_bytes_to_num}, types::{EXCHANGE, MARKET_TYPE_BIT, bit_serialize_message_type, SYMBLE, INFOTYPE, bit_deserialize_message_type}};

pub fn generate_diff(old: &OrderBookMsg, latest: &OrderBookMsg) -> OrderBookMsg {
    let mut diff = OrderBookMsg {
        asks: vec![],
        bids: vec![],
        exchange: latest.exchange.clone(),
        market_type: latest.market_type,
        symbol: latest.symbol.clone(),
        pair: latest.pair.clone(),
        msg_type: latest.msg_type,
        timestamp: latest.timestamp,
        snapshot: latest.snapshot,
        seq_id: latest.seq_id,
        prev_seq_id: latest.prev_seq_id,
        json: latest.json.clone(),
    };
    diff.asks = get_orders(&old.asks, &latest.asks, OrderType::Ask);
    diff.bids = get_orders(&old.bids, &latest.bids, OrderType::Bid);
    diff
}

pub fn encode_orderbook(orderbook: &OrderBookMsg) -> OrderbookResult<Vec<u8>> {
    let mut orderbook_bytes = Vec::<u8>::new();

    // 我們接下來會使用 block 讓一個 scope 的變數乾淨一點。

    // 1. 交易所时间戳: 6 or 8 
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
            .ok_or_else(|| OrderbookError::UnimplementedExchange(either::Left(exchange_str.to_string())))?;
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

    // 7. ask & bid
    {
        let markets = {
            let mut markets = HashMap::new();

            markets.insert("asks", &orderbook.asks);
            markets.insert("bids", &orderbook.bids);

            markets
        };

        for (k, order_list) in markets {
            // 7-1. 字节信息标识
            {
                let info_type_bit = INFOTYPE.get_by_left(k).expect("should be either 'asks' or 'bids'");
                orderbook_bytes.push(*info_type_bit);
            }

            // 7-2. 字节信息体的长度
            {
                let list_len = (order_list.len() * 10) as i64;

                let list_len_hex = long_to_hex(list_len);
                let list_len_hex = format!("{:0>4}", list_len_hex);

                let list_len_hex_byte = hex_to_byte(&list_len_hex)?;
                orderbook_bytes.extend_from_slice(&list_len_hex_byte);
            }

            // 7-3 loops
            for order in order_list {
                // 7-3: data(price(5)、quant(5)) 10*dataLen BYTE[10*dataLen] 信息体
                let price = order.price;
                let quantity_base = order.quantity_base;

                let price_bytes = encode_num_to_bytes(&price.to_string())?;
                let quantity_base_bytes = encode_num_to_bytes(&quantity_base.to_string())?;

                orderbook_bytes.extend_from_slice(&price_bytes);
                orderbook_bytes.extend_from_slice(&quantity_base_bytes);
            }
        }
    }

    // let compressed = compress_to_vec(&bytes, 6);
    // println!("compressed from {} to {}", data.len(), compressed.len());
    Ok(orderbook_bytes)
}

pub fn decode_orderbook(payload: &[u8]) -> OrderbookResult<OrderBookMsg> {
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
            .ok_or(OrderbookError::UnimplementedExchange(either::Right(bit)))?;
        
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

    // 7. ask & bid
    let (asks, bids) = {
        let mut asks: Vec<Order> = Vec::new();
        let mut bids: Vec<Order> = Vec::new();

        while data_byte_ptr.load(Ordering::SeqCst) < payload.len() {
            // 7-1. 字节信息标识
            let data_type_flag = getseek(1)[0].to_be();

            // 7-2. 字节信息体的长度
            let info_len = {
                let raw = getseek(2);

                let mut dst = [0u8; 2];
                dst.copy_from_slice(raw);

                let mut info_len = u16::from_be_bytes(dst);
                info_len /= 10; // 每 10 bits 為一個資料單位

                info_len
            };

            // 7-3.
            for _ in 0..info_len {
                // 7-3: data(price(5)、quant(5)) 10*dataLen BYTE[10*dataLen] 信息体

                let price = {
                    let raw_bytes = getseek(5);
                    decode_bytes_to_num(raw_bytes)
                        .to_f64()
                        .ok_or_else(|| OrderbookError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
                };

                let quantity_base = {
                    let raw_bytes = getseek(5);
                    decode_bytes_to_num(raw_bytes)
                        .to_f64()
                        .ok_or_else(|| OrderbookError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
                };

                let order = Order {
                    price,
                    quantity_base,
                    quantity_quote: 0.0,
                    quantity_contract: None,
                };

                match data_type_flag {
                    // ask
                    1 => asks.push(order),
                    // bid
                    2 => bids.push(order),
                    // unexpected value
                    _ => panic!("unexpected value"),
                }
            }
        }

        (asks, bids)
    };

    let orderbook = OrderBookMsg {
        exchange: exchange_name.to_string(),
        market_type: market_type_name,
        symbol: symble_pair.to_string(),
        pair: symble_pair.to_string(),
        msg_type: message_type_name,
        timestamp: exchange_timestamp as i64,
        seq_id: None,
        prev_seq_id: None,
        asks,
        bids,
        snapshot: true,
        json: String::new(),
    };

    Ok(orderbook)
}

#[derive(thiserror::Error, Debug)]
pub enum OrderbookError {
    #[error("data/hex error: {0}")]
    HexDataError(#[from] HexDataError),

    #[error("failed to get system time: {0}")]
    SystemTimeError(#[from] SystemTimeError),

    #[error("this exchange has not been implemented: {0:?}")]
    UnimplementedExchange(Either<String, u8>),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),
}

pub type OrderbookResult<T> = Result<T, OrderbookError>;
