# Task 63 report — TLS protected-record integrity

## Red evidence

Added `crates/kmipkit-transport/tests/tls_integrity.rs` as a focused derived
integration target. A loopback relay forwards the real TLS 1.3/mTLS handshake
and application traffic. The peer disables session tickets and holds its
response until after it has completed mTLS and read the decrypted request; the
test then arms the relay to flip the final ciphertext/tag byte in the first
server-to-client TLS 1.3 application-data record. The raw-TLS and HTTPS cases
assert rejection, `PossiblySent`, only the safe TLS cause, absence of request
and response sentinels from diagnostics, and connection invalidation.

The baseline failures demonstrate the mapping regression on both adapters:

- `cargo test -p kmipkit-transport --test tls_integrity raw_tls_rejects_a_tampered_post_handshake_application_record --offline -- --exact` — **RED as expected**; the actual cause category is `Io`, while the TLS-record contract expects `Tls`.
- `cargo test -p kmipkit-transport --test tls_integrity https_invalidates_a_connection_after_a_tampered_post_handshake_application_record --offline -- --exact` — **RED as expected**; the actual cause category is `Io`, while the TLS-record contract expects `Tls`.
- `cargo test -p kmipkit-transport --test tls_integrity --offline` — **RED as expected**; 54 source-included adapter/unit tests pass and the two new regression tests fail only because both errors report `Io` instead of `Tls`.
- `cargo fmt --all --check`, `cargo clippy -p kmipkit-transport --test tls_integrity --all-features --offline -- -D warnings`, and `git diff --check` — PASS for the Red test target.

Both failures occur after the relay reports one corrupted protected response
record and after the peer reports a completed TLS 1.3 mTLS handshake plus a
decrypted request. The failures therefore exercise record integrity rather
than handshake rejection. No successful-handshake test is credited as
negative integrity evidence, and no profile or official OASIS test claim is
made.

## Green evidence

The timeout I/O sanitizer preserves a private, fixed TLS-failure marker when
the discarded source is a rustls error. The raw-TLS and HTTPS adapters map
that marker to the existing safe `Tls` cause category. The original rustls
error text and payload are discarded; the exposed source is the fixed
`TLS failure` category. No public API or error text contains peer-controlled
TLS diagnostics.

- `cargo test -p kmipkit-transport --test tls_integrity --offline` — PASS; all
  56 source-included adapter/unit and TLS integrity tests pass, including both
  corrupted-record regressions.
- `cargo test -p kmipkit-transport --all-targets --all-features --offline` —
  PASS for the complete transport package.
- `cargo fmt --all --check`, `git diff --check`, and `cargo clippy -p
  kmipkit-transport --all-targets --all-features --offline -- -D warnings` —
  PASS.

The negative assertions observe exactly one corrupted server application-data
record, reject its plaintext, retain `PossiblySent` because no response byte
was decrypted, expose only the safe TLS category, and verify connection
termination or cache invalidation for the affected adapter.

## Refactor evidence

Renamed the private sanitized marker to `TlsFailure` and aligned its fixed
display text with the general category. This keeps the sanitizer accurate for
all rustls failures while preserving the exact externally tested behavior.

- `cargo test -p kmipkit-transport --all-targets --all-features --offline` —
  PASS, including both record-corruption tests.
- `cargo fmt --all --check`, `git diff --check`, and `cargo clippy -p
  kmipkit-transport --all-targets --all-features --offline -- -D warnings` —
  PASS.
