use std::fs::File;
use std::io::{Write, Read};

use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;
use rust_decimal::Decimal;
use wmjtyd_libstock::data::bbo::BboStructure;
use wmjtyd_libstock::data::fields::exchange_type::Exchange;
use wmjtyd_libstock::data::fields::{PriceDataField, SymbolPairField};
use wmjtyd_libstock::data::serializer::{StructSerializer, StructDeserializer};

fn main() {
    let bbo_structure = BboStructure::builder()
        .exchange_timestamp(1659755147000u64)
        .exchange_type(Exchange::Binance)
        .market_type(MarketType::Spot)
        .symbol(SymbolPairField::from_pair("BTC/USDT"))
        .message_type(MessageType::BBO)
        .asks(
            PriceDataField::builder()
                .price(12345.0)
                .quantity_base(67890.0)
                .build(),
        )
        .bids(
            PriceDataField::builder()
                .price(Decimal::from_str_exact("12345.12345").unwrap())
                .quantity_base(Decimal::from_str_exact("56789.87654").unwrap())
                .build(),
        )
        .build();

    let mut f = File::create("bbo_serialized.bin").unwrap();
    bbo_structure.serialize(&mut f).unwrap();
    f.flush().unwrap();
    drop(f);
    
    let mut f = File::open("bbo_serialized.bin").unwrap();
    let mut serialized = Vec::new();
    f.read_to_end(&mut serialized).unwrap();

    assert_eq!(
        bbo_structure,
        BboStructure::deserialize(&mut serialized.as_slice()).unwrap()
    );
}
