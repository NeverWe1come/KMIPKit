# T041 Green Report — KMIPKIT-0013

## Scope

Make the HTTPS-only HTTP/1 application-protocol constraint explicit while
keeping direct endpoint routing and the existing no-middleware behavior.

## Implementation

`HttpsTransport` now builds and retains a dedicated clone of its validated
rustls client configuration with `alpn_protocols` cleared. This ensures the
HTTP/1-only Hyper driver cannot negotiate HTTP/2, even if the underlying TLS
configuration later gains other ALPN entries. The raw TLS adapter continues
to use its original TLS configuration. HTTPS rustdoc now records that the
transport does not follow redirects or server-supplied endpoints, consult
proxy environment settings, retain cookies, negotiate compression or HTTP/2,
or retry exchanges. These policies follow from the low-level direct socket
and HTTP/1 APIs; no redirect, proxy, cookie, compression, or retry middleware
is introduced.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test https --offline -- --test-threads=1` | Passed: 77/77 |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

The T040 tests passed both before and after this change. The Green commit makes
the ALPN guarantee explicit and documents the policies already enforced by the
low-level implementation; it does not claim a previously failing behavioral
test was repaired.
