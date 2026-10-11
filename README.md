# btc-offline-signer

A Rust CLI for creating a signed Bitcoin transaction without internet access. Built for the Coincheck interview assignment.

**Status:** scaffold only (`main` is empty). Signing, tests, and broadcast proof are not implemented yet.

## Assignment

- Signing must work offline.
- Submit a GitHub/GitLab repository and a URL showing a successful testnet or mainnet broadcast.
- Priorities: readability, maintainability, and understanding of the language/libraries. Tests and Bitcoin knowledge are valuable; feature count is not a priority.

## Build and checks

Requires stable Rust 1.85+ with `rustfmt` and `clippy`, plus GNU make.

```sh
make          # format check, clippy, tests
make build    # checks, then release build
```

The current binary is `target/release/btc-offline-signer`. It does nothing yet. `make fmt` formats code; `make audit` requires cargo-audit. Commit `Cargo.lock`; builds use `--locked`.

## Planned usage

```sh
./target/release/btc-offline-signer --params params.json > signed.hex
```

The signer will validate the payload, show the destination(s), amounts, and fee on stderr, and require confirmation. It will then read one compressed testnet private key in WIF format through a hidden terminal prompt. Never supply the key in arguments or commit a funded key.

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
