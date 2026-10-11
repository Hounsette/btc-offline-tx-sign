# Completion Checklist

Complete one working path before adding features. The [specification](Technical_Specification.md) defines the scope.

## BTCSIGN-001: Validate the payload

- [x] Read bounded JSON; reject duplicate/unknown fields and unsupported shapes.
- [x] Verify previous transaction txid, output index, amount, and script.
- [x] Check testnet addresses, counts, integer amounts, totals, and fee cap.
- [x] Test a valid fixture plus malformed input, wrong txid/vout/script/network, invalid amounts, overspending, and excessive fees. Boundary tests cover the chosen limits.

## BTCSIGN-002: Sign one input and one output

- [x] Build the transaction with the specified version, sequence, locktime, and output.
- [x] In tests, load a disposable key and match it to the previous-output script.
- [x] Compute the sighash, sign, construct the witness, and serialize using the library.
- [x] Check a published/reference signing hash, verify the signature independently, and round-trip the transaction. A changed output or wrong key must fail verification.

## BTCSIGN-003: Add explicit change

- [ ] Support a second output; require it to belong to the signing key.
- [ ] Test recipient/change amounts, fee calculation, foreign change rejection, and signature invalidation after changing either output.

## BTCSIGN-004: Complete the offline CLI

- [ ] Read `--params`, show the payment summary, and require confirmation.
- [ ] Read a compressed testnet WIF through a hidden terminal prompt; validate ownership before signing.
- [ ] Print only complete signed hex on success; no secret material in output/errors.
- [ ] Test CLI argument/file errors, cancellation, malformed/wrong-network/uncompressed keys, key mismatch, and empty stdout on failure. Check hidden input and the successful flow in a real terminal.
- [ ] Run `make`; review dependencies and secret lifetimes. Add simple CI if time permits.

## BTCSIGN-005: Demonstrate and submit

- [ ] Generate a fresh disposable testnet key and fund its P2WPKH address.
- [ ] Fetch the funding transaction online and create the payload.
- [ ] Build and sign with networking disconnected; record the method and tested environment.
- [ ] Broadcast from an online machine and obtain confirmation. Check destinations, amounts, and fee.
- [ ] Put the explorer URL and reproducible instructions in the README; never publish the key.
- [ ] Push to GitHub or GitLab and check that a fresh checkout builds and tests successfully.
- [ ] Submit the repository link and broadcast-result URL. Do not submit with the proof still marked pending.

No formal release tag, approval record, per-step PR, or future-keystore milestone is needed. Use small commits and keep tests passing.
