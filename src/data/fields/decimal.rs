//! The module with a field to store numbers serialized with [`crate::data::num`]'s methods.
//! See [`DecimalField`].

pub use rust_decimal::Decimal;
pub use rust_decimal::Error as DecimalError;

use crate::data::num::Decoder;
use crate::data::num::Encoder;

use super::{FieldDeserializer, FieldError, FieldSerializer};

/// The field to store numbers serialized with [`crate::data::num`]'s methods.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DecimalField<const LEN: usize>(pub Decimal);

impl FieldSerializer<5> for DecimalField<5> {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 5], Self::Err> {
        Ok(self.0.encode()?)
    }
}

impl FieldSerializer<10> for DecimalField<10> {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 10], Self::Err> {
        Ok(self.0.encode()?)
    }
}

impl FieldDeserializer<5> for DecimalField<5> {
    type Err = FieldError;

    fn deserialize(src: &[u8; 5]) -> Result<Self, Self::Err> {
        Ok(Self(Decimal::decode(src)?))
    }
}

impl FieldDeserializer<10> for DecimalField<10> {
    type Err = FieldError;

    fn deserialize(src: &[u8; 10]) -> Result<Self, Self::Err> {
        Ok(Self(Decimal::decode(src)?))
    }
}

impl<const LEN: usize> From<Decimal> for DecimalField<LEN> {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl<const LEN: usize> From<f32> for DecimalField<LEN> {
    fn from(f: f32) -> Self {
        Self(Decimal::from_f32_retain(f).expect("overflow?"))
    }
}

impl<const LEN: usize> From<f64> for DecimalField<LEN> {
    fn from(f: f64) -> Self {
        Self(Decimal::from_f64_retain(f).expect("overflow?"))
    }
}

impl<const LEN: usize> TryFrom<&str> for DecimalField<LEN> {
    type Error = DecimalError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self(Decimal::from_str_exact(s)?))
    }
}
