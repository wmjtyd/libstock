//! The trade-related operations.

use std::io::BufReader;

use crypto_message::TradeMsg;
use typed_builder::TypedBuilder;

use super::{
    fields::{
        EndOfDataFlag, ExchangeTypeField, MarketTypeField, MessageTypeField, PriceDataField,
        SymbolPairField, TimestampField, TradeSideField,
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
    pub exchange_timestamp: TimestampField,

    /// 收到時間戳
    #[builder(default)]
    pub received_timestamp: TimestampField,

    /// 交易所類型 (EXCHANGE)
    pub exchange_type: ExchangeTypeField,

    /// 市場類型 (MARKET_TYPE)
    pub market_type: MarketTypeField,

    /// 訊息類型 (MESSAGE_TYPE)
    pub message_type: MessageTypeField,

    /// SYMBOL
    pub symbol: SymbolPairField,

    /// 交易方向 (Trade side)
    ///
    /// Buy or Sell?
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
        )
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

impl TryFrom<TradeMsg> for TradeStructure {
    type Error = TradeError;

    fn try_from(msg: TradeMsg) -> Result<Self, Self::Error> {
        // Ok(TradeStructure::builder().exchange_timestamp(msg.timestamp))
    }
}

/// Encode a [`TradeMsg`] to bytes.
pub fn encode_trade(trade: &TradeMsg) -> TradeResult<Vec<u8>> {
    // This data should have 32 bytes.
    let mut bytes = Vec::<u8>::with_capacity(32);

    // 1. 交易所时间戳: 6 字节
    bytes.extend_from_slice(&ExchangeTimestampRepr(trade.timestamp).to_bytes());

    // 2. 收到时间戳: 6 字节
    bytes.extend_from_slice(&ReceivedTimestampRepr::try_new_from_now()?.to_bytes());

    // 3. EXCHANGE: 1 字节
    bytes.extend_from_slice(&ExchangeTypeRepr::try_from_str(&trade.exchange)?.to_bytes());

    // 4. MARKET_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MarketTypeRepr(trade.market_type).to_bytes());

    // 5. MESSAGE_TYPE: 1 字节信息标识
    bytes.extend_from_slice(&MessageTypeRepr(trade.msg_type).to_bytes());

    // 6. SYMBOL: 2 字节信息标识
    bytes.extend_from_slice(&SymbolPairRepr::from_pair(&trade.pair).to_bytes());

    // 7. TradeSide: 1 字节信息标识
    bytes.extend_from_slice(&TradeSideRepr(trade.side).to_bytes());

    // 7#. data(price(5)、quant(5))
    bytes.extend_from_slice(&u32::encode_bytes(&trade.price.to_string())?);
    bytes.extend_from_slice(&u32::encode_bytes(&trade.quantity_base.to_string())?);

    Ok(bytes)
}

/// Decode the specified bytes to a [`TradeMsg`].
pub fn decode_trade(payload: &[u8]) -> TradeResult<TradeMsg> {
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

    // 7. TradeSide: 1 字节信息标识
    let trade_side = TradeSideRepr::try_from_reader(&mut reader)?.0;

    // 7#. data(price(5)、quant(5))
    let price = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| TradeError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
    };

    let quantity_base = {
        let raw_bytes = reader.read_exact_array()?;
        u32::decode_bytes(&raw_bytes)
            .to_f64()
            .ok_or_else(|| TradeError::DecimalConvertF64Failed(raw_bytes.to_vec()))?
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
    #[error("field error: {0}")]
    FieldError(#[from] FieldError),

    #[error("I/O reader/writer error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type TradeResult<T> = Result<T, TradeError>;
