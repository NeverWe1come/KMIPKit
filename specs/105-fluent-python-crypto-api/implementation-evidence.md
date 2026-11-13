# KMIPKIT-0105 Implementation Evidence

## Branch and stack

- Branch: `feature/KMIPKIT-0105-python-fluent-crypto-api`.
- Base: `feature/KMIPKIT-0104-java-fluent-crypto-api` at
  `86c9eb2a68d882dd1672402391c39ba918ba9c1f` after rebasing on the Java CI
  fixes and updated manifest snapshots.
- Parent PR: #188, Java fluent cryptographic API.
- PR: #190 (`[WIP] KMIPKIT-0105 Python fluent cryptographic API`), draft,
  based on the KMIPKIT-0104 branch.

## Red

- Command: `/tmp/kmipkit-py105-venv/bin/python -m pytest -q tests/test_fluent_crypto.py`
  from `bindings/python`.
- Result: **expected failure** before production changes; 2 failed. The first
  reported missing `Request.encrypt`; the second reported missing
  `EncryptBuilder`. This confirms the requested consumer surface is absent.
- Commit: `e134c998b`.

## Red — full operation and transport contracts

- Syntax command: `/tmp/kmipkit-py105-venv/bin/python -m py_compile tests/test_fluent_crypto.py`.
- Result: passed.
- Test command: `/tmp/kmipkit-py105-venv/bin/python -m pytest -q tests/test_fluent_crypto.py`.
- Result: **expected failure** before production changes; 12 failed because
  all seven operation-specific request factories/builders and their typed
  setters are absent. The parametrized TLS tests stop at the absent API check,
  before opening sockets.
- The tests cover operation-only setter surfaces, local missing-input and
  closed-handle rejection, typed Cryptographic Parameters and TTLV Data,
  caller-buffer copying, request-header/options separation, one TLS 1.3 mTLS
  exchange per operation, encoded request fields, and typed KMIP failure
  metadata.
- Commit: recorded after this Red commit is created.

## Green and Refactor

### Green

- Added CFFI declarations for all seven existing operation builders, typed
  setters, batch conversion, execution and release.
- Added `Request.encrypt()`, `decrypt()`, `sign()`, `signature_verify()`,
  `mac()`, `mac_verify()` and `hash()` with operation-specific builder classes.
- The builders copy input into a bounded mutable staging buffer, wipe the
  binding-owned copy after the synchronous C call, and preserve Rust-owned
  typed request, batch and response lifetimes.
- `asynchronous_indicator()` now writes through the existing C ABI builder
  setter, so the Request Header carries the selected value on the wire.
- Added typed Hash parameters to the Hash exchange fixture and valid single
  or multipart indicator shapes to the per-operation TLS fixtures.
- Command: `/tmp/kmipkit-py105-venv/bin/python -m pytest -q tests/test_fluent_crypto.py`
  from `bindings/python`.
- Result: **12 passed**.
- Commands: `/tmp/kmipkit-py105-venv/bin/python -m py_compile
  bindings/python/src/kmipkit/fluent.py bindings/python/src/kmipkit/_client_ffi.py
  bindings/python/tests/test_fluent_crypto.py` and `git diff --check`.
- Results: passed.

### Refactor

Pending full binding checks and exact-head CI.

## Red — public API manifest

- Command: `python3 -m unittest
  tools.api_manifest.tests.test_0102_fluent_cryptographic_manifest
  tools.api_manifest.tests.test_0105_fluent_python_crypto_api_manifest`.
- Result: **expected failure**; the 0102 Rust and 0104 Java builder declarations
  are still blocked and have no four-language mappings, and no 0105 Python
  builder exports are declared. The suite recorded 578 failed subcases.
- The contract pins the complete Python public surface, the existing Rust and
  Java mapping targets, approved sources, and requirement linkage before
  completing the public manifest.

## Green — public API manifest

- Command: `python3 -m unittest
  tools.api_manifest.tests.test_0102_fluent_cryptographic_manifest
  tools.api_manifest.tests.test_0105_fluent_python_crypto_api_manifest`.
- Result: **4 tests passed**. The approved Rust and Java exports now map to
  declared C, Java, and Python surfaces; Python-specific ownership and context
  manager methods have their own KMIPKIT-0105 declarations.
- Command: `python3 tools/api_manifest/generate.py --write`.
- Result: generated all 12 public API outputs from the reviewed manifest.
- Command: `python3 tools/api_manifest/generate.py --check`.
- Result: all 12 generated outputs are current.
- Command: `python3 tools/api_manifest/audit_binding_exports.py`.
- Initial result: the default `/tmp` run failed in `ld` with a Bus error. The
  audit was retried with its temporary Cargo target on the worktree-backed
  disk and completed successfully.
- Successful result: C, Java, and Python inventories were complete with zero
  missing exports, kind mismatches, or unobserved manifest declarations.
- Command: `python3 -m unittest
  tools.api_manifest.tests.test_0102_fluent_cryptographic_manifest
  tools.api_manifest.tests.test_0105_fluent_python_crypto_api_manifest`.
- Result: **4 tests passed** after generation.

## Refactor

- Added bilingual Python usage and ownership guidance and corrected the
  language-parity note in both client-execution guides.
- Removed unused local async state from the Python crypto builder; the native
  C builder now remains the single owner of the Request Header indicator.
- Verified the existing `py.typed` package marker and retained inline public
  method annotations.
- Python focused tests: `tests/test_fluent_crypto.py` — **12 passed**.
- Manifest generation check and binding export audit: passed as recorded above.
- Full Python, Rust/FFI, C, Java/JNI, coverage, security, and exact-head CI
  checks remain pending.

## Rebased verification

- Built an isolated Python 3.12.13 environment and installed the editable
  binding with `uv pip install --no-build-isolation --editable
  bindings/python`.
- Focused crypto tests: `target/python312-venv/bin/python -m pytest -q
  bindings/python/tests/test_fluent_crypto.py` — **12 passed**.
- Full Python suite: `target/python312-venv/bin/python -m pytest -q
  bindings/python/tests` — **1,284 passed, 119 subtests passed**.
- Python coverage command: `target/python312-venv/bin/python -m pytest -q
  bindings/python/tests --cov=kmipkit --cov-config=bindings/python/pyproject.toml
  --cov-report=xml:target/coverage-python-0105.xml` — **96.62%**, above the
  85% gate; all 1,284 tests and 119 subtests passed.
- Rust FFI/C ABI: `cargo test -p kmipkit-ffi --all-features --locked` — 471
  unit tests, 55 compiled C consumer tests, and the operation parity test
  passed.
- Rust quality: `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.
- Traceability contracts: `python3 -B -m unittest
  scripts.tests.test_requirement_traceability -v` — 2 passed; feature
  traceability discovery — 31 passed.
- Java/JNI integration build and fluent API shape: `mvn --batch-mode --file
  bindings/java/pom.xml -Dtest=FluentCryptoApiShapeTest test` — 5 tests passed;
  JNI and Java compilation succeeded.
- Manifest generation: `python3 tools/api_manifest/generate.py --check` — all
  12 generated outputs current.
- Binding export audit: `python3 tools/api_manifest/audit_binding_exports.py
  --report target/python105-exports.json --check` — exit code 0 with no audit
  issues.
- Manifest contract: `python3 -m unittest
  tools.api_manifest.tests.test_0105_fluent_python_crypto_api_manifest` — 2
  tests passed.
- All 405 tests across the 108 `test_t102*.py` modules passed in four
  sequential groups after updating the total and manifest fingerprints for
  the KMIPKIT-0105 declarations.
- Rust FFI, C ABI, Java/JNI, full formatting/lint/security gates, coverage,
  and exact-head CI remain pending.

## Rebase on the corrected Java and C heads

- Final stack base: KMIPKIT-0104 Java head
  `fec2849cc77e123023e913a6301544c1a45eecb4`, based on KMIPKIT-0103 C head
  `861ce150fc0bed659ff603150e58d8db18bed1a9`.
- Rebased the Python feature commits onto the corrected Java branch and
  reconciled the public API manifest without discarding the C and Java expert
  attribute declarations. The manifest now records 16,348 declarations and
  exact fluent DX mapping inventories of 256 Rust, 57 C, 164 Java, and 187
  Python exports.
- Marked the Rust and Java crypto declaration mappings ready alongside the
  implemented Python facade, updated the capability inventory, and regenerated
  the 12 public API outputs with `python3 tools/api_manifest/generate.py --write`.
- Refreshed the T102 declaration fingerprints for the combined C, Java, and
  Python crypto API inventory. The focused T102 plus KMIPKIT-0102/0105 manifest
  suites pass: 21 tests.
- Python 3.12.13 focused runtime command:
  `target/python312-venv/bin/python -m pytest -q
  bindings/python/tests/test_fluent_crypto.py` — 12 passed.
- `python3 tools/api_manifest/generate.py --check` confirms all 12 generated
  outputs are current. The rebased command
  `python3 tools/api_manifest/audit_binding_exports.py --report
  target/0105-python-binding-exports.json --check` passed with no audit issues.
- Re-ran the 21 manifest/T102 contracts and all 12 focused Python fluent crypto
  tests after syncing the branch to the final Java base; both passed.

## CI coverage correction

- PR #190 run 43104702058 at head
  `a9a87d9dd3484bad57ab2508286f26aa4f69a080` failed the changed-production
  gate at **167/193 (86.5%)**, below 90%.
- Added real-builder tests for Python input validation, malformed buffer
  views, size limits, local validation/reuse, and exception-safe lifecycle.
  A test-first allocation-failure regression exposed that a temporary secret
  copy was not wiped if CFFI byte-span descriptor allocation failed. Moving
  descriptor allocation/initialization into the existing wiping `try/finally`
  fixes that FR-004 defect without changing the ABI or public API.
- TDD commits: Red `c5d168dd671e403a6ac67a81ebc4b81501ade84d`;
  Green `527eb1f48bd4fd009cd63f6b1c82cbf28ded5bc3`;
  Refactor `fe23ba474231147844cd5e3f0778164bbac15bfd`.
- Focused command: `target/python312-venv/bin/python -m pytest -q bindings/python/tests/test_fluent_crypto.py`
  — **28 passed** after the fix and after refactoring.
- Full Python coverage command: `target/python312-venv/bin/python -m pytest -q bindings/python/tests --cov=kmipkit --cov-config=bindings/python/pyproject.toml --cov-report=xml:target/coverage-python-0105-fix.xml`
  — **1,299 passed, 119 subtests passed, one packaging failure**;
  coverage **96.79%**. The existing wheel test built its native library but
  failed writing the wheel because `/tmp` ran out of space.
- Removed only that run's temporary wheel fixture and retried the exact test
  on the worktree filesystem:
  `target/python312-venv/bin/python -m pytest -q bindings/python/tests/test_shared_parity_fixtures.py::test_built_wheel_artifact_contains_py_typed_and_adjacent_native_library --basetemp=target/python105-wheel-retry`
  — **1 passed**. No test was excluded.
- Local official gate: `python3 scripts/coverage_gate.py aggregate --workspace . --report-dir target/coverage-0105-fix --base fec2849cc77e123023e913a6301544c1a45eecb4 --merge fe23ba474231147844cd5e3f0778164bbac15bfd --scopes-json '["python"]'`
  — **passed**, Python adapters **96.80%** after the aggregator's existing
  documented exclusion; changed production **182/193 (94.30%)** against
  the unchanged 90% minimum.
- `git diff --check`, Python syntax compilation of the changed production
  and test modules, and both `scripts.tests.test_requirement_traceability`
  contracts passed. No generated files, thresholds, or exclusions changed.
- The local Python gate is not an authoritative full exact-head CI result;
  parent review, push, and remote validation remain pending.
