//! Parsing and validation of the `--params` payload. Nothing here needs the private key.

use std::fmt;
use std::io::Read;
use std::marker::PhantomData;

use bitcoin::address::NetworkUnchecked;
use bitcoin::amount::CheckedSum;
use bitcoin::consensus::encode::deserialize_hex;
use bitcoin::{Address, AddressType, Amount, Network, OutPoint, Transaction, TxOut, Txid};
use serde::de::value::MapAccessDeserializer;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

mod error;

pub use error::{BalanceError, ConstraintError, Error, OutputError, ParseError, PrevTxError};

/// Larger payloads are rejected without being parsed.
pub const MAX_PARAMS_BYTES: u64 = 16 * 1024 * 1024;

/// The exercise's fixed fee cap.
pub const MAX_FEE: Amount = Amount::from_sat(1_000_000);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Params {
    /// A plain string: the derived `Network` impl also accepts `{"testnet": null}`.
    network: String,
    inputs: Vec<Object<Input>>,
    outputs: Vec<Object<Output>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    txid: Txid,
    vout: u32,
    prev_tx: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    address: Address<NetworkUnchecked>,
    amount_sats: u64,
}

/// Accepts `T` only as a JSON object. Derived struct impls also accept positional arrays.
struct Object<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
            type Value = Object<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                // The derived impl still rejects duplicate and unknown fields.
                T::deserialize(MapAccessDeserializer::new(map)).map(Object)
            }
        }

        deserializer.deserialize_map(ObjectVisitor(PhantomData))
    }
}

/// A payment that passed validation. Only [`parse`] creates one, and its private fields keep it
/// from changing afterwards. Each example below must not compile; if rustdoc reports that one
/// "compiled successfully", that field has become public.
///
/// ```compile_fail,E0616
/// # fn spend_another_output(mut payment: btc_offline_signer::payload::Payment) {
/// payment.outpoint.vout = 1;
/// # }
/// ```
/// ```compile_fail,E0616
/// # fn change_input_amount(mut payment: btc_offline_signer::payload::Payment) {
/// payment.prev_output.value = bitcoin::Amount::MAX_MONEY;
/// # }
/// ```
/// ```compile_fail,E0616
/// # fn inflate_fee(mut payment: btc_offline_signer::payload::Payment) {
/// payment.outputs[0].value = bitcoin::Amount::ONE_SAT;
/// # }
/// ```
/// ```compile_fail,E0616
/// # fn misreport_fee(mut payment: btc_offline_signer::payload::Payment) {
/// payment.fee = bitcoin::Amount::ZERO;
/// # }
/// ```
#[derive(Debug)]
pub struct Payment {
    outpoint: OutPoint,
    prev_output: TxOut,
    outputs: Vec<TxOut>,
    fee: Amount,
}

impl Payment {
    pub fn outpoint(&self) -> OutPoint {
        self.outpoint
    }

    /// Amount and P2WPKH script of the spent output, taken from the verified previous transaction.
    pub fn prev_output(&self) -> &TxOut {
        &self.prev_output
    }

    /// The recipient, then optional change paying back to the spent output's script.
    pub fn outputs(&self) -> &[TxOut] {
        &self.outputs
    }

    /// Input amount minus the output total.
    pub fn fee(&self) -> Amount {
        self.fee
    }
}

/// Reads at most `MAX_PARAMS_BYTES` of JSON and validates the payment it describes.
///
/// # Errors
///
/// Returns [`Error::Parse`] for read, size, or JSON failures; [`Error::Constraints`]
/// for unsupported networks or counts; [`Error::PrevTx`] for invalid previous-output
/// data; [`Error::Output`] for invalid destinations, amounts, or change; and [`Error::Balance`]
/// for overflowing totals, overspending, or excessive fees.
pub fn parse(reader: impl Read) -> Result<Payment, Error> {
    let mut json = Vec::new();
    reader
        .take(MAX_PARAMS_BYTES + 1)
        .read_to_end(&mut json)
        .map_err(ParseError::Read)?;
    if json.len() as u64 > MAX_PARAMS_BYTES {
        return Err(ParseError::TooLarge.into());
    }
    let Object(params): Object<Params> = serde_json::from_slice(&json).map_err(ParseError::Json)?;

    if params.network != "testnet" {
        return Err(ConstraintError::UnsupportedNetwork(params.network).into());
    }
    let [Object(input)] = params.inputs.as_slice() else {
        return Err(ConstraintError::InputCount(params.inputs.len()).into());
    };
    if !(1..=2).contains(&params.outputs.len()) {
        return Err(ConstraintError::OutputCount(params.outputs.len()).into());
    }

    let prev_output = verify_prev_output(input)?;
    let outputs = params
        .outputs
        .into_iter()
        .map(|Object(output)| to_txout(output))
        .collect::<Result<Vec<_>, _>>()?;
    // `sign` requires the key to own the spent script, so change there belongs to that key.
    if outputs
        .get(1)
        .is_some_and(|change| change.script_pubkey != prev_output.script_pubkey)
    {
        return Err(OutputError::ForeignChange.into());
    }
    let total = outputs
        .iter()
        .map(|output| output.value)
        .checked_sum()
        .ok_or(BalanceError::TotalOverflow)?;
    // The input is at most the maximum supply, so this also rejects totals above it.
    let fee = prev_output
        .value
        .checked_sub(total)
        .ok_or(BalanceError::Overspend {
            outputs: total,
            input: prev_output.value,
        })?;
    if fee > MAX_FEE {
        return Err(BalanceError::FeeAboveCap(fee).into());
    }

    Ok(Payment {
        outpoint: OutPoint::new(input.txid, input.vout),
        prev_output,
        outputs,
        fee,
    })
}

/// Decodes `prev_tx`, checks that it hashes to `txid`, and returns its P2WPKH output `vout`.
fn verify_prev_output(input: &Input) -> Result<TxOut, PrevTxError> {
    let prev_tx: Transaction = deserialize_hex(&input.prev_tx)?;
    let prev_txid = prev_tx.compute_txid();
    if prev_txid != input.txid {
        return Err(PrevTxError::TxidMismatch {
            actual: prev_txid,
            expected: input.txid,
        });
    }
    let prev_output = prev_tx
        .output
        .get(input.vout as usize)
        .ok_or(PrevTxError::MissingOutput(input.vout))?;
    if !prev_output.script_pubkey.is_p2wpkh() {
        return Err(PrevTxError::NotP2wpkh(input.vout));
    }
    if prev_output.value > Amount::MAX_MONEY {
        return Err(PrevTxError::AboveMaxMoney(prev_output.value));
    }
    Ok(prev_output.clone())
}

/// Accepts a testnet P2WPKH address and an amount between one satoshi and the maximum supply.
fn to_txout(output: Output) -> Result<TxOut, OutputError> {
    if !output.address.is_valid_for_network(Network::Testnet) {
        return Err(OutputError::AddressNotTestnet(output.address));
    }
    let address = output.address.assume_checked();
    if address.address_type() != Some(AddressType::P2wpkh) {
        return Err(OutputError::AddressNotP2wpkh(address));
    }
    let value = Amount::from_sat(output.amount_sats);
    if value == Amount::ZERO {
        return Err(OutputError::ZeroAmount);
    }
    if value > Amount::MAX_MONEY {
        return Err(OutputError::AmountAboveMaxMoney(value));
    }
    Ok(TxOut {
        value,
        script_pubkey: address.script_pubkey(),
    })
}
