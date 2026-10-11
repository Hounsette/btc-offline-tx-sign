//! Shared helpers and synthetic fixtures using public test keys. Never fund these keys.

use std::fmt::Display;

use bitcoin::consensus::encode::{deserialize_hex, serialize_hex};
use bitcoin::{Address, ScriptBuf, Transaction};
use serde_json::{Value, json};

use btc_offline_signer::payload::{Error, ParseError, Payment, parse};

/// Public test key 1; the specification's fixture both spends from and pays to it.
pub(crate) const KEY_1_ADDRESS: &str = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
pub(crate) const KEY_1_SCRIPT: &str = "0014751e76e8199196d454941c45d1b3a323f1433bd6";
/// Secret 1, which owns the fixture's previous output.
pub(crate) const KEY_1_WIF: &str = "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87JcbXMTcA";
/// Secret 2.
pub(crate) const KEY_2_WIF: &str = "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87K7XCyj5v";
/// BIP84's first receiving key hash, encoded for testnet.
pub(crate) const RECIPIENT_ADDRESS: &str = "tb1qcr8te4kr609gcawutmrza0j4xv80jy8zmfp6l0";
pub(crate) const RECIPIENT_SCRIPT: &str = "0014c0cebcd6c3d3ca8c75dc5ec62ebe55330ef910e2";
/// BIP173 testnet P2WSH address.
pub(crate) const P2WSH_ADDRESS: &str =
    "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7";
pub(crate) const ABOVE_MAX_MONEY_SATS: u64 = 2_100_000_000_000_001;

/// The specification's fixture: spends 100,000 sats, pays 95,000.
pub(crate) fn fixture() -> Value {
    serde_json::from_str(include_str!("payload.json")).unwrap()
}

/// The fixture with the value at JSON `pointer` replaced.
pub(crate) fn fixture_with(pointer: &str, value: Value) -> Value {
    let mut params = fixture();
    *params.pointer_mut(pointer).unwrap() = value;
    params
}

/// The fixture paying 60,000 sats to the recipient and 35,000 sats of change back to key 1.
pub(crate) fn fixture_with_change() -> Value {
    fixture_with(
        "/outputs",
        json!([
            {"address": RECIPIENT_ADDRESS, "amount_sats": 60_000},
            {"address": KEY_1_ADDRESS, "amount_sats": 35_000},
        ]),
    )
}

/// The fixture with an edited previous transaction, re-encoded with a matching txid.
pub(crate) fn fixture_with_prev_tx(edit: impl FnOnce(&mut Transaction)) -> Value {
    let mut params = fixture();
    let mut prev_tx: Transaction =
        deserialize_hex(params["inputs"][0]["prev_tx"].as_str().unwrap()).unwrap();
    edit(&mut prev_tx);
    params["inputs"][0]["prev_tx"] = json!(serialize_hex(&prev_tx));
    params["inputs"][0]["txid"] = json!(prev_tx.compute_txid());
    params
}

pub(crate) fn script_of(address: &str) -> ScriptBuf {
    address
        .parse::<Address<_>>()
        .unwrap()
        .assume_checked()
        .script_pubkey()
}

pub(crate) fn parse_ok(params: impl Display) -> Payment {
    parse(params.to_string().as_bytes()).unwrap()
}

#[track_caller]
pub(crate) fn parse_error(params: impl Display) -> Error {
    parse(params.to_string().as_bytes()).unwrap_err()
}

/// Asserts a JSON rejection whose serde message contains `reason`.
#[track_caller]
pub(crate) fn assert_json_rejected(params: impl Display, reason: &str) {
    let error = parse_error(params);
    assert!(
        matches!(&error, Error::Parse(ParseError::Json(source)) if source.to_string().contains(reason)),
        "expected {reason:?} in {error:?}"
    );
}
