# Bitcoin Offline Transaction Signer: Technical Specification

Target-release requirements. Resolve marked provisional choices in [Decisions](Decisions.md) before implementation; see [Roadmap](Roadmap.md) for progress and tests.

## 1. Architecture Overview & Constraints

An offline CLI separates public transaction payloads from the private keystore.

- **Offline:** no network calls. Transfer the payload in and signed hex out for online broadcast.
- **Testnet-only:** v0.1.0 accepts only `"testnet"`, rejecting other networks before keystore access or prompts (D2, D12). Use disposable keys never funded on mainnet; network labels do not isolate keys or signatures.
- **Mainnet:** requires BTCSIGN-017, operator review, passing security/release checks, and separate release approval.
- **Trust:** the offline OS, binary, and operator review environment must be trusted.

## 2. Recommended Rust Stack

Proposed stack; confirm BIP38 compatibility under D20–D22.

- **`bitcoin`:** transaction types and signing; P2WPKH inputs only (D1).
- **`bip38` / `rpassword`:** key decryption and terminal passphrases.
- **`secrecy` / `zeroize`:** secret buffers; limits in §5.3.
- **`clap`** (`derive`): CLI arguments.
- **`serde` / `serde_json`:** strict deserialization.
- **`anyhow` / `thiserror`:** errors without secret material.

## 3. Data & Input Specification

### 3.1. The Intent Payload (`params.json`)

Generated online; contains no private keys. This synthetic fixture is not broadcastable. Its previous output belongs to public test key 1; never fund that key.

```json
{
  "network": "testnet",
  "inputs": [
    {
      "txid": "71e3be4e5b89d4263d5902a50180ea5b660a3af3ee8894aaa024ec0f2391a0f6",
      "vout": 0,
      "amount_sats": 100000,
      "prev_tx": "020000000111111111111111111111111111111111111111111111111111111111111111110000000000ffffffff01a086010000000000160014751e76e8199196d454941c45d1b3a323f1433bd600000000"
    }
  ],
  "outputs": [
    {
      "address": "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
      "amount_sats": 95000
    }
  ],
  "fee_sats": 5000
}
```

Each input requires `prev_tx` (full transaction hex). Verify its txid, `vout`, amount, and P2WPKH script before fee checks. Match that script against the offline public-key inventory to select the signing key; no input `pubkey` is accepted (D10, D12).

### 3.2. The Keystore

The encrypted keystore stays offline. File name/format and CLI details are provisional (D7, D19).

- Default: `keys.txt`; override with `--keystore`. Each line: compressed public-key hex, a space, and a BIP38 string (`6P...`).
- Never write plaintext private keys to disk.
- A future keystore retains matching by previous-output script (D11).

### 3.3. Backend Compatibility

- Release notes identify the payload contract, compatible backend version, and a tested backend-generated fixture (D34). No JSON version field is required.
- Reject unsupported payload shapes before keystore access or prompts; never fall back to weaker validation.
- The first release requires `prev_tx` and rejects the earlier `pubkey` payload shape. Update backend examples/fixtures before release; future contract changes require coordinated upgrades.

## 4. Core Execution Flow

1. **Parse Arguments:** `clap` parses the path to `params.json` and, optionally, the keystore file (default `keys.txt`).
2. **Strict Deserialization:** Read `params.json` into typed Rust structs, verify previous outputs, then check amounts, fees, and other security boundaries (see [Section 5](#5-security--anti-dos-safeguards)).
3. **Construct Unsigned Transaction:** Map inputs to `bitcoin::TxIn` and outputs to `bitcoin::TxOut` (resolving `ScriptPubkey`).
4. **Operator Review and Secure Key Decryption:**
   - Validate the keystore, match every input script to a local public key, and require every key before prompting.
   - Show network, every output address/amount, input/output totals, fee, and used public keys on stderr. Compare against independent trusted payment information.
   - Require terminal input exactly `yes`, excluding its line ending (D17). Other input, EOF, or terminal failure aborts: nonzero exit, empty stdout.
   - Prompt for each passphrase with echo disabled (`rpassword`), decrypt in memory, and check the public key. Key representation remains provisional (D25).
5. **Sign Transaction:** Generate sighashes and apply signatures using the approved input script type (D1).
6. **Clear Secrets:** Drop passphrases after decryption and clear owned key buffers after signing or failure (§5.3).
7. **Output Hex:** Print the serialized `bitcoin::Transaction` to stdout for the user to copy.

## 5. Security & Anti-DoS Safeguards

### 5.1. Resource Exhaustion Prevention

- **File Size Limits:** Reject oversize reads; never truncate. D4 proposes 16 MiB (16_777_216 bytes), including previous transactions. Keystore cap: D7. Confirm before implementation.
- **Vector Limits:** A hard cap of 1,000 items on each of the `inputs` and `outputs` arrays bounds transaction work.
- **Strict Parsing:** Serde is configured with `#[serde(deny_unknown_fields)]` to reject bloated, malformed JSON.

### 5.2. Financial Logic Boundaries

- **Satoshi Bounds:** Parse amounts as `u64`; individual amounts and input/output totals must not exceed 2.1 × 10^15 satoshis. Use checked arithmetic.
- **Absolute Fee Cap:** D5 proposes at most 1_000_000 sats (0.01 BTC). Confirm the value and whether to add a separate sats/vbyte limit.
- **Fee Consistency Check:** `fee_sats` must equal `sum(inputs) - sum(outputs)`; any mismatch aborts signing.
- **Duplicate Input Check:** Verifies all provided UTXOs (TxID + vout) are unique before building the transaction.
- **Previous Outputs:** require the supplied transaction to match `txid`, contain `vout`, and have the declared `amount_sats` and a supported P2WPKH script. Calculate fees from verified amounts. This does not prove confirmation or unspent status (D12).

### 5.3. Memory Safety

- Use short-lived secret wrappers that zeroize on drop; review success/error paths for copies (D24, D25).
- Temporary copies may remain after owned buffers are cleared; see [zeroize's limits](https://docs.rs/zeroize/latest/zeroize/#stackheap-zeroing-notes).
- Review and test errors for secret leaks.

## 6. Deliverables & Broadcast Verification

- **Build:** hosting is D30. Commit `Cargo.lock`; [`--locked`](https://doc.rust-lang.org/cargo/commands/cargo-build.html#manifest-options) pins dependencies, not byte-identical binaries.
- **Tests:** each requirement maps to a named test or recorded check in the roadmap.
- **Release:** follow BTCSIGN-016's artifact, signing, compatibility, and recovery checks.
- **Broadcast:** record a confirmed testnet transaction's TxID URL; this is not mainnet approval.
