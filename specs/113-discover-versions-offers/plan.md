# Implementation Plan: Configurable Discover Versions Offers

**Branch**: `feature/KMIPKIT-0113-discover-versions-offers` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification in `specs/113-discover-versions-offers/spec.md`.

## Summary

Add explicit ordered offers, including KMIP 2.0, and an explicit omitted-field
request across Rust, C, Java, and Python. Preserve the current no-argument
KMIP 2.1 offer and keep Message Header version selection independent. Update
the generated public API manifest and its language bindings through the pinned
repository generator.

## Technical Context

**Language/Version**: Rust Edition 2024, C ABI, Java 17, Python 3.12.

**Primary Dependencies**: Existing `kmipkit-protocol` Discover Versions model,
`kmipkit-client` typed batch request path, `kmipkit-ffi`, generated public API
manifest and wrapper generators.

**Storage**: None.

**Testing**: Rust protocol/client tests, C ABI tests compiled and called from
C, Java and Python public-package tests, controlled-peer request/response
tests, and generated-artifact checks.

**Target Platform**: Supported KMIPKit platforms and all four language
packages.

**Project Type**: Multi-crate protocol library with FFI and language bindings.

**Performance Goals**: No additional network round trip and no response or
request copying beyond current typed encoding/decoding paths.

**Constraints**: The 2.1 default must remain unchanged; 2.0 is explicit;
offers must not select the Message Header version; no auto-negotiation or
retry; preserve OASIS field order/repetition; API inventory and wrappers are
generated, not hand-edited.

**Scale/Scope**: One typed operation's request construction and response
validation across four public language surfaces.

## Constitution Check

- **Spec and traceability**: Draft requirements map to exact v2.0/v2.1 OASIS
  sections and ADR-0022. Gate: maintainer approval before implementation.
- **TDD**: Red, Green, and Refactor evidence must be separate commits.
- **Language parity**: All four public surfaces and C ABI consumer tests are
  required.
- **Security**: Offers contain only public protocol version numbers; errors
  remain redacted; no request body is logged.
- **Generated outputs**: API manifests, headers, and wrappers are regenerated
  by pinned tools and checked for diffs.
- **Governance**: Dedicated branch from exact release pin; no approval or
  merge by implementer.

## Phase 0 — Research Findings

See [research.md](research.md). Repository inspection found that the Rust
protocol request already accepts a non-empty vector of catalogued versions,
but the client batch builder exposes only the KMIP 2.1 default. The C, Java,
and Python batch-item constructors are also no-argument-only. The protocol
request model currently uses the empty internal vector to mean “default
KMIP 2.1”; representing an omitted field therefore needs a distinct state.
Response conversion currently validates every successful response against a
non-empty offered slice, so omission requires a distinct no-intersection
validation path.

## Phase 1 — Design and Contracts

- [data-model.md](data-model.md) distinguishes default, explicit offer list,
  and omitted offer field.
- [contracts/discover-versions-api.md](contracts/discover-versions-api.md)
  defines common semantics and proposed language API shapes.
- [quickstart.md](quickstart.md) gives controlled-peer validation scenarios.

## Project Structure

```text
crates/kmipkit-protocol/src/discover_versions.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-ffi/src/extension_registry.rs
crates/kmipkit-ffi/src/extension_registry/client_abi.rs
crates/kmipkit-ffi/tests/
bindings/c/include/kmipkit.h                 # generated
bindings/c/tests/
bindings/java/src/main/java/org/kmipkit/extensions/
bindings/java/src/test/java/org/kmipkit/
bindings/java/native/kmipkit_jni.cpp
bindings/python/src/kmipkit/
bindings/python/tests/
specs/public-api/                            # canonical manifest inputs
```

**Structure Decision**: Extend the existing typed Discover Versions request
and ClientBatchItem path. Do not add a second networking or negotiation layer.

## Implementation Phases (after approval)

1. **Red — protocol semantics**: Add failing tests for default preservation,
   explicit 2.0/2.1 order, omitted field, invalid offered versions, response
   subset enforcement, and unknown versions in no-offer responses.
2. **Green — Rust protocol/client**: Represent all three request states,
   route them through typed batch execution, and validate responses against an
   optional offer list.
3. **Refactor — Rust**: Clarify request accessors and response validation
   without altering the wire contract; document public APIs.
4. **Red — language boundaries**: Add failing C, Java, and Python consumer
   tests for matching default/explicit/omitted behavior and invalid input.
5. **Green/Refactor — FFI and bindings**: Add additive C ABI functions,
   JNI/Python wrappers and regenerate all public API artifacts.
6. Add cross-language normalized tree comparisons, deterministic TTLV byte
   goldens, 2.0/2.1 header-independence tests, and follow-up request checks.
7. Update OASIS traceability, user documentation, API parity inventory, and
   generated declarations. Run the exact required CI workflows and coverage.

## Complexity Tracking

No constitution exception is proposed. The separate omitted-field state is
required to distinguish OASIS's optional Protocol Version field from the
existing no-argument KMIP 2.1 default.
