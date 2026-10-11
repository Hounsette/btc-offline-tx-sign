use std::io::Read;

use serde_json::json;

use btc_offline_signer::payload::{ConstraintError, Error, MAX_PARAMS_BYTES, ParseError, parse};

use crate::common::{assert_json_rejected, fixture, fixture_with, parse_error};

#[test]
fn returns_read_for_reader_failure() {
    use std::error::Error as _;

    struct FailedReader;
    impl Read for FailedReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("device disconnected"))
        }
    }

    let read = parse(FailedReader).unwrap_err();
    assert!(matches!(
        &read,
        Error::Parse(ParseError::Read(source)) if source.kind() == std::io::ErrorKind::Other
    ));
    assert!(read.source().unwrap().is::<std::io::Error>());
}

#[test]
fn returns_too_large_for_payload_above_size_limit() {
    // JSON allows trailing whitespace, so padding keeps the payload valid.
    let mut json = fixture().to_string().into_bytes();
    json.resize(usize::try_from(MAX_PARAMS_BYTES).unwrap(), b' ');
    parse(json.as_slice()).unwrap();

    json.push(b' ');
    let error = parse(json.as_slice()).unwrap_err();
    assert!(matches!(error, Error::Parse(ParseError::TooLarge)));
}

#[test]
fn returns_json_for_malformed_json() {
    use std::error::Error as _;

    let json = fixture().to_string();
    assert_json_rejected("", "EOF while parsing a value");
    assert_json_rejected(&json[..json.len() / 2], "EOF while parsing");
    assert_json_rejected(format!("{json}{{}}"), "trailing characters");

    let error = parse_error("{");
    assert!(matches!(&error, Error::Parse(ParseError::Json(_))));
    assert!(error.source().unwrap().is::<serde_json::Error>());
}

#[test]
fn returns_json_for_duplicate_fields() {
    let json = fixture().to_string();
    assert_json_rejected(
        json.replacen('{', r#"{"network":"testnet","#, 1),
        "duplicate field `network`",
    );
    assert_json_rejected(
        json.replace(r#""vout":0"#, r#""vout":0,"vout":1"#),
        "duplicate field `vout`",
    );
}

#[test]
fn returns_json_for_unknown_fields() {
    let mut params = fixture();
    params["fee_sats"] = json!(5000);
    assert_json_rejected(params, "unknown field `fee_sats`");

    // The input amount comes from prev_tx, never from the payload.
    let mut params = fixture();
    params["inputs"][0]["amount_sats"] = json!(100_000);
    assert_json_rejected(params, "unknown field `amount_sats`");

    let mut params = fixture();
    params["outputs"][0]["label"] = json!("change");
    assert_json_rejected(params, "unknown field `label`");
}

#[test]
fn returns_json_for_unsupported_shapes() {
    let mut params = fixture();
    params["inputs"][0]
        .as_object_mut()
        .unwrap()
        .remove("prev_tx");
    assert_json_rejected(params, "missing field `prev_tx`");

    for (pointer, value, reason) in [
        (
            "/inputs",
            fixture()["inputs"][0].clone(),
            "invalid type: map, expected a sequence",
        ),
        ("/inputs/0/txid", json!("71e3be4e"), "failed to parse hex"),
        ("/inputs/0/vout", json!(-1), "integer `-1`, expected u32"),
        (
            "/inputs/0/vout",
            json!(4_294_967_296_u64),
            "integer `4294967296`, expected u32",
        ),
        (
            "/inputs/0/prev_tx",
            json!(null),
            "invalid type: null, expected a string",
        ),
        (
            "/network",
            json!({"testnet": null}),
            "invalid type: map, expected a string",
        ),
        (
            "/outputs/0/amount_sats",
            json!("95000"),
            "invalid type: string \"95000\", expected u64",
        ),
        (
            "/outputs/0/amount_sats",
            json!(95_000.0),
            "floating point `95000.0`, expected u64",
        ),
        (
            "/outputs/0/amount_sats",
            json!(-95_000),
            "integer `-95000`, expected u64",
        ),
    ] {
        assert_json_rejected(fixture_with(pointer, value), reason);
    }
}

#[test]
fn returns_json_for_arrays_in_place_of_objects() {
    let params = fixture();
    let input = &params["inputs"][0];
    let output = &params["outputs"][0];
    for positional in [
        json!([params["network"], params["inputs"], params["outputs"]]),
        fixture_with(
            "/inputs/0",
            json!([input["txid"], input["vout"], input["prev_tx"]]),
        ),
        fixture_with(
            "/outputs/0",
            json!([output["address"], output["amount_sats"]]),
        ),
    ] {
        assert_json_rejected(positional, "invalid type: sequence, expected a JSON object");
    }
}

#[test]
fn returns_unsupported_network_for_networks_other_than_testnet() {
    for network in [
        "bitcoin", "mainnet", "Testnet", "testnet4", "signet", "regtest",
    ] {
        assert!(matches!(
            parse_error(fixture_with("/network", json!(network))),
            Error::Constraints(ConstraintError::UnsupportedNetwork(actual)) if actual == network
        ));
    }
}

#[test]
fn returns_input_count_for_wrong_number_of_inputs() {
    let input = fixture()["inputs"][0].clone();

    assert!(matches!(
        parse_error(fixture_with("/inputs", json!([]))),
        Error::Constraints(ConstraintError::InputCount(0))
    ));
    assert!(matches!(
        parse_error(fixture_with("/inputs", json!([input, input]))),
        Error::Constraints(ConstraintError::InputCount(2))
    ));
}

#[test]
fn returns_output_count_for_wrong_number_of_outputs() {
    let output = fixture()["outputs"][0].clone();

    assert!(matches!(
        parse_error(fixture_with("/outputs", json!([]))),
        Error::Constraints(ConstraintError::OutputCount(0))
    ));
    assert!(matches!(
        parse_error(fixture_with("/outputs", json!([output, output]))),
        Error::Constraints(ConstraintError::OutputCount(2))
    ));
}

#[test]
fn returns_json_for_invalid_address_checksum() {
    // `bitcoin` falls back to base58 when bech32 decoding fails, so it names that error.
    assert_json_rejected(
        fixture_with(
            "/outputs/0/address",
            json!("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsy"),
        ),
        "base58 error",
    );
}
