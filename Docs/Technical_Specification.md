# Technical Specification

Target behavior for the interview submission; current implementation status is in the [README](../README.md).

## Scope

- Offline Rust CLI; no network access during signing.
- Testnet only: reject other network values and incompatible addresses/keys.
- Exactly one P2WPKH input and one or two P2WPKH outputs: recipient and optional change.
- One compressed testnet WIF private key, read through a hidden terminal prompt.
- Version 2, locktime 0, sequence `0xFFFFFFFD` (RBF), `SIGHASH_ALL`. Use the Bitcoin library for hashing, ECDSA signing, and serialization.

## Input

`--params <PATH>` reads JSON. This synthetic fixture is not broadcastable; its previous output belongs to public test key 1, which must never be funded.

```json
{
  "network": "testnet",
  "inputs": [
    {
      "txid": "71e3be4e5b89d4263d5902a50180ea5b660a3af3ee8894aaa024ec0f2391a0f6",
      "vout": 0,
      "prev_tx": "020000000111111111111111111111111111111111111111111111111111111111111111110000000000ffffffff01a086010000000000160014751e76e8199196d454941c45d1b3a323f1433bd600000000"
    }
  ],
  "outputs": [
    {
      "address": "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
      "amount_sats": 95000
    }
  ]
}
```

Verify `prev_tx` decodes completely, its txid matches `txid`, and output `vout` exists with a P2WPKH script. Read its amount from that output; calculate the fee as input amount minus output totals. Input amount, fee, and public key are not separate payload fields.

Both output addresses are supplied explicitly. The signer never invents change; if two outputs are present, require the second output's script to match the spent output's script. The key must control that script, so change can only return to the signing key, and foreign change is rejected before the key is requested.

## Validation and signing

1. Reject files over 16 MiB, malformed/duplicate/unknown JSON fields, unsupported networks, and invalid input/output counts.
2. Verify the previous output; check destination checksums, network, and P2WPKH type, and that any change returns to the spent script.
3. Use integer satoshis and checked arithmetic. Reject zero outputs, amounts/totals above Bitcoin's maximum supply, overspending, and fees above 1,000,000 sats (the exercise's fixed fee cap).
4. Show every destination and amount, input total, and fee on stderr. Require terminal input exactly `yes`; cancellation, EOF, or other input aborts.
5. Read the WIF without echo. Require a compressed testnet key matching the verified previous-output script.
6. Sign using the verified amount/script. Serialize the complete transaction and print its hex plus one newline to stdout, once.

Errors go to stderr with a nonzero exit. Stdout stays empty unless writing the signed hex fails partway, so callers discard stdout after a nonzero exit. `--help` is informational. Keep secrets out of errors and logs; zeroize owned input buffers and document any library copies that cannot be reliably cleared.

## Limits

The previous transaction proves the referenced amount/script against its txid, not confirmation or unspent status. Network labels do not isolate keys; use disposable testnet keys never used on mainnet. Fee-rate selection and dust policy belong to the online preparation step; a valid signature alone does not guarantee relay.

No BIP38 keystore, multiple keys, multisig, Taproot, or production release workflow is required. See [decisions](Decisions.md).
