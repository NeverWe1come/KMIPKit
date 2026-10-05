# T016 Full Verification Report

Date: 2026-10-05
Branch: `feature/KMIPKIT-0005-ttlv-wire-codec`
Verification base: `release/1.0.0` at `d46e13dfdd83ac05e4b58d24af5b928e355e092d`
Local host: Windows, Rust `1.99.0`; supported-platform CI remains required.

## Changes made

- Moved the client writer and TTLV decoder unit-test sources from `src/` into
  `tests/support/`, with test-only file attributes. The production-source
  preflight now passes without weakening its scan of every Rust source file.
- Added decoder invariant tests for offset overflow, malformed spans, parent
  boundaries, bounded payload copying, checked numeric reads, error display,
  and both Boolean wire values.
- Fixed coverage path normalization for external paths that appear only in
  LLVM function filename tables because of macro expansion. Such entries are
  replaced by `__external_source__`; external `files` entries and workspace
  source paths remain strictly validated.
- Scope resolution: this is a narrow T016 correction to the required coverage
  verification path. A supported LLVM export included a proc-macro registry
  path in a function's filename table without a corresponding coverage file;
  rejecting that metadata made the repository coverage check fail before it
  could assess workspace sources. T016 now authorizes normalization of those
  external function-table entries only. It does not relax validation of
  coverage `files` records, source paths, or the exact changed-source scan and
  does not change KMIPKit's product API or protocol boundary.
- Updated the current verification tree and traceability map to the test
  support paths.

The normalizer change followed test-first evidence: the new regression test
failed on the external `static_assertions` source path before the fix and passed
after it. The Red, Green, and Refactor commits are `90543e5`, `8dfcb3a`, and
`620c8b8` respectively. The Refactor commit extracts the function-table path
logic into a focused helper without changing behavior; the existing regression
test and all script tests pass afterward.

## Verification results

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Pass |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Pass |
| `cargo test --workspace --all-features --locked` | Pass |
| `cargo doc --workspace --all-features --no-deps --locked` | Pass |
| Python script contracts | 49 passed, 3 platform-specific symlink tests skipped |
| Normative catalog unit tests | 162 passed, 7 platform-specific tests skipped |
| PowerShell WSL contracts | Pass |
| OASIS source immutability and source-candidate audit | Pass against the exact release base; 1,411 candidates audited |
| Normative catalog validation | Pass: 4 sources, 1,411 clauses, 4,021 records |
| Generated catalog report, TTLV tags, and result values | All `--check` commands pass |
| Coverage preflight and LLVM path normalization | Pass |
| `git diff --check` | Pass |

Windows line coverage from the normalized workspace export:

| Area | Line coverage |
|---|---:|
| Changed private client writer | 95.83% |
| Changed TTLV decoder | 96.90% |
| `kmipkit-ttlv` | 97.79% |
| `kmipkit-protocol` | 99.21% |
| `kmipkit-transport` | 100% |
| `kmipkit-ffi` | Not applicable; no executable FFI source lines were emitted |
| Rust workspace | 95.32% |

The local coverage thresholds pass. Linux, macOS, MSRV 1.94, and cross-platform
aggregation still require the pull-request workflow.

## Dependency and license scan

- `cargo audit` examined 36 locked crates and reported no advisories.
- `cargo deny check advisories`, bans, and sources passed. `cargo tree --locked
  --target all -d` found no duplicate dependency versions.
- The default `cargo deny check` license phase fails because this repository
  has no `deny.toml` allowlist; it rejects the project's Apache-2.0 packages
  and `zeroize`'s `Apache-2.0 OR MIT` expression. This is a missing repository
  policy configuration, not a newly introduced incompatible license. The
  dependency/advisory/license policy is designated for a separate
  infrastructure specification in `specs/001-cross-platform-ci/spec.md`; this
  feature does not invent or broaden that allowlist.

The bounded 1,000-iteration libFuzzer run and fixture checks remain recorded in
the T014 report. No decoder production behavior changed in T016.
