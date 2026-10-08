# T034 Red Report — KMIPKIT-0013

## Scope and baseline

T034 Red test commits: `d4206ec` (`test(transport): define HTTPS request capture contract (T034 Red)`) and
`1c6c8ab` (`test(transport): cover HTTPS target validation (T034 Red)`), followed by QA correction
`2669cca` (`test(transport): keep HTTPS EOF assertion contract-focused`).
The worktree began at `27ee73b4bc77cb9f6e8ad6cb6700a8269536a965` with the
raw-TLS adapter and shared resolver/worker/deadline modules present, but no
HTTPS adapter source or HTTPS test target. No production source was changed.

Baseline command:

```text
cargo test -p kmipkit-transport --test raw_tls --offline
```

Result: 81 passed, 0 failed, 0 ignored.

## Request test matrix

The registered source-included target is
`crates/kmipkit-transport/tests/https.rs`. It defines a bounded local TLS 1.3
peer requiring a verified client certificate and captures the actual HTTP
request. The adapter test seam injects one deterministic resolver result and
observes KMIPKit-owned staged request cleanup.

| Requirement | Test | Intended assertion |
| --- | --- | --- |
| HTTPS scheme only | `https_configuration_rejects_the_plain_http_scheme` | A plain HTTP endpoint is rejected by validated config before adapter construction |
| Origin-form only; no separate authority | `https_configuration_rejects_absolute_or_different_authority_targets` | Absolute-form and network-path targets with a different authority are rejected |
| Default target and request bytes | `https_default_target_posts_exact_bytes_with_required_headers_and_zeroizes_on_success` | `/kmip`, HTTP/1.1 POST, exact caller bytes, one required value for Host/Content-Type/Content-Length/Cache-Control, verified mTLS, response body unchanged, staged owner zeroized |
| Configured origin target and post-dispatch failure cleanup | `https_configured_target_preserves_ip_authority_and_zeroizes_after_peer_close` | Origin-form `/custom/kmip?version=1`, endpoint IP authority including port remains Host despite TLS-name override, exact caller bytes, `PossiblySent` on peer close, staged owner zeroized |
| Bracketed IPv6 and explicit authority port | `https_request_host_serializes_bracketed_ipv6_authority_and_explicit_port` | Exactly one Host value `[2001:db8::7]:8443` while the request target remains `/kmip` |

The tests use fixed assertion messages and comparisons that do not format the
request sentinel. The peer accept, TLS operations, and request capture are
bounded. Its body parser caps captured request storage at 1 MiB.

The QA correction removed an assertion on `TransportCauseCategory::Other`
from the peer-close case. The contract requires an error with
`RequestDeliveryState::PossiblySent`; it does not prescribe whether incomplete
HTTP/TLS EOF is categorized as `Io`, `Http`, or `Other`. The correction keeps
the delivery-state, captured-request, and zeroization assertions unchanged.

## Red verification

Focused command:

```text
cargo test -p kmipkit-transport --test https --offline
```

Expected Red result: compilation stops at the source-included production
module declaration because `crates/kmipkit-transport/src/https.rs` does not
exist yet. Exact diagnostic:

```text
error: couldn't find file `crates\kmipkit-transport\tests\..\src\https.rs`
  --> crates\kmipkit-transport\tests\https.rs:24:1
24 | mod https;
   | ^^^^^^^^^^
```

This demonstrates that the requested production HTTPS behavior is not yet
implemented. The individual request assertions cannot execute until T035 adds
the adapter; no behavior is claimed as tested at this Red checkpoint.

Formatting and patch checks:

| Command | Result |
| --- | --- |
| `rustfmt --edition 2024 --check --config skip_children=true crates/kmipkit-transport/tests/https.rs` | Passed |
| `git diff --check` | Passed |
| `cargo test -p kmipkit-transport --test https --offline` after `2669cca` | Expected Red; the compiler reports the missing source-included `src/https.rs` module as above |

The direct rustfmt command skips child-module loading because the expected
HTTPS source module is intentionally absent in Red. A normal rustfmt invocation
also stops while resolving that missing `#[path]` child. T035 must compile and
run the entire target before this request matrix can be considered green.
