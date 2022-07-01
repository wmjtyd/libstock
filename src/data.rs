//! The methods for operating with orderbooks, orders and trades.
//!
//! This module includes some methods for converting numbers,
//! see the module [`hex`].

pub mod hex;

#[cfg(feature = "crypto")]
pub mod orderbook;

#[cfg(feature = "crypto")]
pub mod types;

#[cfg(feature = "crypto")]
pub mod fields;

#[cfg(feature = "crypto")]
pub mod order;

#[cfg(feature = "crypto")]
pub mod trade;

#[cfg(feature = "crypto")]
pub mod bbo;

#[cfg(feature = "crypto")]
pub mod kline;
