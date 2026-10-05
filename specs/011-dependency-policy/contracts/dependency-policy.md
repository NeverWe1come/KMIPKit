# Contract: Cargo Dependency Policy

## Inputs

- The checked-out Cargo manifests and `Cargo.lock`.
- The fuzz workspace manifest and `fuzz/Cargo.lock`.
- The reviewed policy in `.cargo/deny.toml`.
- The exception register at
  `specification/compliance/dependency-policy-exceptions.json`.
- The exact cargo-deny release pinned in CI and in contributor instructions.
- Refreshed RustSec advisory data.
- Cargo metadata for both workspace roots, including `workspace_members`,
  package `source`, and `manifest_path` fields.
- Full-feature Cargo metadata for each root with
  `cargo metadata --locked --format-version 1 --all-features` and no
  `--filter-platform`, so every target-specific path dependency is represented.

## Invocation

The repository provides one PowerShell entry point:

```powershell
pwsh -File scripts/Test-DependencyPolicy.ps1
```

CI invokes the same underlying checks from a least-privilege policy job for
both workspaces. The policy job is not filtered by changed file paths and does
not write manifests or lockfiles.

## Decision contract

- Exit `0` only when all configured checks pass, both lockfiles were checked,
  both workspace invocations completed an online advisory refresh, each
  successful invocation records its actual RustSec database commit SHA and
  timestamp, and every configured exception is current and exact.
- Exit nonzero for any denied advisory/license/source/ban/duplicate/wildcard
  finding, invalid exception, missing lockfile, unsupported graph, tool
  failure, or unavailable online advisory refresh/cached-only invocation.
- Emit package, version, affected rule, and public advisory/source evidence as
  appropriate. Do not print credentials or secret material.
- Emit the RustSec database commit SHA and timestamp used by each successful
  workspace invocation. Use a unique temporary `CARGO_HOME` per policy job so
  concurrent jobs cannot move the inspected advisory database; rely on the
  cargo-deny default online fetch, verify the database's configured remote is
  the RustSec Advisory DB, and do not use cached-only options.
- Preserve CI's repository permission as `contents: read`; use no secrets or
  write token. Do not install a GitHub Action for the checker.
- The check analyzes Cargo metadata and lockfiles; it does not prove every
  third-party source file's complete licensing status.

## Policy invariants

1. Only reviewed finite SPDX identifiers and exact license exceptions are
   allowed.
2. Unknown registries, Git sources, out-of-checkout or non-member local paths,
   the architecture-banned crates, and wildcard requirements fail without
   exception. Other duplicate versions (including development-only duplicates)
   and other policy findings can use only an exact current exception. Local
   path escapes and symlink escapes are never excepted.
3. Vulnerable, unsound, unmaintained, or yanked dependencies fail unless an
   exact current exception exists.
4. Exceptions match only their recorded package/version/source/advisory and
   expire no later than 90 days after review.
5. Scheduled runs use the checked-in active release reference; pull-request
   runs use the checked-out pull-request merge tree.
6. Cargo metadata path packages are accepted only when their canonical
   manifest is in the union of root/fuzz workspace members and remains under
   the canonical checkout root.
