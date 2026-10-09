# btc-offline-signer

An air-gapped command-line tool that signs Bitcoin transactions offline. It reads a transaction intent (`params.json`) and a BIP38-encrypted keystore, prompts for the passphrase, and prints the signed transaction hex for broadcast from an online machine.

> **Status:** early development. The signer is not functional yet.

## Prerequisites

- **Rust** stable 1.85 or newer (the project uses edition 2024), with `clippy` and `rustfmt`:

  ```sh
  rustup component add clippy rustfmt
  ```

- **GNU make**
- **cargo-audit** (optional, for `make audit`): `cargo install cargo-audit`

## Build

From the repository root:

```sh
make build
```

`make build` first runs the formatting check, clippy, and the test suite, and only builds if all three pass. The binary is written to `target/release/btc-offline-signer`.

**Building for the offline machine:** build on an online machine (cargo downloads dependencies from crates.io), then copy the binary to the offline signer. It must have the same OS and CPU architecture. The offline machine doesn't need Rust installed.

## Development

| Command | What it does |
|---|---|
| `make` | Runs every check: `cargo fmt --check`, clippy with warnings as errors, and the tests |
| `make build` | Runs every check, then builds the release binary |
| `make test` | Runs the tests only |
| `make lint` | Runs clippy only |
| `make fmt` | Formats the code in place |
| `make audit` | Scans dependencies for known vulnerabilities |
| `make clean` | Removes build output |

Run `make` before every commit. The lint rules live in `Cargo.toml` and `clippy.toml`; see [Docs/Coding_Style.md](Docs/Coding_Style.md) for why each one exists.

### Dependencies and `Cargo.lock`

Every `make` target runs cargo with `--locked`, so builds use exactly the dependency versions recorded in `Cargo.lock`. After adding or changing a dependency in `Cargo.toml`, run `cargo check` once to update `Cargo.lock`, then commit both files together.

## Documentation

- [Technical Specification](Docs/Technical_Specification.md): what the signer does and its security safeguards
- [Coding Style Guide](Docs/Coding_Style.md): how the code is written (test-first, readability, lint rules)
- [Roadmap](Docs/Roadmap.md): incremental milestones, deliverables (`BTCSIGN-XXX`) and the tests for each
- [Decisions](Docs/Decisions.md): open points in the spec, with the answer the roadmap assumes for each
