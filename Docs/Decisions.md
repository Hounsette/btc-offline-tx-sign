# Decisions

This document lists every point that the [Technical Specification](Technical_Specification.md) leaves open, plus every assumption the [Roadmap](Roadmap.md) had to make. The roadmap refers to these decisions by ID, for example (D4).

## 1. How to Use This Document

Each decision has a status:

- **Decided:** confirmed by the project owner.
- **Partly decided:** part of it is confirmed, and the rest still needs an answer.
- **Proposed:** the roadmap already assumes the proposed answer. Confirm it or change it.
- **Open:** there is no proposal yet, so it needs an answer.

To record a decision, set its status to **Decided** and write down the answer and the date. If the answer differs from the proposal, update the deliverables listed under **Needed by** before they start, and update the spec if the decision changes it.

### Answer These First

These decisions shape the most work, and early deliverables depend on them:

1. **D1 and D10: Input types, and one key per input or per transaction.** Both change the `params.json` schema.
2. **D30: Repository host.** The CI pipeline (BTCSIGN-001) depends on it.

## 2. Summary

The table is sorted by the first deliverable that needs each answer.

| ID | Decision | Status | Needed by |
|---|---|---|---|
| D28 | Target platform and toolchain | Open | BTCSIGN-001 |
| D30 | Repository host, branch, visibility, license | Open | BTCSIGN-001 |
| D7 | `keys.txt` name and format | Partly decided | BTCSIGN-002 |
| D19 | CLI arguments and keystore location | Proposed | BTCSIGN-002 |
| D4 | `params.json` size cap | Proposed | BTCSIGN-003, BTCSIGN-017 |
| D1 | Input script types | Proposed | BTCSIGN-004 |
| D2 | Network values and mainnet safety | Proposed | BTCSIGN-004 |
| D10 | Key selection: per input or per transaction | Partly decided | BTCSIGN-004 |
| D13 | `params.json` or PSBT | Decided | BTCSIGN-004 |
| D6 | Empty inputs or outputs | Proposed | BTCSIGN-005 |
| D5 | Fee limit | Proposed | BTCSIGN-006 |
| D15 | Dust and zero-value outputs | Proposed | BTCSIGN-006 |
| D14 | Allowed output types | Proposed | BTCSIGN-007 |
| D18 | Output and exit codes | Proposed | BTCSIGN-008 |
| D3 | Transaction fields | Proposed | BTCSIGN-009 |
| D31 | Expected-hex vectors and test keys | Proposed | BTCSIGN-009 |
| D16 | Standard transaction size | Proposed | BTCSIGN-010 |
| D20 | Passphrase Unicode normalization | Proposed | BTCSIGN-012 |
| D21 | BIP38 EC-multiply keys | Proposed | BTCSIGN-012 |
| D22 | The `bip38` crate | Proposed | BTCSIGN-012 |
| D25 | No WIF string in memory | Proposed | BTCSIGN-012 |
| D8 | Checks before the passphrase prompt | Proposed | BTCSIGN-013 |
| D9 | Passphrase prompts | Proposed | BTCSIGN-013 |
| D17 | Operator confirmation | Proposed | BTCSIGN-013 |
| D23 | Key generation | Open | BTCSIGN-013 |
| D26 | Operating model | Proposed | BTCSIGN-013 |
| D27 | Signing record | Proposed | BTCSIGN-013 |
| D32 | Automated test of a successful run | Proposed | BTCSIGN-013 |
| D24 | Memory protection beyond zeroize | Proposed | BTCSIGN-014 |
| D33 | Testnet for the broadcast proof | Proposed | BTCSIGN-015 |
| D29 | Release integrity | Proposed | BTCSIGN-016 |
| D12 | Trusting input amounts | Decided | BTCSIGN-017 |
| D11 | Real keystore | Partly decided | BTCSIGN-018 |

## 3. Decisions

### D1: Input script types

**Status:** Proposed · **Needed by:** BTCSIGN-004

Which kinds of inputs must the signer be able to spend?

- **Proposed:** native segwit (P2WPKH) only, with compressed keys. The spec example uses a `tb1q…` address.
- **Other options:** also support legacy P2PKH (spec §4 mentions `scriptSig`), wrapped segwit (P2SH-P2WPKH), or Taproot (spec §2 mentions Schnorr signatures).
- **What changes:** each extra type needs a `script_type` field per input in `params.json`, its own signing code, and its own expected-hex tests. Legacy signatures don't commit to the input amount at all, so supporting legacy inputs effectively requires the full previous transactions described in D12.

### D2: Network values and mainnet safety

**Status:** Proposed · **Needed by:** BTCSIGN-004

- **Proposed:** the `network` field in `params.json` accepts only `"mainnet"` and `"testnet"`. The "flag" in spec §1 means this field, not a CLI flag. `"testnet"` covers testnet3 and testnet4, which use the same formats. Signet and regtest are rejected.
- **Question:** should mainnet signing also require the operator to say so on the command line, for example `--mainnet`? Keys aren't tied to a network, so the same key can sign on both. Without such a flag, `params.json` alone decides whether real money moves. Proposed: no flag.
- **Question:** should signet or regtest be allowed for development? Proposed: no.

### D3: Transaction fields

**Status:** Proposed · **Needed by:** BTCSIGN-009

- **Proposed:** version 2, locktime 0, sequence `0xFFFFFFFD` (Replace-By-Fee enabled), `SIGHASH_ALL` only, deterministic low-R ECDSA, and inputs and outputs in the order given in `params.json`.
- **Questions:**
  - Should RBF stay on, so a stuck transaction can be replaced by signing a new one with a higher fee?
  - Should `params.json` be able to set a locktime, for example for a payment that can't be confirmed before a given date? Proposed: no.
  - Should inputs and outputs be sorted (BIP69) for privacy? Proposed: no, the online tool decides the order.

### D4: `params.json` size cap

**Status:** Proposed · **Needed by:** BTCSIGN-003, BTCSIGN-017

- **Proposed:** 1 MiB (1_048_576 bytes), as in Coding_Style §11. The spec says "1 MB".
- **Proposed for BTCSIGN-017:** with full previous transactions in the input file (D12), 1 MiB is too small for 1,000 inputs. Raise the cap to 16 MiB. A batch that doesn't fit is split into several transactions.

### D5: Fee limit

**Status:** Proposed · **Needed by:** BTCSIGN-006

- **Proposed:** a hardcoded cap of 1_000_000 sats (0.01 BTC) on the total fee, the same on both networks. A fee exactly at the cap is accepted, and a zero fee is accepted too.
- **Questions:**
  - Spec §5.2 calls this a fee *rate* check. Should the signer also cap the rate (sats per vbyte)? It can only measure the size after signing, so it would refuse to print an oversized result.
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

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed:** before asking for any passphrase, validate `params.json`, read the keystore, and check that every input's public key is in it.

### D9: Passphrase prompts

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed:** one prompt per key used, in the order the keys first appear in the inputs. Each prompt shows the key's full public key. After decryption, the key's public key must match the one in the keystore. A wrong passphrase stops the whole run.
- **Questions:**
  - Should a wrong passphrase allow a retry? Proposed: no, the operator runs the command again.
  - Should the number of keys per transaction be capped? 1,000 inputs from 1,000 keys means 1,000 passphrases, each taking about a second to check (scrypt). Proposed: no cap. Alternatives: a cap such as 20 keys, or one shared passphrase for all keys.

### D10: Key selection per input or per transaction

**Status:** Partly decided · **Needed by:** BTCSIGN-004

- **Decided (2026-10-09):** `params.json` selects the private keys by public key.
- **To confirm:** where the public key goes. The roadmap and spec currently put a `pubkey` on each input, so one transaction can spend coins owned by several keys. The alternative is a single `pubkey` per transaction: one key, one prompt, and simpler code.

### D11: Real keystore

**Status:** Partly decided · **Needed by:** BTCSIGN-018

- **Decided (2026-10-09):** a future increment replaces `keys.txt` with a real keystore, and `params.json` keeps selecting keys by public key.
- **Open:** which kind of keystore:
  - an encrypted keystore file, where one passphrase unlocks every key
  - the operating system's keyring
  - a hardware device (an HSM or a hardware wallet), where the key never leaves the device, so signing moves to the device as well
  - an HD wallet seed (BIP32), where keys are derived from one secret
- **Open:** whether `keys.txt` stays supported once the real keystore exists.

### D12: Trusting input amounts

**Status:** Decided · **Needed by:** BTCSIGN-017

The signer takes each input's amount from `params.json` and has no way to check it. In native segwit, each signature commits only to its own input's amount. A compromised online machine can exploit this:

1. It sends `params.json` for the same inputs and outputs twice, and each time understates a different input's amount.
2. Each run passes every fee check and produces one valid signature.
3. The attacker combines the valid signatures into one transaction, whose real fee is far higher than either run showed.

The attack needs the operator to sign twice, and it needs a transaction with at least two inputs. Hardware wallets fixed this in 2020 by requiring the full previous transaction for every input.

- **Decided (2026-10-09):** add a `prev_tx` field to each input in a later milestone (BTCSIGN-017, M6), after the v0.1.0 testnet release. The signer checks that the previous transaction hashes to the input's txid, then checks the amount and script against it. Until then the signer trusts the amounts, so v0.1.0 is for testnet only, and the README says so.
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
  - **D10 goes away.** Each input already carries its script, so the signer finds the key whose P2WPKH script matches. No `pubkey` field is needed.
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

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed (the spec as written):** no confirmation step.
- **Option:** before the prompts, print a summary to stderr (network, each output's address and amount, the fee, the keys used) and require the operator to type `yes`.
- **Recommendation:** add it. A compromised online machine can swap a destination address, and this summary is the operator's only chance to notice.

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

**Status:** Proposed · **Needed by:** BTCSIGN-012

BIP38 requires the passphrase to be normalized to Unicode NFC before use. Without normalization, a passphrase containing accented characters can fail when it's typed on a different system. BIP38 publishes a test vector for this case.

- **Proposed:** check whether the `bip38` crate normalizes. If it doesn't, normalize in the signer. That adds a crate not listed in spec §2, such as `unicode-normalization`.
- **Option:** accept only ASCII passphrases.

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

- **Proposed:** never encode the key as a WIF string. Keep the raw 32 bytes in `Zeroizing`, because a WIF string would be one more copy of the secret. Spec §4 step 4 says "derive the WIF private key in memory", so reword it once this is confirmed.

### D26: Operating model

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed:** signing is always interactive and signs one transaction per run. The passphrase is typed at the terminal, never taken from a flag, an environment variable, or a file (Coding_Style §9).
- **Question:** should unattended or batch signing ever be possible? Proposed: no.

### D27: Signing record

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed:** keep no record. The spec says there is no logging.
- **Option:** for audits, append one line per signing to a local file on the offline machine: the time, the txid, and the SHA-256 of `params.json`.

### D28: Target platform and toolchain

**Status:** Open · **Needed by:** BTCSIGN-001 (toolchain), BTCSIGN-016 (release build)

- **Open:** which operating system and CPU does the offline machine run?
- **Proposed, if it's Linux on x86_64:**
  - Build a static binary (`x86_64-unknown-linux-musl`) so it doesn't depend on the offline machine's glibc version.
  - Pin the Rust version in `rust-toolchain.toml` so every build, including CI, uses the same compiler.

### D29: Release integrity

**Status:** Proposed · **Needed by:** BTCSIGN-016

- **Proposed:** publish the SHA-256 hash of each release binary.
- **Option:** sign releases with minisign or GPG.
- **Option:** reproducible builds, where a second person builds the same commit and gets identical bytes. This needs the pinned toolchain from D28. Spec §6 asks for "deterministic builds", but `Cargo.lock` only pins dependency versions. On its own it doesn't make the binary identical byte for byte.

### D30: Repository

**Status:** Open · **Needed by:** BTCSIGN-001

- **Host:** GitHub or GitLab? Spec §6 allows either, and the roadmap assumes GitHub Actions.
- **Default branch:** the repository has only a local `master` branch and no remote. Should the default branch be `main`?
- **Visibility:** public or private? A public repository needs a license, and there is no `LICENSE` file yet.

### D31: Expected-hex vectors and test keys

**Status:** Proposed · **Needed by:** BTCSIGN-009

- **Proposed:** generate the expected transaction hex once, offline, with Bitcoin Core (`createrawtransaction`, `signrawtransactionwithkey`), and record the Core version next to each vector.
- **Question:** who generates the vectors?
- **Question:** which test key signs the spec example? The spec §3.1 example `pubkey` belongs to private key 1, a public, well-known key. Coding_Style §2.4 asks for freshly generated testnet keys instead.
  - Proposed: use private key 1 for the spec example only (it's never funded) and fresh keys for every other test.
  - Alternative: replace the spec example's `pubkey` with a freshly generated key.

### D32: Automated test of a successful run

**Status:** Proposed · **Needed by:** BTCSIGN-013

- **Proposed:** check the successful run by hand (BTCSIGN-013), because the passphrase can only come from the terminal.
- **Option:** automate it with a pseudo-terminal in the tests, using a dev-only crate or an `expect` script, so every `make` checks a full successful run.

### D33: Testnet for the broadcast proof

**Status:** Proposed · **Needed by:** BTCSIGN-015

- **Proposed:** testnet4, broadcast through mempool.space.
- **Option:** testnet3, which is being phased out.
- **Option:** signet, which would require D2 to allow it.
