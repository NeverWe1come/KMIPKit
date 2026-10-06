# Rust workspace and PowerShell commands

KMIPKit is a Cargo workspace. Its seven crates separate protocol-independent
TTLV handling, KMIP models, I/O, client orchestration, the supported Rust API,
the C ABI, and test support. The workspace uses Rust 2024, MSRV 1.94, shared
package metadata, a central dependency graph, and shared lint settings.

## Crate responsibilities

| Crate | Responsibility | Publication |
|---|---|---|
| `kmipkit-ttlv` | Generic TTLV values, framing, encoding, and decoding | Public |
| `kmipkit-protocol` | KMIP structures, values, and validation | Public |
| `kmipkit-transport` | Raw TLS and HTTPS transport | Public |
| `kmipkit-client` | Synchronous request lifecycle and orchestration | Public |
| `kmipkit` | Supported high-level Rust API | Public |
| `kmipkit-ffi` | Stable C ABI and native library targets | Private workspace crate |
| `kmipkit-test-support` | Shared test fixtures and helpers | Private workspace crate |

Internal dependencies flow downward. `kmipkit-ttlv` is transport independent;
protocol code performs no I/O; bindings do not implement KMIP semantics.

## Creating the workspace with Cargo in PowerShell

From the repository root, Cargo creates the library crate directories. The root
`Cargo.toml` then declares them as workspace members and centralizes versions,
license, repository, and lint configuration.

```powershell
$crateNames = @(
    'kmipkit-ttlv',
    'kmipkit-protocol',
    'kmipkit-transport',
    'kmipkit-client',
    'kmipkit',
    'kmipkit-ffi',
    'kmipkit-test-support'
)

New-Item -ItemType Directory -Path crates -Force | Out-Null
foreach ($crateName in $crateNames) {
    cargo new --lib --name $crateName --edition 2024 --vcs none "crates/$crateName"
    if ($LASTEXITCODE -ne 0) {
        throw "cargo new failed for $crateName"
    }
}
```

Cargo creates each crate's initial manifest and `src/lib.rs`. The workspace
manifest and crate manifests are then edited to inherit shared metadata and
express dependencies. In KMIPKit, this setup is already present; do not rerun
the creation loop over existing crate directories.

## Common local commands

Run these from the repository root in PowerShell:

```powershell
# Show the package graph without resolving dependencies.
cargo metadata --format-version 1 --no-deps

# Compile all workspace crates and their targets.
cargo check --workspace --all-targets --all-features

# Format check.
cargo fmt --all --check

# Lint all workspace targets and treat warnings as errors.
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run the workspace tests when implementation specifications add them.
cargo test --workspace --all-features
```

Run the complete dependency-policy check from PowerShell at the repository
root with the single command:

```powershell
pwsh -File .\scripts\Test-DependencyPolicy.ps1
```

It uses the reviewed `cargo-deny` 0.20.2 tool and checks both the root and fuzz
workspaces, their committed lockfiles, all features, development dependencies,
and every target-specific edge in their resolved graphs. Advisory checks run
online and print separate RustSec database commit and timestamp evidence for
each workspace. The command fails on policy violations, failed or
cached-only advisory refresh, invalid path dependencies, missing or stale
lockfiles, an unreviewed runner host, or absent evidence; it preserves both
lockfiles. See the [dependency-policy guide](../security/dependency-policy.md)
for exception review and license-evidence limitations.

`cargo build` produces development artifacts under `target/`. The command
`cargo doc --workspace --no-deps` builds crate documentation. The lockfile is
committed so workspace dependency resolution remains reviewable and reproducible.

## Run workspace tests through WSL from PowerShell

On Windows, KMIPKit supports running the Rust suite in an existing WSL2 Ubuntu
distribution. Use PowerShell 7.3 or newer, WSL2, and Rust 1.94.0 installed in
that distribution. From the repository root, run:

```powershell
pwsh -File .\scripts\Test-Wsl.ps1
```

The command requires exactly one installed Ubuntu distribution. If more than
one is installed, select one by its exact registered name:

```powershell
pwsh -File .\scripts\Test-Wsl.ps1 -Distribution Ubuntu-26.04
```

The runner converts the current checkout path with `wslpath`, sets it as the
WSL working directory with `wsl.exe --cd`, and calls Cargo through
`wsl.exe --exec`. It returns Cargo's exit status and preserves its output. It
does not install Rust, create or convert a distribution, or modify host
configuration. Install missing prerequisites manually and rerun the command.

For script and coverage-gate tests, run:

```powershell
pwsh -File .\scripts\tests\Test-Wsl.ps1
python -m unittest discover -s scripts/tests -p 'test_*.py' -v
```

## Current implementation boundary

The crates currently contain crate-level documentation and compile-time lint
policy only. They do not yet implement KMIP values, operations, encoding,
transport, or C symbols. Each behavior is added through an approved feature
specification and its tests.
