# T025 Red Report — KMIPKIT-0013

## Test source

Initial Red test commit: `3b1579840010d42181e8fb4fad9449fd7d4ff919`.
Review-driven test correction commit: `0560662951a4e4df15938e5a6bc84b700ea9eaf3`.

Added `crates/kmipkit-transport/tests/raw_tls.rs` and registered its explicit
Cargo test target. The cases cover exact caller bytes, TLS 1.3 negotiation,
peer-required mTLS and received client identity, one response frame followed
by connection close, request-owner zeroization after success and peer-close
failure, unknown CA, expired server certificate, hostname mismatch, and caller
CRL revocation. They use `LoopbackTcpListener` and `EphemeralPki` from
`kmipkit-test-support`; rcgen fixtures provide the expired leaf and revoking
CRL.

No concrete adapter type name was added. The integration test source-includes
the future private `src/raw_tls.rs` module and calls a cfg(test)-only `new_for_test`
helper through its opaque `impl Transport` result. That seam accepts the
existing `SecretBufferObserver`; T026 must route it through the same request
owner used by production exchange. This is private test wiring and does not
add a public API. The request staging/zeroization assertions cannot pass until
that production path and observer handoff exist.

The review correction asserts `RequestDeliveryState::NotSent` for unknown CA,
expired certificate, hostname mismatch, and caller CRL rejection. The peer
fixture uses a nonblocking accept loop with a two-second monotonic deadline,
a short poll interval, blocking accepted sockets with read/write timeouts
before TLS, and a six-second operation deadline. It also asserts that the
local peer accepted a connection before treating a handshake rejection as
valid, so a pre-connect adapter error cannot satisfy these cases.

## Red evidence

Command:

```text
cargo test -p kmipkit-transport --test raw_tls --offline
```

Expected Red; the only reported error is the missing T026 source module:

```text
error: couldn't find file `crates\kmipkit-transport\tests\..\src\raw_tls.rs`
  --> crates\kmipkit-transport\tests\raw_tls.rs:32:1
   |
32 | mod raw_tls;
   | ^^^^^^^^^^^^

error: could not compile `kmipkit-transport` (test "raw_tls") due to 1 previous error
```

No runtime behavior is claimed. The named success and policy cases are test
expectations only until T026 implements the adapter.

## Focused checks

- `rustfmt --edition 2024 --config skip_children=true --check crates/kmipkit-transport/tests/raw_tls.rs`: passed.
- `cargo fmt --all -- --config skip_children=true --check`: passed.
- `git diff --check`: passed.
- Plain `cargo fmt --all -- --check` cannot traverse the source-included module
  while `src/raw_tls.rs` is absent; the targeted formatter checks above skip
  child module resolution and validated the changed Rust test file.

The request-owner observer remains a T026 implementation obligation; this Red
commit establishes assertions for both success and a post-write peer-close
failure without adding a test-only model of production ownership.
