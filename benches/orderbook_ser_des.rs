use std::io::Write;
use std::iter;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;
use rust_decimal::Decimal;
use wmjtyd_libstock::data::fields::exchange_type::Exchange;
use wmjtyd_libstock::data::fields::info_type::InfoType;
use wmjtyd_libstock::data::fields::{PriceDataField, SymbolPairField};
use wmjtyd_libstock::data::orderbook::{OrderbookStructure, OrdersBox};
use wmjtyd_libstock::data::serializer::{StructDeserializer, StructSerializer};

pub struct NoneWriter;

impl Write for NoneWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn get_ob_structure() -> OrderbookStructure {
    OrderbookStructure::builder()
        .exchange_timestamp(black_box(1659755147000u64))
        .exchange_type(black_box(Exchange::Binance))
        .market_type(black_box(MarketType::Spot))
        .symbol(SymbolPairField::from_pair(black_box("BTC/USDT")))
        .message_type(black_box(MessageType::L2Event))
        .asks(
            OrdersBox::builder()
                .direction(black_box(InfoType::Asks))
                .orders(
                    iter::repeat(
                        PriceDataField::builder()
                            .price(black_box(12345.0))
                            .quantity_base(black_box(67890.0))
                            .build(),
                    )
                    .take(20)
                    .collect(),
                )
                .build(),
        )
        .bids(
            OrdersBox::builder()
                .direction(black_box(InfoType::Asks))
                .orders(
                    iter::repeat(
                        PriceDataField::builder()
                            .price(Decimal::from_str_exact(black_box("12345.12345")).unwrap())
                            .quantity_base(
                                Decimal::from_str_exact(black_box("56789.87654")).unwrap(),
                            )
                            .build(),
                    )
                    .take(25)
                    .collect(),
                )
                .build(),
        )
        .build()
}

fn construct(c: &mut Criterion) {
    c.bench_function("construct orderbook", |b| b.iter(get_ob_structure));
}

fn serialize(c: &mut Criterion) {
    c.bench_function("serialize orderbook", |b| {
        b.iter_batched_ref(
            get_ob_structure,
            |data| data.serialize(&mut NoneWriter),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn deserialize(c: &mut Criterion) {
    const ENCODED_DATA: &[u8] = include_bytes!("./orderbook_serialized.bin");

    c.bench_function("deserialize orderbook", |b| {
        b.iter_batched_ref(
            || ENCODED_DATA,
            |mut data| OrderbookStructure::deserialize(&mut data),
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, construct, serialize, deserialize);
criterion_main!(benches);
