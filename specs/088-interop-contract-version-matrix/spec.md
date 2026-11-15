# Specification: Versioned interoperability contract and run matrix

**Feature Branch**: `feature/KMIPKIT-0088-interop-contract-matrix`
**Status**: Approved for implementation by maintainer plan authorization
**Owning identifier**: KMIPKIT-0088
**Dependency**: KMIPKIT-0083 at `fdafef8a3eb0c7b4b18d6433b78fde4151493ceb`.

## 1. Purpose

Correct the shared interoperability contract so each case has OASIS-backed
KMIP 2.0/2.1 applicability and references, and preserve every validator run
when reports are compared across languages and execution routes. This is a
shared-contract change only; it does not modify a language validator or claim
vendor interoperability.

## 2. Normative and project references

- OASIS KMIP Specification Version 2.0, OS (2019-07-19), especially §6.1,
  §6.2, §11.36 Operation Enumeration, and the version-specific sections cited
  by generated cases. Pinned source:
  `specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html`.
- OASIS KMIP Specification Version 2.1, OS (2022-12-15), especially §6.1,
  §6.2, §11.36 Operation Enumeration, §§11.61–11.62 Validation enumerations,
  and the version-specific sections cited by generated cases. Pinned source:
  `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.
- KMIPKIT-0083, ADR-0021 and ADR-0022; `AGENTS.md` §§5–6, 8 and 10.

The generator SHALL use only these checked-in, checksum-verified OASIS files;
it SHALL NOT download or scrape remote documents.

## 3. Requirements

| Requirement | Requirement statement | Verification |
|---|---|---|
| KMIPKIT-0088-FR-001 | Every generated case SHALL list exactly the KMIP versions in which its operation, attribute, object type, enumeration, or bit mask is defined by the pinned normative sources. The OASIS inventories SHALL be audited against all shared catalog rows. | Source-inventory coverage tests and generated catalog checks. |
| KMIPKIT-0088-FR-002 | Every case SHALL carry exact OASIS document/section references for each applicable target version. A KMIP 2.1-only case SHALL NOT be represented as KMIP 2.0 applicable merely because a KMIP 2.1 catalog entry exists. | Versioned section-reference tests for all generated rows. |
| KMIPKIT-0088-FR-003 | Negative cases SHALL list their allowed KMIP Result Reason values when the source case defines a bounded set. A validator SHALL be able to distinguish a conformant negative response from an unexpected response without interpreting prose. | Catalog schema, generator, and negative-case contract tests. |
| KMIPKIT-0088-FR-004 | A connection configuration SHALL provide a stable 64-hex `server_configuration_fingerprint` identifying one deployed server configuration. The value SHALL remain the same across runs that change only protocol version, transport, encoding, API route, or language. | Connection schema and stable-fingerprint tests. |
| KMIPKIT-0088-FR-005 | Aggregation SHALL preserve every input report, including repeated runs from the same language/server, and SHALL group matrices by server and KMIP protocol version. | Multi-run aggregation tests. |
| KMIPKIT-0088-FR-006 | Comparability SHALL require matching server identity/version, server configuration fingerprint, protocol version, catalog hash, and KMIPKit library commit. A row SHALL NOT claim parity unless its matrix is comparable. For comparable rows, aggregation SHALL compare execution status, KMIP result status/reason wire codes, library correctness, server compatibility, and observed parity. Named Result Status/Reason values SHALL be normalized to their numeric KMIP codes before comparison. | Positive and negative evidence tests, including enum-name aliases and incomplete evidence. |
| KMIPKIT-0088-FR-007 | JSON, Markdown, and HTML reports SHALL retain execution dimensions and show library correctness, observed server compatibility, differences across languages/routes, comparability state, and cases not executed. Reports SHALL NOT contain secret or raw KMIP body data. | Multi-axis renderer and redaction tests. |
| KMIPKIT-0088-FR-008 | The common CI workflow SHALL regenerate the catalog and run all shared contract, schema, aggregation, and renderer tests. | GitHub Actions and local test suite. |

## 4. Scope and exclusions

This specification covers KMIP 2.0 and 2.1 OASIS standard content only. It
does not add KMIP 1.x, KMIP 3.0, vendor extensions, language implementation
code, or server compatibility claims. A shared catalog's planned case is not
evidence of server support.

The initial audit identifies six operations in the KMIP 2.1 Operation
Enumeration that have no matching KMIP 2.0 client-operation heading or
enumeration value: Get Constraints, Ping, Process, Query Asynchronous Requests,
Set Constraints, and Set Defaults. KMIP 2.0 has 50 in-scope client operation
headings; the other 50 are shared with KMIP 2.1. Vendor Attribute is defined
in KMIP 2.0 §4.53 and KMIP 2.1 §4.60. Validation Authority Type and Validation
Type are also defined in KMIP 2.0 §§11.61–11.62 and in KMIP 2.1 §§11.63–11.64.
These are source-audit inputs; the generator's output and tests are
the authoritative implementation evidence.

## 5. Acceptance

The catalog must regenerate deterministically, cover all source-backed rows,
and fail if a version/reference/reason assignment is missing or stale. The
aggregator must preserve all reports and render their route dimensions without
overwriting same-language runs. Human review and merge remain maintainer
responsibilities.
