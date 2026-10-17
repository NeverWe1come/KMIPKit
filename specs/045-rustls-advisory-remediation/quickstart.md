# Quickstart: Rustls Advisory Remediation

Run from the repository root after checking out the feature branch.

```powershell
python -m unittest scripts.tests.test_rustls_advisory_pin -v
cargo test -p kmipkit-transport --all-features
pwsh -File .\scripts\Test-DependencyPolicy.ps1
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

The dependency-policy command must refresh RustSec online and report evidence for both Cargo workspaces. Do not accept cached-only success or add an advisory waiver. Compare `Cargo.lock` against the Red baseline to ensure only the Rustls resolution changed.
