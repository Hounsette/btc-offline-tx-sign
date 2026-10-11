# Coding Conventions

- Keep parsing, validation, signing, and terminal I/O easy to follow. Split modules only when it helps readability.
- Use `bitcoin` types and library operations; do not implement cryptography.
- Keep I/O at the CLI boundary so signing and validation can be tested without files or prompts.
- Use integer satoshis, checked arithmetic, and names that show units.
- Validate untrusted data before signing; reject unsupported cases with clear errors.
- Never log private keys or expose them through arguments. Keep secrets short-lived and zeroize owned buffers; do not claim all library copies are erased.
- Test observable behavior and meaningful failures. Use synthetic/published fixtures; never fund their keys. Live demonstrations use separate disposable testnet keys.
- Give each reachable error variant a named rejection test (`returns_<variant>_<condition>`). Keep each rejection test focused on one variant, assert its type and relevant fields, and document deferred cases. List them with `cargo test --locked --test payload_test -- --list`.
- Module errors use thiserror: one-level messages, causes in `source()`. The CLI prints only the top-level `Error` (from `main`'s `fail`, enforced by `print_stderr`), whose Display escapes the whole chain.
- Use rustfmt and the existing clippy rules. Add a regression test for each bug; avoid tests that merely repeat implementation details.
- Keep dependencies minimal and commit `Cargo.lock`. Libraries: `bitcoin`, `clap` (without default features), `serde`/`serde_json`, `rustix` (terminal settings), `zeroize`, and `thiserror`.
- Before committing code, run `make`. For documentation-only edits, check examples, links, and consistency instead.

See the [specification](Technical_Specification.md) for behavior and the [checklist](Roadmap.md) for acceptance checks.
