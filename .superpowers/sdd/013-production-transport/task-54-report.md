# T054 user-guide evidence

## Outcome

Added production raw-TLS and HTTPS configuration guidance in English and
Spanish. The guides cover endpoint selection and connection reuse, explicit
mTLS identity and server trust, certificate/key file ownership, TLS policy,
platform trust, session resumption, phase and total deadlines, per-call
overrides, delivery states, response limits, redaction, memory-cleanup limits,
and server interoperability boundaries. Updated the existing typed-execution
guides to remove outdated claims that the production constructor and
transports do not exist. Included both language guides in the client crate's
rustdoc so their Rust examples run as doctests.

## Red, Green, Refactor evidence

- Red: commit `3dcd1b3` linked the not-yet-created bilingual guides into
  rustdoc. `cargo test -p kmipkit-client --doc --offline` failed at the
  expected missing guide files.
- Green: commit `d17ac39` added both guides and updated stale English and
  Spanish transport statements. `cargo test -p kmipkit-client --doc --offline`
  passed 6 doctests and 1 expected compile-fail doctest.
- Refactor: commit `6711d79` added matching timeout-override and delivery-state
  examples in both languages, including a forward-compatible wildcard for the
  non-exhaustive delivery-state enum. The first run correctly exposed the
  missing wildcard; after the fix, `cargo test -p kmipkit-client --doc
  --offline` passed 8 doctests and 1 expected compile-fail doctest.

## Verification

- `cargo test -p kmipkit-client --doc --offline`: PASS — 8 passed, 0 failed;
  one expected compile-fail doctest passed.
- Relative Markdown links under `docs/user-guide/`: PASS — all resolve to
  existing files.
- `git diff --check`: PASS.

## Changed files

- `crates/kmipkit-client/src/lib.rs`
- `docs/user-guide/en/client-execution.md`
- `docs/user-guide/en/production-transports.md`
- `docs/user-guide/es/ejecucion-cliente.md`
- `docs/user-guide/es/transportes-produccion.md`
