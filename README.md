# btc-offline-signer

An offline Bitcoin signer: transaction JSON + encrypted keys → operator confirmation → signed hex for online broadcast.

> **Status:** early development. The signer is not functional yet.

Only the scaffold and local checks exist; there are no tests yet. CI and the pinned static build are [BTCSIGN-001](Docs/Roadmap.md#btcsign-001-ci-pipeline-and-static-release-build). v0.1.0 will reject mainnet; use disposable test keys only (D2, D12).

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

Checks must pass before building `target/release/btc-offline-signer`.

The current build uses host system libraries. The planned musl build removes the glibc dependency (D28); verify releases on the offline machine, which needs no Rust installation.

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

Build incrementally: one input/output → change → multiple inputs → multiple keys → encrypted CLI → testnet proof. The first release verifies full previous transactions and selects keys automatically; see the [roadmap](Docs/Roadmap.md).

Follow the [completion checklist](Docs/Roadmap.md#definition-of-done-every-deliverable). Lints live in `Cargo.toml` and `clippy.toml`.

### Dependencies and `Cargo.lock`

Build, lint, and test use `--locked`. After changing dependencies, run `cargo check`, review the lockfile changes, and commit both files. This pins dependencies, not binary reproducibility (D29).

## Documentation

| Document | Owns |
|---|---|
| [Specification](Docs/Technical_Specification.md) | Required behavior; provisional choices marked |
| [Coding Style](Docs/Coding_Style.md) | Code and test conventions |
| [Roadmap](Docs/Roadmap.md) | Work, tests, status, and evidence |
| [Decisions](Docs/Decisions.md) | Choices and rationale |

This README covers current capabilities and operation.

## Planned operator runbook

Before BTCSIGN-016, replace this checklist with tested commands and rehearse using disposable keys. Keep secrets and real payment records out of evidence.

1. **Install:** compare `sha256sum` with a trusted release record before execution; verify signatures if selected under D29. Stop on mismatch.
2. **Restore:** back up the encrypted keystore and public-key inventory on separate offline media; store passphrase recovery material separately. Restore to another offline location, compare public keys, and test signing. Preserve the original; never export plaintext keys.
3. **Sign:** use an explicit keystore path. Compare every recipient, amount, and fee with independent trusted payment information before confirming. Only payloads and signed hex travel between machines; secrets stay offline.
4. **Release:** run the [release checks](Docs/Roadmap.md#btcsign-016-release-v010) against the exact binary on the target machine and record the results.
5. **Upgrade:** back up first, verify the binary and backend compatibility (spec §3.3), then rehearse signing. Never bypass verification through an older signer or altered payload.
6. **Recover:** stop on failure; check paths, key inventory, and the payment request without exposing secrets. Restore only a verified compatible version. After interruption, check broadcast status online before retrying; never broadcast partial output.
