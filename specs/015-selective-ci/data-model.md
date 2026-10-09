# Data Model: CI Impact Plan

## Impact Plan

One classifier result represents one pull-request merge tree compared to its exact base tree.

| Field | Type | Required | Validation |
|---|---|---:|---|
| `schema_version` | positive integer | yes | Supported schema only; unknown versions fail closed. |
| `base_sha` | 40- or 64-character hexadecimal string | yes | Must match the PR event base SHA. |
| `merge_sha` | 40- or 64-character hexadecimal string | yes | Must match `github.sha`. |
| `classes` | unique list of known path classes | yes | Stable order; each class derived from all old/new paths. |
| `paths` | list of path and class/reason records | yes | Repository-relative POSIX paths; no empty, absolute, or parent-traversal paths. |
| `selected_jobs` | unique list of known pull-request job IDs | yes | Must include all jobs implied by classes; `full` selects every PR-required job. |
| `coverage_scopes` | unique list from supported scope names | yes | Exact mapping from component classes; full mode selects all scopes. |
| `full` | boolean | yes | True when any full/shared/unknown trigger is present or classification falls back. |
| `reason` | concise string | yes | Human-readable explanation safe to escape in Markdown. |
| `fallback` | boolean | yes | True when input, parser, or classifier problem selected full. |

Example (illustrative):

```json
{
  "schema_version": 1,
  "base_sha": "<40-hex-base>",
  "merge_sha": "<40-hex-merge>",
  "classes": ["documentation", "python"],
  "paths": [
    {"path": "docs/development/testing.md", "class": "documentation"},
    {"path": "bindings/python/src/kmipkit/extensions.py", "class": "python"}
  ],
  "selected_jobs": ["docs-contracts", "language-python", "python-coverage", "coverage-gate"],
  "coverage_scopes": ["python"],
  "full": false,
  "reason": "Documentation and Python adapter source changed.",
  "fallback": false
}
```

The plan is an untrusted workflow input at the Summary boundary. The Summary validates its schema, SHA binding, known job/scope identifiers, and internal consistency before accepting any skip. Missing, invalid, mismatched, or unsupported plans make the Summary fail and are treated as full-selection inputs by PR job conditions wherever possible.

## Coverage Scope Names

| Name | Reports and current policy |
|---|---|
| `rust` | Linux, Windows, and macOS LLVM JSON; crate and workspace thresholds plus changed-source threshold. |
| `ffi-c` | Existing Linux C consumer/Rust FFI LLVM JSON; `kmipkit-ffi` threshold and C API exercise. Selected together with `rust` where Rust core/shared CI applies. |
| `java` | Linux JaCoCo XML; Java adapter 85% and selected changed-source 95%. |
| `python` | Linux coverage.py XML; Python adapter 85% and selected changed-source 95%. |
| `jni` | Linux JNI LLVM JSON; JNI bridge 85% and selected changed-source 95%. |

## Job Result

A job result is the `needs.<job_id>.result` outcome for the current workflow run. `success` is passing; `failure`, `cancelled`, `missing`, or `skipped` for a selected job are failing. `skipped` is `not affected` only when the validated plan does not select that job. Schedule-specific jobs remain evaluated by their existing event-specific policy.
