# KMIPKIT-0095: Fluent response representations

**Status**: Approved for implementation.

**Approval provenance**: The maintainer's direct 2026-11-11 instruction approved
implementation of the complete fluent API plan autonomously, including T009.

## Purpose

Let a Rust caller inspect the complete response retained by client execution in
TTLV hex, KMIP JSON, or KMIP XML. The representation must include the common
response header and the ordered response batch, including supported unknown
vendor extension values, while retaining KMIPKit's response ownership and
redaction guarantees.

## Requirements

- **FR-001** `ClientBatchItemResponse` provides explicit methods to render its
  complete retained response as canonical uppercase TTLV hex, KMIP JSON, and
  KMIP XML, using caller-supplied codec limits.
- **FR-002** Each representation includes the complete response header and
  every response batch item in source order. Successfully retained values,
  including non-critical vendor extension payloads, remain present and ordered.
  The root is encoded with the KMIP `ResponseMessage` tag.
- **FR-003** Rendered content is returned only in an explicitly requested
  zeroizing owner. It is excluded from `Debug`, logs, and error text. Errors
  identify only a payload-free encoding category.
- **FR-004** Rendering is a response inspection path. It does not accept a
  caller-authored TTLV request, construct a transport permit, or send data.
- **FR-005** The selected codec limits apply to rendered output and all encoding
  failures remain payload-free.

## Acceptance

1. A checked-in response fixture renders to all three encodings.
2. TTLV output matches the retained header and batch bytes in source order,
   with a `ResponseMessage` root. JSON and XML contain the same ordered header,
   batch, and vendor extension content.
3. Existing response outcome behavior and transport boundaries are unchanged.
4. Focused tests, client crate tests, formatting, export audit, generated API
   manifest, and CI pass on the exact PR head.

## Boundary

This slice adds the Rust response foundation. C, Java, and Python surfaces must
wrap the same Rust-owned response model in their binding deliveries and preserve
their established ownership rules. This slice does not add request encoding,
generic message submission, polling, or automatic retries.

Standard Rust trait implementations for the new error value are Rust language behavior, not separate cross-language KMIPKit product capabilities.

## Normative references

- OASIS KMIP Specification v2.1 §8.4, Table 397: Response Message root and ordered members.
- OASIS KMIP Specification v2.1 §8.5, Table 398: retained Response Header.
- OASIS KMIP Specification v2.1 §8.6, Table 399: ordered Response Batch Items.
- OASIS KMIP Specification v2.1 §9.13, Table 418: Message Extension preservation.
- Codec-specific normative sources and accepted encoding policies remain owned
  by KMIPKIT-0054 through KMIPKIT-0057; this delivery calls those existing codecs.
