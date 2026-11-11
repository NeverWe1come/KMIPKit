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
- Red commit: this commit; exact hash recorded in the Green update.

## Remaining work

Apply the smallest cleanup fix, add meaningful invalid-input/ownership
coverage, rerun focused tests and changed-line measurement, then record Green
and Refactor evidence in separate signed-off commits. Parent owns push and CI.
