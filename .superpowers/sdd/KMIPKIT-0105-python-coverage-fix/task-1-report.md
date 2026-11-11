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

## Remaining verification

Full Python coverage and changed-line measurement are in progress. Parent
owns push and exact-head CI; no remote completion is claimed.
