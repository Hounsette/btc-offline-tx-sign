# Coding Conventions

- Keep parsing, validation, signing, and terminal I/O easy to follow. Split modules only when it helps readability.
- Use `bitcoin` types and library operations; do not implement cryptography.
- Keep I/O at the CLI boundary so signing and validation can be tested without files or prompts.
- Use integer satoshis, checked arithmetic, and names that show units.
- Validate untrusted data before signing; reject unsupported cases with clear errors.
- Never log private keys or expose them through arguments. Keep secrets short-lived and zeroize owned buffers; do not claim all library copies are erased.
- Test observable behavior and meaningful failures. Use synthetic/published fixtures; never fund their keys. Live demonstrations use separate disposable testnet keys.
- Use rustfmt and the existing clippy rules. Add a regression test for each bug; avoid tests that merely repeat implementation details.
- Keep dependencies minimal and commit `Cargo.lock`. Planned libraries: `bitcoin`, `clap`, `serde`/`serde_json`, `rpassword`, `zeroize`, and `anyhow`.
- Before committing code, run `make`. For documentation-only edits, check examples, links, and consistency instead.

See the [specification](Technical_Specification.md) for behavior and the [checklist](Roadmap.md) for acceptance checks.
