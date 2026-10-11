//! The text users see. The `returns_*` tests check which rejection occurs; these check
//! that each message names the rejected values correctly. Group messages are checked
//! directly; the top-level `Error` the CLI prints shows the full chain, escaped for terminal
//! output.

use std::error::Error as _;

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, Txid};
use serde_json::json;

use btc_offline_signer::payload::{
    BalanceError, ConstraintError, Error, OutputError, ParseError, PrevTxError,
};

use crate::common::{ABOVE_MAX_MONEY_SATS, P2WSH_ADDRESS, fixture, fixture_with, parse_error};

/// The message the CLI prints for `error`.
fn shown(error: Error) -> String {
    btc_offline_signer::Error::from(error).to_string()
}

const FIXTURE_TXID: &str = "71e3be4e5b89d4263d5902a50180ea5b660a3af3ee8894aaa024ec0f2391a0f6";
const OTHER_TXID: &str = "0000000000000000000000000000000000000000000000000000000000000001";
const MAINNET_ADDRESS: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";

#[test]
fn messages_name_the_rejected_values() {
    let sats = Amount::from_sat;
    let mainnet: Address<NetworkUnchecked> = MAINNET_ADDRESS.parse().unwrap();
    let p2wsh = P2WSH_ADDRESS
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked();

    assert_eq!(
        ParseError::TooLarge.to_string(),
        "params exceed 16777216 bytes"
    );
    assert_eq!(
        ConstraintError::UnsupportedNetwork("bitcoin".to_owned()).to_string(),
        r#"unsupported network "bitcoin"; only testnet is supported"#
    );
    // Quoting escapes terminal control characters from untrusted input.
    assert_eq!(
        ConstraintError::UnsupportedNetwork("\u{1b}[2J".to_owned()).to_string(),
        r#"unsupported network "\u{1b}[2J"; only testnet is supported"#
    );
    assert_eq!(
        ConstraintError::InputCount(2).to_string(),
        "expected exactly one input, found 2"
    );
    assert_eq!(
        ConstraintError::OutputCount(0).to_string(),
        "expected one or two outputs, found 0"
    );
    assert_eq!(
        PrevTxError::TxidMismatch {
            actual: FIXTURE_TXID.parse::<Txid>().unwrap(),
            expected: OTHER_TXID.parse::<Txid>().unwrap(),
        }
        .to_string(),
        format!("prev_tx has txid {FIXTURE_TXID}, expected {OTHER_TXID}")
    );
    assert_eq!(
        PrevTxError::MissingOutput(1).to_string(),
        "prev_tx has no output 1"
    );
    assert_eq!(
        PrevTxError::NotP2wpkh(0).to_string(),
        "previous output 0 is not P2WPKH"
    );
    assert_eq!(
        PrevTxError::AboveMaxMoney(sats(ABOVE_MAX_MONEY_SATS)).to_string(),
        "previous output amount 2100000000000001 sats exceeds the maximum supply"
    );
    assert_eq!(
        OutputError::AddressNotTestnet(mainnet).to_string(),
        format!("output address {MAINNET_ADDRESS} is not valid on testnet")
    );
    assert_eq!(
        OutputError::AddressNotP2wpkh(p2wsh).to_string(),
        format!("output address {P2WSH_ADDRESS} is not P2WPKH")
    );
    assert_eq!(
        OutputError::AmountAboveMaxMoney(sats(ABOVE_MAX_MONEY_SATS)).to_string(),
        "output amount 2100000000000001 sats exceeds the maximum supply"
    );
    assert_eq!(
        OutputError::ForeignChange.to_string(),
        "change output must pay back to the address being spent"
    );
    assert_eq!(
        BalanceError::Overspend {
            outputs: sats(100_001),
            input: sats(100_000),
        }
        .to_string(),
        "outputs (100001 sats) exceed the input (100000 sats)"
    );
    assert_eq!(
        BalanceError::FeeAboveCap(sats(1_000_001)).to_string(),
        "fee 1000001 sats exceeds the 1000000 sat cap"
    );
}

#[test]
fn shown_includes_library_details_from_sources() {
    let prev_tx = fixture()["inputs"][0]["prev_tx"]
        .as_str()
        .unwrap()
        .to_owned();
    let error = parse_error(fixture_with(
        "/inputs/0/prev_tx",
        json!(format!("{prev_tx}00")),
    ));

    // The payload error's own message alone would hide why decoding failed.
    assert_eq!(error.to_string(), "prev_tx is not a valid transaction");
    assert!(error.source().is_some());
    // Each source appears once, though `Error` forwards its group's source.
    assert_eq!(
        shown(error),
        "prev_tx is not a valid transaction: decoding error: \
         attempted to decode object from an iterator that yielded too many bytes"
    );
}

/// A synthetic cause chain, to check every level is escaped.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct Cause {
    message: &'static str,
    source: Option<Box<Cause>>,
}

fn cause(message: &'static str, source: Option<Cause>) -> Cause {
    Cause {
        message,
        source: source.map(Box::new),
    }
}

#[test]
fn shown_escapes_each_nested_cause_once() {
    // `io::Error` shows the outer cause, then returns the outer cause's source.
    let inner = cause("line\r\nerror: forged \u{7}", None);
    let middle = cause("bad byte \u{1b}[2J", Some(inner));
    let error = Error::from(ParseError::Read(std::io::Error::other(cause(
        "decoder failed",
        Some(middle),
    ))));

    assert_eq!(
        shown(error),
        r"failed to read params: decoder failed: bad byte \u{1b}[2J: line\r\nerror: forged \u{7}"
    );
}

#[test]
fn shown_escapes_control_characters_in_unknown_fields() {
    let field = "攻撃\u{1b}[2J\r\n\t\u{7}\u{9b}31m\u{202e}";
    for pointer in ["", "/inputs/0", "/outputs/0"] {
        let mut params = fixture();
        params.pointer_mut(pointer).unwrap()[field] = json!(0);
        let error = parse_error(params);
        // The source keeps the library message unescaped.
        assert!(error.source().unwrap().to_string().contains(field));
        let shown = shown(error);

        assert!(shown.contains(r"unknown field `攻撃\u{1b}[2J\r\n\t\u{7}\u{9b}31m\u{202e}`"));
        assert!(!shown.chars().any(char::is_control));
    }
}

#[test]
fn shown_escapes_io_errors_and_preserves_readable_text() {
    let error = Error::from(ParseError::Read(std::io::Error::other(
        "device disconnected: 日本語\n\u{1b}]0;FORGED TITLE\u{7}",
    )));
    assert_eq!(
        shown(error),
        r"failed to read params: device disconnected: 日本語\n\u{1b}]0;FORGED TITLE\u{7}"
    );
}

#[test]
fn shown_preserves_already_escaped_network_messages() {
    for network in ["testnet4", "\u{1b}[2J", r#"test'net"\4"#] {
        let network_error = parse_error(fixture_with("/network", json!(network)));
        let expected = ConstraintError::UnsupportedNetwork(network.to_owned()).to_string();
        assert_eq!(network_error.to_string(), expected);
        assert_eq!(shown(network_error), expected);
    }
}

#[test]
fn shown_preserves_already_escaped_serde_messages() {
    for amount in ["95000", "\u{1b}[2J", r#"95'000"\sats"#] {
        let error = parse_error(fixture_with("/outputs/0/amount_sats", json!(amount)));
        let expected = format!("invalid params JSON: {}", error.source().unwrap());
        assert_eq!(shown(error), expected);
    }
}

#[test]
fn shown_preserves_literal_quotes_and_backslashes() {
    let error = Error::from(ParseError::Read(std::io::Error::other(
        r#"can't read "params\file.json""#,
    )));
    assert_eq!(
        shown(error),
        r#"failed to read params: can't read "params\file.json""#
    );
}

#[test]
fn shown_preserves_messages_and_sources_for_all_error_groups() {
    for (error, expected) in [
        (
            Error::from(ParseError::TooLarge),
            "params exceed 16777216 bytes",
        ),
        (
            Error::from(ConstraintError::InputCount(0)),
            "expected exactly one input, found 0",
        ),
        (
            Error::from(PrevTxError::MissingOutput(1)),
            "prev_tx has no output 1",
        ),
        (
            Error::from(OutputError::ZeroAmount),
            "output amount must be positive",
        ),
        (
            Error::from(BalanceError::TotalOverflow),
            "output total overflows",
        ),
    ] {
        assert_eq!(error.to_string(), expected);
        assert!(error.source().is_none());
        assert_eq!(shown(error), expected);
    }
}
