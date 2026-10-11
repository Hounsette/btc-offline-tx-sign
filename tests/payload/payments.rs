use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Amount};
use serde_json::json;

use btc_offline_signer::payload::{BalanceError, Error, MAX_FEE, OutputError};

use crate::common::{
    ABOVE_MAX_MONEY_SATS, KEY_1_SCRIPT, P2WSH_ADDRESS, RECIPIENT_ADDRESS, RECIPIENT_SCRIPT,
    fixture, fixture_with, fixture_with_prev_tx, parse_error, parse_ok,
};

#[test]
fn accepts_specification_fixture() {
    let payment = parse_ok(fixture());

    assert_eq!(
        payment.outpoint.to_string(),
        "71e3be4e5b89d4263d5902a50180ea5b660a3af3ee8894aaa024ec0f2391a0f6:0"
    );
    assert_eq!(payment.prev_output.value, Amount::from_sat(100_000));
    assert_eq!(
        payment.prev_output.script_pubkey.to_hex_string(),
        KEY_1_SCRIPT
    );
    assert_eq!(payment.outputs.len(), 1);
    assert_eq!(payment.outputs[0].value, Amount::from_sat(95_000));
    assert_eq!(
        payment.outputs[0].script_pubkey.to_hex_string(),
        KEY_1_SCRIPT
    );
    assert_eq!(payment.fee, Amount::from_sat(5_000));
}

#[test]
fn pays_the_requested_recipient() {
    let payment = parse_ok(fixture_with("/outputs/0/address", json!(RECIPIENT_ADDRESS)));

    assert_eq!(
        payment.prev_output.script_pubkey.to_hex_string(),
        KEY_1_SCRIPT
    );
    assert_eq!(
        payment.outputs[0].script_pubkey.to_hex_string(),
        RECIPIENT_SCRIPT
    );
}

#[test]
fn returns_address_not_testnet_for_mainnet_address() {
    let mainnet = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    assert!(matches!(
        parse_error(fixture_with("/outputs/0/address", json!(mainnet))),
        Error::Output(OutputError::AddressNotTestnet(address))
            if address == mainnet.parse::<Address<NetworkUnchecked>>().unwrap()
    ));
}

#[test]
fn returns_address_not_p2wpkh_for_p2wsh_address() {
    assert!(matches!(
        parse_error(fixture_with("/outputs/0/address", json!(P2WSH_ADDRESS))),
        Error::Output(OutputError::AddressNotP2wpkh(address))
            if address == P2WSH_ADDRESS.parse::<Address<NetworkUnchecked>>().unwrap().assume_checked()
    ));
}

#[test]
fn returns_zero_amount_for_zero_output() {
    assert!(matches!(
        parse_error(fixture_with("/outputs/0/amount_sats", json!(0))),
        Error::Output(OutputError::ZeroAmount)
    ));
}

#[test]
fn returns_amount_above_max_money_for_excessive_output() {
    assert!(matches!(
        parse_error(fixture_with("/outputs/0/amount_sats", json!(ABOVE_MAX_MONEY_SATS))),
        Error::Output(OutputError::AmountAboveMaxMoney(value))
            if value.to_sat() == ABOVE_MAX_MONEY_SATS
    ));
}

#[test]
fn accepts_maximum_money_amount() {
    let mut params = fixture_with_prev_tx(|prev_tx| prev_tx.output[0].value = Amount::MAX_MONEY);
    params["outputs"][0]["amount_sats"] = json!(Amount::MAX_MONEY.to_sat());
    assert_eq!(parse_ok(params).fee, Amount::ZERO);
}

#[test]
fn returns_overspend_when_outputs_exceed_input() {
    let exact = parse_ok(fixture_with("/outputs/0/amount_sats", json!(100_000)));
    assert_eq!(exact.fee, Amount::ZERO);

    assert!(matches!(
        parse_error(fixture_with("/outputs/0/amount_sats", json!(100_001))),
        Error::Balance(BalanceError::Overspend { outputs, input })
            if outputs.to_sat() == 100_001 && input.to_sat() == 100_000
    ));
}

#[test]
fn returns_fee_above_cap_when_fee_exceeds_limit() {
    let mut params =
        fixture_with_prev_tx(|prev_tx| prev_tx.output[0].value = Amount::from_sat(2_000_000));

    params["outputs"][0]["amount_sats"] = json!(1_000_000);
    assert_eq!(parse_ok(&params).fee, MAX_FEE);

    params["outputs"][0]["amount_sats"] = json!(999_999);
    assert!(matches!(
        parse_error(params),
        Error::Balance(BalanceError::FeeAboveCap(fee)) if fee.to_sat() == 1_000_001
    ));
}
