//! Errors from reading and validating a payment payload.

use bitcoin::address::NetworkUnchecked;
use bitcoin::consensus::encode::FromHexError;
use bitcoin::{Address, Amount, Txid};

use super::{MAX_FEE, MAX_PARAMS_BYTES};

/// Why a payload was rejected. Library details are in `source()`; report the whole chain.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Constraints(#[from] ConstraintError),
    #[error(transparent)]
    PrevTx(#[from] PrevTxError),
    #[error(transparent)]
    Output(#[from] OutputError),
    #[error(transparent)]
    Balance(#[from] BalanceError),
}

/// Reading the bounded payload and decoding its JSON structure.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("failed to read params")]
    Read(#[from] std::io::Error),
    #[error("params exceed {} bytes", MAX_PARAMS_BYTES)]
    TooLarge,
    #[error("invalid params JSON")]
    Json(#[from] serde_json::Error),
}

/// Supported network and input/output counts.
#[derive(Debug, thiserror::Error)]
pub enum ConstraintError {
    #[error("unsupported network {0:?}; only testnet is supported")]
    UnsupportedNetwork(String),
    #[error("expected exactly one input, found {0}")]
    InputCount(usize),
    #[error("expected one or two outputs, found {0}")]
    OutputCount(usize),
}

/// Verification of the previous transaction and the output being spent.
#[derive(Debug, thiserror::Error)]
pub enum PrevTxError {
    #[error("prev_tx is not a valid transaction")]
    Decode(#[from] FromHexError),
    #[error("prev_tx has txid {actual}, expected {expected}")]
    TxidMismatch { actual: Txid, expected: Txid },
    #[error("prev_tx has no output {0}")]
    MissingOutput(u32),
    #[error("previous output {0} is not P2WPKH")]
    NotP2wpkh(u32),
    #[error("previous output amount {} sats exceeds the maximum supply", .0.to_sat())]
    AboveMaxMoney(Amount),
}

/// Destination address, payment amount, and change validation.
#[derive(Debug, thiserror::Error)]
pub enum OutputError {
    #[error("output address {} is not valid on testnet", .0.assume_checked_ref())]
    AddressNotTestnet(Address<NetworkUnchecked>),
    #[error("output address {0} is not P2WPKH")]
    AddressNotP2wpkh(Address),
    #[error("output amount must be positive")]
    ZeroAmount,
    #[error("output amount {} sats exceeds the maximum supply", .0.to_sat())]
    AmountAboveMaxMoney(Amount),
    #[error("change output must pay back to the address being spent")]
    ForeignChange,
}

/// Transaction totals and fee validation.
#[derive(Debug, thiserror::Error)]
pub enum BalanceError {
    // shortcut: current output limits prevent overflow; add a regression test when those limits allow it.
    #[error("output total overflows")]
    TotalOverflow,
    #[error("outputs ({} sats) exceed the input ({} sats)", .outputs.to_sat(), .input.to_sat())]
    Overspend { outputs: Amount, input: Amount },
    #[error("fee {} sats exceeds the {} sat cap", .0.to_sat(), MAX_FEE.to_sat())]
    FeeAboveCap(Amount),
}
