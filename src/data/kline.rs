//! The kline-related operations.

// TODO: change to CandlestickMsg
pub use crypto_message::KlineMsg;
use typed_builder::TypedBuilder;

use super::fields::{
    EndOfDataFlag,
    ExchangeTypeField,
    FieldError,
    KlineIndicatorsField,
    MarketTypeField,
    MessageTypeField,
    PeriodField,
    SymbolPairField,
    TimestampField,
};
use super::serializer::{
    deserialize_block_builder,
    serialize_block_builder,
    StructDeserializer,
    StructSerializer,
};

/// The structure of a K-line (also known as Candlestick).
#[derive(Clone, Debug, PartialEq, Eq, TypedBuilder)]
pub struct KlineStructure {
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

    /// PERIOD
    #[builder(setter(into))]
    pub period: PeriodField,

    /// K 線指標
    pub indicator: KlineIndicatorsField,

    /// 資料結尾
    #[builder(default)]
    pub end: EndOfDataFlag,
}

impl StructSerializer for KlineStructure {
    type Err = KlineError;

    fn serialize(&self, writer: &mut impl std::io::Write) -> Result<(), Self::Err> {
        serialize_block_builder!(
            self.exchange_timestamp,
            self.received_timestamp,
            self.exchange_type,
            self.market_type,
            self.message_type,
            self.symbol,
            self.period,
            self.indicator,
            self.end
            => writer
        );

        Ok(())
    }
}

impl StructDeserializer for KlineStructure {
    type Err = KlineError;

    fn deserialize(reader: &mut impl std::io::Read) -> Result<Self, Self::Err> {
        deserialize_block_builder!(
            reader =>
            exchange_timestamp,
            received_timestamp,
            exchange_type,
            market_type,
            message_type,
            symbol,
            period,
            indicator,
            end
        )
    }
}

impl TryFrom<&KlineMsg> for KlineStructure {
    type Error = KlineError;

    fn try_from(value: &KlineMsg) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .exchange_timestamp(value.timestamp)
            .exchange_type(ExchangeTypeField::try_from_str(&value.exchange)?)
            .market_type(value.market_type)
            .message_type(value.msg_type)
            .symbol(SymbolPairField::from_pair(&value.pair))
            .period(value.period.clone())
            .indicator(
                KlineIndicatorsField::builder()
                    .open(value.open)
                    .high(value.high)
                    .low(value.low)
                    .close(value.close)
                    .volume(value.volume)
                    .build(),
            )
            .build())
    }
}

impl TryFrom<KlineStructure> for KlineMsg {
    type Error = KlineError;

    fn try_from(value: KlineStructure) -> Result<Self, Self::Error> {
        let SymbolPairField { symbol, pair } = value.symbol;

        Ok(Self {
            exchange: value.exchange_type.into(),
            market_type: value.market_type.into(),
            symbol: symbol.to_string(),
            pair,
            msg_type: value.message_type.into(),
            timestamp: value.exchange_timestamp.into(),
            json: String::new(),
            open: value.indicator.open.try_into()?,
            high: value.indicator.high.try_into()?,
            low: value.indicator.low.try_into()?,
            close: value.indicator.close.try_into()?,
            volume: value.indicator.volume.try_into()?,
            period: value.period.into(),
            quote_volume: None,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum KlineError {
    #[error("field error: {0}")]
    FieldError(#[from] FieldError),

    #[error("I/O reader/writer error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type KlineResult<T> = Result<T, KlineError>;

/* 0.3.0 compatible methods */
crate::compat::compat_enc!(
    enc = encode_kline,
    dec = decode_kline,
    crawl = KlineMsg,
    result = KlineResult,
    structure = KlineStructure
);
