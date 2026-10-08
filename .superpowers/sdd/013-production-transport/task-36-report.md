# T036 Refactor Report — KMIPKIT-0013

## Scope and commits

T036 refactors HTTPS request construction without changing its behavior.
The source-only Refactor commit is
`ef118da3e790489a49059418566b6502ba4faec0`
(`refactor(transport): bind HTTPS route and request owner (T036)`). Evidence
and task-ledger updates are committed separately.

The prior T034 Red and T035 Green evidence is recorded in
`.superpowers/sdd/013-production-transport/task-34-report.md` and
`.superpowers/sdd/013-production-transport/task-35-report.md`.

## Refactor

Before the refactor, the validated endpoint host, port, Host authority, and
request target were copied as separate values through the worker call and
passed independently to the request builder. `HttpsRoute` now keeps those
values together from the validated configuration. This makes the origin-form
target and endpoint authority travel as one route; the target cannot supply
the resolver host or HTTP `Host`. TLS server-name selection remains owned by
the validated TLS configuration.

`build_http_request` now accepts that route and the one staged
`SecretBuffer`. `RequestBody::from_owner` constructs one
`Bytes::from_owner(RequestBodyOwner(...))` body and derives both the HTTP
body size hint and `Content-Length` from the same initialized owner. This
removes the separate length/body arguments that could otherwise diverge.
No public API or network behavior changed.

This is a Refactor after the T034 contract was already green, so no synthetic
behavior-failing Red test was introduced. The pre-refactor T034 run is the
baseline and the same contract tests were rerun after the source change.

## Verification

Baseline before editing:

```text
cargo test -p kmipkit-transport --test https --offline
```

Result: 56 passed, 0 failed, 0 ignored.

After the refactor, the same focused command passed 56/56. The full package
command also passed:

```text
cargo test -p kmipkit-transport --all-targets --all-features --offline
```

Result: 324 passed, 0 failed, 0 ignored across 13 targets.

| Command | Result |
| --- | --- |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

All commands ran offline on the Windows host. The T021 symlink fixture used
its fallback because temporary symlink creation was unavailable on this
host; Windows symlink-following remains a platform verification gap.

Independent read-only review of the T036 diff is pending.
