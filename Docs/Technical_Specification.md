# Bitcoin Offline Transaction Signer: Technical Specification

## 1. Architecture Overview & Constraints

This CLI application is designed as an air-gapped offline transaction signer. To meet exchange-grade security standards, it strictly separates public transaction intent (the payload) from cryptographic secrets (the keystore).

- **Offline-First:** Zero network or HTTP calls. All execution happens locally.
- **Air-Gapped Workflow:** Designed to receive a transaction payload via "sneakernet" (e.g., USB drive), sign it locally, and output a signed hex string to be broadcasted by an online machine.
- **Network Flexibility:** Supports `testnet` and `mainnet` flag configurations to satisfy the live broadcast verification requirement.

## 2. Recommended Rust Stack

- **`bitcoin`:** Standard crate for data structures, script evaluation, and ECDSA/Schnorr signatures.
- **`bip38` & `rpassword`:** For secure passphrase prompting and decrypting the encrypted keystore locally.
- **`secrecy` / `zeroize`:** To guarantee private keys are wiped from memory immediately after signing.
- **`clap`** (with `derive`): For robust command-line argument parsing.
- **`serde` / `serde_json`:** For strict type-safe deserialization.
- **`anyhow` / `thiserror`:** For context-aware error handling that explicitly avoids logging secrets.

## 3. Data & Input Specification

### 3.1. The Intent Payload (`params.json`)

This file is generated on an online machine and brought to the offline signer. It contains no private keys.

```json
{
  "network": "testnet",
  "inputs": [
    {
      "txid": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
      "vout": 0,
      "amount_sats": 100000,
      "pubkey": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
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

Each input names the compressed public key that owns it (`pubkey`, hex). The signer uses it to select the matching private key from the keystore.

### 3.2. The Keystore

The cryptographic secrets reside permanently on the offline machine. Using the BIP38 standard demonstrates deep Bitcoin ecosystem knowledge.

- The application reads a local keystore file, `keys.txt` by default (`--keystore` selects another file). Each line holds one key: the compressed public key in hex, a space, and the BIP38 encrypted string (starting with `6P...`).
- The raw Wallet Import Format (WIF) key is never stored on disk in plaintext.
- A later increment may replace `keys.txt` with a dedicated keystore. Inputs keep selecting keys by public key.

## 4. Core Execution Flow

1. **Parse Arguments:** `clap` parses the path to `params.json` and, optionally, the keystore file (default `keys.txt`).
2. **Strict Deserialization:** Read `params.json` into typed Rust structs, applying security boundaries (see [Section 5](#5-security--anti-dos-safeguards)).
3. **Construct Unsigned Transaction:** Map inputs to `bitcoin::TxIn` and outputs to `bitcoin::TxOut` (resolving `ScriptPubkey`).
4. **Secure Key Decryption:**
   - Read the keystore and find the BIP38 string for each public key named by the inputs.
   - For each key, prompt the user for its decryption passphrase via `rpassword` (terminal echo disabled).
   - Derive the WIF private key in memory and check that its public key matches.
5. **Sign Transaction:** Generate Sighashes and apply signatures to the witness or `scriptSig` data.
6. **Zeroize Secrets:** Immediately overwrite the derived private key in memory.
7. **Output Hex:** Print the serialized `bitcoin::Transaction` to stdout for the user to copy.

## 5. Security & Anti-DoS Safeguards

### 5.1. Resource Exhaustion Prevention

- **File Size Limits:** `params.json` reads are strictly bounded (e.g., maximum 1 MB via `std::io::Read::take`) to prevent OOM attacks.
- **Vector Limits:** A hard cap of 1,000 items on the `inputs` and `outputs` arrays prevents CPU exhaustion during signature generation.
- **Strict Parsing:** Serde is configured with `#[serde(deny_unknown_fields)]` to reject bloated, malformed JSON.

### 5.2. Financial Logic Boundaries

- **Satoshi Bounds:** All amounts are parsed as `u64`. A validation step ensures the total sum of inputs/outputs never exceeds the 21 million BTC cap ($2.1 \times 10^{15}$ satoshis).
- **Fee Rate Sanity Check:** A hardcoded maximum fee threshold (e.g., 0.01 BTC max fee) is enforced. If `sum(inputs) - sum(outputs)` exceeds this limit, the CLI aborts immediately to prevent "change omission" loss.
- **Fee Consistency Check:** `fee_sats` must equal `sum(inputs) - sum(outputs)`; any mismatch aborts signing.
- **Duplicate Input Check:** Verifies all provided UTXOs (TxID + vout) are unique before building the transaction.

### 5.3. Memory Safety

- Variables holding the BIP38 passphrase and the decrypted WIF key are wrapped in `secrecy::SecretString` or explicitly dropped using `zeroize`.
- Error handling formats are audited to ensure a failed parse or signature step never prints key material to stderr.

## 6. Deliverables & Broadcast Verification

- **Codebase:** Hosted on GitHub/GitLab, including a fully resolved `Cargo.lock` to guarantee deterministic builds.
- **Testing:** Unit tests covering the JSON parsing boundaries and fee sanity checks using hardcoded mock data.
- **Verification:** An execution run on testnet where the output hex is broadcasted via an explorer (like mempool.space), providing the resulting TxID URL as proof of functionality.
