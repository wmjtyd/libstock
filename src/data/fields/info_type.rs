use std::str::FromStr;

use strum::{EnumString, FromRepr};

use super::{Either, FieldDeserializer, FieldError, FieldResult, FieldSerializer};

/// The info type (`asks` or `bids`) of a message (1 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfoTypeField(pub InfoType);

impl InfoTypeField {
    pub fn try_from_str(str: &str) -> FieldResult<Self> {
        let exchange = InfoType::from_str(str)
            .map_err(|_| FieldError::UnimplementedInfoType(Either::Left(str.to_string())))?;
        Ok(Self(exchange))
    }
}

impl FieldSerializer<1> for InfoTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([self.0 as u8])
    }
}

impl FieldDeserializer<1> for InfoTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0] as usize;
        let name = InfoType::from_repr(bit)
            .ok_or(FieldError::UnimplementedInfoType(Either::Right(bit)))?;

        Ok(Self(name))
    }
}

#[derive(Copy, Clone, FromRepr, strum::Display, EnumString, Debug, PartialEq, Eq, Hash)]
#[strum(serialize_all = "lowercase")]
pub enum InfoType {
    Asks = 1,
    Bids = 2,
}

#[cfg(test)]
mod tests {
    use super::FieldDeserializer;
    use super::{InfoType, InfoTypeField};

    #[test]
    fn test_info_type_try_from_str() {
        assert_eq!(
            InfoTypeField::try_from_str("asks").unwrap().0,
            InfoType::Asks
        );
        assert_eq!(
            InfoTypeField::try_from_str("bids").unwrap().0,
            InfoType::Bids
        );
    }

    #[test]
    fn test_info_expr_from_byte() {
        assert_eq!(InfoTypeField::deserialize(&[1]).unwrap().0, InfoType::Asks);
        assert_eq!(InfoTypeField::deserialize(&[2]).unwrap().0, InfoType::Bids);
    }
}
