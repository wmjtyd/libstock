//! The module with a field to specify the exchange type of a message.
//! See [`ExchangeTypeField`].

use std::str::FromStr;

use either::Either;
use strum::{EnumString, FromRepr};

use super::{FieldDeserializer, FieldError, FieldResult, FieldSerializer};

/// The exchange type of a message (1 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExchangeTypeField(pub Exchange);

impl ExchangeTypeField {
    pub fn try_from_str(str: &str) -> FieldResult<Self> {
        let exchange = Exchange::from_str(str)
            .map_err(|_| FieldError::UnimplementedExchange(Either::Left(str.to_string())))?;
        Ok(Self(exchange))
    }
}

impl FieldSerializer<1> for ExchangeTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        Ok([self.0 as u8])
    }
}

impl FieldDeserializer<1> for ExchangeTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0] as usize;
        let name = Exchange::from_repr(bit)
            .ok_or(FieldError::UnimplementedExchange(either::Right(bit)))?;

        Ok(Self(name))
    }
}

#[derive(Copy, Clone, FromRepr, strum::Display, EnumString, Debug, PartialEq, Eq, Hash)]
#[strum(serialize_all = "lowercase")]
pub enum Exchange {
    Crypto = 1,
    Ftx = 2,
    Binance = 3,
    Huobi = 8,
    Kucoin = 10,
    Okx = 11,
}

#[cfg(test)]
mod tests {
    use super::{Exchange, ExchangeTypeField, FieldDeserializer};

    #[test]
    fn test_exchange_expr_try_from_str() {
        assert_eq!(
            ExchangeTypeField::try_from_str("crypto").unwrap().0,
            Exchange::Crypto
        );
        assert_eq!(
            ExchangeTypeField::try_from_str("ftx").unwrap().0,
            Exchange::Ftx
        );
        assert_eq!(
            ExchangeTypeField::try_from_str("binance").unwrap().0,
            Exchange::Binance
        );
        assert_eq!(
            ExchangeTypeField::try_from_str("huobi").unwrap().0,
            Exchange::Huobi
        );
        assert_eq!(
            ExchangeTypeField::try_from_str("kucoin").unwrap().0,
            Exchange::Kucoin
        );
        assert_eq!(
            ExchangeTypeField::try_from_str("okx").unwrap().0,
            Exchange::Okx
        );
    }

    #[test]
    fn test_exchange_expr_from_byte() {
        assert_eq!(
            ExchangeTypeField::deserialize(&[1]).unwrap().0,
            Exchange::Crypto
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[2]).unwrap().0,
            Exchange::Ftx
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[3]).unwrap().0,
            Exchange::Binance
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[8]).unwrap().0,
            Exchange::Huobi
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[10]).unwrap().0,
            Exchange::Kucoin
        );
        assert_eq!(
            ExchangeTypeField::deserialize(&[11]).unwrap().0,
            Exchange::Okx
        );
    }
}
