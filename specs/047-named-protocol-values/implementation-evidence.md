# KMIPKIT-0047 Implementation Evidence

## TDD commits

- Red: `74231e4f6` adds the catalog/generator contract tests.
- Additional Red refinements: `cb10b7965`, `1b43ce373`, `094848e9a`,
  `a6767e9ea`, `f49ee6a46`, `16a8e028c`, `607b922c9`, and `e3186eaaa` cover
  Java unsigned KMIP values, C public-header auditing, Rust lint safety, empty
  Enumeration groups, normalized-name collisions, and documentation readiness.
- Full-workspace Red: `61fa86fb8` updates the capability-query fixture after
  the workspace suite exposed that the newly mapped `named_protocol_values`
  capability was absent from its fixed expected list.
- Green follow-up: `b676413fd` adds the capability to the Rust facade's local
  availability query, which now includes that
  mapped capability; the subsequent full workspace test run passes.
- Green: `8e3907a04` implements generation and the four language surfaces.
- Refactor: `b50e199c5` centralizes source provenance without changing output.
- Coverage exclusion contract and rationale: `11fc263f7`, `fe55c99d7`.
- Coverage-policy correction Red: `9bf16b9e4` changed boundary tests to reject
  90% TTLV/protocol and changed-code coverage; the old gate failed those tests
  by accepting 90%.
- Coverage-policy correction Green: `7b295798f` raised those gate thresholds
  to the repository contract's 95% and synchronized the current policy docs.
- Coverage-policy correction Refactor: `a29220c54` centralized the threshold
  constants without changing behavior.
- Generated-output CI check Red: `6d07a12b3` added a workflow contract test;
  the focused test failed because the normative-inventory job had no named-
  value generator check.
- Generated-output CI check Green: `d99e95d7a` added the generator's `--check`
  step. The workflow contract test and `generate_named_values.py --check`
  passed.
- Generated C coverage-scope Red: `4ba33ad33` asserted that the exact C helper
  header is an explicit generated-code exclusion; the focused test failed
  because that path was not recorded.
- Generated C coverage-scope Green: `15578e9fe` recorded the exact exclusion
  and corrected the testing guide: the header has `static inline` lookup
  bodies, is not in the aggregate line-coverage inputs, and is exercised by
  the C consumer test.
- Generated C coverage-scope Refactor: `80785a3ee` made the coverage test's
  four generated paths explicit; the focused test passed.
- C literal escaping Red: `07ee17296` added a test for quotes, backslashes,
  newlines, tabs, and UTF-8 bytes. It failed because the generator emitted
  control characters directly into a C string literal.
- C literal escaping Green: `a684b53ae` changed generated C string values to
  fixed-width octal UTF-8 byte escapes; the focused regression test passed.
- C literal escaping Refactor: `7fee54ea4` expanded the escaping loop for
  direct review without changing its output; the focused test and generator
  check passed.
- C renderer call-site Red: `779ef92a2` added a fixture asserting that
  Enumeration names, Bit Mask names, and mask-list names all use the safe
  encoder. It failed because only one of the three outputs was safe.
- C renderer call-site Green: `e03b04c11` routed the Enumeration and Bit Mask
  name helpers through the shared encoder and regenerated the C header; all
  three output assertions passed.
- C renderer call-site Refactor: `5a8b69dbb` centralized the case-line renderer
  used by both KMIP value kinds without changing generated output; the full
  generator test file and `--check` passed.

## Local verification

The following checks passed on the feature worktree:

- `cargo fmt --all --check`
- `cargo test --offline --package kmipkit --test named_values_public_api`
- `cargo clippy --offline --package kmipkit-protocol --package kmipkit --all-targets --all-features -- -D warnings`
- `cargo test --offline --workspace --all-features` (full workspace; passed
  after updating the C capability expectation and facade query).
- `cargo llvm-cov --offline --workspace --all-features --json --output-path
  <temporary-report>` completed on Windows, followed by
  `python -B scripts/coverage_gate.py normalize --workspace . --input
  <temporary-report> --output <temporary-normalized-report>`. The raw Rust
  workspace result was 47,714/51,744 lines (92.21%). After applying only the
  documented exclusions for the generated `profiles_generated.rs` and
  `named_values_generated.rs`, it was 47,680/50,683 (94.07%). Per-crate
  excluded-line results were: `kmipkit-ttlv` 748/761 (98.29%),
  `kmipkit-protocol` 21,517/22,492 (95.67%), `kmipkit-transport` 3,790/4,056
  (93.44%), `kmipkit-ffi` 13,646/15,047 (90.69%), `kmipkit-client`
  6,176/6,429 (96.06%), `kmipkit-test-support` 1,758/1,853 (94.87%), and
  `kmipkit` 45/45 (100%). The 90% workspace floor, 95% TTLV/protocol floors,
  and 85% transport/FFI floors pass for this Windows Rust report. There were
  no changed executable production lines to score: the production additions
  are generated value declarations (covered by consumer tests and the exact
  generated-file exclusions) plus public re-exports. This local result does
  not represent the Linux/macOS or C/Java/Python/JNI reports.
- Rust public facade consumer test for generated values and future raw values.
- MSVC C11 consumer compile with `/W4 /WX` and execution of
  `bindings/c/tests/named_values.c`.
- Java focused Gradle test:
  `bindings/java/gradlew.bat --offline --console=plain test --tests org.kmipkit.protocol.NamedProtocolValuesTest`.
- Python focused binding test:
  `.venv-kmipkit-0047/Scripts/python.exe -m pytest -q bindings/python/tests/test_protocol_values.py`.
- `python -B -m unittest tools.normative_catalog.tests.test_generate_named_values tools.api_manifest.tests.test_t030_package_docs tools.api_manifest.tests.test_t073_doc_reconciliation -v` (26 tests).
- Focused coverage exclusion tests in `scripts/tests/test_coverage_gate.py` and
  `scripts/tests/test_multilanguage_coverage.py` (2 tests).
- Full coverage gate unit suites:
  `PYTHONPATH=scripts/tests python -B -m unittest test_coverage_gate test_multilanguage_coverage -v`
  (94 tests passed, 3 skipped because Windows disallows the symlinks those
  cases require).
- Post-correction coverage/workflow suites:
  `PYTHONPATH=scripts/tests python -B -m unittest test_coverage_gate test_multilanguage_coverage test_workflow -q`
  (190 tests passed, 3 skipped because Windows disallows directory symlinks).
  This run verifies that 90% fails, exactly 95% passes for TTLV/protocol and
  changed production code in both aggregate and scoped evaluation paths, and
  the 85% transport/FFI/binding floors and 90% workspace floor remain intact.
- `PYTHONPATH=scripts/tests python -B -m unittest test_workflow -q` (92 tests
  passed after adding the named-value generator check to the normative job).
- `PYTHONPATH=scripts/tests python -B -m unittest test_coverage_gate.CoverageGateTests.test_catalog_generated_named_value_sources_are_excluded_from_raw_coverage -v` (passed).
- `python -B -m unittest tools.normative_catalog.tests.test_generate_named_values tools.api_manifest.tests.test_binding_exports -q` (61 tests passed after the renderer regression tests).
- `python -B -m unittest tools.normative_catalog.tests.test_generate_named_values -q` (8 tests passed, including C string and renderer escaping regressions).
- `python -B tools/api_manifest/audit_binding_exports.py --c-only --repo-root .` (1,701 exact C declarations matched, no issues).
- `python -B tools/normative_catalog/generate_named_values.py --check` and
  `python -B tools/api_manifest/generate.py --check` (both passed).
- Final follow-up verification after the CI and generated-C coverage corrections:
  `PYTHONPATH=scripts/tests python -B -m unittest test_coverage_gate test_multilanguage_coverage test_workflow -q`
  (191 tests passed, 3 skipped because Windows disallows required directory
  symlinks); `python -B -m unittest tools.normative_catalog.tests.test_generate_named_values tools.api_manifest.tests.test_binding_exports -q` (61 passed); both
  `generate_named_values.py --check` and `api_manifest/generate.py --check`
  passed; `git diff --check` passed.
  The C consumer compiled with WSL GCC using `-std=c11 -Wall -Wextra -Werror`
  and executed successfully against the regenerated header. The C export
  audit matched 1,701/1,701 declarations with no issues.
- Linux verification in WSL `Ubuntu-26.04`: `cargo fmt --all --check`,
  `cargo clippy --offline --workspace --all-targets --all-features -- -D warnings`,
  and `cargo test --offline --workspace --all-features` all passed.
  The full workspace test run included the trybuild compile-fail suites and
  doc tests. `PYTHONPATH=scripts/tests PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest test_coverage_gate test_multilanguage_coverage test_workflow -q` passed all 191 tests without skips. The named-value
  generator check also passed in WSL after setting `GIT_DIR`, `GIT_COMMON_DIR`,
  and `GIT_WORK_TREE` explicitly to bridge the Windows-format linked-worktree
  `.git` pointer.
  The Linux C11 named-value consumer compiled and ran with GCC 15.2.
- `python -B tools/normative_catalog/reconcile_named_value_manifest.py --check --repo-root .`.
- `python -B tools/normative_catalog/generate_named_values.py --check --repo-root .`.
- `python -B tools/api_manifest/audit_binding_exports.py --c-only --repo-root .`:
  1,701/1,701 exact C declarations matched, with no missing, stale, or kind
  mismatches.
- `python -B tools/api_manifest/generate.py --check --repo-root .`.
- `git diff --check`.

The generated value surfaces contain 644 assigned Enumeration values and 33
assigned mask bits. The catalog's 723 Enumeration rows and 44 mask rows include
reserved and wildcard placeholders, which intentionally do not become usable
constants. The API manifest remains the reviewed input; generated outputs are
not edited manually.

The coverage gate records the exact generated named-value paths in Rust, C,
Java, and Python, with the rationale and consumer tests recorded in
`docs/development/testing.md`; the C header contains inline lookup code and is
therefore called out explicitly even though C headers are outside the raw
aggregate coverage inputs. Coverage thresholds are not weakened. The local
Windows LLVM run and the Linux workspace tests are evidence only and do not
replace the aggregate platform and adapter coverage gate. No aggregate
coverage artifact combining all required platform and language reports has
been produced; macOS and the hosted platform controls are unavailable in this
local Windows/WSL environment. Remote CI was not queried under the maintainer's
instruction to ignore it during the GitHub billing issue. The Linux API
manifest `--check` attempt was stopped after nearly five minutes resolving
files on the mounted Windows worktree; the same check passed on Windows. The
Linux named-value generator check, Rust tests/lint/format, and C consumer did
pass. Full adapter aggregate coverage and macOS/hosted reports remain pending,
so T007 stays open.

The earlier security scan for the 0047 range ending at `fe24d0dc55542745f769128825fad6f7fa984018` reported one low-likelihood generated-C string-injection finding (scan `ca8666d0-2c52-40b2-8ca8-1bcfb600284a`). The follow-up commits above encode all generated C names as fixed-width octal UTF-8 bytes and route every lookup form through the shared encoder. Follow-up scan `5ad47d76-3e64-48ee-9ad5-818c4a346567` reviewed the exact fix and CI/coverage changes, completed with no findings, and verified the previous candidate is fixed in the current head. This scoped review is not the independent human security review required before release.

## Integration status

The feature branch is intended to target
`feature/KMIPKIT-0043-t076-shared-client-mapping` as a draft stacked PR. Draft
PR [#114](https://github.com/NeverWe1come/KMIPKit/pull/114) is open against
that branch. The remote branch push succeeded after replacing private commit
email addresses with the account's GitHub noreply address; repository content
is unchanged. Hosted status checks were not queried under the maintainer's
instruction to ignore remote CI during the GitHub billing issue. T007 remains
open for the cross-platform aggregate coverage and hosted platform controls.

## Release synchronization update (2026-11-07)

The #104 squash merge is present in `release/1.0.0`. The feature branch was
synchronized with release commit `e008ef224d49108e9899fd2c9a98a67545c78469`.
The merge preserved the later release features; generated API outputs were
recreated from the release manifest plus KMIPKIT-0047 inputs. The reconciler
now retains the existing capability and declaration order and appends new
declarations deterministically. This avoids reordering the release inventory;
`test_manifest_reconciliation_preserves_existing_declaration_order` covers the
regression.

Local verification on this synchronized tree:

- `cargo fmt --all --check` and
  `cargo clippy --offline -p kmipkit-protocol --all-targets --all-features -- -D warnings` passed.
- `cargo test --offline -p kmipkit --test named_values_public_api` passed (1 test).
- The focused named-value, T073 documentation, and T102 manifest checks passed
  (21 tests); the focused coverage gate suites passed (99 tests, 3 skipped for
  unavailable Windows symlink privileges).
- The C11 named-value consumer compiled with MSVC `/W4 /WX` and ran
  successfully. The C ABI export audit passed (1 test).
- The Java `NamedProtocolValuesTest` Gradle task passed; Python's
  `test_protocol_values.py` passed (1 test) after building the editable package.
- `reconcile_named_value_manifest.py --check`, `generate_named_values.py
  --check`, and `api_manifest/generate.py --check` passed; the latter verified
  all 12 generated API files.

The full `test_generate` module was stopped after a single generated-manifest
case continued for several minutes; the named-header generation and export
audit cases were run separately and passed. No hosted CI status was queried.
T007 remains open for the macOS and hosted aggregate coverage evidence, so the
draft PR remains pending that release control.

The release-sync export reconciliation then exposed three API inventory issues:
Java bitmask helper names used Rust/Python snake_case instead of the generated
Java camelCase, bitmask `name` helpers were declared in languages that do not
provide them, and several exact Java/Python binding details were stale. The
reconciler now records single-bit mask name lookup as a C-only capability,
removes stale Java/Python routes, and matches the two `Client.executeWithOptions`
overloads and exact JNI exclusions. Focused regression tests cover these
contracts. The current binding report was regenerated and the C, Java, and
Python inventories all pass with zero missing, stale, kind-mismatched, or
unclassified exports (1,708 C, 1,881 Java, and 2,000 Python declarations).
`generate.py --check` and `generate_named_values.py --check` also pass. This is
local Windows evidence; no hosted CI status was queried.

The named-value exports are reachable through both the `kmipkit` facade and
the public `kmipkit-protocol` crate, so the manifest records both audited Rust
package paths. The refreshed Rust report at
`specs/043-public-api-bindings/reports/rust-export-report-2026-11-07.json`
matches all 5,999 manifest rows against 19,149 observed paths and 13,150 exact
exclusions, with zero missing, stale, kind, issue, or conditional mismatches.
The Rust crawler's Windows all-features `--check` exits 0. The shared-Rust
inventory report was regenerated at
`specs/043-public-api-bindings/reports/shared-rust-inventory-2026-11-07.json`;
it accounts for 3,697 mapped declarations, 13,150 approved exclusions, and no
unreconciled declarations. The C-header manifest tests now expand the generated
named-value include, and the shared Rust checks allow the exact C-only metadata
accessor mapping. The relevant manifest, shared-inventory, binding-audit, and
named-value suites pass (42 focused tests total). `cargo fmt --all --check`,
workspace Clippy, and `cargo test --offline --workspace --all-features` pass;
the workspace run completed with 1,262 client tests passing plus the remaining
crate tests and doctests. The full `test_manifest_v2` module was not completed
because its repeated large-manifest validations take several minutes; its
affected C-header, metadata, Rust inventory, and mapping tests passed in focused
runs. Remote CI was not queried.
