# Quickstart: Dependency Policy

## Prerequisites

- Rust/Cargo 1.94 or newer available through rustup.
- Python 3.11 or newer for the repository's standard-library policy tests.
- Network access for refreshing the RustSec advisory database.

## Run the full policy check

```powershell
pwsh -File scripts/Test-DependencyPolicy.ps1
```

Expected result: both root and fuzz workspaces are checked with the exact tool
version configured by the repository. The command exits `0` only when the
license, advisory, source, ban, duplicate, wildcard, and exception checks pass.
It also verifies path dependencies against canonical workspace-member
manifests and prints the RustSec database commit SHA and timestamp used by each
workspace check. The command does not edit either lockfile.

## Diagnose a finding

1. Read the package, version, and policy rule printed by the failing check.
2. Prefer updating or removing an affected dependency through a reviewed
   feature change; do not use an automatic fix that changes `Cargo.lock`.
3. For an unavoidable temporary exception, add the exact reviewed record and
   matching policy entry described in the
   [exception schema](data-model.md). The exception must include rationale,
   mitigation, reviewer, evidence link, and an expiry no
   more than 90 days after review.
4. Rerun the same command and include the output in the pull-request evidence.

## Scope and limitation

This check governs Cargo dependency metadata and advisory records. It does not
inspect every source file in every third-party package for undisclosed license
terms and is not a substitute for a complete legal review.
