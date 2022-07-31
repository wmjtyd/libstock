//! The module with a field to specify the market type of a message.
//! See [`MarketTypeField`].

pub use crypto_market_type::MarketType;

use super::{
    abstracts::{derive_hsf, derive_interop_converters},
    bimap::create_bimap,
    FieldDeserializer, FieldError, FieldSerializer,
};

/// The market type of a message (1 byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketTypeField(pub MarketType);

impl FieldSerializer<1> for MarketTypeField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 1], Self::Err> {
        let bit = *MARKET_TYPE_BIT.get_by_left(&self.0).unwrap_or(&0);

        Ok([bit])
    }
}

impl FieldDeserializer<1> for MarketTypeField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 1]) -> Result<Self, Self::Err> {
        let bit = src[0];

        let name = MARKET_TYPE_BIT
            .get_by_right(&bit)
            .unwrap_or(&MarketType::Unknown);

        Ok(Self(*name))
    }
}

derive_interop_converters!(MarketTypeField, MarketType);
derive_hsf!(MarketTypeField, MarketType, 1);

create_bimap!(MARKET_TYPE_BIT {
    MarketType => u8,
    MarketType::Spot => 1,
    MarketType::LinearFuture => 2,
    MarketType::InverseFuture => 3,
    MarketType::LinearSwap => 4,
    MarketType::InverseSwap => 5,
    MarketType::EuropeanOption => 6,
    MarketType::QuantoFuture => 7,
    MarketType::QuantoSwap => 8,
    // Default: MarketType::Unknown => 0,
});
