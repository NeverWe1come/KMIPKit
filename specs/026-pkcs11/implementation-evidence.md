# KMIPKIT-0040 implementation evidence

Status: verified Rust increment; **specification incomplete** while T019/T022
remain open. Scope: OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275 and
nested §§7.26–7.30, Tables 378–382. All fixtures are source-derived.

## Branch and source baseline

The dedicated worktree uses `feature/KMIPKIT-0040-pkcs11`. The existing planning
branch was merged from current `origin/release/1.0.0` at
`a0846037eb9930b24d9b9e3d8c63b2e66b438d1a` without conflicts, force push, or
history rewrite. Pinned OASIS sources were checked against that exact base.
Baseline catalog: 4 sources, 1,411 clauses, 4,044 records; validator passed.
All required operation, enum, nested structure/member and tag records exist.

## Distinct DCO evidence commits

All commits carry `Signed-off-by: NeverWe1come <vgarciaaguilar97@gmail.com>`.

| Phase | Commit | Command / observed evidence |
|---|---|---|
| Planned verification | `a4e8f325` | Exact named table/source verification targets committed before behavior |
| Protocol Red | `769c3fb9` | `cargo test -p kmipkit-protocol pkcs11_operation_tests`: expected E0432 for missing Pkcs11 request/response/error types; no production implementation |
| Protocol fixture correction | `8de8a7bd` | Pending fixture supplies required shared asynchronous correlation; absence of Success Response Payload belongs to shared message validation |
| Protocol Green | `a4112d89` | Same focused command: 9 passed |
| Protocol Refactor | `fe5ad8d5` | Opaque callback accessor reuse; 9 tests remain passing; rustfmt |
| Client Red | `9100cf8e` | `cargo test -p kmipkit-client --lib pkcs11_execution_tests`: expected E0599 for absent variants, typed accessor and explicit client methods |
| Client Green | `024158e0` | Same focused command: 7 passed |
| Protocol lint/boundary cleanup | `b9683fb7` | Malformed optional/return-code, duplicate/depth, sanitized source checks; protocol 12 passed; workspace Clippy passed |
| Client Refactor | `dae83cce` | Shared single-request executor and operation-specific response dispatcher; client 7 passed; format and workspace Clippy passed |

Red test registration hooks were included because this repository discovers
unit suites through external `#[path]` test modules. They add no production
behavior. Numeric separators, borrowed helper arguments and shared match arms
were cleaned up after the passing implementations; no lint gate was relaxed.

## Verified local commands

Runs are on Windows x86_64. Default stable Rust is 1.99.0; the explicit MSRV
check uses Rust 1.94.0. Detailed uncommitted tool logs are under `target/`.

| Command | Result |
|---|---|
| `cargo test -p kmipkit-protocol -p kmipkit-client pkcs11` | PASS: protocol 15; client 9; no failures |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo +1.94.0 check --workspace --all-targets --all-features --locked` | PASS |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked` | PASS |
| `python tools/normative_catalog/validate.py` | PASS: sources=4 clauses=1411 records=4044 |
| `python tools/normative_catalog/report.py --write` then `--check` | PASS; report regenerated only through pinned tool |
| `python tools/api_manifest/generate.py --check` | PASS: 6 current generated outputs; no 0043 mapping added |
| Catalog `generate_ttlv_tags.py`, `generate_result_values.py`, `generate_attribute_types.py`, `generate_attribute_policy.py`, each `--check` | PASS |
| `python tools/normative_catalog/check_immutable_sources.py --base-sha a0846037eb9930b24d9b9e3d8c63b2e66b438d1a` | PASS |
| `python -m unittest discover -s scripts/tests -p 'test_requirement_traceability.py' -v` | PASS: 2 tests |
| `python -m unittest discover -s scripts/tests -p 'test_diagnostic_redaction.py' -v` | PASS: 1 test |
| `python -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v` | PASS after QA fix: 15 tests, 1 fixture skip |
| `git diff --check` | PASS |
| `python -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` | PASS: 213 tests, 7 platform/fixture skips |
| `cargo test --workspace --all-features` | PASS after serial rerun: all tests and doctests passed; first concurrent build had been stopped before tests |
| `cargo llvm-cov --workspace --all-features --locked --json --output-path target/pkcs11-coverage-raw.json` | PASS; full instrumented tests completed and JSON exported |
| `python scripts/coverage_gate.py normalize --workspace . --input target/pkcs11-coverage-raw.json --output target/pkcs11-coverage.json` | PASS: LLVM schema 3.1.0 accepted |
| `pwsh -File scripts/Test-DependencyPolicy.ps1` | PASS: exact cargo-deny 0.20.2; root/fuzz checks, fresh online advisory evidence, negative fixtures and unchanged lockfile hashes |

Full Ubuntu/macOS/Windows plus required adapter coverage reports are needed for
the repository aggregate coverage gate. A Windows-only report is local
measurement and cannot establish the complete multi-platform gate.

## Traceability, security and limitations

Every Table 273/274 row links to stable catalog structures/tags and named
positive/relevant negative verification in `traceability.md`. All Table 275
reasons are exercised, including the three PKCS#11-specific reason values;
unknown Result Reason values remain lossless. No independent §6.1.37 client
normative clause exists, so no synthetic requirement is assigned.

Function and Return Code preserve required nested values and unknown members;
vendor Function Integer is preserved under the approved specification.
Interface and parameter structures remain opaque. Failure/Pending do not
fabricate Success fields. Entire generic response batch items are retained.
Pending asynchronous correlation is caller-enabled and forwarded without
retry/Poll. Invalid requests retain NotSent; partial writes PossiblySent;
malformed received responses ResponseStarted. No transport boundary is added.

Opaque Structure/Item values and SecretBytes use existing owned-memory
zeroization. Debug only shows redacted metadata; Function, Input, Output,
Correlation and generic response sentinel tests verify that values do not
escape through Debug, Display or sanitized errors. Caller-created external
copies are outside KMIPKit-owned memory. Binding/runtime-copy documentation
and C/Java/Python parity remain gated by KMIPKIT-0043 / T019.

No official PKCS#11 fixture is linked in the pinned catalog. No official
vector pass, profile support, certification, FIPS status or interoperability
claim is made. No live PKCS#11 server exchange was executed. The two-independent
server gate in T022 remains open and the existing matrix stays Pending.


## Independent QA correction evidence

- Red `d2f3362d`: two protocol cases failed because wrong/duplicate known
  nested Interface/Input/Output scalars were accepted; two client cases failed
  because an invalid request reached the transport (PossiblySent) and malformed
  nested Output Parameters were accepted as Success.
- Green `daf0ad6b`: known nested 0..1 Interface TextString and Input/Output
  ByteString members are validated. Protocol 15 and client 9 tests passed.
- Refactor `8549fcc7`: one shared required/optional cardinality validator;
  protocol 15 and client 9 remain passing. Workspace Clippy and explicit MSRV
  1.94.0 all-target/all-feature checks were rerun successfully after the fix.

Source interpretation `KMIPKIT-026-SOURCE-001` is recorded in `research.md` and
linked to existing vendor Integer request/response tests. It uses the explicit
§6.1.37 Tables 273/274 vendor context and preserves §7.26 Table 378's outer
Structure and standard Enumeration shape. The same-primary-document mismatch
and observable alternatives remain documented; no official erratum, profile
or complete conformance claim is made.


## Final serial local and real CI coverage evidence

The serial local test, coverage and dependency-policy commands all returned
exit 0. Native `pkcs11.rs` line coverage was 218/226 (96.46%). Windows-only LLVM
file-summary totals are not the project aggregate: native FFI was 22.73% and
native overall totals were 89.70%, because the Linux C-consumer collector and
other platform/adapter reports are not part of this local Rust-only run. No
missing platform or C-consumer artifact is fabricated from that report.

Actual CI run [38181159004](https://github.com/NeverWe1come/KMIPKit/actions/runs/38181159004)
completed **SUCCESS** for implementation head
`95644a0f1978259c9c30004c64eca8435e00d8d7`. All required core/MSRV, platform,
C-consumer, Java/Python/JNI, sanitizer, generated-output, normative inventory,
dependency-policy and final-summary jobs passed. The real selected-scope
coverage gate consumed its required CI artifacts and reported:

| Scope | Covered / measured lines | Coverage / gate |
|---|---:|---|
| Changed Rust | 244 / 252 | 96.83% / 95% |
| TTLV | 735 / 741 | 99.19% / 95% |
| Protocol | 11,997 / 12,377 | 96.93% / 95% |
| Transport | 3,713 / 3,902 | 95.16% / 85% |
| Rust FFI | 2,035 / 2,137 | 95.23% / 85% |
| Java | 618 / 670 | 92.24% / 85% |
| Python | 773 / 783 | 98.72% / 85% |
| JNI | 1,193 / 1,312 | 90.93% / 85%; 21 summary-only uncovered lines included |
| Workspace | 25,127 / 26,010 | 96.61% / 90% |

Local dependency policy verified fresh online RustSec root and fuzz evidence
at commit `7eebec69c352c7191b1f13eb95dd510eeca5d1de`, database commit timestamp
`2026-10-09T10:12:02+02:00`; the refresh succeeded for both workspace scans.
Existing exact exceptions were KMIPKIT-0011-EX-003 and KMIPKIT-0011-EX-004.
Both Cargo.lock hashes and resolved-package invariance checks passed.

Independent QA confirmed both review observations resolved after the nested
member correction and source-context trace. T019 remains blocked by missing
KMIPKIT-0043 on release; T022's remaining condition is real PKCS#11 execution
against two independent supported KMIP 2.1 implementations. The PR remains a
draft, and the whole specification is not declared complete.
