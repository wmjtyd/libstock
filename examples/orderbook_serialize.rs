use std::fs::File;
use std::io::{Read, Write};
use std::iter;

use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;
use rust_decimal::Decimal;
use wmjtyd_libstock::data::fields::exchange_type::Exchange;
use wmjtyd_libstock::data::fields::info_type::InfoType;
use wmjtyd_libstock::data::fields::{PriceDataField, SymbolPairField};
use wmjtyd_libstock::data::orderbook::{OrderbookStructure, OrdersBox};
use wmjtyd_libstock::data::serializer::{StructDeserializer, StructSerializer};

fn main() {
    let orderbook_structure = OrderbookStructure::builder()
        .exchange_timestamp(1659755147000u64)
        .exchange_type(Exchange::Binance)
        .market_type(MarketType::Spot)
        .symbol(SymbolPairField::from_pair("BTC/USDT"))
        .message_type(MessageType::L2Event)
        .asks(
            OrdersBox::builder()
                .direction(InfoType::Asks)
                .orders(
                    iter::repeat(
                        PriceDataField::builder()
                            .price(12345.0)
                            .quantity_base(67890.0)
                            .build(),
                    )
                    .take(20)
                    .collect(),
                )
                .build(),
        )
        .bids(
            OrdersBox::builder()
                .direction(InfoType::Asks)
                .orders(
                    iter::repeat(
                        PriceDataField::builder()
                            .price(Decimal::from_str_exact("12345.12345").unwrap())
                            .quantity_base(Decimal::from_str_exact("56789.87654").unwrap())
                            .build(),
                    )
                    .take(25)
                    .collect(),
                )
                .build(),
        )
        .build();

    let mut f = File::create("orderbook_serialized.bin").unwrap();
    orderbook_structure.serialize(&mut f).unwrap();
    f.flush().unwrap();
    drop(f);

    let mut f = File::open("orderbook_serialized.bin").unwrap();
    let mut serialized = Vec::new();
    f.read_to_end(&mut serialized).unwrap();

    assert_eq!(
        orderbook_structure,
        OrderbookStructure::deserialize(&mut serialized.as_slice()).unwrap()
    );
}
