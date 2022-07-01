//! The orderbook-related operations.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use crypto_msg_parser::{Order, OrderBookMsg};
use rust_decimal::prelude::ToPrimitive;

use super::{
    fields::{
        ExchangeTimestampRepr, ExchangeTypeRepr, InfoTypeRepr, MarketTypeRepr, MessageTypeRepr,
        ReadExt, ReceivedTimestampRepr, StructureError, SymbolPairRepr,
    },
    hex::{NumToBytesExt, HexDataError},
    order::{get_orders, OrderType},
    types::InfoType,
};

pub fn generate_diff(old: &OrderBookMsg, latest: &OrderBookMsg) -> OrderBookMsg {
    let asks = get_orders(&old.asks, &latest.asks, OrderType::Ask);
    let bids = get_orders(&old.bids, &latest.bids, OrderType::Bid);

    OrderBookMsg {
        asks,
        bids,
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
    }
}

/// Encode a [`OrderBookMsg`] to bytes.
pub fn encode_orderbook(orderbook: &OrderBookMsg) -> OrderbookResult<Vec<u8>> {
    // Preallocate 21 bytes.
    let mut orderbook_bytes = Vec::<u8>::with_capacity(21);

    // 1. 交易所时间戳: 8 字节
    orderbook_bytes.extend_from_slice(&ExchangeTimestampRepr(orderbook.timestamp).to_bytes());

    // 2. 收到时间戳: 8 字节
    orderbook_bytes.extend_from_slice(&ReceivedTimestampRepr::try_new_from_now()?.to_bytes());

    // 3. EXCHANGE: 1 字节
    orderbook_bytes
        .extend_from_slice(&ExchangeTypeRepr::try_from_str(&orderbook.exchange)?.to_bytes());

    // 4. MARKET_TYPE: 1 字节信息标识
    orderbook_bytes.extend_from_slice(&MarketTypeRepr(orderbook.market_type).to_bytes());

    // 5. MESSAGE_TYPE: 1 字节信息标识
    orderbook_bytes.extend_from_slice(&MessageTypeRepr(orderbook.msg_type).to_bytes());

    // 6. SYMBOL: 2 字节信息标识
    orderbook_bytes.extend_from_slice(&SymbolPairRepr::from_pair(&orderbook.pair).to_bytes());

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
            orderbook_bytes.extend_from_slice(&{ InfoTypeRepr::try_from_str(k)?.to_bytes() });

            // 7-2. 字节信息体的长度
            orderbook_bytes.extend_from_slice(&{
                let list_len = (order_list.len() * 10) as u16;
                list_len.to_be_bytes()
            });

            // 7-3: data(price(5)、quant(5)) 10*dataLen BYTE[10*dataLen] 信息体
            for order in order_list {
                orderbook_bytes.extend_from_slice(&u32::encode_bytes(&order.price.to_string())?);
                orderbook_bytes
                    .extend_from_slice(&u32::encode_bytes(&order.quantity_base.to_string())?);
            }
        }
    }

    // let compressed = compress_to_vec(&bytes, 6);
    // println!("compressed from {} to {}", data.len(), compressed.len());
    Ok(orderbook_bytes)
}

/// Decode the specified bytes to a [`OrderBookMsg`].
pub fn decode_orderbook(payload: &[u8]) -> OrderbookResult<OrderBookMsg> {
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

    // 7. ask & bid
    let (asks, bids) = {
        let mut asks: Vec<Order> = Vec::new();
        let mut bids: Vec<Order> = Vec::new();

        // Check if the data has left.
        //
        // TODO: when `has_data_left` provided, replace the following
        // to 'reader.has_data_left()`!
        while reader.fill_buf().map(|b| !b.is_empty()).unwrap_or(false) {
            // 7-1. 字节信息标识
            let info_type = InfoTypeRepr::try_from_reader(&mut reader)?.0;

            // 7-2. 字节信息体的长度
            let info_len = {
                let data = reader.read_exact_array()?;
                let info_len_raw = u16::from_be_bytes(data);
                info_len_raw / 10 // 每 10 bits 為一個資料單位
            };

            // 7-3: data(price(5)、quant(5)) 10*dataLen BYTE[10*dataLen] 信息体
            for _ in 0..info_len {
                let price = {
                    let raw_bytes = reader.read_exact_array()?;
                    u32::decode_bytes(&raw_bytes).to_f64().ok_or_else(|| {
                        OrderbookError::DecimalConvertF64Failed(raw_bytes.to_vec())
                    })?
                };

                let quantity_base = {
                    let raw_bytes = reader.read_exact_array()?;
                    u32::decode_bytes(&raw_bytes).to_f64().ok_or_else(|| {
                        OrderbookError::DecimalConvertF64Failed(raw_bytes.to_vec())
                    })?
                };

                let order = Order {
                    price,
                    quantity_base,
                    quantity_quote: 0.0,
                    quantity_contract: None,
                };

                match info_type {
                    InfoType::Asks => asks.push(order),
                    InfoType::Bids => bids.push(order),
                }
            }
        }

        (asks, bids)
    };

    Ok(OrderBookMsg {
        exchange: exchange_type.to_string(),
        market_type,
        symbol: symbol.to_string(),
        pair: pair.to_string(),
        msg_type,
        timestamp: exchange_timestamp,
        seq_id: None,
        prev_seq_id: None,
        asks,
        bids,
        snapshot: true,
        json: String::new(),
    })
}

#[derive(thiserror::Error, Debug)]
pub enum OrderbookError {
    #[error("data/hex error: {0}")]
    HexDataError(#[from] HexDataError),

    #[error("structure error: {0}")]
    StructureError(#[from] StructureError),

    #[error("failed to convert the following bytes to f64: {0:?}")]
    DecimalConvertF64Failed(Vec<u8>),
}

pub type OrderbookResult<T> = Result<T, OrderbookError>;
