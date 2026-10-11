//! The text users see. The `returns_*` tests check which rejection occurs; these check
//! that each message names the rejected values correctly. Group messages are checked
//! directly; `Error` includes the full chain and escapes it for terminal output.

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, Txid};
use serde_json::json;

use btc_offline_signer::payload::{
    BalanceError, ConstraintError, Error, OutputError, ParseError, PrevTxError,
};

use crate::common::{ABOVE_MAX_MONEY_SATS, P2WSH_ADDRESS, fixture, fixture_with, parse_error};

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
fn report_includes_library_details_from_sources() {
    let prev_tx = fixture()["inputs"][0]["prev_tx"]
        .as_str()
        .unwrap()
        .to_owned();
    let error = parse_error(fixture_with(
        "/inputs/0/prev_tx",
        json!(format!("{prev_tx}00")),
    ));

    let report = error.report();
    assert_eq!(error.to_string(), report);
    assert!(
        report.starts_with("prev_tx is not a valid transaction: decoding error: "),
        "{report}"
    );
    assert!(report.ends_with("yielded too many bytes"), "{report}");
}

#[test]
fn report_escapes_control_characters_in_unknown_fields() {
    use std::error::Error as _;

    let field = "攻撃\u{1b}[2J\r\n\t\u{7}\u{9b}31m\u{202e}";
    for pointer in ["", "/inputs/0", "/outputs/0"] {
        let mut params = fixture();
        params.pointer_mut(pointer).unwrap()[field] = json!(0);
        let error = parse_error(params);
        let report = error.report();

        assert_eq!(format!("{error}"), report);
        assert!(report.contains(r"unknown field `攻撃\u{1b}[2J\r\n\t\u{7}\u{9b}31m\u{202e}`"));
        assert!(!report.chars().any(char::is_control));
        assert!(error.source().unwrap().to_string().contains(field));
    }
}

#[test]
fn report_escapes_io_errors_and_preserves_readable_text() {
    let error = Error::from(ParseError::Read(std::io::Error::other(
        "device disconnected: 日本語\n\u{1b}]0;FORGED TITLE\u{7}",
    )));
    assert_eq!(
        error.report(),
        r"failed to read params: device disconnected: 日本語\n\u{1b}]0;FORGED TITLE\u{7}"
    );
    assert_eq!(error.to_string(), error.report());
    let plain = Error::from(ParseError::TooLarge);
    assert_eq!(plain.report(), plain.to_string());
}

#[test]
fn report_preserves_already_escaped_network_messages() {
    for network in ["testnet4", "\u{1b}[2J", r#"test'net"\4"#] {
        let network_error = parse_error(fixture_with("/network", json!(network)));
        let expected = ConstraintError::UnsupportedNetwork(network.to_owned()).to_string();
        assert_eq!(network_error.report(), expected);
        assert_eq!(network_error.to_string(), expected);
    }
}

#[test]
fn report_preserves_already_escaped_serde_messages() {
    use std::error::Error as _;

    for amount in ["95000", "\u{1b}[2J", r#"95'000"\sats"#] {
        let error = parse_error(fixture_with("/outputs/0/amount_sats", json!(amount)));
        assert_eq!(
            error.report(),
            format!("invalid params JSON: {}", error.source().unwrap())
        );
        assert_eq!(error.to_string(), error.report());
    }
}

#[test]
fn report_preserves_literal_quotes_and_backslashes() {
    let error = Error::from(ParseError::Read(std::io::Error::other(
        r#"can't read "params\file.json""#,
    )));
    assert_eq!(
        error.report(),
        r#"failed to read params: can't read "params\file.json""#
    );
    assert_eq!(error.to_string(), error.report());
}

#[test]
fn display_preserves_messages_and_sources_for_all_error_groups() {
    use std::error::Error as _;

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
        assert_eq!(format!("{error}"), expected);
        assert_eq!(error.report(), expected);
        assert!(error.source().is_none());
    }
}
