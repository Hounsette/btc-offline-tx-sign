use bitcoin::{Amount, TxOut, Txid};
use serde_json::json;

use btc_offline_signer::payload::{Error, PrevTxError};

use crate::common::{
    ABOVE_MAX_MONEY_SATS, KEY_1_SCRIPT, P2WSH_ADDRESS, fixture, fixture_with, fixture_with_prev_tx,
    parse_error, parse_ok, script_of,
};

#[test]
fn accepts_segwit_prev_tx_and_nonzero_vout() {
    let mut params = fixture_with_prev_tx(|prev_tx| {
        prev_tx.input[0].witness.push([0x01]);
        let decoy = TxOut {
            value: Amount::from_sat(1),
            script_pubkey: script_of(P2WSH_ADDRESS),
        };
        prev_tx.output.insert(0, decoy);
    });
    params["inputs"][0]["vout"] = json!(1);
    // Version 2, then the segwit marker and flag: the txid must exclude the witness.
    let prev_tx = params["inputs"][0]["prev_tx"].as_str().unwrap();
    assert!(prev_tx.starts_with("020000000001"));

    let payment = parse_ok(&params);
    assert_eq!(payment.outpoint.vout, 1);
    assert_eq!(payment.prev_output.value, Amount::from_sat(100_000));
    assert_eq!(
        payment.prev_output.script_pubkey.to_hex_string(),
        KEY_1_SCRIPT
    );
}

#[test]
fn returns_txid_mismatch_when_txid_does_not_match_prev_tx() {
    let other_txid = "0000000000000000000000000000000000000000000000000000000000000001";
    let fixture_txid: Txid = fixture()["inputs"][0]["txid"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(matches!(
        parse_error(fixture_with("/inputs/0/txid", json!(other_txid))),
        Error::PrevTx(PrevTxError::TxidMismatch { actual, expected })
            if actual == fixture_txid && expected == other_txid.parse::<Txid>().unwrap()
    ));
}

#[test]
fn returns_missing_output_for_out_of_range_vout() {
    assert!(matches!(
        parse_error(fixture_with("/inputs/0/vout", json!(1))),
        Error::PrevTx(PrevTxError::MissingOutput(1))
    ));
}

#[test]
fn returns_decode_for_invalid_prev_tx_encoding() {
    use std::error::Error as _;

    let prev_tx = fixture()["inputs"][0]["prev_tx"]
        .as_str()
        .unwrap()
        .to_owned();
    for bad in [
        "zz".to_owned(),
        prev_tx[..prev_tx.len() / 2].to_owned(),
        format!("{prev_tx}00"),
    ] {
        let error = parse_error(fixture_with("/inputs/0/prev_tx", json!(bad)));
        assert!(matches!(&error, Error::PrevTx(PrevTxError::Decode(_))));
        assert!(
            error
                .source()
                .unwrap()
                .is::<bitcoin::consensus::encode::FromHexError>()
        );
    }
}

#[test]
fn returns_not_p2wpkh_for_p2wsh_prev_output() {
    assert!(matches!(
        parse_error(fixture_with_prev_tx(|prev_tx| {
            prev_tx.output[0].script_pubkey = script_of(P2WSH_ADDRESS);
        })),
        Error::PrevTx(PrevTxError::NotP2wpkh(0))
    ));
}

#[test]
fn returns_above_max_money_for_excessive_prev_output() {
    assert!(matches!(
        parse_error(fixture_with_prev_tx(|prev_tx| {
            prev_tx.output[0].value = Amount::from_sat(ABOVE_MAX_MONEY_SATS);
        })),
        Error::PrevTx(PrevTxError::AboveMaxMoney(value))
            if value.to_sat() == ABOVE_MAX_MONEY_SATS
    ));
}
