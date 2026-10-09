# Coding Style Guide

This project signs Bitcoin transactions with real private keys. Two things matter above everything else:

1. **The code is built test-first.** Every rule in the spec is proven by a test before the code that enforces it exists.
2. **The code is easy to read.** Every read is a security review: a reviewer must be able to convince themselves the code cannot leak a key, lose funds, or sign the wrong thing.

See [Technical_Specification.md](Technical_Specification.md) for *what* we build; this document covers *how* we write it.

## 1. Guiding Principles

- **Test first.** No production code is written without a failing test that demands it.
- **Readable first.** If a reviewer needs to run the code in their head twice, rewrite it.
- **Fail closed.** Any doubt (bad input, overflow, mismatch) aborts with an error. We never "best-effort" sign.
- **Small surface.** Fewest dependencies, fewest files, no abstraction without a second caller. Code that doesn't exist can't leak a key.
- **Parse once, trust after.** Untrusted input is validated in one place; everything downstream works on validated types only.

## 2. Test-Driven Development

### 2.1. The Cycle

Every change follows red → green → refactor:

1. **Red:** write one small test for the next behavior. Run it and watch it fail *for the reason you expect*. A compile error for a not-yet-written function counts.
2. **Green:** write the simplest code that makes it pass. Nothing the test doesn't require.
3. **Refactor:** with all tests green, clean up names, duplication, and structure. Then pick the next test.

### 2.2. Rules

- **No test, no code.** Every function, branch, and error variant exists because a test needed it.
- **Bugs start as tests.** A bug fix begins with a test that reproduces the bug and fails.
- **Safeguards are proven by rejection.** Each rule in spec §5 has a test that feeds bad input and asserts the specific error. Limits are tested on both sides: at the limit passes, one past it fails.
- **Every error variant is produced by at least one test.** An untested variant is either dead code or an unproven path.
- **Commit on green.** A commit holds a test and the code that makes it pass. Never commit a failing suite.
- **Keep the suite fast.** `cargo test` finishes in seconds, so it runs after every change. No network, no real keystore, no sleeps.

### 2.3. Design for Testability

Code that is easy to call from a test is also easy to read:

- **I/O stays at the edges.** Only `main.rs` opens files, prompts for the passphrase, and prints. Every other module takes data (bytes, strings, structs, any `impl Read`) and returns a `Result`.
- **The passphrase is a parameter.** `keystore::decrypt` takes the encrypted string and the passphrase as arguments; `main.rs` does the prompting. Tests pass a known passphrase directly.
- **Signing is deterministic.** ECDSA signatures use RFC 6979 nonces, so the same key and params always produce the same bytes. Signing is tested against an exact expected hex string, not just "it didn't error".

### 2.4. Test Style

Tests are the first documentation a reader opens, so they all follow one shape:

- **One behavior per test**, named as a sentence: `rejects_duplicate_inputs`, `accepts_fee_at_max`.
- **Arrange / act / assert**, separated by blank lines.
- **Start from a known-good fixture and change only the field under test**, so the test shows exactly what matters:

```rust
#[test]
fn rejects_duplicate_inputs() {
    let mut raw = valid_raw_params();
    raw.inputs.push(raw.inputs[0].clone());

    let result = validate(raw);

    assert!(matches!(result, Err(Error::DuplicateInput { .. })));
}
```

- **Assert the specific error variant**, never just `is_err()`. A test that passes for the wrong error proves nothing.
- **Unit tests live next to the code** in a `#[cfg(test)] mod tests` at the bottom of the file, along with that module's fixtures (`valid_raw_params()`).
- **Hardcoded data only**: the spec §3.1 example, published BIP38 test vectors, freshly generated testnet keys. No real keys in the repo, ever.

### 2.5. Starting Test List

Derived from the spec before any code is written. Work through it one test at a time, and add to it whenever a new case comes up.

**`params`** (spec §3.1, §5.1, §5.2)

- `parses_spec_example`: the JSON from spec §3.1
- `rejects_unknown_field`
- `accepts_exactly_1_mib` / `rejects_1_mib_plus_one_byte`
- `accepts_1000_inputs` / `rejects_1001_inputs`, and the same pair for outputs
- `rejects_duplicate_inputs`
- `rejects_total_over_max_money`
- `rejects_outputs_exceeding_inputs`
- `rejects_fee_mismatch`
- `accepts_fee_at_max` / `rejects_fee_over_max`
- `rejects_address_for_wrong_network`

**`keystore`** (spec §3.2)

- `decrypts_bip38_test_vector`
- `rejects_wrong_passphrase`
- `rejects_malformed_keystore`

**`tx`** (spec §4)

- `builds_one_txin_per_input` / `builds_one_txout_per_output`
- `signs_to_expected_hex`: fixed testnet key + spec example → exact hex

**`tests/cli.rs`** (end-to-end, runs the built binary)

- `invalid_params_exit_nonzero_with_empty_stdout`

## 3. Readability

- **`main.rs` reads like the spec.** The top-level flow is one line per step of spec §4, so a reviewer can hold the code and spec side by side (illustrative):

```rust
fn run(args: &Args) -> anyhow::Result<String> {
    let params = load_params(&args.params_path)?;
    let unsigned_tx = tx::build_unsigned(&params)?;
    let private_key = unlock_keystore(&args.keystore_path)?;
    let signed_tx = tx::sign(unsigned_tx, &params, &private_key)?;
    Ok(tx::to_hex(&signed_tx))
}
```

- **Short functions that do one thing.** If you want a comment above a block to say what it does, extract it into a function with that name instead. A function should fit on one screen.
- **Guard clauses, not nesting.** Check, return the error early, and keep the happy path unindented. More than two levels of nesting means restructure.
- **One idea per line.** Name intermediate values instead of nesting calls three deep.
- **Plain over clever.** No custom traits, generics, macros, or explicit lifetimes unless a test-driven need forces them. Iterator chains are fine up to about three steps; past that, use a `for` loop or named variables.
- **Readable without an IDE.** Code is reviewed in a browser, without type hints. Annotate a `let` when its type isn't obvious from the line itself.

## 4. Naming

- Standard Rust conventions: `snake_case` for functions/variables/modules, `UpperCamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- **Full words.** `input_index`, not `idx` or `i` (single letters only in closures one line long). The accepted abbreviations are the domain's own: `tx`, `txid`, `vout`, `utxo`, `sats`, `wif`, `bip38`.
- **Units go in the name.** `amount_sats`, `fee_sats`, `MAX_PARAMS_BYTES`. Never a bare `amount` or `size`.
- **Verbs for functions, nouns for values.** `validate`, `build_unsigned`, `sign` / `unsigned_tx`, `signed_tx`.
- Digit separators in numeric literals: `1_000_000`, not `1000000`.

## 5. Toolchain, Formatting & Lints

- Stable Rust, edition 2024.
- `rustfmt` with default settings. No `rustfmt.toml`.
- Lints are configured in `Cargo.toml` so they apply to every build, not just CI:

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
dbg_macro = "deny"
print_stdout = "deny"             # stdout is reserved for the signed hex (see §10)
float_arithmetic = "deny"         # no floats near money
arithmetic_side_effects = "deny"  # forces checked math (see §8)

[profile.release]
overflow-checks = true            # backstop in case checked math is missed
```

- `clippy.toml` relaxes only what tests need:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
```

- Silencing a lint requires `#[expect(lint, reason = "...")]` at the narrowest scope, never a crate-wide `allow`.

## 6. Project Layout

Binary crate. One module per stage of the execution flow (spec §4). Add a module only when a file outgrows what one reviewer can hold in their head.

```
src/
├── main.rs      # args, opening files, passphrase prompt, the single stdout write
├── params.rs    # bounded read, parse, validate (spec §3.1, §5.1, §5.2)
├── keystore.rs  # BIP38 decryption (spec §3.2)
├── tx.rs        # build unsigned tx, sign, serialize (spec §4)
└── error.rs     # the crate's error enum
tests/
└── cli.rs       # end-to-end: runs the binary, checks stdout and exit code
```

Limits from spec §5 live as `const`s at the top of the module that enforces them, never as inline magic numbers.

## 7. Types & Input Validation

- Every struct deserialized from untrusted input carries `#[serde(deny_unknown_fields)]`.
- Prefer `bitcoin` crate types over primitives: `Txid` not `String`, `Amount` not `u64`, `Address<NetworkUnchecked>` checked with `.require_network()`.
- Reuse constants the `bitcoin` crate already provides (e.g. `Amount::MAX_MONEY`) instead of redefining them.
- `parse` turns bytes into raw params; `validate` turns raw params into a validated type. Signing functions accept only the validated type, so skipping validation is a compile error, not a code-review catch.

## 8. Amounts & Arithmetic

- No floating point anywhere. BTC values are integer satoshis (`Amount` / `u64`).
- No `as` casts on amounts. Use `TryFrom` when a conversion can fail.
- All arithmetic on amounts is checked; overflow is an error, never a wrap or a panic:

```rust
let total_in = params
    .inputs
    .iter()
    .try_fold(Amount::ZERO, |sum, input| sum.checked_add(input.amount))
    .ok_or(Error::AmountOverflow)?;
```

## 9. Secrets Handling

The passphrase and the decrypted private key are the only secrets in the program. Rules for them:

- Held in `secrecy` types (`SecretString`, `SecretBox`) or `zeroize::Zeroizing`. Never a plain `String` or `[u8; 32]` that outlives the line that created it.
- Scoped as tightly as possible: decrypt, sign, and drop as close together as the code allows.
- `expose_secret()` appears only at the call that consumes the secret. Every occurrence is reviewed (`grep -n expose_secret src/`).
- Types holding secrets never derive `Debug`, `Display`, `Clone`, or `Serialize`. If `Debug` is needed, implement it by hand and print `[REDACTED]`.
- The passphrase is read **only** through `rpassword`, in `main.rs`. Never from a CLI flag, env var, or file: those leak through shell history, `/proc`, and backups.

## 10. Errors & Output

- No `unwrap`, `expect`, `panic!`, or slice indexing that can panic in non-test code. Use `?`, `.get()`, and explicit error variants.
- **`thiserror`** for the crate's error enum in `error.rs`. Variants carry only public data (txid, index, amounts) so tests can match on them.
- **`anyhow`** only in `main.rs`, to add context and print the final error chain once.
- Error messages never include secret material, and never format a struct that contains a secret.
- **stdout carries exactly one thing:** the signed transaction hex, written once on success. Prompts, warnings, and errors go to stderr. On any failure, stdout stays empty and the exit code is non-zero.
- No logging framework. One `eprintln!` of the final error is enough for a CLI.

## 11. Network & File I/O

- Zero network access. No crate that pulls in an HTTP client, async runtime, or socket code.
- File reads are bounded, and oversize input is an error, not silent truncation. The reader is a parameter, so tests pass a byte slice instead of a file:

```rust
const MAX_PARAMS_BYTES: u64 = 1_048_576; // spec §5.1: 1 MiB

pub fn read_bounded(mut reader: impl Read) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    reader.by_ref().take(MAX_PARAMS_BYTES).read_to_end(&mut buf)?;
    if reader.read(&mut [0u8; 1])? != 0 {
        return Err(Error::ParamsTooLarge);
    }
    Ok(buf)
}
```

## 12. Dependencies

- Only the crates listed in spec §2. Adding any other crate needs a stated reason in the PR.
- Turn off default features (`default-features = false`) and enable only what's used.
- `Cargo.lock` is committed. Build and test with `--locked`.
- Run `cargo audit` before tagging a release.

## 13. Comments

- Comment *why*, not *what*. Readable code already says what (§3).
- Every security check carries a one-line reference to the spec rule it enforces, so reviewers can trace spec → test → code:

```rust
// Spec §5.2: reject duplicate UTXOs before building the tx.
```

- `///` doc comments on public items. Skip them on private helpers whose name says it all.

## 14. Before Every Commit

Run `make`. It runs these three checks, and all three must pass:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

`make build` runs the same checks before building, so a binary that breaks a rule is never produced. CI, once set up, runs `make`.
