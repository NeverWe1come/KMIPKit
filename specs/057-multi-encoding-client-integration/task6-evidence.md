# KMIPKIT-0057 Task 6 verification evidence

Date: 2026-11-06

## Red, Green, Refactor evidence

- **RED** — `b6c797bce` adds regressions for declared C record-layout fields and the C wire-encoding selector declaration kind. Both focused tests failed against the prior auditor and manifest.
- **GREEN** — `e8ebf8d44` derives C record field declarations from public `types[].c.fields`, marks the three C wire selectors as `macro`, and regenerates the checked-in API parity outputs. The focused tests and binary C ABI audit then passed.
- **REFACTOR** — `ad7746319` extracts the C record-layout inventory into `_declared_c_record_fields`; the focused test stayed green.
- **Follow-up audit baselines** — `109a18a3c` updates exact shared-Rust inventory expectations for the 0057 public surface and teaches the manifest audit test to include the C wire selector macros. The complete API manifest suite passes with these expectations.
- **Hosted C compile regression and fix** — The first hosted run (`42140440923`) exposed duplicate test-local C record declarations and prototypes that conflicted with the generated internal ABI header. Commit `2370bf1e4` removes those duplicate declarations and uses the generated record typedefs. Strict GCC syntax compilation (`cc -std=c11 -Wall -Wextra -Werror -fsyntax-only`) passed locally. The complete follow-up run (`42141649469`) passed its C consumers, sanitizer, Core, platform coverage, aggregate coverage, script-contract, binding, traceability, security, and SonarCloud checks.

## Local verification

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Passed. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed; AWS-LC emitted non-fatal platform/NASM diagnostics on this Windows host. |
| `CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test --workspace --all-features --locked -j 2` | Passed. The three Cosmian-dependent credential-session cases remain intentionally ignored unless their external service is available. |
| `CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo llvm-cov --workspace --all-features --locked --json --output-path target/coverage-raw.json -j 2` | Passed; coverage-gate preflight and normalization passed, and the report was eligible. Local Rust-only line coverage: TTLV 99.19%, protocol 96.21%, client 97.65%, transport 95.34%, FFI 95.47%, Rust workspace 96.39%. These are not the multi-platform/multi-binding aggregate gate. |
| `scripts/coverage_gate.py aggregate` | Not passable locally: required Ubuntu, Windows, macOS, C, Java, Python and JNI artifacts are collected by hosted CI. |
| `bindings/python/.venv/Scripts/python.exe -m pytest -q bindings/python/tests` | 162 passed. |
| `mvn -q -f bindings/java/pom.xml test` | Passed, including native JNI build and tests. |
| `bindings/python/.venv/Scripts/python.exe -B -m unittest discover -s tools/api_manifest/tests` | 269 passed, 3 skipped. |
| `python -B -m unittest tools.normative_catalog.tests.test_generate_wire_names` | 22 passed. Generated `WIRE_NAMES` contains 3 mask families and exactly the 33 catalog components with `allocation == assigned`; reserved and extension components are not emitted symbolically. |
| `python -B -m tools.api_manifest.generate --repo-root . --check` | Passed; all 12 generated API outputs are current. |
| `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --c-only --c-binary target/debug/kmipkit_ffi.dll --check` | Passed: 280 observed C declarations, 259 exact manifest matches, 21 approved exclusions, no missing declarations, stale rows, kind mismatches, or scanner issues. |
| `python -B -m unittest scripts.tests.test_requirement_traceability` and `python -B -m unittest discover -s tools/normative_catalog/tests -p test_feature_traceability.py -v` | Passed (2 and 30 tests); one Windows symlink privilege test is skipped. |
| `scripts/Test-DependencyPolicy.ps1` | Passed; lockfiles unchanged and advisory snapshots current at the recorded repository snapshot. |
| OASIS immutable-source audit (`tools/normative_catalog/audit_sources.py`, base `f46de941c28247beef294e545d49c948fe3836c8`) | Passed; audited 1,411 normative source candidates. |
| Fuzz smoke, Ubuntu WSL: `cargo +nightly fuzz run extension_schema -- -runs=1000 -max_len=4096 -timeout=5` | Passed; libFuzzer completed 428,647 executions with no crash, timeout, or OOM. |

## External gates still open

- Hosted CI and SonarCloud for draft PR [#119](https://github.com/NeverWe1come/KMIPKit/pull/119) passed on the follow-up run `42141649469`, including Core on Ubuntu, macOS, and Windows with stable Rust and Rust 1.94, all platform coverage jobs, the selected-scope aggregate coverage gate, and script contracts on all three operating systems.
- The aggregate coverage gate passed on that run; the local Rust-only percentages above are still not a substitute for its hosted multi-platform and multi-binding reports.
- Two-independent-implementation KMIP interoperability has not been run. Docker Desktop was unavailable on this host (`docker ps` could not connect to the Linux engine); synthetic peers and OASIS codec vectors are not substitutes for that release gate.
- The broader Rust export inventory still contains 907 unmatched paths already present on the 0054 base. This PR does not claim those base findings are resolved; their disposition remains a release-level review item.

The OASIS fixture replay boundary and the non-claims around profile support remain documented in [traceability.md](traceability.md). The hosted gates are complete; Task 6 remains open until interoperability against two independent implementations is run.
