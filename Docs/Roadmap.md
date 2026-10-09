# Development Roadmap

This is the incremental plan for building the signer described in [Technical_Specification.md](Technical_Specification.md), written the way [Coding_Style.md](Coding_Style.md) requires. Every deliverable has an ID (`BTCSIGN-XXX`), is one increment (one branch and one PR), and lists the tests that prove it works.

## 1. How to Use This Roadmap

- **Work top to bottom.** Each deliverable leaves the branch green, and when it's done the program does more than it did before. Section 4 lists what each deliverable depends on, for anyone who wants to work on two at once.
- **Test first.** The tests listed under a deliverable are its starting list, and they extend Coding_Style §2.5. Write each test, see it fail, then write the code that makes it pass. Add more tests as new cases come up.
- **Add dependencies only when needed.** A crate is added in the first deliverable that uses it (listed under **Adds**), with `default-features = false`. Commit `Cargo.lock` in the same commit.
- **Reference the ID.** Keep the usual `Area: Summary` subject line and put the ID in the commit body, for example `Refs: BTCSIGN-005`.
- **Record manual checks.** Some checks can't be automated. Write their results in the PR description.

### Definition of Done (every deliverable)

1. Every listed test exists, failed first, and now passes.
2. `make` passes locally and in CI.
3. Every new error variant is produced by at least one test (Coding_Style §2.2).
4. Every new security check has a `// Spec §x.y:` comment.
5. Nothing is ever written to stdout except the signed hex.

## 2. Decisions

The spec leaves many points open. [Decisions.md](Decisions.md) lists every one of them, with the answer this roadmap assumes, and references such as (D4) point there. Settle a decision before the deliverable that needs it starts. If the answer differs from what the roadmap assumes, update the affected deliverables here first.

## 3. Milestones at a Glance

| Milestone | Deliverables | What works at the end |
|---|---|---|
| M1. Foundation | 001–002 | CI runs `make` and builds the static release binary. The binary reads `--params` (and `--keystore`, which defaults to `keys.txt`) and keeps its stdout and exit-code rules. |
| M2. Intent payload | 003–008 | The CLI rejects every malformed or unsafe `params.json` (spec §5.1, §5.2). |
| M3. Transaction | 009–010 | Code-level tests build and sign transactions with test keys, matching a known-correct hex. |
| M4. Keystore & pipeline | 011–013 | End to end: `params.json` + `keys.txt` + passphrases → signed hex on stdout. |
| M5. Hardening & proof | 014–016 | Security audit done, testnet transaction confirmed, `v0.1.0` tagged. |
| M6. Input verification | 017 | Every input amount is proven by its previous transaction (D12). Required before any mainnet use. |
| M7. Future | 018 | A real keystore replaces `keys.txt`. |

## 4. Deliverable Index

| ID | Title | Depends on |
|---|---|---|
| BTCSIGN-001 | CI pipeline and static release build | — |
| BTCSIGN-002 | CLI skeleton and output rules | — |
| BTCSIGN-003 | Bounded params read | — |
| BTCSIGN-004 | Strict JSON parsing | 003 |
| BTCSIGN-005 | Input/output count and duplicate checks | 004 |
| BTCSIGN-006 | Amount and fee checks | 004 |
| BTCSIGN-007 | Address network check and `ValidatedParams` | 005, 006 |
| BTCSIGN-008 | Params connected to the CLI | 002, 007 |
| BTCSIGN-009 | Build the unsigned transaction | 007 |
| BTCSIGN-010 | Sign and serialize | 009 |
| BTCSIGN-011 | Keystore file format and key lookup | — |
| BTCSIGN-012 | BIP38 decryption and public key check | 011 |
| BTCSIGN-013 | End-to-end pipeline | 008, 010, 012 |
| BTCSIGN-014 | Security audit | 013 |
| BTCSIGN-015 | Testnet broadcast proof | 013 |
| BTCSIGN-016 | Release v0.1.0 | 014, 015 |
| BTCSIGN-017 | Previous transaction verification | 016 |
| BTCSIGN-018 | Real keystore (future) | 016 |

BTCSIGN-011 and 012 depend only on the crate itself, so they can be built alongside M2 or M3.

## 5. M1: Foundation

### BTCSIGN-001: CI pipeline and static release build

**Scope:**
- `rust-toolchain.toml` pins Rust 1.98.1 with `clippy`, `rustfmt`, and the musl target (D28), so rustup installs the same toolchain on every machine, CI included.
- `make build` builds the release binary for `x86_64-unknown-linux-musl`, so the binary carries every library it needs (D28).
- `.github/workflows/ci.yml` runs `make`, `make build`, and `cargo audit` for every push and pull request. CI fails if the binary isn't statically linked (checked with `file`).
- The README's prerequisites become rustup and the `musl-tools` package, because the toolchain file covers the rest. Its build section names the new binary path.

**Manual check:**
- Push a throwaway branch with a formatting error: CI fails. Fix it: CI passes.
- On the build machine, `ldd` on the built binary lists no shared libraries.
- Inside the repository, `rustc --version` prints 1.98.1.

### BTCSIGN-002: CLI skeleton and output rules

**Scope:** `main.rs` uses clap's derive API to read `--params <PATH>` (required) and `--keystore <PATH>` (optional, default `keys.txt`, D7). This deliverable also creates `tests/cli.rs`, which runs the real binary with `std::process::Command` and `env!("CARGO_BIN_EXE_btc-offline-signer")`. No test crates are needed. `--help` and `--version` are the only clap output that goes to stdout, and a signing run never uses them.

**Adds:** `clap` (derive)

**Tests:**
- `missing_params_argument_exits_nonzero_with_empty_stdout` (`tests/cli.rs`)
- `unknown_flag_exits_nonzero_with_empty_stdout` (`tests/cli.rs`)
- `keystore_defaults_to_keys_txt` (unit test in `main.rs`, using clap's `try_parse_from`)

## 6. M2: Intent Payload (spec §3.1, §5.1, §5.2)

All code for M2 lives in `src/params.rs` and `src/error.rs`. Each unit test starts from `valid_raw_params()` or the spec §3.1 JSON and changes only the field under test.

### BTCSIGN-003: Bounded params read

**Scope:** `read_bounded(impl Read) -> Result<Vec<u8>, Error>`, with `MAX_PARAMS_BYTES = 1_048_576` (D4), as shown in Coding_Style §11. An oversize file is an error. It is never silently cut short.

**Adds:** `thiserror`

**Tests:**
- `accepts_exactly_1_mib`
- `rejects_1_mib_plus_one_byte`: returns `Error::ParamsTooLarge`
- `reports_read_failure`: a reader that returns an I/O error gives `Error::Io`

### BTCSIGN-004: Strict JSON parsing

**Scope:** `parse(&[u8]) -> Result<RawParams, Error>` reads the JSON into structs marked `#[serde(deny_unknown_fields)]`. The fields use real types: `Txid`, `vout: u32`, amounts in whole satoshis, a compressed public key for each input's `pubkey` (D1, D10), `Address<NetworkUnchecked>`, and a network enum that only allows `mainnet` and `testnet` (D2).

**Adds:** `serde` (derive), `serde_json`, `bitcoin` (serde)

**Tests:** All of these produce the same parse-error variant. Each test must also check the error message (for example `duplicate field`), so it can't pass because of a different error.
- `parses_spec_example`: the JSON from spec §3.1, checked field by field
- `rejects_unknown_field`: at the top level, inside an input, and inside an output
- `rejects_missing_field`: including an input without `pubkey`
- `rejects_duplicate_key`: two `fee_sats` keys. This stops the online tool and the signer from reading different values.
- `rejects_trailing_data`: extra content after the JSON object
- `rejects_non_integer_amount`: `1.5`, `-1`, `"100"`, `1e5`
- `rejects_amount_above_u64_max`
- `rejects_vout_above_u32_max`
- `rejects_malformed_txid`: wrong length, non-hex characters
- `rejects_malformed_pubkey`: non-hex characters, wrong length, not a point on the curve
- `rejects_uncompressed_pubkey`: a 65-byte `04…` key (D1)
- `rejects_malformed_address`: bad checksum
- `rejects_unsupported_network`: `signet`, `regtest`, `bitcoin`, `Mainnet`

### BTCSIGN-005: Input/output count and duplicate checks

**Scope:** The first part of `validate(RawParams) -> Result<ValidatedParams, Error>`. It allows 1 to 1_000 inputs and 1 to 1_000 outputs (`MAX_INPUTS`, `MAX_OUTPUTS`, D6), and requires every `(txid, vout)` pair to be unique.

**Tests:**
- `accepts_1000_inputs` / `rejects_1001_inputs`
- `accepts_1000_outputs` / `rejects_1001_outputs`
- `rejects_empty_inputs` / `rejects_empty_outputs`
- `rejects_duplicate_inputs`: the error names the txid and vout
- `accepts_same_txid_with_different_vout`

The data in the `accepts_*` tests must also pass every later check. In particular, `fee_sats` must equal the inputs minus the outputs, or BTCSIGN-006 will break these tests.

### BTCSIGN-006: Amount and fee checks

**Scope:** More checks in `validate`. All arithmetic is checked (Coding_Style §8).
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
- `rejects_address_for_wrong_network`: a `bc1…` address in testnet params, and a `tb1…` address in mainnet params
- `accepts_every_standard_output_type`: P2PKH, P2SH, P2WPKH, P2WSH, and P2TR addresses on the params network
- `rejects_unknown_witness_version`: a segwit v2 to v16 address (D14)
- `validates_spec_example`: the spec JSON becomes a `ValidatedParams` with the expected totals and fee

### BTCSIGN-008: Params connected to the CLI

**Scope:** `run()` opens the `--params` file, passes it through `read_bounded`, then parses and validates it. Any failure prints one error to stderr and exits with a non-zero code. The keystore isn't used yet.

**Adds:** `anyhow` (only in `main.rs`)

**Tests** (`tests/cli.rs`, with test files in `tests/fixtures/`):
- `invalid_params_exit_nonzero_with_empty_stdout`
- `missing_params_file_exits_nonzero_with_empty_stdout`
- `oversize_params_file_exits_nonzero_with_empty_stdout`: the test creates the oversize file itself
- `valid_params_exit_zero`: temporary. BTCSIGN-013 replaces it, because from then on valid params lead to the keystore and the passphrase prompt.

## 7. M3: Transaction (spec §4 steps 3, 5, 7)

All code for M3 lives in `src/tx.rs`. The tests use hardcoded testnet keys that never hold funds, and no BIP38, so they stay fast. Each test sets every input's `pubkey` to the public key of the test key that owns it.

**Expected hex:** generate it once with an independent tool and paste it into the test. Use Bitcoin Core's `createrawtransaction` with `replaceable=true` and `signrawtransactionwithkey`. Given the previous transaction details, these commands need no network or UTXO set. Never paste the signer's own output into a test as the expected value.

### BTCSIGN-009: Build the unsigned transaction

**Scope:** `build_unsigned(&ValidatedParams)` sets the transaction fields from D3. It creates one `TxIn` per input (`OutPoint` from txid and vout, with an empty script and witness) and one `TxOut` per output (with the `script_pubkey` taken from the address).

**Tests:**
- `builds_one_txin_per_input` / `builds_one_txout_per_output`
- `preserves_input_and_output_order`
- `sets_version_locktime_and_sequence`
- `output_script_matches_address`
- `unsigned_spec_example_matches_reference_hex`: catches txid byte-order and encoding mistakes

### BTCSIGN-010: Sign and serialize

**Scope:** `sign(unsigned_tx, &ValidatedParams, &keys)`, where `keys` holds the decrypted private keys indexed by public key. For each input, it:
1. Finds the private key for the input's `pubkey`.
2. Builds the P2WPKH script from that `pubkey`.
3. Computes the BIP143 sighash with one shared `SighashCache`, using that input's amount from the params and `SIGHASH_ALL`.
4. Signs with low-R ECDSA.
5. Sets the witness to `[signature, pubkey]`.

`to_hex` serializes the signed transaction in Bitcoin's wire format.

**Tests:**
- `signs_to_expected_hex`: the spec example signed with one test key gives exactly the hex Bitcoin Core produces
- `signs_inputs_from_two_keys_to_expected_hex`: three inputs owned by two keys, with different amounts. This catches inputs, keys, and amounts getting mixed up.
- `rejects_input_without_its_key`: returns `Error::KeyNotFound`. The pipeline already checks this before the prompt (D8), but `sign` must still refuse on its own.
- `every_witness_is_signature_then_pubkey`
- `signing_is_deterministic`: signing twice gives the same bytes
- `signs_1000_inputs`: the spec §5.1 limit finishes in seconds, because one shared `SighashCache` keeps the work linear
- `hex_round_trips`: decoding the output of `to_hex` gives back the same transaction

## 8. M4: Keystore & Pipeline (spec §3.2, §4 steps 4, 6, 7, §5.3)

All keystore code lives in `src/keystore.rs`. It is also the only code BTCSIGN-018 will change, so it is written without a trait or plugin layer.

### BTCSIGN-011: Keystore file format and key lookup

**Scope:** `keystore::parse(&str)` reads the D7 format into BIP38 strings indexed by public key. Looking up a public key that isn't there gives `Error::KeyNotFound`. Nothing is decrypted, so these tests run instantly.

**Tests:**
- `parses_one_entry_per_line`
- `ignores_blank_lines_and_surrounding_whitespace`
- `rejects_malformed_line`: one field, three fields, a bad public key, a value that doesn't start with `6P`
- `rejects_duplicate_pubkey`
- `rejects_empty_keystore`
- `finds_key_by_pubkey`
- `reports_missing_pubkey`: `Error::KeyNotFound` names the public key

### BTCSIGN-012: BIP38 decryption and public key check

**Scope:** `keystore::decrypt(bip38, &SecretString, &expected_pubkey)`. The passphrase is passed in as an argument (Coding_Style §2.3). The decrypted bytes go straight into `Zeroizing`, and the key type has no `Debug`, `Display`, or `Clone`. After decryption, the key's compressed public key must equal `expected_pubkey` (D9). Spec §2 names the `bip38` crate. Before adopting it, check that it is maintained and that `cargo audit` reports no issues (D22).

**Adds:** `bip38`, `secrecy`, `zeroize`

**Tests:**
- `decrypts_bip38_test_vector`: the published BIP38 test vector (non-EC-multiply, compressed) decrypts to the expected key
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
3. Read the keystore file (`keys.txt` unless `--keystore` is given) through `read_bounded`, which now takes a size limit (64 KiB here, D7). Parse it, and check that every input's public key is in it (D8).
4. For each key used, in the order the keys first appear in the inputs: prompt for its passphrase with `rpassword` (no echo, prompt on the terminal, showing the public key) into a `SecretString`, decrypt the key, and check its public key (D9).
5. Sign.
6. Drop the keys.
7. Write the hex and a newline to stdout, once.

**Adds:** `rpassword`

**Tests** (`tests/cli.rs`): every test here must fail **before** the passphrase prompt. `rpassword` reads from `/dev/tty`, so a test that reaches the prompt hangs when run from a developer's terminal.
- `valid_params_without_keys_txt_fail_naming_keys_txt`: runs in an empty temporary directory without `--keystore`. Replaces `valid_params_exit_zero` from BTCSIGN-008.
- `keystore_flag_overrides_default`: `--keystore other.txt` with no such file gives an error naming `other.txt`
- `malformed_keystore_exits_nonzero_before_prompt`
- `oversize_keystore_exits_nonzero_with_empty_stdout`
- `input_key_missing_from_keystore_fails_before_prompt`: the error names the missing public key
- `invalid_params_fail_before_keystore_is_read`: invalid params plus a missing keystore give a params error (D8)

**Manual check** (in a real terminal, because a passphrase can only come from the terminal, Coding_Style §9). Use a `keys.txt` holding the BTCSIGN-010 test keys, encrypted with BIP38:
- One key: `btc-offline-signer --params params.json > signed.hex` asks for one passphrase and exits with 0. `signed.hex` holds exactly one line, and it equals the expected hex of `signs_to_expected_hex`.
- Two keys: there are two prompts, in input order, each showing its public key. The output equals the expected hex of `signs_inputs_from_two_keys_to_expected_hex`.
- The passphrase isn't shown while typing, and the prompt doesn't end up in `signed.hex`.
- Wrong passphrase: non-zero exit, `signed.hex` is empty, and the error doesn't contain the passphrase.

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
1. With the offline tool chosen in D23, generate a new testnet key and get its compressed public key, its BIP38 string, and its P2WPKH address (`tb1q…`). Write the public key and the BIP38 string as one line in `keys.txt`. BIP38 includes a checksum based on an address. If the tool and the `bip38` crate use different networks for that address (mainnet or testnet), decryption reports a wrong passphrase even when it's correct.
2. Send coins to that address from a testnet faucet.
3. Write `params.json` for that UTXO, with the key's public key as the input's `pubkey`, one output to another address you control, and a fee within D5.
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
- Run `make build` and `cargo audit` on a fresh clone, then tag `v0.1.0`. The release binary is the static musl build from `make build` (D28).
- Publish the SHA-256 hash of the release binary.
- State in the README that v0.1.0 is for testnet only, until BTCSIGN-017 verifies input amounts (D12).

**Checks:**
- On a fresh clone, `make build` succeeds and `Cargo.lock` stays unchanged.
- On the offline machine, `sha256sum` of the copied binary matches the published hash, and `--help` runs.

## 10. M6: Input Verification (D12)

This milestone closes the fee bypass attack described in D12. Until it ships, the signer trusts the input amounts in `params.json`, so it must only be used on testnet.

### BTCSIGN-017: Previous transaction verification

**Scope:** Each input gets a required `prev_tx` field that holds the full previous transaction, hex-encoded. `validate` decodes it and checks four things:
1. Its txid equals the input's `txid`. A forged transaction can't pass this check.
2. It has an output at the input's `vout`.
3. That output's amount equals the input's `amount_sats`. From then on, the BTCSIGN-006 fee checks run on true amounts.
4. That output's script is the P2WPKH script of the input's `pubkey`, which proves the input belongs to that key.

Also in this deliverable:
- Raise the `params.json` size cap so 1,000 previous transactions fit (D4).
- Update spec §3.1 (add `prev_tx` to the example) and spec §5.2 (the new checks).
- The online backend now has to supply each previous transaction, for example with Bitcoin Core's `getrawtransaction`.

Signing doesn't change, so the BTCSIGN-010 expected-hex tests pass without changes.

**Tests:**
- `rejects_understated_input_amount`: the D12 attack, where `amount_sats` is lower than the amount in `prev_tx`
- `rejects_overstated_input_amount`
- `rejects_prev_tx_with_different_txid`: a forged previous transaction
- `rejects_vout_missing_from_prev_tx`
- `rejects_prev_output_not_owned_by_pubkey`
- `rejects_malformed_prev_tx`: not hex, cut short, extra bytes at the end
- `rejects_missing_prev_tx`
- `accepts_prev_tx_matching_input`
- `accepts_prev_tx_with_witness_data`: the txid is computed without the witness data, so a segwit funding transaction still matches
- The BTCSIGN-003 size cap tests move to the new limit.

**Manual check:**
- Repeat the BTCSIGN-015 testnet run, filling `prev_tx` with the funding transaction's raw hex (mempool.space serves it at `/testnet4/api/tx/<txid>/hex`). The signed transaction broadcasts and confirms.

## 11. M7: Future

### BTCSIGN-018: Real keystore

**Scope:** Replace `keys.txt` with a dedicated keystore. Its kind is chosen in D11: for example an encrypted keystore file, the OS keyring, a hardware device, or an HD wallet seed. `params.json` doesn't change, so inputs still select keys by public key. The change stays inside `src/keystore.rs` and the keystore step of `run()`. Also decide whether `keys.txt` stays supported.

**Tests:**
- The BTCSIGN-011 to 013 tests that aren't about the `keys.txt` format pass against the new keystore: missing key, wrong passphrase, public key mismatch, and failing before the prompt.
- The new format gets its own tests for malformed, oversize, and empty input.
- The decision on `keys.txt` (still supported, or rejected with a clear error) has a test.

## 12. Spec Traceability

| Spec | Requirement | Deliverables |
|---|---|---|
| §1 | No network access | 014 (dependency check), 015 (offline run) |
| §1 | testnet / mainnet | 004, 007 |
| §3.1 | `params.json` format | 004 |
| §3.1 | Each input's `pubkey` selects its signing key | 004, 010, 011, 013 |
| §3.2 | BIP38 keystore, `keys.txt` by default, never a plaintext WIF key on disk | 002, 011, 012, 013, 018 |
| §4.1 | Argument parsing | 002 |
| §4.2 | Strict deserialization | 003–008 |
| §4.3 | Unsigned transaction | 009 |
| §4.4 | Passphrase prompt and decryption | 012, 013 |
| §4.5 | Signing | 010 |
| §4.6 | Wiping secrets from memory | 012, 013, 014 |
| §4.7 | Hex on stdout | 010, 013 |
| §5.1 | Size limit, item limits, unknown fields | 003, 004, 005, 017 |
| §5.2 | MAX_MONEY, fee cap, fee consistency, duplicate inputs | 005, 006 |
| §5.2 | Input amounts proven by the previous transaction (added by D12) | 017 |
| §5.3 | Secret types, error output audit | 012, 014 |
| §6 | `Cargo.lock`, unit tests, testnet proof | every deliverable, 015, 016 |
