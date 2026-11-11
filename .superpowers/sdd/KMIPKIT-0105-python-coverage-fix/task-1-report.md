# Task 1 report — KMIPKIT-0105 Python coverage correction

## Scope and baseline

- Worktree: `/home/ramp1953/.codex/worktrees/fluent-python-crypto/KMIPKit-fluent-dx-worktree`.
- Branch: `feature/KMIPKIT-0105-python-fluent-crypto-api`.
- Initial HEAD: `a9a87d9dd3484bad57ab2508286f26aa4f69a080`.
- CI evidence supplied by the parent: PR #190, run 43104702058;
  changed-production coverage 167/193 (86.5%), below 90%.
- Read repository rules, constitution, approved specification, tasks,
  traceability, implementation evidence, task brief, and SDD ledger.
- Baseline command: `target/python312-venv/bin/python -m pytest -q bindings/python/tests/test_fluent_crypto.py`.
- Baseline result: **12 passed**. Reused the existing Python 3.12.13
  editable installation and native library; no native build was run.

## Red

- Regression: `test_crypto_builder_wipes_staging_when_span_allocation_fails`.
- Contract: KMIPKIT-0105-FR-004, binding-owned mutable staging must be wiped.
- Injected failure is restricted to allocating the CFFI byte-span descriptor;
  buffer conversion and the builder/native ownership remain real.
- Command: `target/python312-venv/bin/python -m pytest -q bindings/python/tests/test_fluent_crypto.py::test_crypto_builder_wipes_staging_when_span_allocation_fails`.
- Result: **1 failed**, expected assertion failure: staged buffer still
  contained `caller-owned-secret` rather than zero bytes. The original
  `MemoryError` propagated and the caller buffer remained unchanged.
- Root cause: `_set_bytes()` allocated/initialized the byte-span after
  staging but before entering its wiping `try/finally`.
- Red commit: `c5d168dd671e403a6ac67a81ebc4b81501ade84d`
  (`test(python): expose crypto staging allocation cleanup gap`).

## Green

- Moved byte-span allocation and initialization inside the existing wiping
  `try/finally`. No native ABI, supported surface, transport, threshold,
  exclusion, or generated artifact changed.
- Added local type-error tests for typed parameters, structures, expert Data,
  indicators, request options, and non-buffer Data. Added released,
  non-contiguous, and oversized buffer tests, local validation/reuse,
  execution without a client, and context closure after consumer exceptions.
- Command: `target/python312-venv/bin/python -m pytest -q bindings/python/tests/test_fluent_crypto.py`.
- Result: **28 passed**.
- Command: `target/python312-venv/bin/python -m py_compile bindings/python/src/kmipkit/fluent.py bindings/python/tests/test_fluent_crypto.py`.
- Result: passed.
- Command: `git diff --check`.
- Result: passed.
- Green commit: `527eb1f48bd4fd009cd63f6b1c82cbf28ded5bc3`
  (`fix(python): wipe crypto staging on span allocation failure`).

## Full Python coverage and environmental packaging failure

- Command: `target/python312-venv/bin/python -m pytest -q bindings/python/tests --cov=kmipkit --cov-config=bindings/python/pyproject.toml --cov-report=xml:target/coverage-python-0105-fix.xml`.
- Result: **1,299 passed, 119 subtests passed, 1 failed** in 94.02 seconds;
  total Python line coverage **96.79%**, above the unchanged 85% gate.
- Failure: existing wheel-packaging test
  `test_built_wheel_artifact_contains_py_typed_and_adjacent_native_library`
  launched a native build in its temporary fixture. The native compilation
  completed, but wheel archive creation failed with **No space left on
  device** in `/tmp`. This broader test run unintentionally launched native
  compilation despite the task's reuse preference; it is explicitly recorded
  rather than reported as a clean full-suite pass.
- Removed only `/tmp/pytest-of-ramp1953/pytest-1/test_built_wheel_artifact_cont0`,
  created by this run. `df -h /tmp .` then showed 813 MiB available in `/tmp`
  and 764 GiB on the worktree filesystem.
- Following the parent's instruction, the exact packaging test is being
  retried with `--basetemp=target/python105-wheel-retry` on the worktree disk.
  No test was excluded and no coverage threshold changed.

## Refactor

- Consolidated the non-buffer Data cases into the existing setter-type matrix
  without changing the tested inputs or behavior.
- Updated feature traceability to name the new staging-failure regression
  and lifecycle/input contracts.
- Command: `target/python312-venv/bin/python -m pytest -q bindings/python/tests/test_fluent_crypto.py`.
- Result after refactor: **28 passed** in 2.33 seconds.
- Commands: `git diff --check` and
  `target/python312-venv/bin/python -m py_compile bindings/python/src/kmipkit/fluent.py bindings/python/tests/test_fluent_crypto.py`.
- Results: passed.

## Local official coverage gate

- Copied the unmodified full-suite XML report to
  `target/coverage-0105-fix/coverage-python/coverage.xml`, matching the
  repository aggregator's artifact layout.
- Command: `python3 scripts/coverage_gate.py aggregate --workspace . --report-dir target/coverage-0105-fix --base fec2849cc77e123023e913a6301544c1a45eecb4 --merge 527eb1f48bd4fd009cd63f6b1c82cbf28ded5bc3 --scopes-json '["python"]'`.
- Result: **passed**, Python adapters **8858/9151 (96.80%)** after the
  aggregator's one existing documented exclusion; changed production
  **182/193 (94.30%)**, above the unchanged 90% gate.
- The base is the exact Java base SHA recorded in the failed CI log.
- This local selected-scope gate verifies the Python fix; the complete remote
  gate for the final pushed head remains authoritative.

The exact packaging retry is pending.

Parent owns push and exact-head CI; no remote completion is claimed.
