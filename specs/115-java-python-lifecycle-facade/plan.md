# Implementation Plan: Java and Python Fluent Lifecycle Facade

**Branch**: `feature/KMIPKIT-0115-java-python-lifecycle` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

## Summary

Add matching Java and Python fluent builders and direct client convenience
methods for Activate, Archive, Locate, Recover, and Revoke. Both layers reuse
the already published typed batch-item constructors, response accessors, and
single-exchange client execution paths.

## Technical Context

**Language/Version**: Java 17+ and Python 3.12; Rust ABI remains the request and response owner.
**Primary Dependencies**: Existing `kmipkit-java`, JNI, `kmipkit-python`, CFFI, and typed C ABI.
**Storage**: None.
**Testing**: JUnit, pytest, controlled mutual-TLS fixtures, Java/Python package checks, manifest generation, export audits.
**Target Platform**: Supported KMIPKit Java and Python package platforms.
**Project Type**: Multi-language client library.
**Performance Goals**: One client exchange per `execute`, with no additional request encoding layer.
**Constraints**: No arbitrary TTLV sending, implicit KMIP values, retries, polling, or changes to public ABI layouts.
**Scale/Scope**: Five operations, two language facades, typed result and ownership parity.

## Constitution Check

- Approved Spec Kit specification with exact OASIS KMIP 2.1 sections: PASS.
- Rust remains the single owner of protocol behavior and transport: PASS.
- Existing typed request/response routes and fixed ABI remain unchanged: PASS.
- Strict Red, Green, Refactor commits; tests, traceability, generated exports, and docs: REQUIRED.
- No new architecture boundary or ADR is needed: this adds language facades over existing operations.

## Design

### Java flow

`FluentRequest` will create operation-specific builders. Identifier-only
operations use a shared operation enum in the existing `OperationBuilder`;
Locate and Revoke use focused builders because they have distinct typed input
and ownership rules. Each `build()` creates a fresh `ClientBatchItem`, and
`execute()` uses `KmipClient` once. Direct `KmipClient` methods delegate to
these builders.

### Python flow

`Request` will expose the same operation constructors. Identifier-only
operations use the existing `Operation` shape with operation-specific
extension constructors; Locate and Revoke use focused builders. Direct
`Client` methods accept named parameters and delegate to `Request` builders.
All temporary wrappers are closed in `finally`/context-manager scopes.

### Shared contracts

- `RequestOptions` overrides transport timeouts only; the asynchronous
  indicator remains on the builder and batch header.
- Locate requires caller Attributes and copies them through existing typed
  constructors; optional selectors are never defaulted.
- Revoke snapshots reason code/message and creates fresh native owners for
  repeated build/execute calls. Java and Python cannot guarantee that runtime
  string objects are zeroized; temporary encoded native staging follows the
  existing zeroization rules.
- The request builders return existing `ClientBatchItem`/batch responses and
  never infer remote state from a successful status.

## Project Structure

```text
bindings/java/src/main/java/org/kmipkit/client/KmipClient.java
bindings/java/src/main/java/org/kmipkit/fluent/FluentRequest.java
bindings/java/src/main/java/org/kmipkit/fluent/OperationBuilder.java
bindings/java/src/main/java/org/kmipkit/fluent/LocateBuilder.java
bindings/java/src/main/java/org/kmipkit/fluent/RevokeBuilder.java
bindings/java/src/test/java/org/kmipkit/fluent/FluentLifecycleApiShapeTest.java
bindings/java/src/test/java/org/kmipkit/fluent/FluentLifecycleNativeIntegrationTest.java
bindings/python/src/kmipkit/client.py
bindings/python/src/kmipkit/fluent.py
bindings/python/tests/test_fluent_lifecycle.py
docs/user-guide/en/lifecycle-operations.md
docs/user-guide/es/operaciones-ciclo-vida.md
specification/api/public-api.json
specs/115-java-python-lifecycle-facade/
```

## Verification Strategy

1. Commit failing Java and Python builder/consumer tests before production edits.
2. Implement builders and direct methods; prove positive and negative
   construction, ownership cleanup, redaction, and one-exchange responses.
3. Refactor shared Java/Python execution helpers only after the tests pass.
4. Run Java package tests, Python tests/type checks, both controlled mTLS
   fixtures, format/lint, API manifest generation/check, Java/Python export
   audits, traceability, and the repository's relevant CI matrix.

## Complexity Tracking

No constitution violations or new architectural dependencies.
