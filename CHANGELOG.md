# UNRELEASED

## UNRELEASED - Breaking changes

- `ReceivedTimestamp` field is now represented as seconds to fit in `u64`.
- `Symble` is now renamed to `Symbol`.
- `Symbol` field is now serialized as the big endian representation of `u8`.
- Map `EXCHANGE` and `INFOTYPE` are replaced to the enumeration `Exchange` and `InfoType`.
  - Use `strum` to provide the map-like function.
- Some methods in `data::hex` are removed.
  - Replace your current code to the native method (`.to_be_bytes()`)
- Removed and added some new unused error variants.

## UNRELEASED - Features

- `data::orderbook`, `data::trade`: Use `BufReader` instead of our based-on-Atomic seek reader.
- `data::orderbook`, `data::trade`: Use `data::fields` to serialize & deserialize.
- Separate the serialization & deserialization of fields to `data::fields`
- Replace the usize converter of `hex::encode_num_to_bytes` to a robust implementation
  - Related: `Some methods in data::hex are removed.`
- `data::types`: Add `Exchange` and `InfoType` while removing `EXCHANGE` and `INFOTYPE`.
- `data::orderbook`: Improve the logic of `generate_diff`.
