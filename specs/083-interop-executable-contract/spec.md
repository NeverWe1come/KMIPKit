# Specification: Executable KMIP 2.0/2.1 Interoperability Contract

**Feature Branch**: `feature/KMIPKIT-0083-interop-executable-contract`
**Status**: Approved for implementation by maintainer plan authorization
**Owning identifier**: KMIPKIT-0083

## 1. Purpose

Turn the shared interoperability catalog into a versioned, auditable contract
consumed by independent Rust, C, Java, and Python validators. The validators
will use installed KMIPKit packages and report observed behavior against a
configured KMIP 2.0 or 2.1 server. This suite is not a certification claim.

## 2. Normative References

- OASIS KMIP 2.0 Specification, §§2–9.
- OASIS KMIP 2.1 Specification, §§2–9.
- OASIS KMIP Profiles 2.0 and 2.1, applicable profile clauses for selected
  encoding, transport, and operation cases.
- KMIPKit ADR-0021 and KMIPKIT-0066 for supported server-initiated operations.
- KMIPKit ADR-0022 and KMIPKIT-0069–0070 for version selection and version
  boundaries.
- `AGENTS.md` §§5, 6, 8, and 9 for traceability, secure test handling, and
  prohibition on automatic request retry.

## 3. Requirements

| Requirement | Requirement statement | Verification |
|---|---|---|
| KMIPKIT-0083-FR-001 | The shared catalog SHALL identify target protocol versions and SHALL give every case a stable identifier, exact OASIS document and section, preconditions, execution steps, assertions, mutation classification, and implementation state. | Catalog schema and generated-catalog tests. |
| KMIPKIT-0083-FR-002 | The catalog SHALL cover the normative in-scope client-to-server and server-to-client operation inventories, managed objects, attributes, enumerations, bit masks, version gates, negative cases, and representative combinations. | Catalog coverage checks against the checked-in normative catalog. |
| KMIPKIT-0083-FR-003 | A run SHALL probe all version-applicable cases even when a static vendor profile does not advertise the capability. A profile SHALL be treated as a test hint, not measured evidence. | Capability-selection tests and result aggregation tests. |
| KMIPKIT-0083-FR-004 | Mutating cases SHALL require explicit confirmation of a dedicated test environment and explicit mutation enablement. The runner SHALL NOT automatically retry a KMIP request. | Configuration schema tests and runner policy tests. |
| KMIPKIT-0083-FR-005 | Shared result records SHALL separate library/API correctness, observed server compatibility, cross-language parity, and cases not executed. A report SHALL identify server, protocol version, library commit, artifact hashes, configuration fingerprint, transport, encoding, and API level. | Result schema and aggregation tests. |
| KMIPKIT-0083-FR-006 | Reports and diagnostics SHALL NOT contain credentials, private keys, key material, raw KMIP request/response bodies, or unredacted secret values. | Secret-redaction fixtures and report contract tests. |
| KMIPKIT-0083-FR-007 | The shared contract SHALL explicitly record generic TTLV outbound request construction as unavailable in the current public API, without treating the gap as server incompatibility. | Catalog/API limitation metadata validation. |
| KMIPKIT-0083-FR-008 | A validator run SHALL be reproducible from artifacts built from a recorded release commit; the validator SHALL consume the installed language package or library artifact. | CI artifact provenance and consumer smoke tests in language specifications. |

## 4. Scope and Exclusions

The contract covers OASIS KMIP 2.0 and 2.1 only, including their applicable
TTLV, JSON, and XML encodings, transports, client-to-server requests, and
server-to-client operations. Manufacturer extensions, KMIP 1.x, and KMIP 3.0
are excluded. Server compatibility is recorded per exact product version and
configuration; a pass does not imply support for other operations or profiles.

The catalog can define planned cases before external servers exist. Such cases
must remain visibly `planned` until executable implementations and offline
evidence are present; catalog inclusion is not a passing test result.

The pinned OASIS KMIP 2.0 source includes Export (§6.1.18) and Import (§6.1.24).
Therefore these operations are covered in both target versions. This corrects
the conflicting KMIPKIT-0070 §3.1 list, which labels them KMIP 2.1-only. The
version-boundary cases currently cover Get Constraints and Set Constraints,
which are absent from the KMIP 2.0 operation inventory. Any additional names
in KMIPKIT-0070 §3.1 that are not OASIS KMIP 2.0/2.1 operations remain excluded
from this standard-only test catalog.

## 5. Delivery and Acceptance

The common contract is delivered on its own feature branch and draft PR. The
four language validators use separate branches and PRs based on the release
branch after this common change is approved and integrated. Final WIP removal
requires every language CI and the cross-language offline parity gate to pass.
Human approval and merge remain maintainer responsibilities.
