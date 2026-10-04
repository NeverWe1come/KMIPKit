# CI Requirements Quality Checklist

**Purpose**: Review clarity, completeness, and testability of the cross-platform CI, WSL, and coverage requirements.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)

**Note**: This is a reviewer-owned requirements-quality checklist. Unchecked items have not been certified by a human reviewer.

## CI matrix and fork security

- [x] CHK001 Does the spec identify the exact target branches, runner operating systems, and Rust toolchains required for every pull request? [Completeness, FR-001–FR-003]
- [x] CHK002 Are token permissions, secrets, trigger types, checkout credential persistence, and fork-code handling explicit enough to distinguish safe from unsafe workflows? [Security, FR-008, FR-010]
- [x] CHK003 Is the prohibition on package publishing and repository-setting changes stated as an observable workflow boundary? [Scope, FR-010]
- [x] CHK004 Are external action pinning and release identification objectively verifiable? [Measurability, FR-012]

## WSL command contract

- [x] CHK005 Are PowerShell version, WSL version, Ubuntu selection, and toolchain prerequisites explicit? [Completeness, FR-004–FR-007]
- [x] CHK006 Does the spec define deterministic behavior for zero, one, and multiple Ubuntu distributions? [Edge Case, FR-007]
- [x] CHK007 Are spaces, Unicode paths, native argument boundaries, output preservation, and exit-code propagation testable? [Testability, FR-005, FR-007]
- [x] CHK008 Does the spec forbid automatic installation or host/distro mutation while giving actionable prerequisite failures? [Safety, FR-005–FR-006]

## Coverage policy

- [x] CHK009 Are the unavailable, measured, and failed coverage outcomes mutually exclusive and objectively defined, including how every platform represents the initial no-code state? [Clarity, FR-009]
- [x] CHK010 Does the changed-code denominator define executable lines, exact base/merge trees, platform aggregation, zero-line `not applicable`, and missing/malformed report behavior? [Measurability, FR-009]
- [x] CHK011 Are workspace, crate-family, and changed-code thresholds all preserved and mapped to the correct source groups? [Consistency, FR-009]
- [x] CHK012 Is branch coverage explicitly informational and prevented from gating until a reviewed policy change? [Scope, FR-011]
- [x] CHK013 Are generated-source exclusions constrained to documented reasons without allowing production code to evade coverage? [Security, FR-009]

## Boundaries and evidence

- [x] CHK014 Does the spec clearly defer OASIS source integrity, dependency/license policy, generation, traceability, bindings, and branch-protection administration to their own applicable work? [Scope, Context and References]
- [x] CHK015 Does every functional requirement map to a measurable outcome and an implementation task? [Traceability, FR-001–FR-012, SC-001–SC-008]
