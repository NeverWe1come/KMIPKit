# Implementation Plan: Explicit Opaque Object cryptographic context for Register

**Branch**: `feature/KMIPKIT-0044-opaque-object-context` | **Date**: 2026-10-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/029-opaque-object-context/spec.md`

## Summary

Add an explicit caller choice to Register so an Opaque Object can remain generic or be handled as a Managed Cryptographic Object for attributes. Preserve normal wire bytes and caller attribute order. In selected crypto context, enforce Table 293 locally; validate the required Table 4 Opaque Data Type and Opaque Data Value fields without interpreting opaque bytes. Map both 2.3 catalog requirements to implementation and tests. For 2.3-003, link the requirement to the existing Table 4 structure-member elements that carry `source_requiredness: Yes` and correct its summary; retain the audited source-clause locator for the `REQUIRED` header row rather than inventing an unsupported row locator.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94; C ABI, Java JNI, and Python CFFI follow the 1.0 public API manifest.

**Primary Dependencies**: Existing KMIPKit crates and the pinned catalog/code-generation tools. No new runtime dependency is needed.

**Storage**: N/A.

**Testing**: Focused protocol unit tests; client fake-transport tests for zero writes and NotSent; generated C/JNI/CFFI parity tests; catalog traceability validator; the repository format, Clippy, test, coverage, security, and generated-output checks.

**Target Platform**: Supported Linux, Windows, and macOS builds; Java and Python runtimes supported by current bindings.

**Project Type**: Rust client library with C ABI and generated language bindings.

**Performance Goals**: No measurable extra network exchanges; Opaque context and required-attribute checks are linear in the supplied attribute/object children and occur before transport.

**Constraints**: KMIP 2.1 and TTLV only in 1.0; no wire context marker; preserve unknown enum values and attribute order; no parsing of Opaque Data Value; no automatic defaults or lookup; local failures are NotSent; never log object payloads or raw KMIP bodies; never edit pinned OASIS files or generated outputs by hand.

**Scale/Scope**: One Register request model and all 1.0 bindings; two §2.3 requirements plus the existing Register Table 291 and Table 293 requirements. No other operations or server behavior.

## Constitution Check

| Gate | Status | Evidence |
| --- | --- | --- |
| Approved bounded specification and exact normative sources | PASS | `spec.md` cites KMIP 2.1 §2.3/Table 4, §6.1.43/Tables 291 and 293, and §11.34/Table 468. |
| Strict Red, Green, Refactor with separate signed commits | PASS | Tasks require failing regression tests first, implementation second, cleanup/docs third. |
| Requirement-to-source/code/test traceability | PASS | IDs 2.3-002 and 2.3-003 plus cross-references to existing Register IDs 6.1.43-001, 6.1.43-007, 6.1.43-009-001/002, and 6.1.43-010-001/002 have explicit mapping tasks. |
| Rust-only core and safe FFI boundaries | PASS | Core work stays in `kmipkit-protocol`; binding surfaces follow the public API manifest and generated workflow. |
| Lossless values and secret redaction | PASS | No opaque data interpretation; generic values preserved; tests and docs prohibit payload logging. |
| No automatic request retry or implicit server policy | PASS | Invalid request preflight is NotSent; valid requests use the normal single request execution path. |
| 1.0 language parity | CONDITIONAL PASS | KMIPKIT-0043 was merged as PR #100 and its manifest is available on `release/1.0.0` (merge commit `b499366b`). This feature now adds its Register context mapping and adapter surfaces; final parity checks remain in progress. |

No constitution exception is requested. The public API manifest dependency is a sequencing gate, not a reason to weaken parity.

## Phase 0: Research

See [research.md](research.md). Source inspection resolved the Table 293 applicability, Table 4 child field shapes, general opaque attribute exemptions, and catalog locator discrepancy. No unresolved product behavior remains.

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/register-opaque-context.md](contracts/register-opaque-context.md), and [quickstart.md](quickstart.md).

Implementation stays in the current Rust workspace and generated binding directories. Protocol validation and request model changes live in `crates/kmipkit-protocol`; client delivery evidence is verified through the existing `crates/kmipkit-client` fake-transport boundary; the catalog input and deterministic generated traceability outputs change together. Binding changes are made through the accepted manifest and pinned generator only.

## Sequencing and release dependencies

1. Land KMIPKIT-0043's public API manifest contract on `release/1.0.0` before changing generated binding outputs. This dependency is satisfied by PR #100 (`b499366b`).
2. Implement the Rust request model and tests with Red, Green, and Refactor commits.
3. Update catalog requirement ownership and the Table 4 locator for KMIPKIT-REQ-SPEC-2.3-003; regenerate all derived inventory and report files.
4. Generate equivalent C, Java, and Python selectors and run their parity tests.
5. Run convergence, independent QA/security review, CI, and cross-platform checks. Open or update a draft PR; do not merge or publish.

## Project Structure

### Documentation (this feature)

```text
specs/029-opaque-object-context/
├── plan.md
├── research.md
├── data-model.md
├── contracts/
│   └── register-opaque-context.md
├── quickstart.md
├── checklists/
│   ├── requirements.md
│   └── conformance.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/kmipkit-protocol/src/register.rs
crates/kmipkit-protocol/tests/unit/register_operation_tests.rs
crates/kmipkit-client/src/
crates/kmipkit-client/tests/
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md       # generated
crates/kmipkit-ffi/                             # generated/public C surface
bindings/java/                                  # generated/public JNI surface
bindings/python/                                # generated/public CFFI surface
```

**Structure Decision**: Extend the existing Register protocol model and validation, using the established fake-transport client boundary and generated binding pipeline. Do not create a separate crate or runtime service.

## Complexity Tracking

No constitution violations or additional architectural components are introduced.
