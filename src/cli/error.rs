//! Errors from the command-line flow.

use std::io;
use std::path::PathBuf;

/// Failures of the command-line flow itself. The CLI prints them as part of [`crate::Error`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open params file {path:?}")]
    OpenParams { path: PathBuf, source: io::Error },
    #[error("terminal I/O failed")]
    Terminal(#[source] io::Error),
    #[error("signing cancelled")]
    Cancelled,
    // No source: base58 errors can quote characters of the key.
    #[error("private key is not a valid WIF")]
    InvalidKey,
    #[error("failed to write the signed transaction")]
    WriteTransaction(#[source] io::Error),
}
