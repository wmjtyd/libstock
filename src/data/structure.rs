//! The prefered encoded data structures of `libstock`.

use std::ops::RangeBounds;

use super::hex::{long_to_hex, hex_to_byte, HexDataError};

/// Make a slice to a fixed array.
/// 
/// # Panic
/// 
/// See [`slice::copy_from_slice`].
/// 
/// ```should_panic
/// use wmjtyd_libstock::data::structure::slice_to_fixed_array;
///
/// let slice = &[1, 2, 3];
/// // The array's length (4) is larger than the slice's length (3)!
/// let fixed_array = slice_to_fixed_array::<4>(&[1, 2, 3], None);
/// ```
/// 
/// ```should_panic
/// use wmjtyd_libstock::data::structure::slice_to_fixed_array;
///
/// let slice = &[1, 2, 3];
/// // The array's length (3) is smaller than the slice's length (4)!
/// let fixed_array = slice_to_fixed_array::<3>(&[1, 2, 3, 4], None);
/// ```
/// 
/// # Examples
/// 
/// ```
/// use wmjtyd_libstock::data::structure::slice_to_fixed_array;
///
/// let slice = &[1, 2, 3];
/// let fixed_array = slice_to_fixed_array::<3>(slice, None);
/// 
/// assert_eq!(&fixed_array, slice);
/// ```
///
/// ```
/// use wmjtyd_libstock::data::structure::slice_to_fixed_array;
///
/// let slice = &[1, 2, 3];
/// let fixed_array = slice_to_fixed_array::<5>(&[1, 2, 3], Some(2..));
/// 
/// assert_eq!(&fixed_array, &[0, 0, 1, 2, 3]);
/// ```
pub fn slice_to_fixed_array<const T: usize>(
    ref_: &[u8],
    offset: Option<impl RangeBounds<usize> + std::slice::SliceIndex<[u8], Output = [u8; T]>>
) -> [u8; T] {
    let mut array = [0u8; len];

    if let Some(offset) = offset {
        array[offset].copy_from_slice(ref_);
    } else {
        array.copy_from_slice(ref_);
    }

    array
}

#[macro_export]
macro_rules! slice2array {
    (@op newarray ($type:ty: $len:expr)) => {
        let mut array = [0$type; $len];
    };

    (($type:ty: $len:expr) $ref_:expr, $offset:expr) => {
        slice2array!(@op newarray ($type: $len))

        array[offset].copy_from_slice($ref_);
        array
    };

    (($type:ty: $len:expr) $ref_:expr) => {
        let mut array = [0$type; $len];

        array.copy_from_slice($ref_);
        array
    };

    (($type:ty: $len:expr) $ref_:expr, $offset:expr) => {
        slice2array!((u8: $len) $ref_ $offset)
    };

    (($type:ty: $len:expr) $ref_:expr) => {
        slice2array!((u8: $len) $ref_)
    };
}

/// The timestamp of exchange.
pub struct ExchangeTimestamp {
    ts: i64,
}

impl ExchangeTimestamp {
    /// Create a new `ExchangeTimestamp` from the specified timestamp.
    pub fn from_ts(ts: i64) -> Self {
        Self { ts }
    }

    /// Create a new `ExchangeTimestamp` from the encoded bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        todo!()
        // let payload = getseek(6);
        // let mut buf = [0u8; 16];
        // buf[10..].copy_from_slice(payload);

        // i128::from_be_bytes(buf)
    }

    /// Encode the timestamp to a bytes.
    pub fn to_bytes(&self) -> StructureResult<Vec<u8>> {
        let exchange_timestamp = self.ts;
        let exchange_timestamp_hex = long_to_hex(exchange_timestamp);
        let exchange_timestamp_hex_byte = hex_to_byte(&exchange_timestamp_hex)
            .map_err(StructureError::HexToByteFailed)?;
        
        Ok(exchange_timestamp_hex_byte)
    }
}


#[derive(thiserror::Error, Debug)]
pub enum StructureError {
    #[error("unable to convert a hex to byte")]
    HexToByteFailed(HexDataError),
}

pub type StructureResult<T> = Result<T, StructureError>;
