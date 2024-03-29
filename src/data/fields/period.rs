//! The module with a field to specify the period of a message.
//! See [`PeriodField`].

use super::abstracts::{derive_hsf, derive_interop_converters};
use super::bimap::create_bimap;
use super::{Either, FieldDeserializer, FieldError, FieldSerializer};

/// The period of a message (1 byte).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeriodField(pub String);

impl FieldSerializer<1> for PeriodField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        let bit = *PERIOD
            .get_by_left(self.0.as_str())
            .ok_or_else(|| FieldError::UnimplementedPeriod(Either::Left(self.0.to_string())))?;

        Ok([bit])
    }
}

impl FieldDeserializer<1> for PeriodField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0] as u8;

        let name = PERIOD
            .get_by_right(&bit)
            .ok_or(FieldError::UnimplementedPeriod(Either::Right(bit)))?;

        Ok(Self(name.to_string()))
    }
}

derive_interop_converters!(PeriodField, String);

impl From<&str> for PeriodField {
    fn from(src: &str) -> Self {
        Self(src.to_string())
    }
}

derive_hsf!(PeriodField, String, 1);

create_bimap!(PERIOD {
    &'static str => u8,
    "1m" => 1,
    "5m" => 2,
    "30m" => 3,
    "1h" => 4,
});
