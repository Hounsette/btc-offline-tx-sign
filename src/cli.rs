//! The command-line flow: validate the payload, confirm it, then read the key and sign.

use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use bitcoin::consensus::encode::serialize_hex;
use bitcoin::{Address, Network, PrivateKey, Script};
use zeroize::Zeroizing;

use crate::payload::{self, Payment};
use crate::signing;

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

/// Validates the payment in `params` and shows it on `terminal`. Once `read_reply` returns the
/// line `yes`, signs with the WIF that `read_key` prompts for and returns the transaction hex.
///
/// The WIF and reply are zeroized when dropped. Copies made elsewhere are not reliably cleared:
/// base58 decoding and encoding inside `bitcoin`, and moves of the `Copy` key type.
///
/// # Errors
///
/// Returns [`Error::OpenParams`] or a [`payload::Error`] before anything is shown,
/// [`Error::Terminal`] if a prompt cannot be shown or read, [`Error::Cancelled`] for any reply
/// but `yes` or an interrupted key prompt, and [`Error::InvalidKey`] or a [`signing::Error`] if
/// the key cannot spend the input.
pub fn run(
    params: &Path,
    terminal: &mut impl Write,
    read_reply: impl FnOnce() -> io::Result<String>,
    read_key: impl FnOnce() -> io::Result<Zeroizing<String>>,
) -> Result<String, crate::Error> {
    let file = File::open(params).map_err(|source| Error::OpenParams {
        path: params.to_owned(),
        source,
    })?;
    let payment = payload::parse(file)?;

    show(terminal, &payment).map_err(Error::Terminal)?;
    // A key pasted at the wrong prompt would land here.
    let reply = Zeroizing::new(read_reply().map_err(Error::Terminal)?);
    if !matches!(reply.as_str(), "yes\n" | "yes\r\n") {
        return Err(Error::Cancelled.into());
    }

    let wif = read_key().map_err(|error| match error.kind() {
        io::ErrorKind::Interrupted => Error::Cancelled,
        _ => Error::Terminal(error),
    })?;
    let mut key = PrivateKey::from_wif(&wif).map_err(|_| Error::InvalidKey)?;
    // `from_wif` ignores the compression marker's value; a canonical WIF re-encodes to itself.
    let signed = if *Zeroizing::new(key.to_wif()) == *wif {
        signing::sign(&payment, &key).map_err(crate::Error::from)
    } else {
        Err(Error::InvalidKey.into())
    };
    key.inner.non_secure_erase();
    Ok(serialize_hex(&signed?))
}

/// Shows the spent output, each destination, and the fee, then asks for confirmation.
fn show(terminal: &mut impl Write, payment: &Payment) -> io::Result<()> {
    let input = payment.prev_output();
    writeln!(
        terminal,
        "Spend   {} sats from {}",
        input.value.to_sat(),
        payment.outpoint()
    )?;
    for (label, output) in ["Pay", "Change"].into_iter().zip(payment.outputs()) {
        writeln!(
            terminal,
            "{label:<7} {} sats to {}",
            output.value.to_sat(),
            address(&output.script_pubkey)
        )?;
    }
    writeln!(terminal, "Fee     {} sats", payment.fee().to_sat())?;
    write!(terminal, "Type yes to sign: ")?;
    terminal.flush()
}

fn address(script: &Script) -> String {
    // Outputs are validated P2WPKH, so the script fallback is unreachable.
    Address::from_script(script, Network::Testnet)
        .map_or_else(|_| script.to_string(), |address| address.to_string())
}
