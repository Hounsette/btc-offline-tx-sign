//! Building and signing the transaction for a validated payment.

use bitcoin::absolute::LockTime;
use bitcoin::secp256k1::{Message, Secp256k1};
use bitcoin::sighash::{EcdsaSighashType, P2wpkhError, SighashCache};
use bitcoin::transaction::Version;
use bitcoin::{
    CompressedPublicKey, PrivateKey, ScriptBuf, Sequence, Transaction, TxIn, Witness, ecdsa,
};

use crate::payload::Payment;

/// Why a payment could not be signed with the given key.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("private key is uncompressed; P2WPKH requires a compressed key")]
    UncompressedKey,
    #[error("private key does not control the previous output")]
    KeyMismatch,
    // Unreachable and untested: the key check proves the script is P2WPKH, and input 0 exists.
    #[error("failed to compute the signature hash")]
    Sighash(#[from] P2wpkhError),
}

/// Builds the transaction for `payment` and signs its input with `key`.
///
/// The transaction is version 2 with locktime 0 and RBF signalled by sequence `0xFFFFFFFD`. The
/// BIP143 signature (`SIGHASH_ALL`) commits to the verified previous output's amount and script.
///
/// # Errors
///
/// Returns [`Error::UncompressedKey`] or [`Error::KeyMismatch`] if `key` cannot spend the
/// previous output.
pub fn sign(payment: &Payment, key: &PrivateKey) -> Result<Transaction, Error> {
    let secp = Secp256k1::signing_only();
    let public_key =
        CompressedPublicKey::from_private_key(&secp, key).map_err(|_| Error::UncompressedKey)?;
    let prev_output = payment.prev_output();
    if ScriptBuf::new_p2wpkh(&public_key.wpubkey_hash()) != prev_output.script_pubkey {
        return Err(Error::KeyMismatch);
    }

    let mut tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: payment.outpoint(),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
        }],
        output: payment.outputs().to_vec(),
    };
    let sighash = SighashCache::new(&tx).p2wpkh_signature_hash(
        0,
        &prev_output.script_pubkey,
        prev_output.value,
        EcdsaSighashType::All,
    )?;
    let signature = secp.sign_ecdsa(&Message::from(sighash), &key.inner);
    tx.input[0].witness = Witness::p2wpkh(&ecdsa::Signature::sighash_all(signature), &public_key.0);
    Ok(tx)
}
