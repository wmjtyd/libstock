//! The orderbook-related operations.

use std::io::{Read, Write};

pub use crypto_message::OrderBookMsg;
use super::fields::price_data::Order;

use typed_builder::TypedBuilder;

use super::{
    fields::{
        info_type::InfoType, EndOfDataFlag, ExchangeTypeField, FieldError, InfoTypeField,
        MarketTypeField, MessageTypeField, PriceDataField, SymbolPairField, TimestampField,
    },
    order::{get_orders, OrderType},
    serializer::{
        serialize_block_builder, FieldDeserializer, FieldSerializer, StructDeserializer,
        StructSerializer,
    },
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

/// The box storing the direction and the orders.
///
/// This type is designed especially for the [`OrderbookStructure`].
/// As its size is variant, we don't implement it as a [`Field`](super::fields::Field),
/// and its serialization and deserialization method need to be written manually.
#[derive(Clone, Debug, PartialEq, Eq, Hash, TypedBuilder)]
pub struct OrdersBox {
    #[builder(setter(into))]
    direction: InfoTypeField,

    orders: Vec<PriceDataField>,
}

impl OrdersBox {
    /// Calculate the size of `orders`
    /// according to its length.
    ///
    /// As a PriceDataField occupied 10 bytes,
    /// we calculate the size with:
    ///
    /// ```plain
    /// L = The length of Vec<PriceDataField> × 10 bytes
    /// ```
    fn get_orders_size(length: usize) -> usize {
        length * 10
    }

    /// Calculate the length of `orders`
    /// according to its size.
    fn get_orders_length(size: usize) -> usize {
        size / 10
    }

    fn serialize_orders_size(length: usize) -> [u8; 2] {
        (Self::get_orders_size(length) as u16).to_be_bytes()
    }

    fn deserialize_orders_size(src: &[u8; 2]) -> usize {
        Self::get_orders_length(u16::from_be_bytes(*src).into())
    }
}

impl OrdersBox {
    /// Serialize the input and write the whole serialized
    /// content to the writer.
    pub fn serialize_to_writer(&self, writer: &mut impl Write) -> OrderbookResult<()> {
        /* Direction */
        self.direction.serialize_to_writer(writer)??;

        /* Orders's size */
        writer.write_all(&Self::serialize_orders_size(self.orders.len()))?;

        /* Orders */
        for order in &self.orders {
            order.serialize_to_writer(writer)??;
        }

        Ok(())
    }
}

impl OrdersBox {
    /// Read from the writer, and deserialize it.
    pub fn deserialize_from_reader(reader: &mut impl Read) -> OrderbookResult<Self> {
        /* Direction */
        let direction = InfoTypeField::deserialize_from_reader(reader)??;

        /* Orders's size */
        let mut size_buf = [0u8; 2];
        reader.read_exact(&mut size_buf)?;
        let order_len = Self::deserialize_orders_size(&size_buf);

        /* Orders */
        let mut orders = Vec::with_capacity(order_len);

        for _ in 0..order_len {
            let order = PriceDataField::deserialize_from_reader(reader)??;
            orders.push(order);
        }

        Ok(Self { direction, orders })
    }
}

/// The structure of a order book.
///
/// You can take advantage of `builder()`
/// to construct some fields automatically.
#[derive(Clone, Debug, PartialEq, Eq, TypedBuilder)]
pub struct OrderbookStructure {
    /// 交易所時間戳
    #[builder(setter(into))]
    pub exchange_timestamp: TimestampField,

    /// 收到時間戳
    #[builder(default)]
    pub received_timestamp: TimestampField,

    /// 交易所類型 (EXCHANGE)
    #[builder(setter(into))]
    pub exchange_type: ExchangeTypeField,

    /// 市場類型 (MARKET_TYPE)
    #[builder(setter(into))]
    pub market_type: MarketTypeField,

    /// 訊息類型 (MESSAGE_TYPE)
    #[builder(setter(into))]
    pub message_type: MessageTypeField,

    /// SYMBOL
    pub symbol: SymbolPairField,

    /// 賣方 (asks) 的資料
    pub asks: OrdersBox,

    /// 買方 (bids) 的資料
    pub bids: OrdersBox,

    /// 資料結尾
    #[builder(default)]
    pub end: EndOfDataFlag,
}

impl StructSerializer for OrderbookStructure {
    type Err = OrderbookError;

    fn serialize(&self, writer: &mut impl Write) -> Result<(), Self::Err> {
        serialize_block_builder!(
            self.exchange_timestamp,
            self.received_timestamp,
            self.exchange_type,
            self.market_type,
            self.message_type,
            self.symbol
            => writer
        );

        // OrdersBox is not a standard FieldSerializer.
        self.asks.serialize_to_writer(writer)?;
        self.bids.serialize_to_writer(writer)?;

        self.end.serialize_to_writer(writer)??;

        Ok(())
    }
}

impl StructDeserializer for OrderbookStructure {
    type Err = OrderbookError;

    fn deserialize(reader: &mut impl Read) -> Result<Self, Self::Err> {
        let exchange_timestamp = TimestampField::deserialize_from_reader(reader)??;
        let received_timestamp = TimestampField::deserialize_from_reader(reader)??;
        let exchange_type = ExchangeTypeField::deserialize_from_reader(reader)??;
        let market_type = MarketTypeField::deserialize_from_reader(reader)??;
        let message_type = MessageTypeField::deserialize_from_reader(reader)??;
        let symbol = SymbolPairField::deserialize_from_reader(reader)??;

        // OrdersBox is not a standard FieldDeserializer.
        let asks = OrdersBox::deserialize_from_reader(reader)?;
        let bids = OrdersBox::deserialize_from_reader(reader)?;

        let end = EndOfDataFlag::deserialize_from_reader(reader)??;

        Ok(Self {
            exchange_timestamp,
            received_timestamp,
            exchange_type,
            market_type,
            message_type,
            symbol,
            asks,
            bids,
            end,
        })
    }
}

impl TryFrom<&OrderBookMsg> for OrderbookStructure {
    type Error = OrderbookError;

    fn try_from(value: &OrderBookMsg) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .exchange_timestamp(value.timestamp)
            .exchange_type(ExchangeTypeField::try_from_str(&value.exchange)?)
            .market_type(value.market_type)
            .message_type(value.msg_type)
            .symbol(SymbolPairField::from_pair(&value.pair))
            .asks(
                OrdersBox::builder()
                    .direction(InfoType::Asks)
                    .orders(value.asks.iter().map(TryInto::try_into).collect::<Result<Vec<PriceDataField>, _>>()?)
                    .build(),
            )
            .bids(
                OrdersBox::builder()
                    .direction(InfoType::Bids)
                    .orders(value.bids.iter().map(TryInto::try_into).collect::<Result<Vec<PriceDataField>, _>>()?)
                    .build(),
            )
            .build())
    }
}

impl TryFrom<OrderbookStructure> for OrderBookMsg {
    type Error = OrderbookError;

    fn try_from(value: OrderbookStructure) -> Result<Self, Self::Error> {
        let SymbolPairField { symbol, pair } = value.symbol;
        let asks = value.asks.orders.into_iter().map(TryInto::try_into).collect::<Result<Vec<Order>, _>>()?;
        let bids = value.bids.orders.into_iter().map(TryInto::try_into).collect::<Result<Vec<Order>, _>>()?;

        Ok(Self {
            exchange: value.exchange_type.into(),
            market_type: value.market_type.into(),
            symbol: symbol.to_string(),
            pair,
            msg_type: value.message_type.into(),
            timestamp: value.exchange_timestamp.into(),
            snapshot: true,
            asks,
            bids,
            seq_id: None,
            prev_seq_id: None,
            json: String::new()
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OrderbookError {
    #[error("field error: {0}")]
    FieldError(#[from] FieldError),

    #[error("I/O reader/writer error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type OrderbookResult<T> = Result<T, OrderbookError>;

/* 0.3.0 compatible methods */
crate::compat::compat_enc!(
    enc = encode_orderbook,
    dec = decode_orderbook,
    crawl = OrderBookMsg,
    result = OrderbookResult,
    structure = OrderbookStructure
);
