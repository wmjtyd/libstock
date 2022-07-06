//! wmjtyd-libstock - utilities for operating stocks
//!
//! The utilities for operating stocks. For example:
//!
//! - Using methods under [`mod@data`], you can operate with orderbooks,
//!   orders and trades.
//! - Using methods under [`mod@file`], you can operate with files
//!   and create a daemon to write files with the well-defined format
//!   to the well-defined directory.
//! - Using methods under [`mod@flag`] to use thread-safe, lock-free flags.
//! - Using methods under [`mod@slack`] to send notifications to Slack with Slack Hook.
//!
//! These utilities are especial for [wmjtyd/crypto-market](https://github.com/wmjtyd/crypto-market);
//! however, you can also use it in your project. We licensed it under `Apache-2.0`, the same
//! license to `crypto-market`! 😄
//!
//! ## License
//!
//! Apache-2.0

pub mod data;
pub mod file;
pub mod flag;

#[cfg(feature = "slack")]
pub mod slack;
