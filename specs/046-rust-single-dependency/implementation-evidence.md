# KMIPKIT-0046 Implementation Evidence

## Red

`cargo check --manifest-path tests/fixtures/rust-single-dependency/Cargo.toml --offline` failed as expected on the absent `kmipkit::protocol`, `kmipkit::ttlv`, `kmipkit::client_config`, and `kmipkit::transport_config` facade paths. The first run also identified a missing fixture `main`; that harness issue was corrected before recording this Red result. The request-type imports deliberately include all 56 currently defined request structs, eight of which are not yet re-exported by `kmipkit-protocol`.

## Green

Pending.

## Refactor

Pending.

## Local checks

Pending.

## Limitations

Hosted CI may remain unavailable because of the repository billing issue. The generic TTLV crate exposes a bounded decoder but no general-purpose encoder; encoding remains through typed client requests.
