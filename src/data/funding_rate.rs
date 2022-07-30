//! The funding rate related operations.

use crypto_message::FundingRateMsg;
use rust_decimal::prelude::ToPrimitive;
use typed_builder::TypedBuilder;

use super::{
    fields::{
        DecimalField, EndOfDataFlag, ExchangeTypeField, FieldError, MarketTypeField,
        MessageTypeField, SymbolPairField, TimestampField,
    },
    serializer::{
        deserialize_block_builder, serialize_block_builder, FieldSerializer, StructDeserializer,
        StructSerializer,
    },
};

pub type FundingRateField = DecimalField<10>;

pub type EstimatedRateField = DecimalField<10>;

/// The structure of a funding rate.
///
/// You can take advantage of `builder()`
/// to construct some fields automatically.
#[derive(Clone, Debug, PartialEq, Eq, TypedBuilder)]
pub struct FundingRateStructure {
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

    /// Funding rate
    pub funding_rate: FundingRateField,

    /// Funding time
    pub funding_time: TimestampField,

    /// Estimated rate
    pub estimated_rate: EstimatedRateField,

    /// 資料結尾
    #[builder(default)]
    pub end: EndOfDataFlag,
}

impl StructSerializer for FundingRateStructure {
    type Err = FundingRateError;

    fn serialize(&self, writer: &mut impl std::io::Write) -> Result<(), Self::Err> {
        serialize_block_builder!(
            self.exchange_timestamp,
            self.received_timestamp,
            self.exchange_type,
            self.market_type,
            self.message_type,
            self.symbol,
            self.funding_rate,
            self.funding_time,
            self.estimated_rate,
            self.end
            => writer
        );

        Ok(())
    }
}

impl StructDeserializer for FundingRateStructure {
    type Err = FundingRateError;

    fn deserialize(reader: &mut impl std::io::Read) -> Result<Self, Self::Err> {
        deserialize_block_builder!(
            reader =>
            exchange_timestamp,
            received_timestamp,
            exchange_type,
            market_type,
            message_type,
            symbol,
            funding_rate,
            funding_time,
            estimated_rate,
            end
        )
    }
}

impl TryFrom<&FundingRateMsg> for FundingRateStructure {
    type Error = FundingRateError;

    fn try_from(msg: &FundingRateMsg) -> Result<Self, Self::Error> {
        Ok(Self::builder()
            .exchange_timestamp(TimestampField(msg.timestamp as u64))
            .exchange_type(ExchangeTypeField::try_from_str(&msg.exchange)?)
            .market_type(MarketTypeField(msg.market_type))
            .message_type(MessageTypeField(msg.msg_type))
            .symbol(SymbolPairField::from_pair(&msg.pair))
            .funding_rate(FundingRateField::from(msg.funding_rate))
            .funding_time(TimestampField(msg.funding_time as u64))
            .estimated_rate(EstimatedRateField::from(
                msg.estimated_rate
                    .ok_or(FundingRateError::MissingEstimatedRate)?,
            ))
            .build())
    }
}

impl TryFrom<FundingRateStructure> for FundingRateMsg {
    type Error = FundingRateError;

    fn try_from(s: FundingRateStructure) -> Result<Self, Self::Error> {
        let SymbolPairField { symbol, pair } = s.symbol;

        Ok(Self {
            exchange: s.exchange_type.0.to_string(),
            market_type: s.market_type.0,
            symbol: symbol.to_string(),
            pair,
            msg_type: s.message_type.0,
            timestamp: s.exchange_timestamp.0 as i64,
            funding_rate: s.funding_rate.0.to_f64().expect("overflow?"),
            funding_time: s.funding_time.0 as i64,
            estimated_rate: s.estimated_rate.0.to_f64(),
            json: String::new(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FundingRateError {
    #[error("field error: {0}")]
    FieldError(#[from] FieldError),

    #[error("estimated_rate in FundingRateMsg is None")]
    MissingEstimatedRate,

    #[error("I/O reader/writer error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("The data is ended too early.")]
    NoEndCharacter,
}

pub type FundingRateResult<T> = Result<T, FundingRateError>;
