# Development Roadmap

This is the incremental plan for building the signer described in [Technical_Specification.md](Technical_Specification.md), written the way [Coding_Style.md](Coding_Style.md) requires. Every deliverable has an ID (`BTCSIGN-XXX`), is one increment (one branch and one PR), and lists the tests that prove it works.

## 1. How to Use This Roadmap

- Follow deliverable dependencies (§4): resolve decisions → failing behavior test → implement → verify → update docs → merge → mark Done.
- Add crates when first needed, with `default-features = false`, and commit `Cargo.lock`.
- Use `Area: Summary` commit subjects and `Refs: BTCSIGN-XXX` in the body.
- At each milestone boundary, review open decisions, stale docs, and completion evidence; detail the next milestone.

### Definition of Done (every deliverable)

1. Listed behavior tests failed first and now pass; behavior/build changes pass `make`. For prose-only changes, review accuracy and links.
2. Required CI checks pass before merge.
3. Every new error variant is produced by at least one test (Coding_Style §2.2).
4. Every new security check has a `// Spec §x.y:` comment.
5. Signing stdout contains only signed hex on success and stays empty on failure; `--help` and `--version` are separate commands.
6. Update affected docs, examples, and status in the same PR, or explain why none need changes.
7. The PR links deliverable/decision IDs to named tests or manual checks, results, and limitations. Exclude secrets and secret-bearing transcripts. Link the merged PR/evidence in §4 before marking Done.

## 2. Decisions

[Decisions.md](Decisions.md) records answers and provisional assumptions referenced here as D1, D2, etc. Resolve needed choices before implementation; update affected spec, examples, and acceptance criteria when accepting or changing a decision. Accepted means approved; Done requires implementation evidence.

## 3. Milestones at a Glance

| Milestone | Deliverables | What works at the end |
|---|---|---|
| M1. Foundation | 001–002 | CI runs `make` and builds the static release binary. The binary reads `--params` (and `--keystore`, which defaults to `keys.txt`) and keeps its stdout and exit-code rules. |
| M2. Verified payload | 003–008, 017 | Previous outputs, amounts, and fees are checked before signing. |
| M3. Incremental signing | 009–010, 019–021 | One input → change → multiple inputs → multiple keys, with independent signature checks. |
| M4. Keystore & pipeline | 011–013 | End to end: `params.json` + `keys.txt` + passphrases → signed hex on stdout. |
| M5. Hardening & proof | 014–016 | Security audit done, testnet transaction confirmed, `v0.1.0` tagged. |
| M6. Future | 018 | A real keystore replaces `keys.txt`. |

## 4. Deliverable Index

| ID | Title | Depends on | Status | PR / evidence |
|---|---|---|---|---|
| BTCSIGN-001 | CI pipeline and static release build | — | Not started | — |
| BTCSIGN-002 | CLI skeleton and output rules | — | Not started | — |
| BTCSIGN-003 | Bounded params read | — | Not started | — |
| BTCSIGN-004 | Strict JSON parsing | 003 | Not started | — |
| BTCSIGN-005 | Input/output count and duplicate checks | 004 | Not started | — |
| BTCSIGN-006 | Amount and fee checks | 017 | Not started | — |
| BTCSIGN-007 | Address network check and `ValidatedParams` | 005, 006 | Not started | — |
| BTCSIGN-008 | Params connected to the CLI | 002, 007 | Not started | — |
| BTCSIGN-009 | Build the unsigned transaction | 007 | Not started | — |
| BTCSIGN-010 | Sign one input and one output | 009 | Not started | — |
| BTCSIGN-011 | Keystore file format and key lookup | — | Not started | — |
| BTCSIGN-012 | BIP38 decryption and public key check | 011 | Not started | — |
| BTCSIGN-013 | End-to-end pipeline | 008, 021, 012 | Not started | — |
| BTCSIGN-014 | Security audit | 013 | Not started | — |
| BTCSIGN-015 | Testnet broadcast proof | 013 | Not started | — |
| BTCSIGN-016 | Release v0.1.0 | 014, 015 | Not started | — |
| BTCSIGN-017 | Previous transaction verification | 005 | Not started | — |
| BTCSIGN-018 | Real keystore (future) | 016 | Not started | — |
| BTCSIGN-019 | Add change | 010 | Not started | — |
| BTCSIGN-020 | Multiple inputs, one key | 019 | Not started | — |
| BTCSIGN-021 | Multiple keys and automatic selection | 020, 011 | Not started | — |

Statuses: Not started, In progress, Done. Evidence links to the PR and verification results.

IDs remain stable; follow dependencies, not numeric order. Build 017 before 006, then 009 → 010 → 019 → 020 → 021. Keystore work (011–012) can proceed independently. Early signing uses in-memory fixtures; the first signing CLI requires all verification.

## 5. M1: Foundation

### BTCSIGN-001: CI pipeline and static release build

**Scope:**
- `rust-toolchain.toml` pins Rust 1.98.1 with `clippy`, `rustfmt`, and the musl target (D28), so rustup installs the same toolchain on every machine, CI included.
- `make build` builds the release binary for `x86_64-unknown-linux-musl`, so the binary carries every library it needs (D28).
- Resolve hosting (D30), then run `make`, `make build`, `cargo audit`, and a static-link check (`file`) on pushes and PRs. On GitHub, use `.github/workflows/ci.yml`.
- Before feature work, protect the default branch with required checks; record check names and the branch rule.
- The README's prerequisites become rustup and the `musl-tools` package, because the toolchain file covers the rest. Its build section names the new binary path.

**Manual check:**
- Push a throwaway branch with a formatting error: CI fails. Fix it: CI passes.
- A temporary failing test blocks merging; fixing it restores a passing gate. Record both results (D30).
- On the build machine, `ldd` on the built binary lists no shared libraries.
- Inside the repository, `rustc --version` prints 1.98.1.

### BTCSIGN-002: CLI skeleton and output rules

**Scope:** `main.rs` uses clap's derive API to read `--params <PATH>` (required) and `--keystore <PATH>` (optional, default `keys.txt`, D7). This deliverable also creates `tests/cli.rs`, which runs the real binary with `std::process::Command` and `env!("CARGO_BIN_EXE_btc-offline-signer")`. No test crates are needed. `--help` and `--version` are the only clap output that goes to stdout, and a signing run never uses them.

**Adds:** `clap` (derive)

**Tests:**
- `missing_params_argument_exits_nonzero_with_empty_stdout` (`tests/cli.rs`)
- `unknown_flag_exits_nonzero_with_empty_stdout` (`tests/cli.rs`)
- `keystore_defaults_to_keys_txt` (unit test in `main.rs`, using clap's `try_parse_from`)

## 6. M2: Verified Payload (spec §3.1, §5.1, §5.2)

All code for M2 lives in `src/params.rs` and `src/error.rs`. Each unit test starts from `valid_raw_params()` or the spec §3.1 JSON and changes only the field under test.

### BTCSIGN-003: Bounded params read

**Scope:** `read_bounded(impl Read) -> Result<Vec<u8>, Error>`, with `MAX_PARAMS_BYTES = 16_777_216` (D4), as shown in Coding_Style §11. An oversize file is an error. It is never silently cut short.

**Adds:** `thiserror`

**Tests:**
- `accepts_exactly_16_mib`
- `rejects_16_mib_plus_one_byte`: returns `Error::ParamsTooLarge`
- `reports_read_failure`: a reader that returns an I/O error gives `Error::Io`

### BTCSIGN-004: Strict JSON parsing

**Scope:** `parse(&[u8]) -> Result<RawParams, Error>` reads JSON into `#[serde(deny_unknown_fields)]` structs: `Txid`, `vout: u32`, whole-satoshi amounts, required `prev_tx` hex (D10, D12), `Address<NetworkUnchecked>`, and testnet-only network (D2, D12; mainnet gate in spec §1).

**Adds:** `serde` (derive), `serde_json`, `bitcoin` (serde)

**Tests:** All of these produce the same parse-error variant. Each test must also check the error message (for example `duplicate field`), so it can't pass because of a different error.
- `parses_spec_example`: the JSON from spec §3.1, checked field by field
- `rejects_unknown_field`: at the top level, inside an input, and inside an output
- `rejects_missing_field`: including an input without `prev_tx`
- `rejects_duplicate_key`: two `fee_sats` keys. This stops the online tool and the signer from reading different values.
- `rejects_trailing_data`: extra content after the JSON object
- `rejects_non_integer_amount`: `1.5`, `-1`, `"100"`, `1e5`
- `rejects_amount_above_u64_max`
- `rejects_vout_above_u32_max`
- `rejects_malformed_txid`: wrong length, non-hex characters
- `rejects_input_pubkey_field`: the earlier proposed payload shape is unsupported
- `rejects_malformed_address`: bad checksum
- `rejects_unsupported_network`: `signet`, `regtest`, `bitcoin`, `Mainnet`
- `rejects_mainnet`: otherwise valid params with `network: "mainnet"` are rejected (D12)

### BTCSIGN-005: Input/output count and duplicate checks

**Scope:** The first part of `validate(RawParams) -> Result<ValidatedParams, Error>`. It allows 1 to 1_000 inputs and 1 to 1_000 outputs (`MAX_INPUTS`, `MAX_OUTPUTS`, D6), and requires every `(txid, vout)` pair to be unique.

**Tests:**
- `accepts_1000_inputs` / `rejects_1001_inputs`
- `accepts_1000_outputs` / `rejects_1001_outputs`
- `rejects_empty_inputs` / `rejects_empty_outputs`
- `rejects_duplicate_inputs`: the error names the txid and vout
- `accepts_same_txid_with_different_vout`

The data in the `accepts_*` tests must also pass every later check. In particular, `fee_sats` must equal the inputs minus the outputs, or BTCSIGN-006 will break these tests.

### BTCSIGN-017: Previous transaction verification

**Scope:** Before fee checks, decode each required `prev_tx` and verify its txid, the existence of `vout`, equality with `amount_sats`, and a P2WPKH output script. Retain the verified amount and script in `ValidatedParams`. Key matching is BTCSIGN-021. Mainnet stays disabled.

**Tests:**
- `rejects_prev_tx_with_different_txid`
- `rejects_vout_missing_from_prev_tx`
- `rejects_understated_input_amount` / `rejects_overstated_input_amount`
- `rejects_unsupported_prev_output_script`
- `rejects_malformed_prev_tx`: non-hex, truncated, or trailing bytes
- `rejects_missing_prev_tx`
- `accepts_prev_tx_matching_input`
- `accepts_prev_tx_with_witness_data`: compare txid, not wtxid
- `validates_spec_previous_transaction`: matches the example's txid, amount, and script

The first backend fixture includes full previous transactions. Old payloads fail before keystore access or prompts (BTCSIGN-008); no weaker fallback or post-release format switch.

### BTCSIGN-006: Amount and fee checks

**Scope:** After BTCSIGN-017 verifies every previous output, check amounts and fees in `validate`. All arithmetic is checked (Coding_Style §8); totals use verified output amounts.
- Each amount and each total is at most `Amount::MAX_MONEY` (2.1 × 10^15 sats).
- The total of the outputs is at most the total of the inputs.
- `fee_sats` equals the total of the inputs minus the total of the outputs.
- The fee is at most `MAX_FEE_SATS = 1_000_000` (D5).

**Tests:**
- `rejects_input_amount_over_max_money` / `rejects_output_amount_over_max_money`
- `accepts_input_total_at_max_money` / `rejects_input_total_over_max_money`: every single amount is valid, but the total is not
- `rejects_output_total_over_max_money`
- `rejects_outputs_exceeding_inputs`
- `rejects_fee_mismatch`: a declared fee that is too high, and one that is too low
- `accepts_fee_at_max` / `rejects_fee_over_max`

### BTCSIGN-007: Address network check and `ValidatedParams`

**Scope:** Every output address is checked with `require_network` against the `network` field. `validate` returns `ValidatedParams`, the only type the transaction code accepts. Code that skips validation therefore doesn't compile.

**Tests:**
- `rejects_address_for_wrong_network`: a `bc1…` address in testnet params
- `accepts_every_standard_output_type`: P2PKH, P2SH, P2WPKH, P2WSH, and P2TR addresses on the params network
- `rejects_unknown_witness_version`: a segwit v2 to v16 address (D14)
- `validates_spec_example`: the spec JSON becomes a `ValidatedParams` with the expected totals and fee

### BTCSIGN-008: Params connected to the CLI

**Scope:** `run()` opens the `--params` file, passes it through `read_bounded`, then parses and validates it. Any failure prints one error to stderr and exits with a non-zero code. The keystore isn't used yet.

**Adds:** `anyhow` (only in `main.rs`)

**Tests** (`tests/cli.rs`, with test files in `tests/fixtures/`):
- `invalid_params_exit_nonzero_with_empty_stdout`
- `payload_without_prev_tx_fails_before_keystore_or_prompt`: old payload plus missing keystore gives a params error
- `mainnet_params_fail_before_keystore_or_prompt`: mainnet params plus missing keystore give a network error, nonzero exit, empty stdout (D12)
- `missing_params_file_exits_nonzero_with_empty_stdout`
- `oversize_params_file_exits_nonzero_with_empty_stdout`: the test creates the oversize file itself
- `valid_params_exit_zero`: temporary. BTCSIGN-013 replaces it, because from then on valid params lead to the keystore and the passphrase prompt.

## 7. M3: Incremental Signing (spec §4 steps 3, 5, 7)

All code for M3 lives in `src/tx.rs`. The tests use hardcoded testnet keys that never hold funds, and no BIP38, so they stay fast. Fixtures include matching previous transactions and verified output scripts. Increase complexity only after the preceding increment passes.

**Expected hex:** generate it once with an independent tool and paste it into the test. Use Bitcoin Core's `createrawtransaction` with `replaceable=true` and `signrawtransactionwithkey`. Given the previous transaction details, these commands need no network or UTXO set. Never paste the signer's own output into a test as the expected value.

### BTCSIGN-009: Build the unsigned transaction

**Scope:** `build_unsigned(&ValidatedParams)` sets the transaction fields from D3. It creates one `TxIn` per input (`OutPoint` from txid and vout, with an empty script and witness) and one `TxOut` per output (with the `script_pubkey` taken from the address).

**Tests:**
- `builds_one_txin_per_input` / `builds_one_txout_per_output`
- `preserves_input_and_output_order`
- `sets_version_locktime_and_sequence`
- `output_script_matches_address`
- `unsigned_spec_example_matches_reference_hex`: catches txid byte-order and encoding mistakes

### BTCSIGN-010: Sign one input and one output

**Scope:** Start with a test that signs a fixed digest, verifies it, and rejects a changed digest/key. Then sign one P2WPKH input and one output using one disposable test key and `ValidatedParams`; no JSON I/O or encryption in these tests.

Match the key's P2WPKH script to the verified previous output, compute the BIP143 sighash with `SighashCache`, sign with low-R ECDSA, and set witness `[signature, pubkey]`. `to_hex` serializes the transaction.

**Tests:**
- `signs_and_verifies_fixed_digest`
- `sighash_matches_reference_vector`
- `signs_to_expected_hex`: one input and one output; independently verify the signature too
- `rejects_input_without_its_key`: mismatching key/script
- `every_witness_is_signature_then_pubkey`
- `signing_is_deterministic`
- `hex_round_trips`

### BTCSIGN-019: Add change

**Scope:** One input, recipient output, and explicit change output. The backend supplies both destinations; the signer does not invent change. Reuse amount and fee checks (006).

**Tests:**
- `signs_recipient_and_change_to_expected_hex`
- `changing_either_output_invalidates_signature`
- Retain overspending, fee mismatch, and fee-cap rejection tests.

### BTCSIGN-020: Multiple inputs, one key

**Scope:** Sign each input with the same key and its own verified amount/script. Share one `SighashCache` across the transaction.

**Tests:**
- `signs_multiple_inputs_with_one_key`: different amounts; independently verify every signature
- `signs_1000_inputs`: the spec limit finishes in seconds
- Retain duplicate-input rejection and verified-total fee checks.

### BTCSIGN-021: Multiple keys and automatic selection

**Scope:** Match verified input scripts against P2WPKH scripts derived from the keystore's public keys. Resolve every input before prompts; report an unmatched input by txid/vout. Return unique used keys in first-use order; BTCSIGN-013 decrypts each once. `sign(unsigned_tx, &ValidatedParams, &keys)` must also reject missing/mismatched keys on its own.

**Tests:**
- `selects_keys_by_verified_script`: no input `pubkey`; no decryption needed for lookup
- `signs_inputs_from_two_keys_to_expected_hex`: three inputs, two keys, different amounts; verify every signature
- `rejects_input_without_its_key`
- `used_keys_are_unique_in_first_input_order`

## 8. M4: Keystore & Pipeline (spec §3.2, §4 steps 4, 6, 7, §5.3)

All keystore code lives in `src/keystore.rs`. It is also the only code BTCSIGN-018 will change, so it is written without a trait or plugin layer.

### BTCSIGN-011: Keystore file format and key lookup

**Scope:** `keystore::parse(&str)` reads the D7 format into BIP38 strings indexed by public key. Looking up a public key that isn't there gives `Error::KeyNotFound`. Nothing is decrypted, so these tests run instantly.

**Tests:**
- `parses_one_entry_per_line`
- `ignores_blank_lines_and_surrounding_whitespace`
- `rejects_malformed_line`: one field, three fields, a bad or uncompressed public key, a value that doesn't start with `6P`
- `rejects_duplicate_pubkey`
- `rejects_empty_keystore`
- `finds_key_by_pubkey`
- `reports_missing_pubkey`: `Error::KeyNotFound` names the public key

### BTCSIGN-012: BIP38 decryption and public key check

**Scope:** `keystore::decrypt(bip38, &SecretString, &expected_pubkey)`. The passphrase is passed in as an argument (Coding_Style §2.3). The decrypted bytes go straight into `Zeroizing`, and the key type has no `Debug`, `Display`, or `Clone`. After decryption, the key's compressed public key must equal `expected_pubkey` (D9). Spec §2 names the `bip38` crate. Before adopting it, check that it is maintained and that `cargo audit` reports no issues (D22).

**Adds:** `bip38`, `secrecy`, `zeroize`

**Tests:**
- `decrypts_bip38_test_vector`: the published BIP38 test vector (non-EC-multiply, compressed) decrypts to the expected key
- `decrypts_unicode_nfc_bip38_test_vector`: verify the published Unicode vector internally (D20); an uncompressed vector proves normalization but remains rejected by the production API (D1)
- `canonically_equivalent_passphrases_decrypt_identically`: precomposed and decomposed accented forms normalize to NFC and decrypt the same compressed test key (D20)
- `rejects_wrong_passphrase`: returns `Error::WrongPassphrase`
- `rejects_malformed_bip38`: a bad Base58Check checksum, a wrong length
- `rejects_uncompressed_key`: the BIP38 uncompressed test vector (D1)
- `rejects_pubkey_mismatch`: the passphrase is right, but `expected_pubkey` belongs to another key. Returns `Error::PubkeyMismatch`.
- `wrong_passphrase_error_does_not_contain_passphrase`: checks both `Display` and `Debug` of the error
- `rejects_ec_multiply_key`: EC-multiply keys (`6Pf…`/`6Pn…`) get their own error (D21)

BIP38 decryption runs scrypt, which is slow in debug builds. If the test suite slows down, compile scrypt with optimizations in the dev profile (`[profile.dev.package.scrypt] opt-level = 3`).

### BTCSIGN-013: End-to-end pipeline

**Scope:** `run()` follows the spec §4 order, one line per step (Coding_Style §3):
1. Load and validate the params.
2. Build the unsigned transaction.
3. Read the keystore file (`keys.txt` unless `--keystore` is given) through `read_bounded`, which now takes a size limit (64 KiB here, D7). Parse it and resolve every verified input script against its public-key inventory (D8, D10, BTCSIGN-021).
4. Show the summary on stderr and require terminal confirmation per spec §4.4 (D17), before secret prompts. Then, in first-input key order, use `rpassword` to read each passphrase into `SecretString` without echo, display its public key, decrypt, and verify the public key (D9).
5. Sign.
6. Drop the keys.
7. Write the hex and a newline to stdout, once.

**Adds:** `rpassword`

**Tests** (`tests/cli.rs`): fail **before** terminal prompts to avoid hanging.
- `valid_params_without_keys_txt_fail_naming_keys_txt`: runs in an empty temporary directory without `--keystore`. Replaces `valid_params_exit_zero` from BTCSIGN-008.
- `keystore_flag_overrides_default`: `--keystore other.txt` with no such file gives an error naming `other.txt`
- `malformed_keystore_exits_nonzero_before_prompt`
- `oversize_keystore_exits_nonzero_with_empty_stdout`
- `input_key_missing_from_keystore_fails_before_prompt`: the error names the unmatched input
- `invalid_params_fail_before_keystore_is_read`: invalid params plus a missing keystore give a params error (D8)
- Retain `mainnet_params_fail_before_keystore_or_prompt` from BTCSIGN-008.

**Unit tests** (D17; ordinary values/readers, no terminal or PTY):
- `summary_includes_network_all_outputs_totals_fee_and_used_pubkeys`: include multiple outputs and keys, check amounts and input-order key display, and never truncate outputs
- `summary_uses_verified_input_totals_and_fee`
- `confirmation_accepts_only_exact_yes`: allow the line ending, reject other text including surrounding spaces and case changes
- `confirmation_rejects_eof_or_read_error`: fail without proceeding to secret handling

**Manual check** (real terminal, Coding_Style §9; BIP38-encrypted disposable BTCSIGN-010 keys, never production keys):
- Run `btc-offline-signer --params params.json > signed.hex`. Compare the complete summary with independent trusted payment information before typing `yes` (D17).
- One key: confirmation precedes one passphrase prompt; exit 0; exactly one output line matching `signs_to_expected_hex`.
- Two keys: prompts show public keys in input order; exit 0; output matches `signs_inputs_from_two_keys_to_expected_hex`.
- Passphrases aren't echoed; summary and prompts never enter `signed.hex`.
- Rejected confirmation, EOF, cancellation, and missing controlling terminal each give nonzero exit, no passphrase prompt, and empty stdout.
- Wrong passphrase: nonzero exit, empty stdout, no passphrase in the error.

## 9. M5: Hardening & Proof (spec §5.3, §6)

### BTCSIGN-014: Security audit

**Scope:** No new features. This deliverable fixes whatever the audit finds. Any gap found starts as a failing test (Coding_Style §2.2).

**Checks:**
- Every `Error` variant is produced by a named test (list them in the PR). Variants that nothing produces are deleted.
- No error message includes a passphrase, a key, or a struct that holds either.
- Every result of `grep -n expose_secret src/` is the line that uses up the secret.
- Each passphrase is dropped right after its key is decrypted, and the keys right after signing. `secp256k1::SecretKey` is `Copy` and isn't wiped from memory when dropped. Keep it in a single scope, call `non_secure_erase()`, and write down the remaining risk in a code comment.
- No networking code: `cargo tree -e normal` lists no HTTP, async, or socket crates (`hyper`, `reqwest`, `ureq`, `tokio`, `mio`, `socket2`). Add this check to CI.
- `cargo audit` reports no issues, and every dependency uses `default-features = false`.

### BTCSIGN-015: Testnet broadcast proof

**Steps:**
1. With the offline tool chosen in D23, generate a new disposable testnet key, never shared with mainnet or production, and get its compressed public key, its BIP38 string, and its P2WPKH address (`tb1q…`). Write the public key and the BIP38 string as one line in `keys.txt`. BIP38 includes a checksum based on an address. If the tool and the `bip38` crate use different networks for that address (mainnet or testnet), decryption reports a wrong passphrase even when it's correct.
2. Send coins to that address from a testnet faucet.
3. Write `params.json` with the full funding transaction in `prev_tx`, its txid/vout and amount, one output to another address you control, and a fee within D5. Omit input `pubkey`.
4. Sign on the offline machine, or under `unshare -rn` so the process has no network access. Bring the hex back.
5. Broadcast the hex on mempool.space (for testnet4, `mempool.space/testnet4/tx/push`) and wait for one confirmation.

**Checks:**
- The transaction confirms, and its URL (`https://mempool.space/testnet4/tx/<txid>`) is added to the README.
- Signing the same params twice gives the same hex.
- The fee shown on mempool.space equals `fee_sats`.
- Optional: a second transaction spending UTXOs of two different keys confirms, which proves D9 on the live network.

### BTCSIGN-016: Release v0.1.0

**Scope:**
- Update the README with usage (`--params`, `--keystore`), the params format (link to spec §3.1), the `keys.txt` format, the offline workflow, and the testnet proof link. Remove "The signer is not functional yet."
- On a fresh clone, run `make`, `make build`, and `cargo audit` (D28); tag only after all checks pass for the release binary.
- Publish its SHA-256 through D29's trusted release-record or signature procedure.
- Document testnet-only restrictions (D12), compatibility/fixtures/migration (spec §3.3), and working [operator runbook](../README.md#planned-operator-runbook) commands (D34).

**Checks:**
- On a fresh clone, `make build` succeeds and `Cargo.lock` stays unchanged.
- On the offline machine, verify the copied binary against D29's trusted hash/signature before running `--help`; independently verify any signing key.
- Repeat the complete BTCSIGN-013 terminal checklist on **each exact release binary** (D32). Record commit, version, SHA-256, target, OS/CPU, tester, date, fixtures, commands, and results; repeat if the binary changes.
- Supported backend fixtures produce expected hex; unsupported payload shapes fail before keystore access or prompts, without fallback (D34).
- Rehearse backup restoration and failed-install recovery with disposable keys; record compatible versions and results. Test rollback when a verified older release preserves required security checks and the payload contract (D34).

## 10. M6: Future

### BTCSIGN-018: Real keystore

**Scope:** Replace `keys.txt` with a dedicated keystore. Its kind is chosen in D11: for example an encrypted keystore file, the OS keyring, a hardware device, or an HD wallet seed. `params.json` doesn't change; selection still matches verified scripts to the local public-key inventory. The change stays inside `src/keystore.rs` and the keystore step of `run()`. Also decide whether `keys.txt` stays supported.

**Tests:**
- The BTCSIGN-011 to 013 tests that aren't about the `keys.txt` format pass against the new keystore: missing key, wrong passphrase, public key mismatch, and failing before the prompt.
- The new format gets its own tests for malformed, oversize, and empty input.
- The decision on `keys.txt` (still supported, or rejected with a clear error) has a test.

## 11. Spec Traceability

| Spec | Requirement | Deliverables |
|---|---|---|
| §1 | No network access | 014 (dependency check), 015 (offline run) |
| §1 | Testnet-only release; mainnet rejected (D12) | 004, 007, 008, 013, 017 |
| §3.1 | `params.json` format | 004 |
| §3.1 | Required previous transactions and automatic script-based key selection | 004, 017, 021, 013 |
| §3.2 | BIP38 keystore, `keys.txt` by default, never a plaintext WIF key on disk | 002, 011, 012, 013, 018 |
| §3.3 | Payload compatibility, backend fixture, migration (D34) | 016, 017 |
| §4.1 | Argument parsing | 002 |
| §4.2 | Strict deserialization | 003–008 |
| §4.3 | Unsigned transaction | 009 |
| §4.4 | Confirmation before decryption (D17); NFC passphrases (D20) | 012, 013 |
| §4.5 | Incremental signing | 010, 019–021 |
| §4.6 | Clear owned secret buffers and document residual copies | 012, 013, 014 |
| §4.7 | Hex on stdout | 010, 013 |
| §5.1 | Size limit, item limits, unknown fields | 003, 004, 005, 017 |
| §5.2 | MAX_MONEY, fee cap, fee consistency, duplicate inputs | 005, 006 |
| §5.2 | Input amounts verified against the previous transaction (D12) | 017 |
| §5.3 | Secret types, error output audit | 012, 014 |
| §6 | Toolchain/CI (D28, D30), verified release (D29, D32), recovery (D34) | 001, 013, 015, 016 |
