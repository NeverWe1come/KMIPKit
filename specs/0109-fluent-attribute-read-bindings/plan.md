# Implementation Plan: Fluent Attribute Read Bindings

- **Branch**: `feature/KMIPKIT-0109-fluent-attribute-read-bindings`
- **Date**: 2026-11-12
- **Spec**: `specs/0109-fluent-attribute-read-bindings/spec.md`

## Summary

Add semantic Get Attributes and Get Attribute List entry points to C, Java,
and Python on top of the KMIPKIT-0108 Rust builders and existing typed
operation models. C calls the Rust-owned execution path through new fixed-width
ABI functions. Java and Python provide idiomatic builders that internally
create one typed batch item and execute through their existing clients.

## Technical Context

- **Languages**: Rust 2024/MSRV 1.94 for FFI; C ABI; Java; Python.
- **Dependencies**: Existing `kmipkit-client`, `kmipkit-ffi`, CFFI/JNI layers,
  typed response wrappers, controlled TLS fixtures, and API manifest generator.
- **Storage**: None.
- **Testing**: Rust FFI unit tests, C consumer tests, Java JUnit/JNI consumer
  tests, Python pytest/CFFI tests, and controlled TLS tests.
- **Constraints**: No new transport, codec, retry, Poll, or raw-message API.
  Existing fixed-width statuses and handle ownership remain unchanged.
- **Scope**: Two read operations across three bindings; Rust builder behavior
  is inherited from KMIPKIT-0108.

## Constitution Check

- Approved feature specification with exact KMIP 2.1 clauses: satisfied.
- Rust owns protocol behavior; language facades reuse typed requests: satisfied.
- TDD and distinct Red/Green/Refactor commits: required before review.
- Public API generation and bilingual user documentation: required.
- No architecture or protocol boundary change: satisfied.

## Project Structure

- `crates/kmipkit-ffi/src/extension_registry/client_abi/fluent.rs`: direct C
  execution functions, validated inputs, and fixed-width status mapping.
- `bindings/c/include/kmipkit.h` and C consumer tests: stable public ABI and
  compiled consumer coverage.
- `bindings/java/src/main/java/org/kmipkit/fluent/`: operation builders and
  `FluentRequest` entry points; Java consumer/JNI tests under `src/test`.
- `bindings/python/src/kmipkit/fluent.py` and `client.py`: builders and
  request facade; pytest consumer tests under `bindings/python/tests`.
- `specification/api/public-api.json`: generator input; generated API output
  is updated only by `tools/api_manifest/generate.py`.
- `docs/user-guide/{en,es}/client-execution.md`: language usage examples.

## Delivery Sequence

1. Add failing C, Java, and Python consumer/controlled-transport tests.
2. Implement the direct C execution functions over existing typed request
   constructors and the single-exchange execution helper.
3. Add Java builders and `FluentRequest` methods, preserving existing response
   wrappers and resource closing behavior.
4. Add Python builders and request methods, preserving type validation,
   exceptions, and context-managed response ownership.
5. Update manifest inputs, generate all exports and parity fixtures, update
   traceability and both user guides.
6. Run language-specific suites, Clippy, coverage, API generation/audits, then
   open a `[WIP]` PR and resolve all CI failures on its exact SHA.

## Complexity Tracking

No constitution violations or architectural exceptions are introduced.
