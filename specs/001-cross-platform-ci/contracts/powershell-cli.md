# PowerShell CLI Contract

## Command

```powershell
pwsh -File .\scripts\Test-Wsl.ps1 [-Distribution <UbuntuDistributionName>]
```

Run from the repository root with PowerShell 7.3 or newer, WSL2, one installed Ubuntu distribution, and Rust 1.94.0 already installed in that distribution. The command does not install or update prerequisites.

## Behavior

1. Verify `pwsh` version, repository root, WSL availability, installed distribution list, and WSL version 2.
2. If `-Distribution` is absent, require exactly one distribution whose name begins with `Ubuntu`; if more than one exists, show the names and request an explicit command argument.
3. If supplied, require the exact selected name to be an installed Ubuntu distribution running as WSL2.
4. Translate the current checkout with `wslpath -u`; preserve native output.
5. Set the translated checkout as the WSL working directory with `wsl.exe --cd <linux-repository-root>`.
6. Run `cargo +1.94.0 test --workspace --all-features` through `wsl.exe --exec`, passing each argument separately and without a shell command string.

## Exit and output contract

- Exit `0` only when the Cargo test process exits `0`.
- Return the Cargo test's non-zero exit status unchanged.
- Missing WSL, Ubuntu, toolchain, invalid selection, unsupported WSL version, invalid path conversion, and PowerShell version failures return non-zero with an actionable message on stderr.
- Cargo stdout/stderr remains visible to the caller.
- No automatic install, distro creation, distro conversion, or host configuration change occurs.

## Safety constraints

Repository paths with spaces and Unicode characters are passed as native arguments. No values are interpolated into `bash -c`, `sh -c`, or another shell command string.
