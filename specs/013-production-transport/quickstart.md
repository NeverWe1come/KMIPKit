# Quickstart: Verify Production Transports

This guide describes the acceptance scenarios for the production transport
feature. It becomes runnable after KMIPKIT-0013 is implemented. It does not
claim that a public server or a formal OASIS profile test is available.

## Prerequisites

- Rust 1.94 or later and the repository's pinned toolchain.
- A built KMIPKit workspace with KMIPKIT-0013 enabled.
- The test harness's ephemeral CA, server certificate, client certificate,
  and matching unencrypted client key. Production tests do not use the host's
  public network or real credentials.

## Run focused checks

```powershell
cargo test -p kmipkit-transport --test tls_policy
cargo test -p kmipkit-transport --test resolver
cargo test -p kmipkit-transport --test raw_tls
cargo test -p kmipkit-transport --test https
cargo test -p kmipkit-transport --test timeout_delivery
cargo test -p kmipkit-client --test production_client
```

Expected results: valid TLS 1.3/mTLS exchanges succeed; TLS 1.2, invalid
trust/name/validity/CRL, malformed framing, unsupported HTTP behavior, and
timeout scenarios fail closed with the expected delivery state. Raw TLS
closes after each frame; HTTPS reuses a healthy connection. All response
limits are enforced before KMIPKit body-owner growth, no failed KMIP request
is replayed, and public client calls remain synchronous even when invoked by
a caller already inside a Tokio runtime.

## Run repository gates

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

The same targeted and workspace scenarios must pass on Linux, Windows, and
macOS CI. Review [timeout and delivery semantics](contracts/timeout-and-delivery.md)
when a timeout assertion differs from the observed phase.
