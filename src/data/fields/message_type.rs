//! The module with a field to specify the type of a message.
//! See [`MessageTypeField`].

pub use crypto_msg_type::MessageType;

use super::abstracts::derive_interop_converters;
use super::{Field, FieldDeserializer, FieldError, FieldSerializer};

/// The type of a message (1 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageTypeField(pub MessageType);

impl FieldSerializer<1> for MessageTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([bit_serialize_message_type(self.0)])
    }
}

impl FieldDeserializer<1> for MessageTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        Ok(Self(bit_deserialize_message_type(src[0])))
    }
}

derive_interop_converters!(MessageTypeField, MessageType);

// MessageType does not implement Hash; thus, it is not a
// high-standard field (hsf).
//
// However, it implements Clone,
impl Field<1> for MessageTypeField {}

/// Serialize [`MessageType`] to 1 bit identifier.
fn bit_serialize_message_type(mt: MessageType) -> u8 {
    match mt {
        MessageType::Trade => 1,
        MessageType::BBO => 2,
        MessageType::L2TopK => 3,
        MessageType::L2Snapshot => 4,
        MessageType::L2Event => 5,
        MessageType::L3Snapshot => 6,
        MessageType::L3Event => 7,
        MessageType::Ticker => 8,
        MessageType::Candlestick => 9,
        MessageType::OpenInterest => 10,
        MessageType::FundingRate => 11,
        MessageType::LongShortRatio => 12,
        MessageType::TakerVolume => 13,
        _ => 0,
    }
}

/// Deserialize a 1 bit identifier to a [`MessageType`].
fn bit_deserialize_message_type(id: u8) -> MessageType {
    match id {
        1 => MessageType::Trade,
        2 => MessageType::BBO,
        3 => MessageType::L2TopK,
        4 => MessageType::L2Snapshot,
        5 => MessageType::L2Event,
        6 => MessageType::L3Snapshot,
        7 => MessageType::L3Event,
        8 => MessageType::Ticker,
        9 => MessageType::Candlestick,
        10 => MessageType::OpenInterest,
        11 => MessageType::FundingRate,
        _ => MessageType::Other,
    }
}
