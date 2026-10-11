// Marks helpers as test code so clippy.toml's allow-unwrap-in-tests covers them.
#![cfg(test)]

//! Signing payments with public test keys. Never fund these keys.

// Each test binary uses only some of the shared helpers.
#[allow(dead_code)]
mod common;

use bitcoin::absolute::LockTime;
use bitcoin::consensus::encode::{deserialize_hex, serialize_hex};
use bitcoin::secp256k1::{Message, Secp256k1};
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::transaction::Version;
use bitcoin::{
    Amount, CompressedPublicKey, NetworkKind, PrivateKey, ScriptBuf, Sequence, Transaction, TxOut,
    ecdsa,
};
use serde_json::json;

use btc_offline_signer::payload::Payment;
use btc_offline_signer::signing::{Error, sign};

use crate::common::{
    KEY_1_WIF, KEY_2_WIF, P2WSH_ADDRESS, RECIPIENT_ADDRESS, RECIPIENT_SCRIPT, fixture_with,
    fixture_with_change, fixture_with_prev_tx, parse_ok, script_of,
};

/// The fixture paying another recipient. Its own output pays back to the spent script, which
/// would hide a signature hash computed over the wrong script.
fn payment() -> Payment {
    parse_ok(fixture_with("/outputs/0/address", json!(RECIPIENT_ADDRESS)))
}

fn key(wif: &str) -> PrivateKey {
    PrivateKey::from_wif(wif).unwrap()
}

fn public_key(wif: &str) -> CompressedPublicKey {
    CompressedPublicKey::from_private_key(&Secp256k1::new(), &key(wif)).unwrap()
}

/// The BIP143 hash for P2WPKH input `index`, computed apart from `sign` and checked against
/// BIP143's published example.
fn sighash(tx: &Transaction, index: usize, prev_output: &TxOut) -> Message {
    let hash = SighashCache::new(tx)
        .p2wpkh_signature_hash(
            index,
            &prev_output.script_pubkey,
            prev_output.value,
            EcdsaSighashType::All,
        )
        .unwrap();
    Message::from(hash)
}

/// Whether the input's witness signature is valid for `public_key` over the current transaction.
fn verifies(tx: &Transaction, prev_output: &TxOut, public_key: &CompressedPublicKey) -> bool {
    let signature = ecdsa::Signature::from_slice(&tx.input[0].witness[0]).unwrap();
    public_key
        .verify(
            &Secp256k1::verification_only(),
            &sighash(tx, 0, prev_output),
            &signature,
        )
        .is_ok()
}

#[test]
fn sighash_matches_bip143_p2wpkh_example() {
    let tx: Transaction = deserialize_hex(concat!(
        "0100000002fff7f7881a8099afa6940d42d1e7f6362bec38171ea3edf433541db4e4ad969f00000000",
        "00eeffffffef51e1b804cc89d182d279655c3aa89e815b1b309fe287d9b2b55d57b90ec68a01000000",
        "00ffffffff02202cb206000000001976a9148280b37df378db99f66f85c95a783a76ac7a6d5988ac90",
        "93510d000000001976a9143bde42dbee7e4dbe6a21b2d50ce2f0167faa815988ac11000000",
    ))
    .unwrap();
    // The second input spends 6 BTC from a P2WPKH script.
    let prev_output = TxOut {
        value: Amount::from_sat(600_000_000),
        script_pubkey: ScriptBuf::from_hex("00141d0f172a0ecb48aee1be1f2687d2963ae33f71a1").unwrap(),
    };

    assert_eq!(
        sighash(&tx, 1, &prev_output).to_string(),
        "c37af31116d1b27caf68aae9e3ac82f1477929014d5b917657d0eb49478cb670"
    );
}

#[test]
fn signs_verifiable_p2wpkh_transaction() {
    let payment = payment();
    let tx = sign(&payment, &key(KEY_1_WIF)).unwrap();

    assert_eq!(tx.version, Version::TWO);
    assert_eq!(tx.lock_time, LockTime::ZERO);
    assert_eq!(tx.output, payment.outputs());
    assert_eq!(tx.output[0].script_pubkey.to_hex_string(), RECIPIENT_SCRIPT);
    assert_eq!(tx.input.len(), 1);
    let input = &tx.input[0];
    assert_eq!(input.previous_output, payment.outpoint());
    assert_eq!(input.sequence, Sequence(0xFFFF_FFFD));
    assert!(input.script_sig.is_empty());
    // P2WPKH witness: DER signature plus sighash byte, then the compressed public key.
    assert_eq!(input.witness.len(), 2);
    let signature = ecdsa::Signature::from_slice(&input.witness[0]).unwrap();
    assert_eq!(signature.sighash_type, EcdsaSighashType::All);
    assert_eq!(&input.witness[1], public_key(KEY_1_WIF).to_bytes());

    let decoded: Transaction = deserialize_hex(&serialize_hex(&tx)).unwrap();
    assert_eq!(decoded, tx);
    assert!(verifies(
        &decoded,
        payment.prev_output(),
        &public_key(KEY_1_WIF)
    ));
}

#[test]
fn signs_payment_spending_nonzero_vout() {
    // A decoy output moves the spent output to index 1.
    let mut params = fixture_with_prev_tx(|prev_tx| {
        let decoy = TxOut {
            value: Amount::from_sat(1),
            script_pubkey: script_of(P2WSH_ADDRESS),
        };
        prev_tx.output.insert(0, decoy);
    });
    params["inputs"][0]["vout"] = json!(1);
    params["outputs"][0]["address"] = json!(RECIPIENT_ADDRESS);
    let payment = parse_ok(params);
    let tx = sign(&payment, &key(KEY_1_WIF)).unwrap();

    assert_eq!(tx.input[0].previous_output, payment.outpoint());
    assert_eq!(tx.input[0].previous_output.vout, 1);
    assert!(verifies(&tx, payment.prev_output(), &public_key(KEY_1_WIF)));

    // The signature commits to the outpoint, so pointing at the decoy invalidates it.
    let mut changed = tx.clone();
    changed.input[0].previous_output.vout = 0;
    assert!(!verifies(
        &changed,
        payment.prev_output(),
        &public_key(KEY_1_WIF)
    ));
}

#[test]
fn signature_fails_for_changed_output_or_input_amount() {
    let payment = payment();
    let tx = sign(&payment, &key(KEY_1_WIF)).unwrap();
    let key_1 = public_key(KEY_1_WIF);

    let mut changed = tx.clone();
    changed.output[0].value = Amount::from_sat(94_999);
    assert!(!verifies(&changed, payment.prev_output(), &key_1));

    let mut changed = tx.clone();
    changed.output[0].script_pubkey = ScriptBuf::new_p2wpkh(&public_key(KEY_2_WIF).wpubkey_hash());
    assert!(!verifies(&changed, payment.prev_output(), &key_1));

    // BIP143 commits to the spent amount, so a different input value also invalidates it.
    let other_amount = TxOut {
        value: Amount::from_sat(99_999),
        ..payment.prev_output().clone()
    };
    assert!(!verifies(&tx, &other_amount, &key_1));
}

#[test]
fn signs_change_and_fails_after_changing_either_output() {
    let payment = parse_ok(fixture_with_change());
    let tx = sign(&payment, &key(KEY_1_WIF)).unwrap();
    let key_1 = public_key(KEY_1_WIF);

    assert_eq!(tx.output, payment.outputs());
    assert!(verifies(&tx, payment.prev_output(), &key_1));

    for index in 0..2 {
        let mut changed = tx.clone();
        changed.output[index].value = Amount::from_sat(1);
        assert!(!verifies(&changed, payment.prev_output(), &key_1));

        let mut changed = tx.clone();
        changed.output[index].script_pubkey = script_of(P2WSH_ADDRESS);
        assert!(!verifies(&changed, payment.prev_output(), &key_1));
    }

    // Moving the fee into change, or swapping the outputs, also invalidates it.
    let mut changed = tx.clone();
    changed.output[1].value = Amount::from_sat(40_000);
    assert!(!verifies(&changed, payment.prev_output(), &key_1));
    let mut changed = tx.clone();
    changed.output.swap(0, 1);
    assert!(!verifies(&changed, payment.prev_output(), &key_1));
}

#[test]
fn signature_fails_for_other_public_key() {
    let payment = payment();
    let tx = sign(&payment, &key(KEY_1_WIF)).unwrap();

    assert!(!verifies(
        &tx,
        payment.prev_output(),
        &public_key(KEY_2_WIF)
    ));
}

#[test]
fn returns_key_mismatch_for_other_key() {
    assert!(matches!(
        sign(&payment(), &key(KEY_2_WIF)),
        Err(Error::KeyMismatch)
    ));
}

#[test]
fn returns_uncompressed_key_err_for_uncompressed_key() {
    // The right secret still fails: P2WPKH commits to the compressed public key.
    let uncompressed = PrivateKey {
        compressed: false,
        ..key(KEY_1_WIF)
    };
    assert!(matches!(
        sign(&payment(), &uncompressed),
        Err(Error::UncompressedKey)
    ));
}

#[test]
fn returns_mainnet_key_for_mainnet_key() {
    // The right secret still fails: only testnet keys are accepted.
    let mainnet = PrivateKey {
        network: NetworkKind::Main,
        ..key(KEY_1_WIF)
    };
    assert!(matches!(sign(&payment(), &mainnet), Err(Error::MainnetKey)));
}
