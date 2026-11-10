# Implementation Plan: KMIPKIT-0050 Parameters, Request Authentication, and Direct Client Operations

**Branch**: `feature/KMIPKIT-0050-parameters-auth-client` | **Date**: 2026-11-05 | **Spec**: [spec.md](spec.md)

**Input**: Approved Phase 5 feature description in [spec.md](spec.md)

## Summary

Add a public typed Cryptographic Parameters value/builder for all 18 Table 59 members, retain the current generic TTLV APIs, attach optional caller-owned `Authentication` to a single `ClientBatch` Request Message, and close the inventory of 56 direct Rust `Client` methods and `_with_options` variants. All request construction routes delegate to existing typed batch and operation code.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94.
**Primary Dependencies**: Existing KMIPKit protocol, TTLV, client, and zeroization types; no new dependencies expected.
**Storage**: None. Authentication and parameters are caller-owned, per-call values.
**Testing**: Rust unit/integration tests with fake transport; compile-time public API consumer; manifest and source audits.
**Target Platform**: Rust workspace platforms already supported by KMIPKit.
**Project Type**: Rust client library.
**Performance Goals**: No additional exchange, retry, Poll, or credential lookup; direct methods perform one exchange.
**Constraints**: Preserve unknown TTLV values, generic `Structure` call sites, redaction, zeroization, TLS 1.3/mTLS boundaries, operation payload behavior, delivery-state semantics, and async Pending/correlation behavior.
**Scale/Scope**: 18 known Cryptographic Parameters members, 56 client-initiated operation values, 14 missing direct methods, and 10 missing option variants.

## Constitution Check

- Client-only KMIP v2.1 scope and TTLV encoding are unchanged.
- Authentication remains optional and caller-supplied; no credentials are retained in `ClientConfiguration` or `Client`.
- Rust APIs are additive; generic TTLV and existing operation methods remain supported.
- Secret values remain redacted and owned secret memory must be zeroized; no raw request body enters errors or logs.
- Tests and traceability must cite exact OASIS clauses, and generated manifest outputs must use the official generator.
- No unsafe, transport, TLS policy, FFI, language-binding, or server behavior is added.

## Design Decisions

1. **Typed parameters**: `CryptographicParameters` owns a validated original `Structure`; `CryptographicParametersBuilder` stores one typed value per known Table 59 member and emits members in the exact source-table order: Block Cipher Mode, Padding Method, Hashing Algorithm, Key Role Type, Digital Signature Algorithm, Cryptographic Algorithm, Random IV, IV Length, Tag Length, Fixed Field Length, Invocation Field Length, Counter Length, Initial Counter Value, Salt Length, Mask Generator, Mask Generator Hashing Algorithm, P Source, and Trailer Field. An explicit extension path retains unknown `Item`s. Parsing validates known member type, singleton count, and the order of known fields; `with_ttlv`/`into_ttlv` preserve the generic structure and unknown field positions.
2. **Existing operation integration**: Keep all existing methods taking `Structure`. Add named typed setters that consume `CryptographicParameters` for Encrypt, Decrypt, Derive Key, Derivation Parameters, Sign, Signature Verify, MAC, MAC Verify, Encryption Key Information, and MAC Signature Key Information.
3. **Authentication**: Store `Option<Authentication>` only in `ClientBatch`. The common Request Header writer moves it once after Attestation Type and before Batch Error Continuation Option, matching KMIP v2.1 §8.2 Table 395. A one-item `ClientBatch` is the request-scoped form; no duplicate direct-method credential arguments or client-global precedence rules are introduced. The validated type cannot represent empty Authentication, so malformed generic structures fail during typed construction before attachment. Follow ADR-0012: the candidate callsite test proves the zeroizing request owner survives through transport success and partial-write error, is cleared after return, reports delivery state, and does not retry. Keep the PR draft until CI passes this test against the candidate callsite.
4. **Direct methods**: Add each missing direct method by converting its typed request into an existing `ClientBatchItem` and using the same internal exchange path. Add `_with_options` to all 14 new direct methods and the 10 existing methods identified in `gap-matrix.md`.
5. **Manifest**: Add every newly exposed public API path to the reviewed manifest using exact operation/requirement capabilities or narrowly justified exact exclusions for trait-generated methods. Regenerate outputs only with `tools/api_manifest/generate.py`.

## Project Structure

```text
crates/kmipkit-protocol/src/cryptographic_parameters.rs
crates/kmipkit-protocol/src/{encrypt,decrypt,derive_key,sign,signature_verify,mac,mac_verify,key_wrapping}.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-protocol/tests/cryptographic_parameters_public.rs
crates/kmipkit-protocol/tests/cryptographic_parameters_integration.rs
crates/kmipkit-client/tests/client_authentication.rs
crates/kmipkit-client/tests/direct_operation_methods.rs
tools/api_manifest/manifest.json and generated outputs
specs/050-parameters-auth-client/*
```

## Verification Strategy

- Establish Red tests before production changes for typed parameter API, authentication header output, and all missing direct methods/options.
- Reuse the existing fake transport and request decoder to prove exact header scope, one exchange, no retries, result/delivery parity, and redaction.
- Run all protocol/client focused tests, formatting, Clippy, relevant workspace tests, coverage gates available locally, public-route audit, manifest generator `--check`, and traceability checks.
- Keep the stacked PR in draft if hosted cross-platform CI or any required coverage control is unavailable.

## Risks

- `ClientBatch` currently owns typed operation values but no authentication. A new secret-bearing field must be carried through internal validation, profile clones, serialization, and drop paths without unintended secret copies.
- Typed parameter parsing must validate only schema-defined fields and keep unknown extensions open and accessible.
- Direct wrapper signatures must match the existing request/error/options conventions, especially Ping and asynchronous operations.
- No test or local Windows run can stand in for the required hosted matrix or independent interoperability evidence.
