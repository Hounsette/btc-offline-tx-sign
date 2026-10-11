//! The text users see. The `returns_*` tests check which rejection occurs; these check
//! that each message names the rejected values correctly. `Error` shows its group's
//! message unchanged, so the groups are checked directly.

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount, Txid};
use serde_json::json;

use btc_offline_signer::payload::{
    BalanceError, ConstraintError, OutputError, ParseError, PrevTxError,
};

use crate::common::{
    ABOVE_MAX_MONEY_SATS, P2WSH_ADDRESS, fixture, fixture_with, parse_error, report,
};

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

    // The top-level message alone would hide why decoding failed.
    assert_eq!(error.to_string(), "prev_tx is not a valid transaction");
    let report = report(&error);
    assert!(
        report.starts_with("prev_tx is not a valid transaction: decoding error: "),
        "{report}"
    );
    assert!(report.ends_with("yielded too many bytes"), "{report}");
}
