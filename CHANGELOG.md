# libstock's changelog

## UNRELEASED: 0.4.0

### How to migrate your old codebase to 0.4.0-native architecture?

#### Compatibility Layer

**To enable it, enable `compat-v0_3` feature.**

The built-in compatibility layer emulates what
v0.3.x used to behave, including the *serialization*,
*deserialization*, *number encoding* and *number decoding*.

With this emulation, you can migrate your code with less pain–
most of the current components will mostly behave as before. However,
all of them are marked as `deprecated`, and we may remove
this layer in the future major version (such as 0.5.x).

DO NOT DEPEND ON THIS LAYER, and migrate your codebase as long as possible! This layer may slow down your program, and the layer may have some unexpected changes compared to the 0.3.x version (for example, the Error variant).

#### Encoding (Now called *Serializing*)

Assuming your code is using `encode_bbo` (or `encode_orderbook`, whatever):

```rs
let encoded: Vec<u8> = encode_bbo(your_bbo_msg).unwrap();
```

First, you have to convert your `Msg` in crypto-crawler,
to libstock's `Structure`. You can approach it with `try_into()`:

```rs
// OrderBookMsg → OrderbookStructure, KlineMsg → KlineStructure,
// TradeMsg → TradeStructure, FundingRateMsg → FundingRateStructure,
// so on.
let bbo_structure = BboStructure::try_from(your_bbo_msg)?
```

Our new serialization method accepts *any* type implementing `Write` –
in other words, you can pass your Socket, File and anything else writable
to the `serialize()` method:

```rs
// Use our serialization method.
use wmjtyd_libstock::data::serializer::StructureSerializer;

bbo_structure.serialize(your_file)?;
```

If you prefer the current way, you need to manage your own buffer
(however, you can pass any buffer implementing `Write` for your case.)

```rs
let mut buffer = Vec::new();
bbo_structure.serialize(buffer)?;
```

#### Decoding (Now called *Deserializing*)

Assuming your code is using `decode_bbo` (or `decode_orderbook`, whatever):

```rs
let decoded: BboMsg = decode_bbo(src).unwrap();
```

First, use our deserialization method to construct the Structure.
It accepts any type implementing `Read` trait, so you can just
pass your socket, file or anything else readable to the `deserialize()` method.

```rs
// Use our deserialization method.
use wmjtyd_libstock::data::serializer::StructureDeserializer;

let decoded = BboStructure::deserialize(&mut your_data_source);
```

Note that if your data source is a `Vec<T>`, you **must** turn it to a mutable-immutable slice.
It is counterintuitive, [but it works](https://doc.rust-lang.org/std/io/trait.Read.html#impl-Read-2) ;)

```rs
let decoded = BboStructure::deserialize(&mut your_vec.as_slice());
```

Finally, convert your Structure back to the crypto-crawler's Msg:

```rs
let msg = BboMsg::try_from(decoded)?;
```

#### Number Encoding and Decoding

We have rewritten the encoding and decoding methods for clearer
architecture, robust and performance. However, it introduces
some breaking change though it can be easily resolved.

Assuming you have such this code:

```rs
let num = 1234_5678.to_string();
u32::encode_bytes(&num)
```

First, You need to convert your integer, float number, or number string
to `rust_decimal::Decimal`. We have re-exported it in `data::num`
so you won't need to add another dependencies by yourself.

Take the above as an example, we can rewrite it to:

```rs
// Integer
let num = rust_decimal::from(1234_5678);
// Float; you should do your own error handling.
let num = rust_decimal::from_f32(1234.5678).map_err(...)?;
// Number string; you should do your own error handling.
let num = rust_decimal::from_str_exact("1234.5678")?;
```

After that, we include our new encode API and encode the `num`:

```rs
use wmjtyd_libstock::data::num::Encoder;

let num_bytes: [u8; 5] = num.encode()?;
```

To decode the number, use `Decoder::decode`:

```rs
use wmjtyd_libstock::data::num::Decoder;

let num = Decoder::decode(&num_bytes)?;
```

#### Fields

We did not write the compatibility layer for fields,
as it used to be internally used in the serialization and deserialization, and most people may not operate with this.

Some notable big changes to migrate your codebase:

- `data::types` has merged into `data::fields`.
- All the `-Repr` are renamed to `-Field`.
- `try_from_reader` → `deserialize_from_reader`
- `from_bytes` (or `try_from_bytes`) → `deserialize`
- `to_bytes` (or `try_to_bytes`) → `serialize`
- `SymbolPairField` is no longer a tuple – its fields has been named now.
- `ReadExt` has been merged to `Deserializer`.

#### `message`

Though we reshaped and rewrote all the message modules,
there is not really big changes for users.

**Migrate `nanomsg` writers**

Assuming you used to have such a code:

```rs
use std::io::Write;
use wmjtyd_libstock::message::nanomsg::{Nanomsg, NanomsgProtocol};

let nanomsg = Nanomsg::new("ipc:///tmp/cl-nanomsg-old-api-w.ipc", NanomsgProtocol::Pub);

if let Ok(mut nanomsg) = nanomsg {
    nanomsg.write_all(b"Hello World!").ok();
}
```

First, update your `Nanomsg` import to `NanomsgPublisher`.
We have separated `Nanomsg` to `NanomsgPublisher` and `NanomsgSubscriber`,
which implementing the corresponding `Publisher` and `Subscriber`:

```rs
use wmjtyd_libstock::message::nanomsg::NanomsgPublisher;
```

And then, construct your `NanomsgPublisher`. **Note that no any
additional parameter will pass it this construction**, and we
will bind our address later.

```rs
let nanomsg = NanomsgPublisher::new();
```

`new()` returns `Result`, so error handling is needed here.
After that, let's bind our address to Nanomsg:

```rs
use wmjtyd_libstock::message::traits::Bind;

if let Ok(mut nanomsg) = nanomsg {
    nanomsg.bind("ipc:///tmp/cl-nanomsg-new-api-w.ipc").ok();
```

Note that we includes `traits::Bind` here. To use the `bind`
method, you *must* introduce our `Bind` trait here. It is
necessary for our high-level abstraction. Finally, we introduce
`Write` trait, just like what we did before.

```rs
use std::io::Write;  // or use wmjtyd_libstock::message::traits::Write;
    nanomsg.write_all(b"Hello World!").ok();
}
```

Besides, we provide the async API of `nanomsg` now!

```rs
use tokio::io::AsyncWriteExt;  // or use wmjtyd_libstock::message::traits::AsyncWriteExt;
    nanomsg.write_all(b"Hello World!").await.ok();
}
```

The full synchronous example:

```rs
use wmjtyd_libstock::message::nanomsg::NanomsgPublisher;
use wmjtyd_libstock::message::traits::{Bind, Write};

let nanomsg = NanomsgPublisher::new();

if let Ok(mut nanomsg) = nanomsg {
    nanomsg.bind("ipc:///tmp/cl-nanomsg-new-api-w.ipc").ok();
    nanomsg.write_all(b"Hello World!").ok();
}
```

**Migrate `zeromq` writers**

Just like `nanomsg`, but use `zeromq::ZeromqPublisher`. Asynchronous Example:

```rs
use wmjtyd_libstock::message::traits::{AsyncWriteExt, Bind};
use wmjtyd_libstock::message::zeromq::ZeromqPublisher;

let zeromq = ZeromqPublisher::new();

if let Ok(mut zeromq) = zeromq {
    zeromq.bind("ipc:///tmp/cl-zeromq-new-api-w-a.ipc").ok();
    zeromq.write_all(b"Hello World!").await.ok();
}
```

**Migrate `nanomsg` readers**

*WIP*. Really similar to what we did in writers.
Example of this new API:

```rs
use wmjtyd_libstock::message::nanomsg::NanomsgSubscriber;
use wmjtyd_libstock::message::traits::{Connect, Read, Subscribe};

let nanomsg = NanomsgSubscriber::new();

if let Ok(mut nanomsg) = nanomsg {
    nanomsg.connect("ipc:///tmp/cl-nanomsg-new-api-r.ipc").ok();
    nanomsg.subscribe(b"").ok();

    let mut buf = [0; 12];
    nanomsg.read_exact(&mut buf).ok();
    assert_eq!(b"Hello World!", &buf);
}
```

**Migrate `zeromq` readers**

*WIP*. All of the APIs are followed our new rule and
thus the logic is just the same. Example of this new API:

```rs

```

### 0.4.0 – Breaking Changes

- `message` has been largely rewritten.
- Methods under `data` module are mostly reshaped. You may need to
  rewrite your code to adapt this new architectiure.
  - We have abstracted a `Structure` – To serialize your
    `BboMsg`, do `.try_into()` first.
- Added a End-Of-Data flag (`\0`) to all the current structure.

### 0.4.0 – Features

- Rewrite `data` module for robust and performance.
- Implement `Eq` for `file/writer`.
- Implement a general serializer and deserializer for field and structure.
  - `StructSerializer`: We now accept any type implemented `std::io::Write`,
     instead of `BufWriter` only!
  - `StructDeserializer`: We now accept any type implemented `std::io::Read`,
     instead of `BufReader` only!
- Add the ability to convert other structures other than what
  `crypto-crawler` provides! 🎉
  - Just implement the `TryFrom` between your structure and our structure.
  - Our incoming C binding adds the ability to operate with `libstock`'s
    serialization and deserialization feature in C/C++.
  - Our work-in-progress Java binding also adds such the ability
    like C/C++'s.
- Derive more useful traits for our structures, such as `Debug`, `PartialEq`,
  `Eq`, `Hash`, `Clone`.
  - Also our fields ;)

### 0.4.0 – Chores

- Update dependencies.
  - WIP: merge crypto-crawler changes to upstream

### 0.4.0 – CI

- Automatically abort the old CI tasks by
  setting the `concurrent` flag.
- Set `rust-toolchain.toml` to beta toolchain.

## 0.3.0

### 0.3.0 – Breaking changes

- `file/reader`: `open()` returns `Result` instead of `Option` now.
  - For better backtrace
- `file/writer`: Set the size of the length field of written data structure to 2 bytes.
  - It used to be 'usize'. It is an implementation mistake.
- `file/hex`: removed two deprecated methods
- `file/reader`: move path to `<date>/<name>.csv`
- Updated the fields of `data`.
- `file/datadir`: need to pass a timestamp

### 0.3.0 – Features

- `slack`: For sending notifications to Slack with Slack Hook.
- `message`: A basic encap of `nanomsg` and `zeromq` for subscribing and publishing.
  - (beta 2) implement DerefMut to `Socket` for `message`
- `message`: Add `Subscribe` trait for generalize `subscribe()` method.
  - `zeromq` and `nanomsg` both have implemented this.
- `file/hex`: Add the encode/decode capability for negative numbers
  - WIP: currently dirty but works.
- `message/zeromq`: migrate to the ZeroMQ implemented in Rust.

### 0.3.0 – Bug fixes

- `file/reader`: Make `read()` returns the exact data
  - Currently, it always returns `[]` due to an implementation mistake.
- `file/writer`: Flush buffer after the data written ended.
- (beta 2) `message`: use `connect` instead of `bind` for Sub
- `data/*/decode`: `ReceivedTimestampRepr` should be 6 bytes instead of 8 bytes.

### 0.3.0 – Refactoring

- Upgrade `crypto-crawler-rs` to `92aee0d37e228e53dd994a17058a7f819e005446`
  - Clean up the unnecessary dependencies.
- Deprecated `file/ident`.

### 0.3.0 – Tests

<!-- <!> has been disabled by default. -->
<!-- - `file`: Add integration test for writer & reader -->
- Add encode/decode tests to `data/bbo`
  - *WIP:* add other encode/decode tests to `data/*`!

### 0.3.0 – Chores

- `file/reader`: add `info` logger on new() for better debugging

### 0.3.0 – CI

- Install `nanomsg` for `message` test.
- Introduce `cargo nextest` for faster build.

## 0.2.0

### 0.2.0 - Breaking changes

- `Symble` is now renamed to `Symbol`.
- `Symbol` field is now serialized as the big endian representation of `u8`.
- Map `EXCHANGE` and `INFOTYPE` are replaced to the enumeration `Exchange` and `InfoType`.
  - Use `strum` to provide the map-like function.
- Some methods in `data::hex` are removed.
  - Replace your current code to the native method (`.to_be_bytes()`)
  - (beta 5) For UNIX timestamp, use `unix_ms_to_six_byte_hex` and `six_byte_hex_to_unix_ms` instead.
- Removed and added some new unused error variants.

### 0.2.0 - Features

- `data::orderbook`, `data::trade`: Use `BufReader` instead of our based-on-Atomic seek reader.
- `data::orderbook`, `data::trade`: Use `data::fields` to serialize & deserialize.
- Separate the serialization & deserialization of fields to `data::fields`
- Replace the usize converter of `hex::encode_num_to_bytes` to a robust implementation
  - Related: `Some methods in data::hex are removed.`
- `data::types`: Add `Exchange` and `InfoType` while removing `EXCHANGE` and `INFOTYPE`.
- `data::orderbook`: Improve the logic of `generate_diff`.
- `data::hex`: Deprecate old inextensible methods (`encode_num_to_bytes` & `decode_num_to_bytes`)
  - use `u32::encode_bytes` and `u32::decode_bytes` for 5-byte encoding & decoding
- `data::hex`: Add the u64 to 10-byte encode & decode support
  - use `u64::encode_bytes` and `u64::decode_bytes` for 10-byte encoding & decoding
- `data::types`: Add more exchanges, and add `PERIOD`.
- Add the encoder and decoder of `bbo` and `kline`.
- `file::writer`: Add `tracing` log.
- `file::writer`: Refactored for extensibility.
- Change to wmjtyd's `crypto-crawler` and `crypto-msg-parser`.
- (beta 5) Add methods to serialize and deserialize UNIX timestamp to 6-byte `u64`.

### 0.2.0 - CI

- Also test if `cargo doc` can generate correct documentation.
- Run `cargo clippy` and `cargo fmt --check`

### 0.2.0 - Examples

- Add a production example of `file::writer`.
- (beta 4) `data::hex` add the testcase of `i8` → `u8` and `u8` → `i8`

### 0.2.0 - Bug fixes

- `file::writer`: Don't create the directory if it has been existed.
- `file::writer`: `.stop()` can't work properly.
- (beta 4) `{InfoTypeExpr,ExchangeTypeRepr}::try_from_str` should be from lowercase
  - Added the test case of this.
- (beta 5) UNIX timestamp should serialize and deserialize to 6-byte `u64`.
