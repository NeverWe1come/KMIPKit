# T035 Green Report — KMIPKIT-0013

## Scope and commits

T035 implements the HTTPS/HTTP 1.1 request path and concrete
`exchange_with_options` entry point after T034 added the request-capture
contract. The source Green commit is
`366ae182490f676d9a9f12e7a64fb7579b8ee332`
(`feat(transport): implement HTTPS adapter (T035 Green)`). Task/report and
ledger evidence are recorded separately from the source change.

T034 Red evidence remains in commits `d4206ec`, `1c6c8ab`, and its QA
correction `2669cca`; its report is
`.superpowers/sdd/013-production-transport/task-34-report.md`. The Red target
initially stopped at the intentionally absent `src/https.rs`, so it did not
execute behavioral assertions before Green.

## Design

`HttpsTransport::new` accepts the validated `TransportConfig` and constructs
the existing rustls policy without starting DNS or opening a socket. The
endpoint connection helper keeps the endpoint-derived host, effective HTTPS
port, and exact authority together; the origin-form target cannot replace
the TLS server name or `Host` authority. The concrete adapter is re-exported
from `kmipkit-transport` and implements both `Transport::exchange` and
`exchange_with_options`.

Each call captures one absolute total deadline before lazy worker startup,
validates finite phase deadlines, and uses the existing worker, bounded
resolver, sequential TCP/TLS candidate handshake, and `DeadlineIo` path.
The KMIP request is staged in the existing zeroizing `SecretBuffer` owner,
then handed to Hyper as a single HTTP/1.1 body frame using
`Bytes::from_owner`. The adapter dispatches only after verified TLS and
Hyper sender readiness. It sends a direct POST with the configured origin
target, endpoint-derived `Host`, `application/octet-stream`, exact
`Content-Length`, and `Cache-Control: no-cache`. Each exchange closes its
connection after completion; it does not retry.

The response body is streamed through a bounded buffer which checks the
configured cap before growth and zeroizes initialized bytes on error/drop.
Strict HTTP status and response-header validation, parser boundary cases,
and reused-connection unsolicited-response tests belong to T037/T038 and
are not claimed as implemented by T035.

## Verification

Focused command:

```text
cargo test -p kmipkit-transport --test https --offline
```

Result: 56 passed, 0 failed, 0 ignored. The run includes the T034 request
capture, HTTPS-only target validation, exact body/header checks, TLS 1.3
mTLS peer, configured-target behavior, and zeroization observer assertions.

Full package command:

```text
cargo test -p kmipkit-transport --all-targets --all-features --offline
```

Result: 324 passed, 0 failed, 0 ignored across 13 targets, including
`secret_redaction` (22/22), `secret_redaction_current` (9/9), HTTPS (56/56),
and raw TLS (81/81).

| Command | Result |
| --- | --- |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

All commands ran offline on the Windows host. The T021 symlink test used its
documented fallback because temporary symlink creation was unavailable on
this host; Windows symlink-following remains a platform verification gap.

## Remaining scope

T035 establishes the direct HTTPS HTTP/1 request path. It does not complete
the response validation, parser hardening, or connection reuse test matrix
assigned to T037–T039. Independent QA found a P2 cancellation gap: the local
`JoinHandle` was aborted only after `exchange_on_worker` returned normally, so
worker cancellation could drop the handle and detach the Hyper driver. The
test-first correction is tracked as T035a in
`.superpowers/sdd/013-production-transport/task-35a-report.md`. Its Green
guard is committed; T037 remains paused until independent read-only
re-review.
