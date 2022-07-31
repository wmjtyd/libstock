//! The module with a field for storing the timestamp of a message.
//! See [`TimestampField`].

use std::time::SystemTime;

use crate::data::num::{six_byte_hex_to_unix_ms, unix_ms_to_six_byte_hex};

use super::{FieldDeserializer, FieldError, FieldResult, FieldSerializer, abstracts::{derive_hsf, derive_interop_converters}};

/// The general timestamp field (6 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimestampField(pub u64);

impl TimestampField {
    /// Create a new `ReceivedTimestamp` from the current time.
    pub fn new_from_now() -> FieldResult<Self> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        debug_assert!(u64::try_from(now.as_millis()).is_ok());
        let now_sec = now.as_millis() as u64;

        Ok(Self(now_sec))
    }
}

impl Default for TimestampField {
    fn default() -> Self {
        Self::new_from_now().expect("failed to get the system time")
    }
}

impl FieldSerializer<6> for TimestampField {
    type Err = FieldError;

    fn serialize(&self) -> Result<[u8; 6], Self::Err> {
        Ok(unix_ms_to_six_byte_hex(self.0))
    }
}

impl FieldDeserializer<6> for TimestampField {
    type Err = FieldError;

    fn deserialize(src: &[u8; 6]) -> Result<Self, Self::Err> {
        Ok(Self(six_byte_hex_to_unix_ms(src)))
    }
}

derive_interop_converters!(TimestampField, u64);
derive_hsf!(TimestampField, u64, 6);
