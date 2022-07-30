//! The module with a field to specify the symbol of a message.
//! See [`SymbolPairField`].

use typed_builder::TypedBuilder;

use super::{bimap::create_bimap, FieldDeserializer, FieldError, FieldSerializer};

/// Exchange-specific trading symbol or id, recognized by RESTful API.
pub type Symbol = u16;
/// Unified pair, base/quote, e.g., `BTC/USDT`.
pub type Pair = String;

/// The symbol of a message (2 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct SymbolPairField {
    pub symbol: Symbol,
    #[builder(setter(into))]
    pub pair: Pair,
}

impl SymbolPairField {
    pub fn from_pair(pair: &str) -> Self {
        let symbol = *SYMBOL_PAIR.get_by_right(&pair).unwrap_or(&0);

        Self {
            symbol,
            pair: pair.to_string(),
        }
    }
}

impl From<(Symbol, Pair)> for SymbolPairField {
    fn from((symbol, pair): (Symbol, Pair)) -> Self {
        Self { symbol, pair }
    }
}

impl FieldSerializer<2> for SymbolPairField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 2], Self::Err> {
        Ok(self.symbol.to_be_bytes())
    }
}

impl FieldDeserializer<2> for SymbolPairField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 2]) -> Result<Self, Self::Err> {
        let symbol: Symbol = u16::from_be_bytes(*src);
        let pair: Pair = SYMBOL_PAIR
            .get_by_left(&symbol)
            .unwrap_or(&"UNKNOWN")
            .to_string();

        Ok(Self { symbol, pair })
    }
}

create_bimap!(SYMBOL_PAIR {
    u16 => &'static str,
    1 => "BTC/USDT",
    2 => "BTC/USD",
    3 => "USDT/USD",
    4 => "ETH/USDT",
    5 => "ETH/USD",
});
