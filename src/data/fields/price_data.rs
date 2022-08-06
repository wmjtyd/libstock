//! The module with a field to specify the price data,
//! which includes the price and quantity base.
//!
//! For more information, see [`PriceDataField`].

pub use crypto_message::Order;
use typed_builder::TypedBuilder;

use super::{DecimalField, Field, FieldDeserializer, FieldError, FieldSerializer};

/// The price data (10 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct PriceDataField {
    /// 價格 (10 bytes)
    ///
    /// NOTE: crypto-crawler 是用浮點數儲存價格的。
    /// 這可能造成非常嚴重的誤差（0.1+0.2=0.300000004），
    /// 因此是 Bug，遲早要改成 String。
    #[builder(setter(into))]
    pub price: DecimalField<10>,

    /// 基本量 (10 bytes)
    ///
    /// NOTE: crypto-crawler 是用浮點數儲存價格的。
    /// 這可能造成非常嚴重的誤差（0.1+0.2=0.300000004），
    /// 因此是 Bug，遲早要改成 String。
    #[builder(setter(into))]
    pub quantity_base: DecimalField<10>,
}

impl FieldSerializer<20> for PriceDataField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 20], Self::Err> {
        let mut bytes = [0; 20];

        bytes[..10].copy_from_slice(&self.price.serialize()?);
        bytes[10..].copy_from_slice(&self.quantity_base.serialize()?);

        Ok(bytes)
    }
}

impl FieldDeserializer<20> for PriceDataField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 20]) -> Result<Self, Self::Err> {
        let price = arrayref::array_ref![src, 0, 10];
        let quantity_base = arrayref::array_ref![src, 10, 10];

        Ok(Self {
            price: DecimalField::deserialize(price)?,
            quantity_base: DecimalField::deserialize(quantity_base)?,
        })
    }
}

impl TryFrom<PriceDataField> for Order {
    type Error = FieldError;

    fn try_from(value: PriceDataField) -> Result<Self, Self::Error> {
        Ok(Order {
            price: value.price.try_into()?,
            quantity_base: value.quantity_base.try_into()?,
            quantity_quote: 0.0,
            quantity_contract: None,
        })
    }
}

impl TryFrom<&Order> for PriceDataField {
    type Error = FieldError;

    fn try_from(value: &Order) -> Result<Self, Self::Error> {
        Ok(PriceDataField {
            price: value.price.into(),
            quantity_base: value.quantity_base.into(),
        })
    }
}

impl Field<20> for PriceDataField {}
