//! Errors from reading and validating a payment payload.

use std::fmt;
use std::iter::successors;

use bitcoin::address::NetworkUnchecked;
use bitcoin::consensus::encode::FromHexError;
use bitcoin::{Address, Amount, Txid};

use super::{MAX_FEE, MAX_PARAMS_BYTES};

/// Why a payload was rejected. Display includes the full chain, escaped for terminal output.
#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    Constraints(ConstraintError),
    PrevTx(PrevTxError),
    Output(OutputError),
    Balance(BalanceError),
}

impl Error {
    fn inner(&self) -> &(dyn std::error::Error + 'static) {
        match self {
            Self::Parse(error) => error,
            Self::Constraints(error) => error,
            Self::PrevTx(error) => error,
            Self::Output(error) => error,
            Self::Balance(error) => error,
        }
    }

    /// Formats the full error chain with control characters escaped for terminal output.
    /// Preserves quotes and backslashes in already formatted messages.
    /// Original errors remain available through `source()` and must not be printed directly.
    pub fn report(&self) -> String {
        // Start at the inner error: formatting self would recurse through Display.
        let chain = successors(Some(self.inner()), |error| error.source())
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(": ");
        let mut escaped = String::with_capacity(chain.len());
        for c in chain.chars() {
            match c {
                '"' | '\'' | '\\' => escaped.push(c),
                c => escaped.extend(c.escape_debug()),
            }
        }
        escaped
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.report())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner().source()
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

impl From<ConstraintError> for Error {
    fn from(error: ConstraintError) -> Self {
        Self::Constraints(error)
    }
}

impl From<PrevTxError> for Error {
    fn from(error: PrevTxError) -> Self {
        Self::PrevTx(error)
    }
}

impl From<OutputError> for Error {
    fn from(error: OutputError) -> Self {
        Self::Output(error)
    }
}

impl From<BalanceError> for Error {
    fn from(error: BalanceError) -> Self {
        Self::Balance(error)
    }
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
