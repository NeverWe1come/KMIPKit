# Feature Specification: Rustls Advisory Remediation

**Feature Branch**: `feature/KMIPKIT-0045-rustls-advisory-remediation`
**Created**: 2026-10-14
**Status**: Approved
**Input**: User authorized autonomous execution of the KMIPKit roadmap and corrective work required by its security gates. The pull-request dependency-policy check reported the pinned Rustls release affected by current RustSec advisories.

## Scope and Exclusions

This feature updates the existing exact Rustls dependency pin and its root lockfile resolution, then records regression, policy, and traceability evidence. It does not change KMIP protocol behavior, TLS policy, Rustls features, public APIs, the dependency-policy implementation, its exception register, or CI rules. It does not add an advisory waiver.

## User Scenarios & Testing

### User Story 1 - Maintain a patched TLS dependency (Priority: P1)

KMIPKit maintainers need the 1.0 client to use a Rustls release with published fixes for every current advisory affecting the pinned release, while keeping the reviewed TLS architecture and reproducible dependency graph.

**Why this priority**: TLS protects KMIP credentials and key-management traffic. A known affected TLS dependency blocks the required dependency-policy gate and weakens the security basis of the release.

**Independent Test**: Run the regression contract against the workspace dependency pin and lockfile, then run the online dependency-policy command against both Cargo workspaces.

**Acceptance Scenarios**:

1. **Given** the current exact Rustls pin is below the published patched release, **When** the regression contract runs, **Then** it fails and identifies the expected minimum patched release.
2. **Given** the pin and lockfile are updated to the reviewed patched Rustls release, **When** the contract and fresh online dependency-policy checks run, **Then** they pass without an advisory exception.
3. **Given** the Rustls release is updated, **When** the existing TLS client builds and its tests run, **Then** it continues to use the approved Rustls and aws-lc-rs backend with the existing TLS behavior.

## Edge Cases

- If a later RustSec refresh reports another affected dependency, the policy check remains failed; this feature does not add a waiver or hide a finding.
- If updating Rustls requires unrelated runtime dependency or feature changes, keep those changes out of scope and report the incompatibility for a separately reviewed decision.
- If the advisory source cannot be refreshed, the dependency-policy check is not considered successful.

## Requirements

### Functional Requirements

- **FR-001**: KMIPKit MUST pin Rustls to a stable 0.23.x release at or above 0.23.46, the minimum patched release stated by the four RustSec advisories affecting 0.23.45 on 2026-10-14.
- **FR-002**: The primary Cargo lockfile MUST resolve the exact reviewed Rustls pin, and the update MUST NOT change unrelated package versions.
- **FR-003**: The dependency-policy check MUST pass for both the root and fuzz workspaces with online advisory refresh and no exception for the identified Rustls advisories.
- **FR-004**: The update MUST preserve the approved Rustls TLS backend and feature set, TLS 1.3-only/mTLS product policy, and existing public API behavior.
- **FR-005**: The change MUST include executable regression coverage, requirement traceability, and implementation evidence for the advisory disposition and verification.

### Key Entities

- **Rustls dependency pin**: The exact stable Rustls version selected by the root Cargo workspace and resolved by its lockfile.
- **Advisory evidence**: The RustSec identifiers, patched-version range, refreshed database evidence, and policy-check result for this change.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All four advisories RUSTSEC-2026-0346, RUSTSEC-2026-0347, RUSTSEC-2026-0348, and RUSTSEC-2026-0349 report no finding against the committed root dependency graph.
- **SC-002**: Both workspace dependency-policy scans complete with a successful online advisory refresh and no new exception or policy relaxation.
- **SC-003**: The regression contract, affected Rust tests, formatting, lint, and required CI checks pass on the final feature head.
- **SC-004**: The existing Rustls backend and declared features are unchanged, and only the intended Rustls package resolution changes in Cargo.lock.

## Assumptions

- RustSec's pinned advisory database on 2026-10-14 identifies Rustls 0.23.46 or later as patched for all four current advisories.
- The exact pin is a maintenance/security correction within the existing TLS architecture; it does not change KMIP wire behavior or supported TLS policy.
- The approved dependency-policy specification remains the authority for how advisory data is refreshed and evaluated.
- This is a dependency-maintenance specification, so OASIS KMIP normative references and conformance vectors do not apply.
