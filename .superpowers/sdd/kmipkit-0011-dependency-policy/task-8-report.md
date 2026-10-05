# T008 report: dependency policy PowerShell runner

## Scope

Added `scripts/Test-DependencyPolicy.ps1` for the KMIPKIT-0011 T008 runner contract. No CI, policy, exception, manifest, lockfile, or test files were changed.

The runner checks the four reviewed Cargo host triples against `rustc -vV`, installs and verifies cargo-deny 0.20.2 inside a unique temporary `CARGO_HOME`, generates locked all-feature metadata for the root and fuzz manifests without target filtering, and runs the T007 metadata and exception validator before scanning. It then invokes cargo-deny online for each workspace separately with `--workspace --all-features --locked`. It captures external command output without forwarding raw diagnostics, validates one unambiguous RustSec Git checkout by its `origin` remote, and prints the commit SHA and ISO timestamp after each successful workspace check. It compares SHA-256 hashes for both lockfiles and removes the temporary Cargo home at exit.

The RustSec checkout discovery uses cargo-deny 0.20.2's `advisory-dbs/advisory-db-<stable-url-hash>` layout. The first real run established that path assumption by passing the scan and failing only at post-scan evidence lookup; the next run verified and exercised the corrected discovery.

## TDD evidence

### Red

Before creating the runner, ran:

```text
python -m unittest scripts.tests.test_dependency_policy.DependencyPolicyRunnerContractTests
```

Result: 8 failures, all asserting the missing `scripts/Test-DependencyPolicy.ps1` contract; there were no test errors.

### Green

After implementation, ran:

```text
python -m unittest -v scripts.tests.test_dependency_policy.DependencyPolicyRunnerContractTests scripts.tests.test_dependency_policy.DependencyPolicyApiTests scripts.tests.test_dependency_policy.DependencyExceptionTests
```

Result: 38 tests passed, 1 expected skip, 0 failures. The skipped symlink test requires Windows directory-symlink privileges unavailable on this host (`WinError 1314`).

PowerShell syntax validation with `[System.Management.Automation.Language.Parser]::ParseFile(...)` passed.

## Windows runner evidence

Command:

```text
pwsh -NoLogo -NoProfile -File scripts/Test-DependencyPolicy.ps1
```

Result: passed on `x86_64-pc-windows-msvc`. The runner installed and verified cargo-deny 0.20.2; root and fuzz all-feature metadata and exception validation passed; both separate online cargo-deny workspace checks passed.

| Workspace | Verified RustSec remote | Commit | Commit timestamp |
|---|---|---|---|
| root | `https://github.com/RustSec/advisory-db` | `ef6173cbc5c50ec8166f9a5b28f07834144373ee` | `2026-10-03T10:14:03+02:00` |
| fuzz | `https://github.com/RustSec/advisory-db` | `ef6173cbc5c50ec8166f9a5b28f07834144373ee` | `2026-10-03T10:14:03+02:00` |

Lockfiles remained unchanged:

| Lockfile | SHA-256 |
|---|---|
| `Cargo.lock` | `08cbbb0bbfb0db6e567eec2db83d2b63788cc6ffada8531be0d50033a8ff0231` |
| `fuzz/Cargo.lock` | `ea34d89d36fa78841f0b1c63064726f09a353f62e68725cc7cdc99d32c6c7778` |

`git diff --check` passed before recording this report. No unrelated or pre-existing worktree changes were present.

## Concerns

No remaining implementation concerns. The API/exception tests retain the expected Windows-only symlink privilege skip described above.
