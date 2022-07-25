# libstock's changelog

## Unreleased: 0.3.0

### Unreleased: 0.3.0 – Breaking changes

- `file/reader`: `open()` returns `Result` instead of `Option` now.
  - For better backtrace
- `file/writer`: Set the size of the length field of written data structure to 2 bytes.
  - It used to be 'usize'. It is an implementation mistake.
- `file/hex`: removed two deprecated methods
- `file/reader`: move path to `<date>/<name>.csv`
- Updated the fields of `data`.
- `file/datadir`: need to pass a timestamp

### Unreleased: 0.3.0 – Features

- `slack`: For sending notifications to Slack with Slack Hook.
- `message`: A basic encap of `nanomsg` and `zeromq` for subscribing and publishing.
  - (beta 2) implement DerefMut to `Socket` for `message`
- `message`: Add `Subscribe` trait for generalize `subscribe()` method.
  - `zeromq` and `nanomsg` both have implemented this.
- `file/hex`: Add the encode/decode capability for negative numbers
  - WIP: currently dirty but works.
- `message/zeromq`: migrate to the ZeroMQ implemented in Rust.

### Unreleased: 0.3.0 – Bug fixes

- `file/reader`: Make `read()` returns the exact data
  - Currently, it always returns `[]` due to an implementation mistake.
- `file/writer`: Flush buffer after the data written ended.
- (beta 2) `message`: use `connect` instead of `bind` for Sub
- `data/*/decode`: `ReceivedTimestampRepr` should be 6 bytes instead of 8 bytes.

### Unreleased: 0.3.0 – Refactoring

- Upgrade `crypto-crawler-rs` to `92aee0d37e228e53dd994a17058a7f819e005446`
  - Clean up the unnecessary dependencies.
- Deprecated `file/ident`.

### Unreleased: 0.3.0 – Tests

<!-- <!> has been disabled by default. -->
<!-- - `file`: Add integration test for writer & reader -->
- Add encode/decode tests to `data/bbo`
  - _WIP:_ add other encode/decode tests to `data/*`!

### Unreleased: 0.3.0 – Chores

- `file/reader`: add `info` logger on new() for better debugging

### Unreleased: 0.3.0 – CI

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
