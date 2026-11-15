# T047 Green evidence

## Implementation

- Added `Client::new` using only validated `TransportConfig`, selecting the HTTPS or raw-TLS
  production adapter without resolving or opening a socket during construction.
- Retained the immutable extension `ClientConfiguration` independently from transport config.
- Passed `RequestOptions` through each current typed operation method and exposed options-bearing
  variants while keeping the default methods source-compatible.
- Kept arbitrary `Transport` injection test-only; updated the top-level Rust facade with the typed
  client surface and shared `RequestOptions`.
- Preserved early extension-registry provenance validation and sanitized `InvalidInput`/`NotSent`
  behavior. Same-configuration extension requests still reach the production exchange path.

## Evidence

- Green implementation commit: `82eac36` (`feat(client): connect typed client to production transports`).
- Red contracts: `6e2ea68`, `f89f1b0`, and test-name alignment `c18e542`.
- `cargo test -p kmipkit-client --all-targets --all-features --offline`: passed; 203 library tests,
  every integration target, the 9 production-client tests, and the trybuild injection contract passed.
- `cargo clippy -p kmipkit-client -p kmipkit --all-targets --all-features --offline -- -D warnings`:
  passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.

## Scope remaining

T048 performs a separate ownership refactor and re-runs the T046/T046a contracts. User-guide and
architecture documentation updates remain assigned to T052/T054.
