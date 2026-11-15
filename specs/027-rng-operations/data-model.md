# Data Model: KMIP 2.1 Remote RNG Operations

Sources: OASIS KMIP Specification v2.1 §§6.1.49–6.1.50, Tables 316–321; §11.36 defines operation values. Shared Data structure ownership remains with its current catalog feature assignment.

## RNG Retrieve

| Direction | Field | Cardinality | TTLV/model and handling |
|---|---|---|---|
| Request | Data Length | exactly 1 | Integer; preserve the supplied quantity without inventing a range rule not present in the approved source. |
| Success response | Data | exactly 1 | Byte String; opaque secret, preserve exact bytes, redact, and zeroize owned memory. |

## RNG Seed

| Direction | Field | Cardinality | TTLV/model and handling |
|---|---|---|---|
| Request | Data | exactly 1 | Byte String; caller-provided secret, preserve exact bytes, redact, and zeroize owned memory. |
| Success response | Data Length | exactly 1 | Integer; preserve zero as a successful value per KMIPKIT-REQ-SPEC-6.1.50-001. |

## Shared result and invariants

- Operation values: RNG Retrieve 0x25; RNG Seed 0x26, as cataloged.
- Common Result Status/Reason/allowed Message, Pending, correlation, delivery evidence, and generic unknown fields are preserved.
- A call sends one request; no retry, implicit Poll, local random generation, or local cryptographic operation occurs.
- Generic TTLV remains available. No response bytes are logged or formatted.
