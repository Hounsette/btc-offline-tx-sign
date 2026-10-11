//! Errors from signing a validated payment.

use bitcoin::sighash::P2wpkhError;

/// Why a payment could not be signed with the given key.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("private key is for mainnet; only testnet keys are supported")]
    MainnetKey,
    #[error("private key is uncompressed; P2WPKH requires a compressed key")]
    UncompressedKey,
    #[error("private key does not control the previous output")]
    KeyMismatch,
    // Unreachable and untested: the key check proves the script is P2WPKH, and input 0 exists.
    #[error("failed to compute the signature hash")]
    Sighash(#[from] P2wpkhError),
}
