//! The module with a field to specify the [`TradeSide`] of a message.
//! See [`TradeSideField`].

pub use crypto_message::TradeSide;

use super::abstracts::derive_interop_converters;
use super::{Field, FieldDeserializer, FieldError, FieldResult, FieldSerializer};

/// The [`TradeSide`] of a message (1 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeSideField(pub TradeSide);

impl FieldSerializer<1> for TradeSideField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([bit_serialize_trade_side(self.0)])
    }
}

impl FieldDeserializer<1> for TradeSideField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        Ok(Self(bit_deserialize_trade_side(src[0])?))
    }
}

derive_interop_converters!(TradeSideField, TradeSide);

impl Field<1> for TradeSideField {}

/// Serialize [`TradeSide`] to 1 bit identifier.
fn bit_serialize_trade_side(side: TradeSide) -> u8 {
    match side {
        TradeSide::Buy => 1,
        TradeSide::Sell => 2,
    }
}

/// Deserialize a 1 bit identifier to a [`TradeSide`].
fn bit_deserialize_trade_side(id: u8) -> FieldResult<TradeSide> {
    Ok(match id {
        1 => TradeSide::Buy,
        2 => TradeSide::Sell,
        _ => Err(FieldError::UnexpectedTradeSide(id))?,
    })
}
