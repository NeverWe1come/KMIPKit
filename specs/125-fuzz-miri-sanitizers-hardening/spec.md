# Specification: Pre-release hardening — fuzzing, Miri, sanitizers, extended negative tests (KMIPKIT-0125)

## 1. Overview and scope

Closes the release hardening requirements for `1.0.0` ("Fuzzing, Miri, sanitizers, and extended negative testing").
It provides reproducible, automated evidence that decoders, protocol validation boundaries, and FFI layers reject hostile or malformed input without panics, memory corruption, undefined behavior, or resource exhaustion.

## 2. Normative and project references

- OASIS KMIP Profiles v2.1 §5.4.1 (XML), §5.5.1 (JSON), §5.12.3 (Tape Library Alternative Name).
- OASIS KMIP Specification v2.1 §6.2 (Server-to-Client operations).
- AGENTS.md §5 (negative tests for `MUST NOT` / `SHALL NOT`), §6, §7, §8.
- ADR-0017 single-dependency facade, KMIPKIT-0011 dependency policy.
- `docs/development/testing.md` (Scheduled: Miri, sanitizers, bounded fuzzing).

## 3. Requirements

- **KMIPKIT-0125-FR-001 (Fuzz targets)**: The fuzz workspace includes bounded targets for TTLV, extension schemas, JSON decode/encode, XML preflight, document framing, and Server-to-Client / multi-version decoding. All targets enforce strict length and depth bounds.
- **KMIPKIT-0125-FR-002 (Opt-in fuzz hooks)**: Internal codecs and framer components are exposed to fuzz targets via a dedicated, off-by-default `fuzzing` feature with `#[doc(hidden)]` visibility, preserving the public API surface.
- **KMIPKIT-0125-FR-003 (Miri verification)**: Safe crates (`kmipkit-ttlv`, `kmipkit-protocol`) are verified under Miri with strict provenance and symbolic alignment checks.
- **KMIPKIT-0125-FR-004 (Sanitizers)**: AddressSanitizer and UndefinedBehaviorSanitizer verify all C ABI consumers and JNI C++ tests.
- **KMIPKIT-0125-FR-005 (Extended negative tests)**: Hostile input tests verify XML (DTD, entity, depth), JSON (non-hex, overflow, malformed), and framing bounds.
- **KMIPKIT-0125-FR-006 (§5.12.3 closure & contract)**: A negative test verifies non-injection of Alternative Name by the client; decision record `KMIPKIT-DEC-035` records application responsibility. Contract test verifies 100% of in-scope prohibited requirements.
- **KMIPKIT-0125-FR-007 (Hardening campaign & evidence)**: An automated script runs all hardening checks and outputs verified documentation.
