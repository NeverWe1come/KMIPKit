# Implementation Plan: Dependency Policy Gates

**Branch**: `feature/KMIPKIT-0011-dependency-policy-spec` | **Date**: 2026-10-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/011-dependency-policy/spec.md`

## Summary

Add a pinned, unified Cargo dependency-policy check to pull-request CI and the
scheduled review. Use `cargo-deny` for advisory, license, duplicate/ban, and
source checks; check both root and fuzz workspaces; maintain explicit policy
and time-limited exception records; and document one local command. Keep the
job read-only, secret-free, locked, and independent of the existing OASIS
integrity workflow.

## Technical Context

**Language/Version**: Rust 1.94 workspace; Python 3.11+ standard library for
the policy-exception consistency validator, if the implementation gate
confirms this is the narrowest maintainable option.

**Primary Dependencies**: Exact reviewed `cargo-deny` release; no runtime or
published-package dependency is added. The pinned release and platform support
must be recorded in `dependency-review.md` before the workflow uses it.

**Storage**: Checked-in `Cargo.lock`, `fuzz/Cargo.lock`, `.cargo/deny.toml`,
and a checked-in machine-readable exception record. Advisory databases use an
isolated temporary Cargo home and are refreshed by each online workspace
invocation. The policy command uses `--locked` but not `--offline`,
`--frozen`, or cached-only fetch options.

**Testing**: Python `unittest` fixtures for exception-policy validation and
workflow contracts; `cargo deny check` against the root and fuzz workspaces;
existing cross-platform CI. No protocol tests or OASIS vectors apply.

**Target Platform**: One least-privilege Linux policy job on each pull request
and scheduled run. Its dependency target set must cover every target triple
used by the current Linux, Windows, and macOS CI matrix, including the
self-hosted Linux ARM64 path. Verify and record the exact target list in T001.

**Project Type**: Rust workspace CI policy and contributor documentation.

**Constraints**: No secrets, write permissions, mutable action references,
lockfile changes, auto-fixes, unpinned policy tools, unbounded exceptions,
new runtime dependencies, or OASIS source changes. The only manifest edits
are the two first-party unpublished fuzz metadata corrections in FR-013; they
must follow Red tests and preserve resolved package versions. Policy database
failures must fail visibly rather than be skipped.

**Scale/Scope**: One policy configuration and validator covering two Cargo
lockfiles, wired to existing PR and scheduled CI. No SBOM, attestation,
release-signing, branch-protection, Java, or Python package policy.

## Constitution Check

### Before research

- **Specification and traceability**: Pass. This infrastructure specification
  has testable FR/SC identifiers. OASIS clauses do not apply; the affected
  project policy sources are linked in `research.md`.
- **Test first and evidence**: Pass. Tasks require red and green test commits
  followed by a distinct refactor/evidence commit before the draft PR.
- **Human-governed review**: Pass. The change stays on a dedicated feature
  branch; an agent may open a draft PR but may not approve or merge it.
- **Secure defaults**: Pass. CI remains read-only and secret-free; unknown
  licenses, sources, stale scans, and policy exceptions fail closed.
- **Architecture and product boundaries**: Pass. No Rust runtime API,
  transport, protocol, or package behavior changes.

### After design

Pass. The design adds repository policy/configuration, a narrow validator,
workflow wiring, documentation, and two first-party metadata corrections to
the unpublished fuzz workspace. It does not change accepted ADRs,
constitution, OASIS inputs, runtime dependencies, resolved package versions,
or published artifacts. Administrator-owned branch protection remains an
external limitation and will not be claimed as enforced.

## Design decisions

1. Use one `cargo-deny` policy surface instead of adding a redundant
   `cargo-audit` workflow: the official `cargo-deny` checks cover advisories,
   licenses, banned/duplicate crates, and sources. The exact binary version
   is pinned only after the independent tool review.
2. Check both `Cargo.lock` and `fuzz/Cargo.lock`. Check root and fuzz graphs in
   distinct invocations with the same checked-in policy so a secondary Cargo
   workspace cannot evade the policy.
3. Set `graph.all-features = true`, `licenses.include-dev = true`, and
   `bans.multiple-versions-include-dev = true`. Do not set
   `graph.targets`: cargo-deny then evaluates resolved dependency edges for
   every target, including target-specific packages outside the current CI
   matrix. Capture the CI runner host triples in T001 and verify the policy
   runner's actual host triple at runtime; policy coverage does not depend on
   the Linux runner's host.
4. Require a finite SPDX allowlist. The repository's own `Apache-2.0` license
   is not evidence that every third-party license is acceptable. T001 records
   the exact current graph and reviewed license evidence; an unreviewed
   expression remains denied.
5. Explicitly set cargo-deny levels to `advisories.unsound = "all"`,
   `advisories.unmaintained = "all"`, `advisories.yanked = "deny"`,
   `sources.unknown-registry = "deny"`, `sources.unknown-git = "deny"`,
   `bans.multiple-versions = "deny"`, and `bans.wildcards = "deny"`;
   vulnerability advisories are errors by tool definition. Explicitly allow
   only the reviewed crates.io registry and set `sources.required-git-spec =
   "rev"` so a later reviewed Git source cannot use a mutable branch or tag.
   Do not rely on warning defaults. Ban the exact packages `native-tls`,
   `openssl`, and `openssl-sys` under ADR-0005's rustls TLS-backend decision;
   do not add broad or pattern-based bans. Any necessary exception is exact,
   reviewed, and expires within 90 days.
6. Run `cargo metadata --locked --format-version 1 --all-features` for both
   workspace roots without `--filter-platform`, so metadata includes all
   targets and features like cargo-deny's policy graph. Validate every
   source-less path package against the canonical union of root/fuzz workspace
   members; reject paths or symlink targets outside the repository. Test an
   optional-feature-only path edge. Do not treat cargo-deny's registry and Git
   source check as enforcement for local paths.
7. Give each policy job an isolated temporary `CARGO_HOME`. Run the root and
   fuzz cargo-deny checks online; do not pass `--offline` or `--frozen`.
   cargo-deny refreshes the RustSec database before each check. The runner
   script captures `rustc -vV` and fails if its host triple is not in the
   reviewed target set. After each successful workspace invocation, verify
   the configured RustSec Git remote and record that invocation's database
   commit SHA and timestamp. On schedule, check out the configured active release branch
   because GitHub schedule events execute the workflow from the default
   branch. The daily scheduled CI trigger already exists.
8. Do not alter existing license/source policy in other tooling. OASIS source
   integrity, action SHA pinning, and future release SBOM/provenance/signatures
   remain separate controls.
9. Resolve the T001 fuzz baseline within this feature, after Red tests: declare
   `Apache-2.0` on the unpublished first-party fuzz package and state an
   explicit version requirement matching the existing `kmipkit-ttlv` path
   package. These metadata-only corrections must not change either lockfile or
   the resolved package versions and must not be treated as third-party
   license approval.

## Implementation Phases

### Phase 0: verify baseline and tool

- Record exact release/base SHA, both lockfile paths and graph roots, target
  triples, and existing CI workflow behavior.
- Complete independent review of the exact `cargo-deny` release, install
  path, license, platform support, Rust/MSRV requirements, security history,
  maintenance, and transitive tool dependencies in
  `dependency-review.md`.
- Inspect every current package license/advisory/source/duplicate finding.
  Do not accept existing findings automatically; record an exact disposition
  or leave the implementation gate blocked.
- Confirm no new external GitHub Action is needed and CI permissions remain
  read-only.

### Phase 1: Red commits

- Add failing policy-validator and workflow-contract fixtures before adding
  policy configuration or CI commands.
- Cover both workspace invocations, all finding categories, exact exception
  matching and expiry, locked operation, no secret/write permissions, and
  scheduled release checkout behavior. Include assertions for the existing
  fuzz manifest's FR-013 license/version metadata and the two baseline findings.
- Run the tests and record behavioral failures, not syntax or harness errors.

### Phase 2: Green commits

- Add the reviewed finite policy to `.cargo/deny.toml` and a structured
  exception register with no blanket ignores.
- In the same Green change, apply only FR-013's explicit license and version
  metadata to the existing unpublished fuzz package/path dependency; verify no
  lockfile or resolved-version change occurs.
- Add the standard-library exception validator and its local/CI entry point.
- Add the pinned policy job to `.github/workflows/ci.yml` for every PR; add
  the scheduled job to the existing daily trigger and make it scan the
  explicitly configured active release ref.
- Invoke the checker for both workspaces without updating Cargo manifests or
  lockfiles. Fail when an online refresh cannot complete or on any rule
  violation; emit the fetched RustSec revision and timestamp per workspace.

### Phase 3: Converge and final evidence

- Refactor shared parsing/report logic without changing policy outcomes.
- Update English developer/testing/security documentation and the roadmap's
  Phase A status in the same implementation PR.
- Run `/speckit-converge` after implementation. If it adds buildable tasks,
  complete them with Red/Green/Refactor evidence and repeat convergence.
- After convergence reports no gaps, run focused fixtures, local checks, all
  existing CI checks, and policy validation for supported matrix targets.
  Obtain independent QA and security reviews; if either review adds work,
  return to implementation, convergence, verification, and both reviews until
  clear. Record actual results and limitations; do not claim branch protection
  enforcement.
- Only then open a draft PR. Do not approve or merge it.

## Project Structure

### Documentation (this feature)

```text
specs/011-dependency-policy/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── dependency-review.md
├── implementation-evidence.md (created during implementation)
├── tasks.md
├── contracts/
│   └── dependency-policy.md
└── checklists/
    ├── requirements.md
    ├── policy-review.md
    └── security.md
```

### Source Code (repository root)

```text
.cargo/deny.toml
.github/workflows/ci.yml
fuzz/Cargo.toml
docs/development/testing.md
docs/development/rust-workspace.md
docs/security/dependency-policy.md
scripts/dependency_policy.py
scripts/Test-DependencyPolicy.ps1
scripts/tests/test_dependency_policy.py
scripts/tests/test_workflow.py
specification/compliance/dependency-policy-exceptions.json
```

**Structure Decision**: Keep the policy configuration next to Cargo metadata,
workflow wiring in the existing CI file, a single standard-library validator
under `scripts/`, and exception data with compliance documentation. The
implementation task must confirm no redundant helper or generated artifact
is needed before coding.

## Complexity Tracking

No constitution violations or architectural exceptions are required.
