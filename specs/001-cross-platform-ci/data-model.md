# Data Model: CI and Local Test Invocation

This feature introduces workflow records only; it does not add application data.

## CI Run

| Field | Meaning | Validation |
|---|---|---|
| merge_commit | GitHub pull-request merge commit checked out for validation | Equals the workflow's checked-out `github.sha`; full Git object ID |
| base_commit | Pull request base commit | Full Git object ID; present in the full checkout and used in a two-tree diff against `merge_commit` |
| head_commit | Pull request source-branch head | Recorded as PR metadata; not the tree used for coverage when the merge ref is checked |
| operating_system | `ubuntu-latest`, `windows-latest`, or `macos-latest` | One of the required runner labels |
| rust_toolchain | Rust 1.94 series or stable | Toolchain install and invocation succeed |
| check | Formatting, Clippy, tests, rustdoc, or coverage | Required check name in the workflow |
| result | Passed, failed, or unavailable | `unavailable` only when no eligible executable production source exists |
| report | Tool output or coverage artifact | Coverage JSON has required schema and belongs to the checked out repository |

## WSL Test Invocation

| Field | Meaning | Validation |
|---|---|---|
| repository_root | Current Windows checkout | Exists and is a repository root |
| distribution | Selected installed Ubuntu distribution | Unique by default or supplied explicitly; WSL version 2 |
| linux_repository_root | `wslpath` translation of checkout | Non-empty absolute WSL path |
| command | Cargo workspace test invocation | `cargo +1.94.0 test --workspace --all-features`; no shell string |
| exit_status | Native test process status | Returned unchanged; preflight failures are non-zero |

## Coverage Result

| Field | Meaning | Validation |
|---|---|---|
| platform_reports | One status artifact for each required OS | Exactly one artifact per OS; either a valid LLVM JSON report or explicit no-code unavailable sentinel |
| source_path | Workspace-relative Rust source path | Must normalize inside the checkout |
| executable_lines | Physical lines derived from LLVM file segments; function code-region start lines must be present; function regions reconcile shared lines before remaining summary-uncovered residuals are added per platform | Union across required OS reports plus conservative uncovered residuals |
| execution_count | Hit count per executable line | Sum across reports; non-negative integer |
| changed_lines | Executable source lines changed from PR base to checked merge tree | Computed from exact base and merge commit trees; zero changed executable lines => changed-code metric is not applicable |
| coverage_percent | Covered executable lines / executable lines | No production executable source => overall coverage unavailable; otherwise thresholds evaluated; no changed executable lines => changed-code metric is not applicable, never 100% |
| threshold | Applicable policy | 95% changed and ttlv/protocol; 85% transport/FFI/bindings; 90% workspace |
| branch_summary | Branch counts/coverage where emitted | Report only; never gates this feature |

## State transitions

`coverage result`: `unavailable` only after a complete scan of every `*.rs` file under `crates/*/src/`, regardless of filename or nested directory, succeeds and finds no function body. Any generated-source exclusion requires a documented reason. The scan ignores comments and string/character literals and distinguishes declarations ending in `;`. An unreadable file or unclassifiable syntax is conservatively eligible, requiring coverage. Inline `#[cfg(test)]` modules in production source fail preflight and must move to excluded test paths. Each OS emits a sentinel artifact only for a valid no-body result. `measured` when all three LLVM reports and data are valid and applicable thresholds pass. `failed` when data is absent/incomplete/malformed after eligible source is found, a changed executable line lacks a valid measurement, preflight finds inline tests, or a threshold is missed. `changed-code metric`: `not applicable` when there are no changed executable Rust lines, without changing the state of other metrics. A failed, unavailable, or not-applicable result cannot be serialized as a passing percentage.
