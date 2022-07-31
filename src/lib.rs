//! wmjtyd-libstock - utilities for operating stocks
//!
//! The utilities for operating stocks. For example:
//!
//! - Using methods under [`mod@data`], you can operate with data structures,
//!   such as orderbook, bbo, kline, etc.
//! - Using methods under [`mod@file`], you can operate with files
//!   and create a daemon to write files with the well-defined format
//!   to the well-defined directory.
//! - Using methods under [`mod@flag`] to use thread-safe, lock-free flags.
//! - Using methods under [`mod@slack`] to send notifications to Slack with Slack Hook.
//! - Using methods under [`mod@message`] to subscribe and publish based on Nanomsg or Zeromq.
//!
//! ## License
//!
//! Apache-2.0

pub mod data;
pub mod file;
pub mod flag;

#[cfg(feature = "slack")]
pub mod slack;

pub mod message;

mod compat;
