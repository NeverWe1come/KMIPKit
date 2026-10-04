# Implementation Plan: Cross-Platform CI and Local Tests

**Branch**: `feature/KMIPKIT-0001-cross-platform-ci` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

## Summary

Add pull-request CI for Linux, Windows, and macOS on Rust 1.94 and stable; a PowerShell 7.3+ entry point that runs the current checkout's tests in an existing WSL2 Ubuntu distribution; and coverage collection that fails closed once production Rust code exists. Coverage policy remains in `docs/development/testing.md`, including 95% changed executable-line coverage, 95% TTLV/protocol, 85% transport/FFI/bindings, and 90% workspace coverage.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94; PowerShell 7.3+ for the WSL entry point; Python 3 standard library for the coverage gate.
**Primary Dependencies**: GitHub-hosted runners; pinned `actions/checkout`, `actions/upload-artifact`, and `actions/download-artifact`; `cargo-llvm-cov` 0.9.1 with `--locked`; LLVM JSON coverage export.
**Storage**: Workflow artifacts retained for the run; no persistent application storage.
**Testing**: `cargo fmt`, Clippy, `cargo test`, `cargo doc`; self-contained PowerShell tests; Python `unittest` tests for coverage report and diff handling.
**Target Platform**: Ubuntu, Windows, and macOS GitHub-hosted runners; Windows PowerShell 7.3+ with existing WSL2 Ubuntu for local testing.
**Project Type**: Rust library workspace with CI and contributor tooling.
**Performance Goals**: No protocol performance changes. CI should avoid duplicate test runs where the matrix already exercises the same OS/toolchain pair.
**Constraints**: No runner secrets or write permissions; no automatic machine/WSL setup; no mutable action tags; no coverage passing when eligible source or report data is missing; no coverage exclusions without a reason.
**Scale/Scope**: One workflow, local runner scripts and tests, coverage gate and fixtures, and updates to testing/development docs. No OASIS source, package, release, branch-protection, dependency-policy, or generator changes.

## Constitution Check

| Principle | Check |
|---|---|
| Specification and traceability | Pass. CI-only infrastructure has no KMIP normative clauses; requirements FR-001–FR-012 are mapped to executable validation in `tasks.md`. |
| Test first and evidence based | Pass. Script behavior is tested first; Red, Green, and Refactor are separate signed development commits. |
| One core, explicit language boundaries | Pass. No protocol, FFI, or language API behavior changes. |
| Secure defaults and lossless handling | Pass. Fork-safe `pull_request` workflow, read-only permissions, no persisted checkout credentials, no secrets, and pinned actions. |
| Human governed, reviewable changes | Scope and implementation remain on the feature branch; produce a draft PR and do not change `master`, release branches, repository protections, or publish artifacts. |
| Rust quality and coverage | Pass. Preserve Rust 1.94 and all existing line thresholds; changed executable lines use a strict three-platform report gate. |

## Design Decisions

1. Use ordinary `pull_request` events targeting `master` and `release/**`; do not use privileged `pull_request_target` or `workflow_run` triggers. Set workflow permissions to `contents: read`, omit secrets, and disable persisted checkout credentials. Keep GitHub's default pull-request merge-ref checkout and use `fetch-depth: 0` so the checked tree (`github.sha`) and exact base commit are both present.
2. Run formatting, Clippy, tests, and rustdoc on a 3-OS × (Rust 1.94, stable) matrix. Install toolchains/components with the preinstalled `rustup`; avoid setup actions. Rust `1.94` resolves the latest patch in that series; repository MSRV remains 1.94.
3. Run coverage on stable for all three OSes. Each runner emits a full LLVM JSON report and uploads it under an OS-specific artifact name. When a preflight confirms there are no executable production function bodies, each runner emits an explicit unavailable status artifact instead. A dependent Linux job downloads all three artifacts and runs a Python standard-library gate against the exact checked merge tree and PR base.
4. Derive executable line counts from LLVM JSON file segments and require parsed line and covered-line counts to match each file's LLVM summary. Validate function code-region schemas and map each region file ID through its function filename table. This preserves nested-region counts and keeps lines in the denominator when function records are missing. Compute the diff with `git diff <base.sha> <github.sha>` using the two exact trees (not a merge-base diff), decoding Git-quoted UTF-8 paths and failing on malformed paths. Missing reports, invalid JSON/schema, segment coordinates outside source files, paths outside the checkout, missing diff commits, or no report for eligible code are errors. If the source preflight finds no executable production function bodies, all three runners emit an unavailable status artifact. If a PR changes no executable Rust lines, report only the changed-code metric as not applicable; continue evaluating package/workspace thresholds. Keep tests outside crate `src/`; files under `src/` remain in the coverage denominator regardless of filename.
5. The stable coverage gate checks changed executable lines at 95% when any changed executable lines exist, workspace at 90%, TTLV/protocol at 95%, and transport/FFI/bindings at 85%. A zero changed executable-line denominator is `not applicable`, never 100%. Run branch coverage separately on nightly in a non-gating informational job because cargo-llvm-cov requires nightly for branch data and documents it as unstable.
6. Local WSL execution uses PowerShell 7.3+ native argument passing, explicitly selects one existing WSL2 Ubuntu distribution, translates the current Windows checkout path with `wslpath`, resolves the rustup-managed Cargo path from the distribution user's `HOME`, sets the checkout as WSL's working directory through `wsl.exe --cd`, and invokes `cargo +1.94.0 test --workspace --all-features` without a shell command string. It never installs software or mutates WSL.
7. Coverage source eligibility preflight scans every `*.rs` file under `crates/*/src/`, regardless of filename or nested directory, because Rust path/module attributes can select test-like files as production modules. Tests belong in external crate test directories outside `src/`. The scanner ignores comments and string/character literals and recognizes a function body only when a function definition has a body rather than ending in `;`. A file that cannot be read or syntax that cannot be classified is conservatively eligible, so coverage is required. Inline `#[cfg(test)]` modules in production source fail preflight with guidance to move tests outside `src/`. Only a complete, successful scan that finds no body may emit unavailable sentinels; any eligible source followed by a report/tool failure is a gate failure.

## Project Structure

```text
.github/workflows/ci.yml
docs/development/testing.md
docs/development/rust-workspace.md
scripts/Test-Wsl.ps1
scripts/KmipKit.WslTest.psm1
scripts/coverage_gate.py
scripts/tests/Test-Wsl.ps1
scripts/tests/test_coverage_gate.py
specs/001-cross-platform-ci/
├── contracts/powershell-cli.md
├── data-model.md
├── quickstart.md
├── research.md
└── tasks.md
```

**Structure Decision**: Keep contributor automation in `scripts/`, the single workflow in `.github/workflows/`, and executable validation adjacent under `scripts/tests/`. Spec Kit artifacts remain under the feature directory.

## Implementation Phases

1. Complete requirement-quality checklist, task mapping, and consistency analysis. Resolve critical findings in the artifacts before code.
2. Write failing PowerShell and Python tests for WSL parsing/selection/argument safety, report parsing, diff line extraction, executable region merging, all thresholds, and malformed/missing data. Record RED and commit it.
3. Implement the PowerShell module/entry point and coverage gate minimally; verify GREEN on Windows PowerShell 7.6.5 and installed WSL2 Ubuntu. Commit implementation separately.
4. Refactor for clear errors, path normalization, strict schema handling, and maintainability while preserving GREEN; commit refactor separately.
5. Add the pinned, least-privilege GitHub Actions workflow and documentation; validate YAML, action pins, workflow expressions, local command behavior, and full Rust checks.
6. Run convergence, QA and security reviews, fix findings, run available checks, and prepare a draft PR with commit and verification evidence.

## Risks and Mitigations

- LLVM JSON schemas can evolve. Pin `cargo-llvm-cov`, validate the root/data/files/functions/function-filenames/regions shape and region tuple widths, and fail closed on unknown/incomplete report data.
- Coverage paths differ across hosted OSes. Normalize absolute paths only when they resolve under the checked-out workspace; compare normalized relative paths and reject ambiguous collisions.
- PowerShell native argument handling changed in PowerShell 7.3. Require that version explicitly and test paths with spaces, punctuation, and non-ASCII characters.
- Hosted runner image/toolchain downloads can fail transiently. Surface the failing step and do not convert infrastructure failures into successful checks.
- The initial Rust workspace has no executable production source. Each OS emits an unavailable coverage artifact until code exists; Python/PowerShell tests still run and CI core jobs still validate the workspace.
- The LLVM report must correspond to GitHub's checked-out pull-request merge commit; the base and merge tree IDs come from the pull-request event and full checkout history.
- Cargo-llvm-cov line coverage runs on stable; its branch mode runs separately on nightly with a non-blocking job result.

## Complexity Tracking

No constitution violations or architecture boundary changes.
