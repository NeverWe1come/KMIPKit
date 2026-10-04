# Research: Cross-Platform CI and Local Tests

## Decision: Use a read-only `pull_request` workflow

**Decision**: Trigger on pull requests targeting `master` or `release/**`. Set only `contents: read`; do not use privileged PR triggers, secrets, branch mutation, publishing, or persisted checkout credentials.

**Rationale**: Fork-controlled workflow and source code must not execute with privileged base-repository credentials. GitHub documents the elevated context of `pull_request_target`; the workflow does not need it.

**Alternatives considered**: `pull_request_target` plus carefully gated checkout was rejected because no privileged capability is needed. A write token for status/report publication was rejected; standard check results and run artifacts are sufficient.

Sources: [GitHub Actions events](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows), [workflow permissions](https://docs.github.com/en/actions/using-jobs/assigning-permissions-to-jobs), [actions/checkout security guidance](https://github.com/actions/checkout).

## Decision: Run core checks on three OSes and two Rust channels

**Decision**: Use Ubuntu, Windows, and macOS hosted runners, each with the Rust 1.94 toolchain line and current stable. Install toolchains with rustup already available on hosted images; do not add a setup action.

**Rationale**: This directly verifies the required platform and MSRV matrix with fewer action dependencies. Rust 1.94 remains the declared minimum; choosing the latest patch within 1.94 does not change the MSRV.

**Alternatives considered**: A floating stable-only matrix was rejected because it would not validate the MSRV. A third-party Rust setup action was rejected as unnecessary for hosted runners.

Sources: [Rust release archive](https://blog.rust-lang.org/releases/), [GitHub-hosted runners](https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners).

## Decision: Use strict LLVM JSON coverage reports from every required OS

**Decision**: Pin `cargo-llvm-cov` 0.9.1, export JSON on Linux, Windows, and macOS, upload uniquely named artifacts, and aggregate in a read-only Linux job. Keep the default PR merge-ref checkout with full history; compare its exact `github.sha` tree to the exact event base SHA. Derive changed executable lines from LLVM file segments, check segment totals as lower bounds against each file summary, and require function code-region start lines to appear in the mapped file segment map. Summary totals can exceed unique physical lines when source locations are shared across functions; use function regions to reconcile shared physical-line coverage within one export mapping, count remaining uncovered residuals as uncovered, sum those residuals across platforms, and include them in changed-file coverage. Fail closed when a workspace source path occurs in multiple export mappings until reconciliation can preserve each mapping's uncovered evidence. Keep Rust tests in external crate test directories outside `src/`; all `.rs` files under `src/` remain in the denominator regardless of name because path/module attributes can select them as production code.

**Rationale**: LLVM JSON includes per-function source regions and execution counts. Its JSON exporter defines the top-level `data` array as one or more export objects, each representing a `CoverageMapping` ([LLVM `CoverageExporterJson.cpp`](https://github.com/llvm/llvm-project/blob/main/llvm/tools/llvm-cov/CoverageExporterJson.cpp)). The checker accepts the `llvm.coverage.json.export` type and reviewed schema versions 3.0.x and 3.1.x; other types or versions fail closed until explicitly reviewed. LLVM defines its JSON export version as a major/minor/patch triple. LCOV provides only line-based records and may omit changed source lines; a changed-line gate must not treat an absent record as covered. Three-platform reports account for platform-specific `cfg` code. Missing artifacts, malformed JSON, repeated workspace source paths across export mappings, missing required schema fields, invalid paths, and an eligible source file without coverage data are errors. When a PR changes no executable Rust lines, report the changed-code metric as `not applicable`, not 100%. When no production function bodies exist, each platform emits an explicit unavailable sentinel so aggregation can distinguish the expected initial state from missing reports. Branch coverage requires the nightly toolchain and is unstable, so its report runs separately and cannot block required CI checks.

**Alternatives considered**: `diff-cover` over LCOV was rejected as the enforcement source because lines absent from the LCOV report and zero measured lines can be silently omitted. A single Linux-only report was rejected because platform-specific executable code could be omitted.

Sources: [LLVM `llvm-cov export`](https://www.llvm.org/docs/CommandGuide/llvm-cov.html#export-command), [LLVM source-based coverage documentation](https://clang.llvm.org/docs/SourceBasedCodeCoverage.html), [cargo-llvm-cov JSON and install documentation](https://github.com/taiki-e/cargo-llvm-cov), [cargo-llvm-cov limitations](https://github.com/taiki-e/cargo-llvm-cov#known-limitations).

The cargo-llvm-cov documentation also confirms support for the Windows MSVC runner target used by GitHub-hosted Windows runners. Source files under `src/` are not omitted based only on test-like names because Rust path attributes can select them as production modules.

## Decision: Use PowerShell 7.3+ and direct WSL argv

**Decision**: Provide `pwsh -File scripts/Test-Wsl.ps1 [-Distribution <name>]`. Select exactly one installed Ubuntu distribution unless explicitly named; require WSL2; convert the repository path using `wslpath`; pass it as the working directory through `wsl.exe --cd`; call `wsl.exe` with `--exec` and argument arrays; propagate native exit status.

**Rationale**: PowerShell 7.3 improved native argument passing. Avoiding `bash -c` keeps the Windows path and test arguments out of an interpolated shell program. Preflight failures are read-only and actionable.

**Alternatives considered**: PowerShell 5.1 was rejected for the primary interface because its native argument-passing semantics require additional quoting machinery. Automatic WSL installation, distribution conversion, or Rust installation was rejected as a host mutation.

Sources: [PowerShell native argument passing](https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_parsing), [WSL basic commands](https://learn.microsoft.com/windows/wsl/basic-commands), [WSL `--cd` command introduction](https://learn.microsoft.com/windows/wsl/release-notes), [WSL file systems](https://learn.microsoft.com/windows/wsl/filesystems).

## Verified action pins

The official Git tag refs were resolved with `git ls-remote` on 2026-10-04:

| Action release | Full commit SHA |
|---|---|
| `actions/checkout` v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `actions/upload-artifact` v7.0.1 | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` |
| `actions/download-artifact` v8.0.1 | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` |

## Unresolved research

No design-critical questions remain. Before implementation, tests will verify the pinned coverage tool's actual JSON schema against checked-in fixtures and, where the environment permits, a generated report. CI validation will fail closed on schema drift.
