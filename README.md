# btc-offline-signer

A Rust CLI for creating a signed Bitcoin transaction without internet access. Built for the Coincheck interview assignment.

**Status:** payload validation ([src/payload.rs](src/payload.rs)), signing with optional change ([src/signing.rs](src/signing.rs)), and the terminal flow ([src/cli.rs](src/cli.rs)) are implemented and tested ([tests/](tests/)). A Testnet3 transaction is confirmed; see the [broadcast proof](testnet_test/broadcast-proof.md).

## Build and checks

Requires a Unix-like OS (tested on Linux), Rust installed through rustup, GNU make, and Python 3.9+ for the tests. [rust-toolchain.toml](rust-toolchain.toml) selects Rust 1.98.1 with `rustfmt` and `clippy`; Cargo rejects compilers older than 1.98.1. Update the toolchain pin and `package.rust-version` together when changing the required compiler version.

Install the pinned toolchain while online before building:

```sh
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
```

```sh
make          # format check, clippy, tests
make test     # Rust and Python tests
make build    # checks, then release build
```

The Python tests run the debug binary on a pseudo-terminal: errors, cancellation, bad keys, hidden key input, and signing. Run them alone with `python3 tests/cli_test.py`.

The binary is `target/release/btc-offline-signer`. `make fmt` formats code; `make audit` requires cargo-audit. Commit `Cargo.lock`; builds use `--locked`.

## Usage

```sh
./target/release/btc-offline-signer --params params.json > signed.hex
```

The signer validates the payload, shows the destination(s), amounts, and fee on stderr, and asks you to type `yes` on the terminal. It then reads one compressed testnet private key in WIF format through a hidden terminal prompt and writes only the signed transaction hex to stdout. Ctrl-C or Ctrl-D at the key prompt cancels and restores the terminal. Never supply the key in arguments or commit a funded key.

Failures exit nonzero with a message on stderr. Stdout stays empty unless writing the signed hex itself fails, which can leave part of it behind, so discard stdout after any nonzero exit.

The key is read unbuffered into a fixed-size buffer; it and the reply are zeroized after use. Copies inside `bitcoin` are not reliably cleared: base58 decoding and re-encoding, and moves of the `Copy` private key type (its last copy is overwritten on a best-effort basis).

Scope: one P2WPKH input, one recipient, optional explicit change, testnet only. Full previous-transaction data enables offline verification. See the [input format](docs/technical-specification.md#input).

## Demonstration

Use a fresh disposable testnet key. Prepare the payload online, sign on a disconnected machine, then transfer only the signed hex back online to broadcast. Verify confirmation, destinations, amounts, and fee.

**Broadcast proof:** [Confirmed Bitcoin Testnet3 transaction](https://blockstream.info/testnet/tx/b12dcfb1aedbf839ede0469e5c065ef0460769a2474766bbb216bdea8b9291bc). It returned 192,693 sats to the sending address with a 200-sat fee. The [demonstration record](testnet_test/broadcast-proof.md) includes the funding transaction, payload, signed hex, and build/sign/broadcast commands. Network isolation during signing was not recorded for this run.

## Documentation

- [Specification](docs/technical-specification.md): behavior and input format.
- [Checklist](docs/roadmap.md): remaining implementation and submission work.
- [Decisions](docs/decisions.md): scope and tradeoffs.
- [Coding conventions](docs/coding-style.md): a few rules for readable, safe code.

This is a prototype, not a production wallet. It does not establish whether an input is confirmed or still unspent; the broadcasting node checks chain state. Never use mainnet keys or funds.
