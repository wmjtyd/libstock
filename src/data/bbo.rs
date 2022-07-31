//! The generalized BBO serialization and deserialization module.
//!
//! Currently supported formats:
//!
//! - crypto-crawler-rs's [`BboMsg`]
//!
//! You can still implement your own format by implementing the [`TryFrom`]
//! between your structure and [`BboStructure`].

pub use crypto_message::BboMsg;
use typed_builder::TypedBuilder;

use super::{
    fields::{
        EndOfDataFlag, ExchangeTypeField, FieldError, MarketTypeField, MessageTypeField,
        PriceDataField, SymbolPairField, TimestampField,
    },
    serializer::{
        deserialize_block_builder, serialize_block_builder, StructDeserializer, StructSerializer,
    },
};

/// The structure of BBO.
///
/// You can take advantage of `builder()`
/// to construct some fields automatically.
#[derive(Clone, Debug, PartialEq, Eq, TypedBuilder)]
pub struct BboStructure {
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

    /// 最優賣出報價資訊 (asks)
    pub asks: PriceDataField,

    /// 最優買入報價資訊 (bids)
    pub bids: PriceDataField,

    /// 資料結尾
    #[builder(default)]
    pub end: EndOfDataFlag,
}

impl StructSerializer for BboStructure {
    type Err = BboError;

    fn serialize(&self, writer: &mut impl std::io::Write) -> Result<(), Self::Err> {
        serialize_block_builder!(
            self.exchange_timestamp,
            self.received_timestamp,
            self.exchange_type,
            self.market_type,
            self.message_type,
            self.symbol,
            self.asks,
            self.bids,
            self.end => writer
        );

        Ok(())
    }
}

impl StructDeserializer for BboStructure {
    type Err = BboError;

    fn deserialize(reader: &mut impl std::io::Read) -> Result<Self, Self::Err> {
        deserialize_block_builder!(
            reader =>
            exchange_timestamp,
            received_timestamp,
            exchange_type,
            market_type,
            message_type,
            symbol,
            asks,
            bids,
            end
        )
    }
}

impl TryFrom<&BboMsg> for BboStructure {
    type Error = BboError;

    fn try_from(value: &BboMsg) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .exchange_timestamp(value.timestamp)
            .exchange_type(ExchangeTypeField::try_from_str(&value.exchange)?)
            .market_type(value.market_type)
            .message_type(value.msg_type)
            .symbol(SymbolPairField::from_pair(&value.pair))
            .asks(
                PriceDataField::builder()
                    .price(value.ask_price)
                    .quantity_base(value.ask_quantity_base)
                    .build(),
            )
            .bids(
                PriceDataField::builder()
                    .price(value.bid_price)
                    .quantity_base(value.bid_quantity_base)
                    .build(),
            )
            .build())
    }
}

impl TryFrom<BboStructure> for BboMsg {
    type Error = BboError;

    fn try_from(value: BboStructure) -> Result<Self, Self::Error> {
        let SymbolPairField { symbol, pair } = value.symbol;

        Ok(Self {
            exchange: value.exchange_type.into(),
            market_type: value.market_type.into(),
            msg_type: value.message_type.into(),
            pair,
            symbol: symbol.to_string(),
            timestamp: value.exchange_timestamp.into(),
            ask_price: value.asks.price.try_into()?,
            ask_quantity_base: value.asks.quantity_base.try_into()?,
            ask_quantity_quote: 0.0,
            ask_quantity_contract: None,
            bid_price: value.bids.price.try_into()?,
            bid_quantity_base: value.bids.quantity_base.try_into()?,
            bid_quantity_quote: 0.0,
            bid_quantity_contract: None,
            id: None,
            json: String::new(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BboError {
    #[error("field error: {0}")]
    FieldError(#[from] FieldError),

    #[error("I/O reader/writer error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type BboResult<T> = Result<T, BboError>;

#[cfg(test)]
mod tests {
    use crypto_market_type::MarketType;
    use crypto_message::BboMsg;

    use crate::data::serializer::{StructDeserializer, StructSerializer};

    use super::BboStructure;

    // FIXME: add Clone, PartialEq, Eq, Debug, Hash... to crypto-crawler-rs.
    fn construct_bbomsg(unknown_market_type: bool) -> BboMsg {
        BboMsg {
            exchange: "crypto".into(),
            market_type: {
                if unknown_market_type {
                    crypto_market_type::MarketType::AmericanOption
                } else {
                    crypto_market_type::MarketType::Spot
                }
            },
            symbol: "1".into(),
            pair: "BTC/USDT".into(),
            msg_type: crypto_msg_type::MessageType::BBO,
            timestamp: 12345678,
            json: "".into(),
            bid_price: 1.0,
            bid_quantity_base: 2.0,
            bid_quantity_quote: 0.0,
            bid_quantity_contract: None,
            ask_price: 4.0,
            ask_quantity_base: 5.0,
            ask_quantity_quote: 0.0,
            ask_quantity_contract: None,
            id: Some(114514),
        }
    }

    #[test]
    fn test_bbo_encode_decode() {
        let payload = construct_bbomsg(false);

        let bbo_structure = BboStructure::try_from(&payload).unwrap();
        let mut buffer = Vec::new();

        bbo_structure.serialize(&mut buffer).unwrap();
        let decoded_structure = BboStructure::deserialize(&mut buffer.as_slice()).unwrap();
        let decoded_msg = BboMsg::try_from(decoded_structure).unwrap();

        assert_eq!(payload.exchange, decoded_msg.exchange);
        assert_eq!(payload.market_type, decoded_msg.market_type);
        assert_eq!(payload.symbol, decoded_msg.symbol);
        assert_eq!(payload.pair, decoded_msg.pair);
        assert_eq!(payload.msg_type, decoded_msg.msg_type);
        assert_eq!(payload.timestamp, decoded_msg.timestamp);
        // assert_eq!(payload.json, decoded.json);
        assert_eq!(payload.bid_price, decoded_msg.bid_price);
        assert_eq!(payload.bid_quantity_base, decoded_msg.bid_quantity_base);
        // assert_eq!(payload.bid_quantity_quote, decoded.bid_quantity_quote);
        assert_eq!(payload.ask_price, decoded_msg.ask_price);
        assert_eq!(payload.ask_quantity_base, decoded_msg.ask_quantity_base);
        // assert_eq!(payload.ask_quantity_quote, decoded.ask_quantity_quote);
        // assert_eq!(payload.id, decoded.id);
    }

    #[test]
    fn test_bbo_encode_decode_unknown_markettype() {
        let payload = construct_bbomsg(true);

        let bbo_structure = BboStructure::try_from(&payload).unwrap();
        let mut buffer = Vec::new();

        bbo_structure.serialize(&mut buffer).unwrap();
        let decoded_structure = BboStructure::deserialize(&mut buffer.as_slice()).unwrap();
        let decoded_msg = BboMsg::try_from(decoded_structure).unwrap();

        assert_eq!(payload.exchange, decoded_msg.exchange);
        assert_eq!(MarketType::Unknown, decoded_msg.market_type);
        assert_eq!(payload.symbol, decoded_msg.symbol);
        assert_eq!(payload.pair, decoded_msg.pair);
        assert_eq!(payload.msg_type, decoded_msg.msg_type);
        assert_eq!(payload.timestamp, decoded_msg.timestamp);
        // assert_eq!(payload.json, decoded.json);
        assert_eq!(payload.bid_price, decoded_msg.bid_price);
        assert_eq!(payload.bid_quantity_base, decoded_msg.bid_quantity_base);
        // assert_eq!(payload.bid_quantity_quote, decoded.bid_quantity_quote);
        assert_eq!(payload.ask_price, decoded_msg.ask_price);
        assert_eq!(payload.ask_quantity_base, decoded_msg.ask_quantity_base);
        // assert_eq!(payload.ask_quantity_quote, decoded.ask_quantity_quote);
        // assert_eq!(payload.id, decoded.id);
    }
}

/* 0.3.0 compatible methods */
crate::compat::compat_enc!(
    enc = encode_bbo,
    dec = decode_bbo,
    crawl = BboMsg,
    result = BboResult,
    structure = BboStructure
);
