# Design Decisions

Scope revised on 2026-10-11. Earlier production-oriented decisions are preserved in commit `ca9c856`; this document supersedes them for the assignment.

| Choice | Reason |
|---|---|
| Rust and the `bitcoin` library | Demonstrate readable Rust and Bitcoin transaction handling without implementing cryptography. |
| P2WPKH, one input, one key | Enough to satisfy the assignment and explain every signing step. |
| Explicit optional change | Demonstrates UTXO spending and fee calculation without adding coin selection. |
| Custom JSON | A small, visible transaction-construction interface; PSBT interoperability is outside this submission. |
| Full previous transaction | Verify the referenced amount/script offline and avoid redundant input amount/public-key fields. |
| Hidden WIF prompt | The brief does not require encrypted storage; no key files, BIP38, or keystore inventory to maintain. |
| Testnet only | Meets the brief without risking real funds. Use one documented testnet deployment for the demonstration. |
| Library defaults for deterministic ECDSA | Verify signatures and signing hashes; do not require a particular low-R optimization. |
| Standard Cargo build | Document the tested compiler and OS. Add a musl build only if the actual offline target needs it. |
| Focused tests and a real broadcast | Direct evidence of correctness; production release governance is outside scope. |

Keep strict validation, bounded input, checked amounts, fee review, secret-safe errors, and tests for rejection paths. Testnet-only does not justify weaker signing logic.

Defer multiple inputs/keys, encrypted storage, HD wallets, other input types, large-batch tuning, artifact signing, and upgrade/recovery procedures. No new abstraction is needed for these possible extensions.

The submission must include a GitHub/GitLab repository and a URL proving the broadcast result.