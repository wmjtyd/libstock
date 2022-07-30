//! Methods for operating with hexadecimal strings.

mod decimal_dec;
mod decimal_enc;
mod timestamp;

mod test_utils;

pub use decimal_dec::*;
pub use decimal_enc::*;
pub use rust_decimal::{Decimal, Error as DecimalError};
pub use timestamp::*;

#[derive(thiserror::Error, Debug)]
pub enum NumError {}
