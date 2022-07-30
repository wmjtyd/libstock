//! The methods for operating with data structures.
//!
//! This module includes some methods for converting numbers,
//! see the module [`num`].

pub mod num;
pub mod serializer;

// #[cfg(feature = "crypto")]
// pub mod orderbook;

#[cfg(feature = "crypto")]
pub mod fields;

#[cfg(feature = "crypto")]
pub mod order;

// #[cfg(feature = "crypto")]
// pub mod trade;

#[cfg(feature = "crypto")]
pub mod bbo;

// #[cfg(feature = "crypto")]
// pub mod kline;

#[cfg(feature = "crypto")]
pub mod funding_rate;
