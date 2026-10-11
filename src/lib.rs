//! Offline signer for one-input P2WPKH testnet transactions.

pub mod cli;
mod error;
pub mod payload;
pub mod signing;

pub use error::Error;
