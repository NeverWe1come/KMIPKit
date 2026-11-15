# Quickstart: CI Impact Routing

## Local classifier

From the repository root, run the classifier against two commits:

```text
python -B scripts/ci_impact.py classify --base <base-sha> --merge <merge-sha> --output impact-plan.json
```

The command emits the validated plan as JSON and a short routing decision to stdout. Unknown paths and safely recoverable diff/classification errors return a full-CI plan with a fallback reason. Invalid command-line SHA values or inability to create the requested output file return non-zero.

## Focused contracts

```text
python -B -m unittest scripts.tests.test_ci_impact -v
python -B -m unittest scripts.tests.test_ci_summary -v
python -B -m unittest scripts.tests.test_coverage_gate scripts.tests.test_multilanguage_coverage -v
python -B -m unittest scripts.tests.test_workflow -v
```

## PR expectations

- Documentation-only PR: docs/traceability validators and Summary run; build/test/coverage jobs are authorized skips.
- Isolated language PR: changed language tests run on Linux, Windows, and macOS; its Linux coverage scope is gated.
- Shared, high-risk, unknown, or CI/tooling path: full PR validation runs.
- Any selected failure, cancellation, missing result, missing coverage report, or unexpected skip keeps the Summary red.
- Scheduled runs do not use the PR impact plan.
