# btc-offline-signer

A Rust CLI for creating a signed Bitcoin transaction without internet access. Built for the Coincheck interview assignment.

**Status:** payload validation ([src/payload.rs](src/payload.rs)), signing with optional change ([src/signing.rs](src/signing.rs)), and the terminal flow ([src/cli.rs](src/cli.rs)) are implemented and tested ([tests/](tests/)). The broadcast proof is pending.

## Assignment

- Signing must work offline.
- Submit a GitHub/GitLab repository and a URL showing a successful testnet or mainnet broadcast.
- Priorities: readability, maintainability, and understanding of the language/libraries. Tests and Bitcoin knowledge are valuable; feature count is not a priority.

## Build and checks

Requires a Unix-like OS (tested on Linux) and stable Rust 1.85+ with `rustfmt` and `clippy`, plus GNU make.

```sh
make          # format check, clippy, tests
make build    # checks, then release build
```

The binary is `target/release/btc-offline-signer`. `make fmt` formats code; `make audit` requires cargo-audit. Commit `Cargo.lock`; builds use `--locked`.

## Usage

```sh
./target/release/btc-offline-signer --params params.json > signed.hex
```

The signer validates the payload, shows the destination(s), amounts, and fee on stderr, and asks you to type `yes` on the terminal. It then reads one compressed testnet private key in WIF format through a hidden terminal prompt and writes only the signed transaction hex to stdout. Ctrl-C or Ctrl-D at the key prompt cancels and restores the terminal. Never supply the key in arguments or commit a funded key.

Failures exit nonzero with a message on stderr. Stdout stays empty unless writing the signed hex itself fails, which can leave part of it behind, so discard stdout after any nonzero exit.

The key is read unbuffered into a fixed-size buffer; it and the reply are zeroized after use. Copies inside `bitcoin` are not reliably cleared: base58 decoding and re-encoding, and moves of the `Copy` private key type (its last copy is overwritten on a best-effort basis).

Scope: one P2WPKH input, one recipient, optional explicit change, testnet only. Full previous-transaction data enables offline verification. See the [input format](Docs/Technical_Specification.md#input).

## Demonstration

Use a fresh disposable testnet key. Prepare the payload online, sign on a disconnected machine, then transfer only the signed hex back online to broadcast. Verify confirmation, destinations, amounts, and fee.

**Broadcast proof:** pending. Before submission, replace this with the confirmed transaction's explorer URL, the tested network, build/run commands, and a short note on how signing was isolated from the network. Do not publish the private key or terminal input.

## Documentation

- [Specification](Docs/Technical_Specification.md): behavior and input format.
- [Checklist](Docs/Roadmap.md): remaining implementation and submission work.
- [Decisions](Docs/Decisions.md): scope and tradeoffs.
- [Coding conventions](Docs/Coding_Style.md): a few rules for readable, safe code.

This is an interview prototype, not a production wallet. It does not establish whether an input is confirmed or still unspent; the broadcasting node checks chain state. Never use mainnet keys or funds.
