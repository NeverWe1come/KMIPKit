# Quickstart: Validate Cross-Platform CI and WSL Tests

## Local PowerShell tests through WSL2

Prerequisites: PowerShell 7.3+, WSL2, Ubuntu, and Rust 1.94.0 already installed in that Ubuntu distribution.

From the repository root:

```powershell
pwsh -File .\scripts\Test-Wsl.ps1
```

If more than one Ubuntu distribution is installed, pass the exact registered name:

```powershell
pwsh -File .\scripts\Test-Wsl.ps1 -Distribution Ubuntu-26.04
```

The command runs `cargo +1.94.0 test --workspace --all-features` in the current checkout and returns its exit status. It does not provision software.

## Local automation tests

```powershell
pwsh -File .\scripts\tests\Test-Wsl.ps1
python -m unittest discover -s scripts/tests -p 'test_*.py' -v
```

## Coverage workflow

On pull requests, GitHub Actions tests the default PR merge commit on all three operating systems and on Rust 1.94 and stable. Stable coverage reports are aggregated across Ubuntu, Windows, and macOS. When no executable production function bodies exist, each platform emits an explicit `unavailable` artifact. When eligible code exists, the job enforces the project thresholds and rejects missing or malformed coverage data. A PR with no changed executable Rust lines reports the changed-code metric as `not applicable`; the workspace and package gates still run. A separate nightly Linux job attempts branch coverage with a non-blocking result.

See [testing strategy](../../docs/development/testing.md) for the gates and [the CLI contract](contracts/powershell-cli.md) for WSL selection and exit behavior.
