# Testnet3 broadcast proof

The CLI produced a signed P2WPKH transaction that was broadcast using `curl` and
confirmed on Bitcoin Testnet3. The saved [signed transaction](signed.hex) matches
the transaction returned by Blockstream's API byte for byte.

**Result:** [Confirmed transaction on Blockstream](https://blockstream.info/testnet/tx/b12dcfb1aedbf839ede0469e5c065ef0460769a2474766bbb216bdea8b9291bc)

## Transaction details

| Field | Value |
|---|---|
| Network | Bitcoin Testnet3 |
| Transaction ID | `b12dcfb1aedbf839ede0469e5c065ef0460769a2474766bbb216bdea8b9291bc` |
| Funding output | Output `0` of the [faucet transaction](https://blockstream.info/testnet/tx/9003c250d26eb5699ff3618229c1f38908c6c5ee5fb18fb9a44475bb33c316fe) |
| Input amount | 192,893 sats (0.00192893 tBTC) |
| Recipient | `tb1qdm4xudmk0jpgmxhfnfhlyyrct6njvrs2ngcenc` |
| Output amount | 192,693 sats (0.00192693 tBTC) |
| Transaction fee | 200 sats (0.00000200 tBTC) |
| Confirmation block | 5,158,204 |
| Verification date | October 11, 2026 (JST) |

This is a self-transfer: the only output pays back to the input's address. There
is no separate change output. The input, recipient, output amount, and fee were
checked against the published transaction and [payload.json](payload.json).

## Build, sign, and broadcast

The commands below describe the workflow, run from the repository root. The
funding output in this record is now spent; another demonstration requires a
fresh unspent output and an updated payload.

The release build was checked on Linux x86_64 with Rust 1.95.0; `make build`
passed formatting, Clippy, and all 79 tests.

```sh
make build
```

The signing command reads the payload, shows the amount and fee, asks for `yes`,
and reads the compressed testnet WIF through a hidden terminal prompt:

```sh
./target/release/btc-offline-signer \
  --params testnet_test/payload.json \
  > testnet_test/signed.hex
```

Only use the output if the signer exits successfully. Broadcasting requires an
Internet connection and uses the signed hex without the private key:

```sh
curl --fail-with-body --silent --show-error \
  --data @testnet_test/signed.hex \
  --write-out '\n' \
  https://blockstream.info/testnet/api/tx
```

The broadcast transaction ID is recorded above. Neither the payload nor the
signed hex contains the private key.

## Offline operation

Signing is designed to work without network access, using the complete previous
transaction in the payload. Network isolation during this run was not recorded,
so this record establishes the successful broadcast and confirmation only.
To verify offline operation separately, run the signer with network access
disabled and record the isolation method; transfer the resulting hex to an
online machine for broadcasting.
