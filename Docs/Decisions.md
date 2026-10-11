# Decisions

This document lists every point that the [Technical Specification](Technical_Specification.md) leaves open, plus every assumption the [Roadmap](Roadmap.md) had to make. The roadmap refers to these decisions by ID, for example (D4).

## 1. How to Use This Document

Each decision has a status:

- **Decided:** confirmed by the project owner.
- **Partly decided:** part of it is confirmed, and the rest still needs an answer.
- **Proposed:** the roadmap already assumes the proposed answer. Confirm it or change it.
- **Open:** there is no proposal yet, so it needs an answer.

Record the answer, date, rationale, and approver (project owner by default); retain superseded answers. Update affected docs and roadmap checks in the same PR.

**Decided means approved, not implemented.** Track delivery and evidence in the [roadmap](Roadmap.md#4-deliverable-index). Proposals remain provisional. Changes dated 2026-10-09 reflect the owner's documentation review; 2026-10-11 records approval of incremental signing and early input verification.

### Answer These First

These decisions shape the most work, and early deliverables depend on them:

1. **D30: Repository host.** The CI pipeline (BTCSIGN-001) depends on it.
2. **D4 and D7: Payload cap and keystore format.** Resolve before their parsers.

## 2. Summary

Deliverable dependencies in the roadmap determine implementation order.

| ID | Decision | Status | Needed by |
|---|---|---|---|
| D28 | Target platform and toolchain | Decided | BTCSIGN-001 |
| D30 | Repository host, branch, visibility, license | Open | BTCSIGN-001 |
| D7 | `keys.txt` name and format | Partly decided | BTCSIGN-002 |
| D19 | CLI arguments and keystore location | Proposed | BTCSIGN-002 |
| D4 | `params.json` size cap | Proposed | BTCSIGN-003, BTCSIGN-017 |
| D1 | Input script types | Decided | BTCSIGN-004 |
| D2 | Network values and mainnet safety | Partly decided | BTCSIGN-004, BTCSIGN-017 |
| D10 | Automatic key selection | Decided | BTCSIGN-004, BTCSIGN-021 |
| D13 | `params.json` or PSBT | Decided | BTCSIGN-004 |
| D6 | Empty inputs or outputs | Proposed | BTCSIGN-005 |
| D5 | Fee limit | Proposed | BTCSIGN-006 |
| D15 | Dust and zero-value outputs | Proposed | BTCSIGN-006 |
| D14 | Allowed output types | Proposed | BTCSIGN-007 |
| D18 | Output and exit codes | Proposed | BTCSIGN-008 |
| D3 | Transaction fields | Proposed | BTCSIGN-009 |
| D31 | Expected-hex vectors and test keys | Proposed | BTCSIGN-009 |
| D16 | Standard transaction size | Proposed | BTCSIGN-010 |
| D20 | Passphrase Unicode normalization | Decided | BTCSIGN-012 |
| D21 | BIP38 EC-multiply keys | Proposed | BTCSIGN-012 |
| D22 | The `bip38` crate | Proposed | BTCSIGN-012 |
| D25 | No WIF string in memory | Proposed | BTCSIGN-012 |
| D8 | Checks before the passphrase prompt | Decided | BTCSIGN-013 |
| D9 | Passphrase prompts | Proposed | BTCSIGN-013 |
| D17 | Operator confirmation | Decided | BTCSIGN-013 |
| D23 | Key generation | Open | BTCSIGN-013 |
| D26 | Operating model | Partly decided | BTCSIGN-013 |
| D27 | Signing record | Proposed | BTCSIGN-013 |
| D32 | Successful run and release verification | Partly decided | BTCSIGN-013, BTCSIGN-016 |
| D24 | Memory protection beyond zeroize | Proposed | BTCSIGN-014 |
| D33 | Testnet for the broadcast proof | Proposed | BTCSIGN-015 |
| D29 | Release integrity | Partly decided | BTCSIGN-016 |
| D34 | Backend compatibility and recovery | Decided | BTCSIGN-016, BTCSIGN-017 |
| D12 | Trusting input amounts | Decided | BTCSIGN-017 |
| D11 | Real keystore | Partly decided | BTCSIGN-018 |

## 3. Decisions

### D1: Input script types

**Status:** Decided · **Needed by:** BTCSIGN-004

- **Decided (2026-10-11):** P2WPKH inputs with compressed keys only. Build one-input signing first, then change, multiple inputs, and multiple keys.
- Legacy, wrapped SegWit, multisig, and Taproot inputs need separate decisions and tests.

### D2: Network values and mainnet safety

**Status:** Partly decided · **Needed by:** BTCSIGN-004, BTCSIGN-017

- **Decided (2026-10-09):** v0.1.0 accepts only `"testnet"` (testnet3/4 formats), rejecting other networks before keystore access or prompts (spec §1). This replaces immediate mainnet support to enforce D12.
- Use disposable test keys that never hold mainnet funds: network labels do not isolate keys or signatures.
- Mainnet remains disabled after BTCSIGN-017 pending separate release approval of input verification, operator review, network tests, and release checks.
- **Open:** future mainnet CLI opt-in. Signet/regtest also need a new decision and tests.

### D3: Transaction fields

**Status:** Proposed · **Needed by:** BTCSIGN-009

- **Proposed:** version 2, locktime 0, sequence `0xFFFFFFFD` (Replace-By-Fee enabled), `SIGHASH_ALL` only, deterministic low-R ECDSA, and inputs and outputs in the order given in `params.json`.
- **Questions:**
  - Should RBF stay on, so a stuck transaction can be replaced by signing a new one with a higher fee?
  - Should `params.json` be able to set a locktime, for example for a payment that can't be confirmed before a given date? Proposed: no.
  - Should inputs and outputs be sorted (BIP69) for privacy? Proposed: no, the online tool decides the order.

### D4: `params.json` size cap

**Status:** Proposed · **Needed by:** BTCSIGN-003, BTCSIGN-017

- **Proposed:** 16 MiB (16_777_216 bytes) from the first release, including required previous transactions (D12). The byte cap applies independently of the 1,000-input cap; split batches that exceed it.

### D5: Fee limit

**Status:** Proposed · **Needed by:** BTCSIGN-006

- **Proposed:** a hardcoded cap of 1_000_000 sats (0.01 BTC) on the total fee. A fee exactly at the cap is accepted, and a zero fee is accepted too. Reassess the cap before any future mainnet release.
- **Questions:**
  - Spec §5.2 caps the total fee. Should the signer also cap the rate (sats per vbyte), using the final signed size before printing a result?
  - Is 0.01 BTC the right cap for mainnet? A 1,000-input consolidation is about 68,000 vbytes, so at 15 sats per vbyte its fee already exceeds the cap.
  - Should the cap be configurable? Proposed: no, because a flag that raises the cap weakens the safeguard.

### D6: Empty inputs or outputs

**Status:** Proposed · **Needed by:** BTCSIGN-005

- **Proposed:** reject a transaction with zero inputs or zero outputs.

### D7: `keys.txt` name and format

**Status:** Partly decided · **Needed by:** BTCSIGN-002 (name), BTCSIGN-011 (format)

- **Decided (2026-10-09):** the keystore starts as a text file that the signer uses by default. A real keystore comes later (D11).
- **To confirm:** the file name. You wrote `key.txt`. The roadmap uses `keys.txt`, which is the spec's name and fits a file that holds several keys.
- **Proposed format:** one key per line: the compressed public key in hex, a space, then the BIP38 string. Blank lines and whitespace around a line are ignored. Any other malformed line, or a public key listed twice, is rejected. The read is capped at 64 KiB, which is about 500 keys.
- **Questions** (proposed answer to each: no):
  - Allow `#` comments?
  - Allow an optional label per key (for example `cold-1`), shown in the passphrase prompt in place of the raw public key?
  - Refuse a `keys.txt` that other users can read, the way `ssh` refuses world-readable private keys?

### D8: Checks before the passphrase prompt

**Status:** Decided · **Needed by:** BTCSIGN-013

- **Decided (2026-10-09):** validate params, keystore, and required-key presence before confirmation or passphrases (spec §4, D17). Reject invalid requests before involving the operator or secrets; BTCSIGN-013 checks the order.

### D9: Passphrase prompts

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed:** one prompt per key used, in the order the keys first appear in the inputs. Each prompt shows the key's full public key. After decryption, the key's public key must match the one in the keystore. A wrong passphrase stops the whole run.
- **Questions:**
  - Should a wrong passphrase allow a retry? Proposed: no, the operator runs the command again.
  - Should the number of keys per transaction be capped? 1,000 inputs from 1,000 keys means 1,000 passphrases, each taking about a second to check (scrypt). Proposed: no cap. Alternatives: a cap such as 20 keys, or one shared passphrase for all keys.

### D10: Automatic key selection

**Status:** Decided · **Needed by:** BTCSIGN-004, BTCSIGN-021

- **Decided (2026-10-11):** omit input `pubkey`. Match each verified previous-output script against P2WPKH scripts derived from the offline public-key inventory. Reject missing keys before prompts; decrypt only used keys.
- **Superseded (2026-10-09):** `params.json` selected keys by explicit public key; its placement was unresolved.

### D11: Real keystore

**Status:** Partly decided · **Needed by:** BTCSIGN-018

- **Decided (2026-10-11):** a future increment replaces `keys.txt` while preserving script-based key selection (D10).
- **Superseded (2026-10-09):** the replacement retained explicit public keys in `params.json`.
- **Open:** which kind of keystore:
  - an encrypted keystore file, where one passphrase unlocks every key
  - the operating system's keyring
  - a hardware device (an HSM or a hardware wallet), where the key never leaves the device, so signing moves to the device as well
  - an HD wallet seed (BIP32), where keys are derived from one secret
- **Open:** whether `keys.txt` stays supported once the real keystore exists.

### D12: Trusting input amounts

**Status:** Decided · **Needed by:** BTCSIGN-017

Without previous transactions, the signer cannot verify input amounts. In native segwit, each signature commits only to its own input's amount. A compromised online machine can exploit this:

1. It sends `params.json` for the same inputs and outputs twice, and each time understates a different input's amount.
2. Each run passes every fee check and produces one valid signature.
3. The attacker combines the valid signatures into one transaction, whose real fee is far higher than either run showed.

The attack needs the operator to sign twice, and it needs a transaction with at least two inputs. Hardware wallets fixed this in 2020 by requiring the full previous transaction for every input.

- **Decided (2026-10-11):** require `prev_tx` from the first release. BTCSIGN-017 verifies txid, output index, declared amount, and P2WPKH script before fee checks; key matching follows D10. Mainnet remains separately gated (D2).
- **Superseded (2026-10-09):** defer verification until after v0.1.0 and label amounts unverified.
- **Considered:**
  - Trust the amounts permanently, and document that the operator must never sign the same inputs twice with different amounts.
  - Taproot inputs only. BIP341 signatures commit to every input's amount, but this conflicts with D1.
  - A local record of every input signed (txid, vout, amount), so the signer refuses the same input with a different amount. It needs no new field, but the record must never be lost, and it doesn't help if a second signer holds the same key.
  - PSBT, ruled out by D13.

### D13: `params.json` or PSBT

**Status:** Decided · **Needed by:** BTCSIGN-004

- **Decided (2026-10-09):** keep the custom `params.json` format from spec §3.1. The signer serves an exchange whose own backend creates the input file, so PSBT's main advantage, working with other wallets, doesn't apply. The fee bypass attack still has to be handled within `params.json` (D12).
- **Considered:**
- **Option:** use PSBT (BIP174), the standard format for offline signing. Wallets such as Bitcoin Core and Sparrow create PSBTs and broadcast the result. The `bitcoin` crate already parses PSBTs. M2's JSON parsing would be replaced by PSBT parsing and validation, and spec §3.1 and §5.1 would change. Effects on other decisions:
  - **D1 is still needed.** PSBT works for every input type, but the signer still implements signing for each type it accepts. Native segwit (P2WPKH) only remains the simplest choice.
  - **D12 is solved** if every input must include its full previous transaction, which the signer checks against the txid.
  - **D10 stays the same.** Match the previous-output script against the local inventory; neither format needs an input `pubkey`.
  - **D2 changes.** A PSBT has no network field, and its outputs are scripts, not addresses. Signing doesn't depend on the network, so `network` would only be a CLI flag for displaying addresses.
  - **The fee check changes.** `fee_sats` disappears, because the fee is simply the inputs minus the outputs. It is still capped (D5).
  - **The unsigned transaction is built by the online wallet**, so BTCSIGN-009 goes away.
- **Why now:** switching after M2 throws most of M2 away.

### D14: Allowed output types

**Status:** Proposed · **Needed by:** BTCSIGN-007

- **Proposed:** accept P2PKH, P2SH, P2WPKH, P2WSH, and P2TR addresses for the params network. Reject addresses for future segwit versions (v2 to v16): until a soft fork defines those versions, anyone can spend coins sent to them.
- **Proposed:** no `OP_RETURN` (data) outputs, because outputs only hold an address.
- **Proposed:** the same address may appear in more than one output.

### D15: Dust and zero-value outputs

**Status:** Proposed · **Needed by:** BTCSIGN-006

- **Proposed:** no check. The spec allows these outputs, and nodes refuse to relay such transactions, so no funds are at risk.
- **Option:** reject outputs below the dust limit for their type (for example 294 sats for P2WPKH, 546 sats for P2PKH), or at least zero-value outputs.

### D16: Standard transaction size

**Status:** Proposed · **Needed by:** BTCSIGN-010

The spec §5.1 limits of 1,000 inputs and 1,000 outputs can produce a transaction larger than nodes relay (400,000 weight units, about 100,000 vbytes). 1,000 P2WPKH inputs alone are about 68,000 vbytes. Such a transaction signs fine but can't be broadcast. No funds are at risk.

- **Proposed:** no check.
- **Option:** refuse to print a transaction over 400,000 weight units, or lower the item limits.

### D17: Operator confirmation

**Status:** Decided · **Needed by:** BTCSIGN-013

- **Decided (2026-10-09):** after D8 checks, display spec §4's payment summary on stderr and require exact terminal `yes` before passphrases. Other responses, EOF, or terminal failure abort with non-zero exit and empty stdout.
- **Reason:** detect substituted recipients by comparison with independent trusted payment records. This replaces the earlier no-confirmation proposal; BTCSIGN-013 and D32 own verification.

### D18: Output and exit codes

**Status:** Proposed · **Needed by:** BTCSIGN-008

- **Proposed:** stdout gets the hex and a newline, once, and nothing else. The exit code is 0 on success and non-zero otherwise: 1 for signer errors, 2 for bad arguments (clap's default).
- **Questions** (proposed answer to each: no):
  - Print the txid to stderr after signing, so the operator can match it after broadcast?
  - Add an `--out <FILE>` option? `> signed.hex` already does the job.
  - Show the hex as a QR code, so it can leave the offline machine without a USB drive?
  - Use a distinct exit code for each kind of error, for scripts?

### D19: CLI arguments and keystore location

**Status:** Proposed · **Needed by:** BTCSIGN-002

- **Proposed:** named flags, `--params <PATH>` (required) and `--keystore <PATH>` (optional). The default `keys.txt` is looked up in the current directory.
- **Option:** positional arguments.
- **Option:** a fixed default location for the keystore, such as next to the binary or in a config directory. Spec §3.2 says the keystore lives permanently on the offline machine, while `params.json` arrives on a USB drive. If the operator runs the signer from the USB drive's folder, a current-directory default would look for `keys.txt` on the USB drive.

### D20: Passphrase Unicode normalization

**Status:** Decided · **Needed by:** BTCSIGN-012

- **Decided (2026-10-09):** support BIP38 NFC normalization so equivalent Unicode passphrases decrypt identically. Add normalization if the library lacks it; protect its buffers as secrets.
- BTCSIGN-012 specifies Unicode-vector tests without weakening D1's compressed-key restriction.
- **Rejected:** ASCII-only input excludes valid BIP38 passphrases.

### D21: BIP38 EC-multiply keys

**Status:** Proposed · **Needed by:** BTCSIGN-012

- **Proposed:** reject EC-multiply keys (`6Pf…`/`6Pn…`) with their own error.
- **Option:** support them, tested with the published BIP38 test vectors.

### D22: The `bip38` crate

**Status:** Proposed · **Needed by:** BTCSIGN-012

- **Proposed:** use the `bip38` crate named in spec §2, provided it is maintained and `cargo audit` reports no issues.
- **If it isn't acceptable:** implement BIP38 decryption directly with the `scrypt` and `aes` crates, which aren't in spec §2 and add crypto code to review, or choose another crate.

### D23: Key generation

**Status:** Open · **Needed by:** BTCSIGN-013 (the manual check needs encrypted test keys), BTCSIGN-015

The signer only reads keys. Something must create each key, encrypt it with BIP38, and output its public key and P2WPKH address.

- **Option:** an existing offline tool. Which one? It must output the compressed public key, the BIP38 string, and the P2WPKH address. Its BIP38 checksum must also use the same network as the `bip38` crate does, or correct passphrases are rejected.
- **Option:** a `keygen` subcommand in this binary. This adds random number generation and encryption code to the signer.

### D24: Memory protection beyond zeroize

**Status:** Proposed · **Needed by:** BTCSIGN-014

- **Proposed:** `secrecy` and `zeroize` wrappers, plus `non_secure_erase()` on the `secp256k1` secret key. That key type is `Copy`, so stray copies may stay in memory. Disable swap and core dumps at the operating system level on the offline machine, and document this in the README.
- **Option:** lock secret memory and disable core dumps from inside the signer. Both need `unsafe` code or new crates, which conflicts with `unsafe_code = "forbid"` and spec §2.

### D25: No WIF string in memory

**Status:** Proposed · **Needed by:** BTCSIGN-012

- **Proposed:** never encode the key as a WIF string. Keep the raw 32 bytes in `Zeroizing`, because a WIF string would be one more copy of the secret. Spec §4 step 4 leaves the representation provisional; update it when this is confirmed.

### D26: Operating model

**Status:** Partly decided · **Needed by:** BTCSIGN-013

- **Decided (2026-10-09):** v0.1 is interactive, signing one payload transaction with D17 review and terminal-only passphrases (Coding_Style §9).
- **Open:** future batch/unattended operation needs a new decision covering approval and secret handling.

### D27: Signing record

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed:** keep no record. The spec says there is no logging.
- **Option:** for audits, append one line per signing to a local file on the offline machine: the time, the txid, and the SHA-256 of `params.json`.

### D28: Target platform and toolchain

**Status:** Decided · **Needed by:** BTCSIGN-001

- **Decided (2026-10-09):** the offline machine is a standard Linux PC with an x86_64 CPU.
- **Decided (2026-10-09):** build for `x86_64-unknown-linux-musl` in `make build` and CI from BTCSIGN-001. Static linking avoids glibc dependence; verify compatibility on the actual offline target. CI catches incompatible dependencies when added.
- **Decided (2026-10-09):** pin Rust **1.98.1** in `rust-toolchain.toml`, so every build, including CI, uses the same compiler. Reproducible builds (D29) depend on this too. The reasons and the upgrade rule are below.

**Why a static binary.** A Rust program built for Linux the usual way (the `x86_64-unknown-linux-gnu` target) doesn't include the C standard library, glibc. Instead, it loads the copy installed on the machine that runs it, which must provide the symbol versions required by the binary. The binary is built on the online machine and copied to the offline one (see the [README build section](../README.md#build)), so the two machines' glibc versions must be compatible:

- The current build machine runs Ubuntu with glibc 2.39. Even today's empty `main()` needs glibc 2.34 or newer (`objdump -T target/release/btc-offline-signer`). It therefore won't start on Debian 11 or Ubuntu 20.04, which have glibc 2.31, and fails with `version 'GLIBC_2.34' not found`.
- The build machine gets updated and the offline machine doesn't, so the gap only grows.

The musl target builds a different C library, musl, into the binary itself. The resulting static artifact avoids the glibc version mismatch; supported OS/CPU combinations are recorded with release test results.

**Costs:**
- The build machine needs the target (`rustup target add x86_64-unknown-linux-musl`) and a musl C compiler (the `musl-tools` package on Debian and Ubuntu), because the `secp256k1` crate compiles C code. Neither is installed on the current build machine.
- Fixes to bundled musl or other bundled libraries require rebuilding and verifying a new release; OS library updates do not update those copies.

**Considered and rejected:** keeping the default target and building on the same Linux distribution and version as the offline machine, for example in a container. It works, but the two have to be kept in step for as long as the signer is used.

**Why Rust 1.98.1.** Rust has no long-term-support release. Fixes go only into new releases and are never backported to old ones. So the safest pin is the newest release that has had time for its own fixes to arrive. As of 2026-10-09:

| Version | Released | Verdict |
|---|---|---|
| 1.95.0 (installed on the build machine today) | 2026-04-16 | Unsafe. It has an LLVM miscompilation, present since at least 1.87 and fixed in 1.97.1. Its Cargo also lacks the fixes for CVE-2026-5222 and CVE-2026-5223 (fixed in 1.96; these affect third-party registries only) and for the libssh2 CVEs fixed in 1.96.1. |
| 1.98.0 | 2026-08-20 | Unsafe. In some cases it builds a trait object's vtable with a null pointer, which is undefined behavior. Error handling relies on trait objects (`dyn Error`). Fixed in 1.98.1. |
| **1.98.1** | 2026-09-03 | **Chosen.** It contains every fix listed above, and in five weeks no further point release has been needed. |
| 1.99.0 | 2026-10-01 | Too new. Four of the last five releases (1.94, 1.96, 1.97, 1.98) needed a point release 1 to 5 weeks after launch, three of them to fix miscompilations. Its announcement lists no security fixes, so staying on 1.98.1 misses nothing. |

**The toolchain file:**

```toml
[toolchain]
channel = "1.98.1"
profile = "minimal"
components = ["clippy", "rustfmt"]
targets = ["x86_64-unknown-linux-musl"]
```

BTCSIGN-001 adds this file. Verify the active compiler for rustup overrides; install `musl-tools` through the system package manager.

**Upgrade rule:**
- Pin the latest point release of the previous minor version. When 1.100.0 ships (expected around 2026-11-12, six weeks after 1.99.0), move to the latest 1.99.x.
- If a release fixes a security issue that affects this project, move to it straight away.
- Watch [the Rust release announcements](https://blog.rust-lang.org/releases) for both.
- Make each upgrade its own commit (`Build: Update Rust to 1.xx.y`) and run `make` and `make build`. The BTCSIGN-010 expected-hex tests also catch a compiler that miscompiles the signing code.

### D29: Release integrity

**Status:** Partly decided · **Needed by:** BTCSIGN-016

- **Decided (2026-10-09):** publish each binary's SHA-256 and BTCSIGN-016 release evidence; verify before execution. A matching hash alone does not authenticate its source.
- **Open before release:** choose a trusted release-record channel or signatures with an independently verified minisign/GPG key.
- **Optional:** reproducible builds require independent byte comparisons. `Cargo.lock` pins dependency resolution, not compiler, linker, environment, or all build inputs.

### D30: Repository

**Status:** Open · **Needed by:** BTCSIGN-001

- **Host:** GitHub or GitLab? Configure CI for the chosen host.
- **Default branch:** the repository has only a local `master` branch and no remote. Should the default branch be `main`?
- **Visibility:** public or private? A public repository needs a license, and there is no `LICENSE` file yet.

### D31: Expected-hex vectors and test keys

**Status:** Proposed · **Needed by:** BTCSIGN-009

- **Proposed:** generate the expected transaction hex once, offline, with Bitcoin Core (`createrawtransaction`, `signrawtransactionwithkey`), and record the Core version next to each vector.
- **Question:** who generates the vectors?
- **Question:** which test key signs the spec example? Its previous-output script belongs to public, well-known private key 1; never fund this fixture key.
  - Proposed: use private key 1 for the spec example only (it's never funded) and fresh keys for every other test.
  - Alternative: regenerate the spec's previous transaction for a fresh test key.

### D32: Successful run and release verification

**Status:** Partly decided · **Needed by:** BTCSIGN-013, BTCSIGN-016

- **Decided (2026-10-09):** run BTCSIGN-013's terminal checklist on every exact release binary, on the offline target. Record BTCSIGN-016's evidence without secrets; a passing build or `--help` is insufficient.
- **Method:** manual. **Open:** CI automation through a pseudo-terminal crate or `expect`, with a timeout. Retain release-artifact and target checks without weakening passphrase handling.

### D33: Testnet for the broadcast proof

**Status:** Proposed · **Needed by:** BTCSIGN-015

- **Proposed:** testnet4, broadcast through mempool.space.
- **Option:** testnet3, which is being phased out.
- **Option:** signet, which would require D2 to allow it.

### D34: Backend compatibility and recovery

**Status:** Decided · **Needed by:** BTCSIGN-016, BTCSIGN-017

- **Decided (2026-10-09):** follow spec §3.3 for release/backend compatibility, fixtures, and migration. Required `prev_tx` breaks old payloads; reject them without fallback. No JSON version field is added.
- **Updated (2026-10-11):** `prev_tx` and automatic key selection ship in the first release; reject the earlier proposed `pubkey` format. No released-format migration is needed yet.
- Rehearse the [operator runbook](../README.md#planned-operator-runbook) before release using disposable keys; keep secrets out of evidence. BTCSIGN-016/017 own verification.
- **Reason:** incompatible backend payloads or unrestorable keys can defeat an otherwise correct signer.
