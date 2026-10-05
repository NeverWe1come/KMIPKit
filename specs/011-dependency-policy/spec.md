# Feature Specification: Dependency Policy Gates

**Feature Branch**: `feature/KMIPKIT-0011-dependency-policy-spec`
**Created**: 2026-10-05
**Status**: Draft
**Input**: Complete KMIPKit roadmap, Phase A infrastructure gap, and the existing every-PR dependency, advisory, source, and license policy requirement.

## Context and scope

KMIPKit's testing guide requires dependency, advisory, source, and license
policy checks on every pull request, and scheduled vulnerability review. The
current CI checks Rust quality, cross-platform behavior, coverage, and source
integrity, but does not yet enforce the Cargo dependency policy. This feature
defines those checks for the existing Rust workspace and fuzz workspace.

This is repository-infrastructure work, not a KMIP protocol feature. OASIS
requirements and conformance vectors are not applicable. It does not change
the product architecture, runtime dependencies, supported platforms, or
published package behavior.

The current fuzz-workspace baseline also requires two narrowly scoped
first-party metadata corrections: declare the existing unpublished fuzz
package's `Apache-2.0` license and add a version requirement matching the
existing `kmipkit-ttlv` path package. These corrections are part of this
feature, must follow the Red/Green test sequence, and must preserve both
lockfiles and the resolved package versions. They do not change published or
runtime dependencies.

## User Scenarios & Testing

### User Story 1 - Detect an unacceptable dependency before merge (Priority: P1)

A maintainer reviews a pull request that changes a manifest or lockfile. The
same policy checks run for every pull request and identify a vulnerable,
unlicensed, disallowed-license, duplicated, banned, or untrusted-source crate
before the change is accepted.

**Why this priority**: An unreviewed dependency can affect the security,
licensing, and reproducibility of every KMIPKit consumer.

**Independent Test**: Run the policy command against fixtures representing
each denied condition and confirm a nonzero result; run it against an accepted
dependency graph and confirm success without modifying either lockfile.

**Acceptance Scenarios**:

1. **Given** a pull request with a dependency containing a denied advisory,
   **When** its policy job runs, **Then** the job fails with the affected
   package and advisory identifier.
2. **Given** a dependency with a license outside the explicit reviewed
   allowlist or without usable license evidence, **When** the policy job runs,
   **Then** the job fails and does not infer approval from KMIPKit's own
   `Apache-2.0` license.
3. **Given** an unapproved registry or Git source, a denied package, or an
   unapproved duplicate version, **When** the policy job runs, **Then** the job
   fails and identifies the matching policy rule.
4. **Given** either Cargo workspace, **When** the policy job runs, **Then** it
   checks the root and fuzz lockfiles, development dependencies, and every
   supported target represented by the resolved dependency graphs without
   rewriting manifests or lockfiles.
5. **Given** the existing unpublished fuzz package, **When** its reviewed
   metadata remediation is applied, **Then** its license matches the
   repository's `Apache-2.0` license, the existing local `kmipkit-ttlv` path
   edge states that package's current version, cargo-deny no longer reports
   these two metadata findings, and neither lockfile nor resolved package
   version changes.

### User Story 2 - Review a narrowly scoped policy exception (Priority: P1)

A maintainer encounters a finding that cannot be fixed immediately. They can
see its exact scope, rationale, mitigation, owner, and expiry in a reviewed
record, and the corresponding exception cannot silently approve other crates,
versions, sources, or advisories.

**Why this priority**: A policy exception must remain auditable and bounded
without weakening checks for future dependency changes.

**Independent Test**: Add a fixture exception and verify it applies only to
the stated package/version or advisory; verify missing, expired, broad, or
undocumented exceptions fail the policy validation.

**Acceptance Scenarios**:

1. **Given** a current, documented exception matching one exact finding,
   **When** the policy runs, **Then** only that finding is accepted and the
   report includes the exception reference.
2. **Given** an exception that is expired, unbounded, or has no matching
   review record, **When** the policy runs, **Then** the policy job fails.
3. **Given** a new version or different crate that was not named in an
   exception, **When** the policy runs, **Then** the exception does not apply.

### User Story 3 - Reproduce and refresh the dependency review (Priority: P2)

A contributor can run the same pinned policy locally, and scheduled automation
checks the active release dependency graph against refreshed advisory data even
when no manifest or lockfile has changed.

**Why this priority**: A pull-request-only scan can miss advisories published
after the last dependency change.

**Independent Test**: Follow one documented command from a clean checkout and
confirm it checks both lockfiles; verify the scheduled workflow is configured
to run at least weekly against the active release graph and fails on an
unexcepted finding.

**Acceptance Scenarios**:

1. **Given** a clean checkout with the pinned policy tool available, **When** a
   contributor follows the documented local command, **Then** it runs the same
   policy checks and exits with the same finding categories as CI.
2. **Given** no source change for a week, **When** scheduled policy review
   occurs, **Then** the latest advisory data is refreshed and findings are
   reported against the active release branch.

### Edge Cases

- Cargo metadata is incomplete, malformed, or inconsistent with a lockfile:
  fail closed and preserve the original files.
- A dependency has an SPDX expression containing alternatives or conjunctive
  licenses: apply SPDX expression semantics; do not reduce it to a guessed
  single license.
- A package has no machine-readable license evidence: fail until the exact
  package is reviewed and a narrow clarification is recorded.
- The advisory source is unreachable, or only cached advisory data was
  available without a successful online refresh during this run: do not report
  a successful fresh scan; fail the pull-request check or identify the
  scheduled run as unsuccessful. A fetched database commit may be older than
  seven days if RustSec has published no newer commit; freshness is established
  by the successful online refresh attempt, not by guessing from commit age.
- A Git dependency points to a mutable branch or tag: reject it; any future
  Git source exception must pin an immutable revision and have a reviewed
  policy record.
- A tool installation or policy database update fails: surface the failure;
  do not turn it into a pass or skip.

## Requirements

### Functional Requirements

- **FR-001**: The policy MUST evaluate the complete resolved dependency graphs
  for the primary and fuzz Rust workspaces, including development dependencies
  and target-specific packages represented in their lockfiles, with all
  declared features enabled.
- **FR-002**: Pull-request CI MUST check both committed lockfiles without
  changing Cargo manifests or lockfiles; a missing or stale lockfile MUST fail.
- **FR-003**: The policy MUST reject every dependency license expression that
  is not covered by an explicit, finite, reviewed SPDX allowlist or an exact
  documented exception. Missing or invalid license data MUST fail unless an
  exact exception provides human-reviewed license evidence and approval; a
  waiver without that evidence MUST fail.
- **FR-004**: The policy MUST fail on known vulnerability and unsoundness
  advisories, unmaintained advisories, and yanked package versions unless a
  current exact exception is recorded. Advisory data MUST be refreshed during
  pull-request and scheduled runs; unavailable data or cached-only operation
  MUST NOT produce a successful fresh-scan result. CI MUST use online advisory
  refresh mode and MUST NOT use `--offline` or `--frozen` for this check. The
  run MUST emit the RustSec database commit SHA and commit timestamp used by
  each successful workspace check.
- **FR-005**: The policy MUST reject dependency sources outside the explicitly
  allowed registry and local-workspace path sources. Every path-sourced
  package MUST resolve, after symlink canonicalization, to a manifest in the
  union of root and fuzz workspace members. Paths outside the checkout and
  symlink escapes MUST be rejected without exception. Git dependencies and
  additional registries MUST be denied by default; a future Git-source
  exception MUST name an immutable revision and have a review record.
- **FR-006**: The policy MUST always reject the exact crates `native-tls`,
  `openssl`, and `openssl-sys` under the accepted rustls TLS-backend decision
  in ADR-0005; exceptions MUST NOT waive these architecture bans. It MUST
  reject wildcard version requirements without exceptions and duplicate
  package versions including development dependencies unless a narrow,
  documented exception covers the exact versions. Each banned crate MUST
  cite ADR-0005; this feature MUST NOT invent general-purpose or
  pattern-based package bans.
- **FR-007**: Every exception MUST name the affected rule and exact package,
  version, source, or advisory; include a rationale, mitigation, review date,
  owner, and expiry; and be checked for expiry and matching scope. Blanket
  ignores and undocumented exceptions MUST fail validation.
- **FR-008**: The policy tool MUST be pinned to an exact reviewed version and
  installed without an unpinned third-party GitHub Action. Tool upgrades MUST
  update the recorded version and independent tool review together.
- **FR-009**: Pull-request policy jobs MUST use read-only repository
  permissions, must not access secrets, and MUST run for all pull requests
  targeting `master` or `release/*` regardless of which files changed.
- **FR-010**: A scheduled workflow MUST recheck the active release dependency
  graphs against refreshed advisory data at least once every seven days and
  expose a failure rather than silently skip it.
- **FR-011**: Contributor documentation MUST provide one reproducible local
  command covering both workspaces and explain how policy exceptions and tool
  upgrades are reviewed.
- **FR-012**: The policy report MUST identify each finding by package, version,
  a safely redacted source reference, license or advisory as applicable, and
  policy rule; it MUST NOT emit credentials or secret material.
- **FR-013**: The implementation MUST correct the existing unpublished fuzz
  package metadata by declaring `Apache-2.0` and giving its existing local
  `kmipkit-ttlv` dependency an explicit version matching that package. These
  first-party metadata corrections MUST be covered by failing tests before
  the Green commit and MUST leave both lockfiles and resolved package versions
  unchanged.

### Explicit Exclusions

- OASIS source immutability and normative catalog generation checks, which
  already have dedicated validation.
- Repository rulesets, branch protection, secrets, credentials, or other
  administrator-controlled settings.
- Package publishing, artifact signing, SBOM/provenance attestation, and full
  reproducible-build verification, which remain in release-hardening scope.
- Maven, PyPI, Java, Python, or future language-package dependency policy.
- Automatic dependency updates, vulnerability remediation, or changes to
  application/runtime dependencies as part of this infrastructure feature.
  The only manifest edits in scope are the two metadata corrections to the
  existing unpublished fuzz package and its existing local path dependency
  described by FR-013.
- `cargo-vet` supply-chain audit records or a second redundant advisory tool.
- Legal advice or a claim that automated license metadata is a complete
  source-code license audit.

### Key Entities

- **Dependency graph**: A lockfile-backed set of workspace and transitive
  packages, including the root and fuzz workspaces.
- **Policy finding**: A package-scoped advisory, license, version, ban, or
  source violation reported by a policy check.
- **Policy exception**: A reviewed, expiring record that narrowly suppresses
  one exact class of finding for a named package/version/source or advisory.
- **Policy run**: A pull-request or scheduled evaluation with a pinned tool and
  refreshed advisory database, whose status is visible in CI.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every pull request to `master` or `release/*` runs policy checks
  against both lockfiles, independent of changed-file filters.
- **SC-002**: Every test fixture for vulnerable, unsound, unmaintained, yanked,
  disallowed-license, unknown-registry, unapproved-Git-source, external or
  symlink-escaping local path, banned-package, normal/dev duplicate-version,
  and expired-exception cases fails before implementation is considered ready.
- **SC-003**: A clean accepted graph passes without any manifest or lockfile
  or lockfile diff caused by a policy invocation; all denied test cases fail
  with a rule-specific finding.
- **SC-004**: The scheduled workflow's interval is no longer than seven days,
  and its successful result records the active-release ref and the RustSec
  advisory database commit SHA and timestamp used by each workspace check.
- **SC-005**: All exception records are exact, documented, unexpired, and
  traceable to a review; there are no blanket ignores.
- **SC-006**: A contributor can run the complete local policy check using one
  documented command and the exact CI tool version.
- **SC-007**: The policy job uses no secrets and requests no repository write
  permission.
- **SC-008**: After FR-013 is implemented, the root and fuzz policy runs no
  longer report a wildcard for the existing local fuzz dependency or missing
  license data for `kmipkit-ttlv-fuzz`, while both committed lockfiles and the
  resolved package versions remain unchanged by the metadata correction.

## Assumptions

- The current policy applies to Rust manifests and lockfiles only; language
  adapters do not yet have package manifests.
- The latest active release branch is the authoritative graph for scheduled
  scans; pull-request CI evaluates its own checked-out merge tree.
- Local path dependencies are allowed only when Cargo metadata identifies the
  package manifest as a member of the root or fuzz workspace; the known fuzz
  edge to the root `kmipkit-ttlv` crate remains valid.
- The fuzz package is first-party, unpublished, and covered by the repository's
  Apache-2.0 `LICENSE`; its isolated Cargo workspace must repeat that metadata
  explicitly. The path dependency's requirement will match the current local
  package version and must be intentionally updated if that package version
  changes later.
- `cargo-deny` is the preferred unified checker because its supported checks
  cover advisories, licenses, banned/duplicate packages, and dependency
  sources. The exact release pin and all configuration semantics will be
  independently verified before implementation.
- An explicit policy allowlist is intentionally stricter than inferring
  dependency approval from the project's `Apache-2.0` package license.
- This feature defines a CI check, but required-check enforcement depends on
  administrator-managed GitHub branch rules and is outside this PR.

## Clarification outcome

No `[NEEDS CLARIFICATION]` markers remain. The chosen tool family, workspace
scope, CI targets, exception boundary, and exclusions follow the existing
testing guide, the accepted project constraints, and current official tool
documentation. The T001 scan's two first-party fuzz metadata findings are now
bounded by FR-013; the exact third-party license allowlist and tool release
pin remain implementation inputs that must be recorded with their independent
review. The repository license does not silently approve third-party licenses.
