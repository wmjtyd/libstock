//! The module with a field to specify the flag indicating the end of data.
//! See [`EndOfDataFlag`].

use super::{Field, FieldDeserializer, FieldError, FieldSerializer};

/// The flag indicating the end of data. (1 byte).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndOfDataFlag;

impl FieldSerializer<1> for EndOfDataFlag {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([b'\0'])
    }
}

impl FieldDeserializer<1> for EndOfDataFlag {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        if src[0] != b'\0' {
            Err(FieldError::DataEndedTooEarly)
        } else {
            Ok(Self)
        }
    }
}

impl Field<1> for EndOfDataFlag {}
