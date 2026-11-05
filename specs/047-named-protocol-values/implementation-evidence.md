# KMIPKIT-0047 Implementation Evidence

## TDD commits

- Red: `74231e4f6` adds the catalog/generator contract tests.
- Additional Red refinements: `97ef086e5`, `c2d6956d4`, `1f412c2e3`,
  `fd775b3a2`, `1b547d604`, `c4f1b7545`, `1041708cd`, and `b89d76c06` cover
  Java unsigned KMIP values, C public-header auditing, Rust lint safety, empty
  Enumeration groups, and normalized-name collisions.
- Full-workspace Red: `1ee468edd` updates the capability-query fixture after
  the workspace suite exposed that the newly mapped `named_protocol_values`
  capability was absent from its fixed expected list.
- Green follow-up: `bee7b0295` adds the capability to the Rust facade's local
  availability query, which now includes that
  mapped capability; the subsequent full workspace test run passes.
- Green: `0ec3e4f0d` implements generation and the four language surfaces.
- Refactor: `892304402` centralizes source provenance without changing output.
- Coverage exclusion contract and rationale: `db34dec26`, `fc7e48126`.

## Local verification

The following checks passed on the feature worktree:

- `cargo fmt --all --check`
- `cargo test --offline --package kmipkit --test named_values_public_api`
- `cargo clippy --offline --package kmipkit-protocol --package kmipkit --all-targets --all-features -- -D warnings`
- `cargo test --offline --workspace --all-features` (full workspace; passed
  after updating the C capability expectation and facade query).
- `cargo llvm-cov --offline --workspace --all-features --summary-only` (passed
  on Windows; this single-platform raw workspace line summary is 89.65%, while
  the repository release gate aggregates the Linux, Windows, macOS, C, Java,
  Python, and JNI reports and is not represented by this local run).
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

The coverage gate excludes only the exact generated named-value files in Rust,
Java, and Python, with the rationale and consumer tests recorded in
`docs/development/testing.md`. Coverage thresholds are not weakened. Full
The local Windows LLVM run is evidence only and does not replace the aggregate
platform and adapter coverage gate. Remote platform CI and full adapter
aggregate coverage are still pending.

The Codex Security diff scan for base `7ea1fac2b8eb55e40a57d1da0c86fbf6d420b6f1`
and head `a782fa24027922bb5dff81f3f15343293f0b84de` completed with no findings
(scan `011da8a8-e23b-429b-8a52-10ebc0fd4641`). It reviewed the 18 changed files
and relevant project security guidance. This is a scoped diff review, not the
independent human security review required before release.

## Integration status

The feature branch is intended to target
`feature/KMIPKIT-0043-t076-shared-client-mapping` as a draft stacked PR. The
remote push, PR creation, and current PR checks will be recorded after those
actions complete.
