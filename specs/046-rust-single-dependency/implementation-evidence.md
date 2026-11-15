# KMIPKIT-0046 Implementation Evidence

## Red

`cargo check --manifest-path tests/fixtures/rust-single-dependency/Cargo.toml --offline` failed as expected on the absent `kmipkit::protocol`, `kmipkit::ttlv`, `kmipkit::client_config`, and `kmipkit::transport_config` facade paths. The first run also identified a missing fixture `main`; that harness issue was corrected before recording this Red result. The request-type imports deliberately include all 56 currently defined request structs, which are already exported by the lower-level protocol crate but were not reachable via the `kmipkit` facade.

## Green

`cargo check --manifest-path tests/fixtures/rust-single-dependency/Cargo.toml --offline` passed after adding the facade namespaces and configuration exports. The isolated consumer has exactly one direct KMIPKit dependency and compiles imports for all 56 protocol request structs, constructs the supported transport/client configuration from public types, type-checks typed execution/results, and decodes/builds generic TTLV values. No network connection is made.

## Refactor

Added `protocol`, `ttlv`, `client_config`, and selective `transport_config` facade namespaces while preserving existing root exports. The external fixture covers all 56 operation request types, representative operation responses, `ClientBatchResponse`, `PendingOutcome`, mTLS client construction, and generic TTLV decoding using one direct KMIPKit dependency.

Added `tools/api_manifest/reconcile_0046_aliases.py`, which fails closed on incomplete reports or unmapped paths and deterministically reconciles exact aliases, capability mappings, operation declaration references, and exact exclusions. The reviewed manifest records 1,318 additional declarations and 5,660 exact exclusions; the four namespace grouping paths refer to accepted ADR-0017. The helper is tested for determinism, order independence, exact mapping, and fail-closed behavior.

Regenerated the public API outputs with `tools/api_manifest/generate.py`. The checked-in manifest grew beyond the previous 23 MiB bound, so the generator now permits up to 40 MiB; the limit remains finite and has oversized-input tests. The legacy KMIPKIT-0012 projection now filters newer requirement IDs and preserves the original `ClientBatchItemOutcome` kind so the generated registry slice remains compatible. The isolated generator test repo relocates generated files used as evidence to avoid treating evidence inputs as preexisting outputs.

Updated the English and Spanish Rust package guides and added a CI matrix check that compiles the external fixture on the supported OS/toolchain matrix.

## Local checks

Passed:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --all-features --locked`
- `cargo check --locked --manifest-path tests/fixtures/rust-single-dependency/Cargo.toml --offline`
- `python scripts/test_user_guide_examples.py` (20 Rust examples compiled)
- `python tools/api_manifest/generate.py --check` (12 generated outputs verified)
- `python -B -m unittest tools.api_manifest.tests.test_manifest_v2 -q` (153 tests passed; 1,268.438 seconds)
- `python tools/api_manifest/audit_rust_exports.py --repo-root . --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json` (23,918 observed paths; 6,942 matched; 16,976 exact exclusions; zero missing, mismatched, or unobserved paths)
- Reconciler and manifest-limit unit tests (8 passed), the facade metadata projection, generated-fixture setup, malformed-input behavior, and legacy registry projection tests (4 passed)
- Duplicate C parameter validation passed; the symlink test was skipped because Windows denied symlink creation (WinError 1314)

The first full run of `test_generate.py` (86 tests) exposed stale assumptions in the legacy registry projection and generated-evidence fixture. Those issues were corrected and their affected tests passed when rerun. The complete API manifest v2 module initially exposed missing exact inventory fixtures, stale expectations that all 56 operations were verified, an outdated Rust export report, and C-header parsing that omitted included generated headers. These fixtures and projections were synchronized with the current manifest and official audit output; the complete module then passed all 153 tests. A final `tools/api_manifest/generate.py --check` confirmed all 12 generated outputs remain current. The symlink-specific test cannot exercise its target behavior on this Windows account.

## Limitations

Hosted CI may remain unavailable because of the repository billing issue. Coverage was not run, consistent with the temporary branch-specific coverage pause; cross-platform CI remains pending. The generic TTLV crate exposes a bounded decoder but no general-purpose generic TTLV encoder; outbound encoding remains through typed client requests.
