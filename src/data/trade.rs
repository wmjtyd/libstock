//! The trade-related operations.

pub use crypto_message::TradeMsg;
use typed_builder::TypedBuilder;

use super::{
    fields::{
        EndOfDataFlag, ExchangeTypeField, FieldError, MarketTypeField, MessageTypeField,
        PriceDataField, SymbolPairField, TimestampField, TradeSideField,
    },
    serializer::{
        deserialize_block_builder, serialize_block_builder, StructDeserializer, StructSerializer,
    },
};

/// The structure of a trace data.
///
/// You can take advantage of `builder()`
/// to construct some fields automatically.
#[derive(Clone, Debug, PartialEq, Eq, TypedBuilder)]
pub struct TradeStructure {
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

    /// 交易方向 (Trade side)
    ///
    /// Buy or Sell?
    #[builder(setter(into))]
    pub trade_side: TradeSideField,

    /// 交易價格資訊
    pub trace_price: PriceDataField,

    /// 資料結尾
    #[builder(default)]
    pub end: EndOfDataFlag,
}

impl StructSerializer for TradeStructure {
    type Err = TradeError;

    fn serialize(&self, writer: &mut impl std::io::Write) -> Result<(), Self::Err> {
        serialize_block_builder!(
            self.exchange_timestamp,
            self.received_timestamp,
            self.exchange_type,
            self.market_type,
            self.message_type,
            self.symbol,
            self.trade_side,
            self.trace_price,
            self.end
            => writer
        );

        Ok(())
    }
}

impl StructDeserializer for TradeStructure {
    type Err = TradeError;

    fn deserialize(reader: &mut impl std::io::Read) -> Result<Self, Self::Err> {
        deserialize_block_builder!(
            reader =>
            exchange_timestamp,
            received_timestamp,
            exchange_type,
            market_type,
            message_type,
            symbol,
            trade_side,
            trace_price,
            end
        )
    }
}

impl TryFrom<&TradeMsg> for TradeStructure {
    type Error = TradeError;

    fn try_from(msg: &TradeMsg) -> Result<Self, Self::Error> {
        Ok(TradeStructure::builder()
            .exchange_timestamp(msg.timestamp)
            .exchange_type(ExchangeTypeField::try_from_str(&msg.exchange)?)
            .market_type(msg.market_type)
            .message_type(msg.msg_type)
            .symbol(SymbolPairField::from_pair(&msg.pair))
            .trade_side(msg.side)
            .trace_price(
                PriceDataField::builder()
                    .price(msg.price)
                    .quantity_base(msg.quantity_base)
                    .build(),
            )
            .build())
    }
}

impl TryFrom<TradeStructure> for TradeMsg {
    type Error = TradeError;

    fn try_from(value: TradeStructure) -> Result<Self, Self::Error> {
        let SymbolPairField { symbol, pair } = value.symbol;

        Ok(TradeMsg {
            exchange: value.exchange_type.into(),
            market_type: value.market_type.into(),
            msg_type: value.message_type.into(),
            pair,
            symbol: symbol.to_string(),
            timestamp: value.exchange_timestamp.into(),
            side: value.trade_side.into(),
            price: value.trace_price.price.try_into()?,
            quantity_base: value.trace_price.quantity_base.try_into()?,
            quantity_quote: 0.0,
            quantity_contract: None,
            trade_id: String::new(),
            json: String::new(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TradeError {
    #[error("field error: {0}")]
    FieldError(#[from] FieldError),

    #[error("I/O reader/writer error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type TradeResult<T> = Result<T, TradeError>;

/* 0.3.0 compatible methods */
crate::compat::compat_enc!(
    enc = encode_trade,
    dec = decode_trade,
    crawl = TradeMsg,
    result = TradeResult,
    structure = TradeStructure
);
