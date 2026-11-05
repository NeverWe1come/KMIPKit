# Feature Specification: KMIP 2.1 Multi-Encoding Scope and Common Codec Contract

**Feature Branch**: `feature/KMIPKIT-0054-multi-encoding-governance`
**Created**: 2026-11-05
**Status**: Approved for implementation by direct maintainer instruction on 2026-11-05
**Input**: Maintainer-approved addition of TTLV, JSON, and XML encodings to the 1.0.0 client.

## User Scenarios & Testing

### User Story 1 - Maintain an auditable 1.0 encoding scope (Priority: P1)

A maintainer and implementer can determine which encodings and transports
belong to 1.0, which ADR controls the scope, and which feature specifications
own implementation.

**Independent Test**: Review the accepted ADR, constitution, `AGENTS.md`,
project definition, and roadmap and confirm that all describe the same scope
and feature-ID ownership.

**Acceptance Scenarios**:

1. **Given** the original TTLV-only scope in ADR-0002, **When** a reviewer
   reads the current governance set, **Then** ADR-0017 clearly supersedes only
   the encoding decision and preserves remaining 1.0 boundaries.
2. **Given** the reserved Phase 5 identifiers 0047–0053, **When** the roadmap
   lists multi-encoding work, **Then** the new specifications use 0054–0057.
3. **Given** a codec implementer, **When** they read this approved feature,
   **Then** they can identify the common model, normative sources, security
   limits, parallel work split, and public API integration gate.

### User Story 2 - Build JSON and XML codecs independently (Priority: P1)

Two codec teams can consume a stable common semantic item model and catalog
metadata, implement their respective OASIS representations, and run official
vectors without editing the shared public API manifest or client dispatcher.

**Independent Test**: Validate the dependency/ownership contract and branch
matrix; separately execute each codec's official vectors and malformed-input
tests against the common item tree.

**Acceptance Scenarios**:

1. **Given** the existing ordered generic `Item` tree, **When** a codec reads
   or writes a KMIP item, **Then** it preserves child ordering and every
   representable raw value and tag.
2. **Given** a catalog-known tag or enumeration, **When** encoded, **Then**
   the codec can use the catalog's canonical symbolic name; unknown values
   remain representable using the profile's hexadecimal representation.
3. **Given** JSON and XML codec branches, **When** developed independently,
   **Then** neither modifies `specification/api/public-api.json` or the
   public client dispatch path.

## Scope and Normative Sources

The pinned sources are `specification/oasis/kmip-2.1/upstream/` and the
verified inventory in `specification/oasis/kmip-2.1/SOURCES.md`. Upstream
copies are immutable. Normative references:

| Source | Sections | Use |
|---|---|---|
| OASIS KMIP Specification v2.1 | §§10.1–10.2.3 | TTLV baseline and delegation of JSON/XML message protocols to KMIP Profiles. |
| OASIS KMIP Profiles v2.1 | §§5.3.1–5.3.2 | HTTPS client/server content types and the documented source discrepancy. |
| OASIS KMIP Profiles v2.1 | §§5.4.1.1–5.4.1.3, 5.4.1.6.7, 5.4.2 | XML name normalization, tag/type/value representations, XML client clauses. |
| OASIS KMIP Profiles v2.1 | §§5.5.1.1–5.5.1.3, 5.5.1.6.7, 5.5.2 | JSON name normalization, tag/type/value representations, JSON client clauses. |
| OASIS KMIP Profiles v2.1 | §§5.4.4.1, 5.5.4.1 | Official `MSGENC-XML-M-1-21` and `MSGENC-JSON-M-1-21` acceptance evidence. |

ADR-0017 controls the project-specific scope and the contextual HTTPS
interpretation. It does not change OASIS text or claim official clarification.

## Functional Requirements

- **KMIPKIT-0054-FR-001**: The product governance documents MUST define the
  1.0 encoding set as TTLV, JSON, and XML over raw TLS and HTTPS/HTTP 1.1,
  with TLS and operation-direction boundaries unchanged.
- **KMIPKIT-0054-FR-002**: The original TTLV-only encoding decision MUST be
  linked to its superseding ADR; no governance document may continue to
  identify JSON/XML as post-1.0 work.
- **KMIPKIT-0054-FR-003**: The common codec contract MUST use the existing
  ordered `kmipkit-ttlv::Item` semantic model. A repository-owned offline
  generator MUST produce private Rust tables of assigned tag names,
  Enumeration families and values, and tag-to-Enumeration-family mappings
  from the reviewed catalog; codec specifications consume them for
  profile-specific symbolic normalization. The codecs MUST preserve
  unknown allocation-valid tags, raw Enumeration values, bitmask bits, and
  extension values represented by the model.
- **KMIPKIT-0054-FR-004**: The common contract MUST define shared limits,
  payload-free/redacted error behavior, and the conversion boundary needed by
  codecs, without exposing a new public encoding API.
- **KMIPKIT-0054-FR-005**: Symbolic tag and Enumeration names MUST be
  resolved from checked-in generated catalog metadata. JSON/XML codecs MUST
  use profile-compliant hexadecimal or numeric forms for unknown tags and
  Enumeration values; builds MUST NOT fetch or scrape OASIS pages.
- **KMIPKIT-0054-FR-006**: KMIPKIT-0054, -0055, and -0056 MUST NOT modify
  `specification/api/public-api.json`, generated public API output, or public
  client dispatch. KMIPKIT-0057 owns the coordinated `WireEncoding` type,
  client defaults/override, manifest, generator outputs, and Rust/C/Java/Python
  parity.
- **KMIPKIT-0054-FR-007**: JSON and XML codec implementations MUST be
  independently testable over the shared contract after this specification is
  accepted, with no cross-dependency between their codec modules.
- **KMIPKIT-0054-FR-008**: Raw TLS text encodings MUST use one complete
  document per exchange, incrementally detected under decoder limits, with
  no KMIPKit-specific wire prefix, and the connection MUST close after the
  response.
- **KMIPKIT-0054-FR-009**: HTTPS JSON/XML bodies MUST follow ADR-0017's
  contextual interpretation of Profiles §§5.3.1–5.3.2. The design record MUST
  label the source conflict as unresolved by OASIS and MUST NOT assert that
  the selected interpretation is an erratum.
- **KMIPKIT-0054-FR-010**: The local received-byte limit MUST apply to actual
  TTLV/JSON/XML wire bytes; protocol `Maximum Response Size` MUST retain the
  OASIS definition based on the equivalent TTLV response size.
- **KMIPKIT-0054-FR-011**: Encoding selection MUST never trigger automatic
  format detection for fallback, retry, or renegotiation. KMIPKIT-0057 MUST
  use the selected request encoding to decode the response and report a
  payload-free mismatch error with request-delivery evidence.
- **KMIPKIT-0054-FR-012**: The feature MUST reserve identifiers 0047–0053
  for Phase 5 and assign 0054 to governance/common contract, 0055 to JSON,
  0056 to XML, and 0057 to client/transport integration.

## Success Criteria

- **KMIPKIT-0054-SC-001**: All canonical governance sources describe the same
  1.0 encoding/transport scope and ADR-0017 is indexed as accepted.
- **KMIPKIT-0054-SC-002**: The requirements in this specification map to the
  governance, ADR, or downstream feature owner; traceability is complete.
- **KMIPKIT-0054-SC-003**: The JSON and XML implementations can be developed
  and tested on separate worktrees against the same common item contract;
  their changesets have no public manifest or dispatcher file in common.
- **KMIPKIT-0054-SC-004**: Two generator runs over the same reviewed catalog
  produce byte-identical private naming tables; `--check` detects stale or
  missing output without rewriting it.
- **KMIPKIT-0054-SC-005**: Every 0054–0057 specification identifies its
  correct dependency and exact OASIS source sections; 0055 uses Profiles §5.5
  and 0056 uses Profiles §5.4.

## Dependencies and Exclusions

- KMIPKIT-0054 stabilizes governance and internal codec contract before
  implementation of 0055 or 0056.
- KMIPKIT-0055 and KMIPKIT-0056 depend on the accepted 0054 contract and may
  be implemented in parallel in dedicated worktrees.
- KMIPKIT-0057 depends on both codec specifications and their implementations;
  it owns client, transport, manifest, generated output, and four-language
  public surface integration.
- This feature adds no codec implementation, `WireEncoding` public type,
  client setting, transport dispatcher, public API manifest/generated output,
  dependency, or conformance claim. It does add a private catalog-derived
  tag-name table for the parallel codec branches.
- Exact JSON/XML profile serialization and parsing rules belong to 0055 and
  0056 respectively; this feature fixes the shared ownership and safety
  contract only.

## Security and Test Requirements

- Decode untrusted inputs under the project defaults of 16 MiB, depth 64, and
  100,000 elements, with checks before size-driven allocation; limits remain
  configurable under the established `CodecLimits` contract.
- Never log, format, persist, retain, or include raw bodies or secret values
  in errors. KMIPKit-owned initialized wire buffers are zeroized at the end of
  their permitted lifecycle, subject to the documented external-copy limits.
- 0055 and 0056 MUST replay their exact official mandatory profile vectors,
  including `MSGENC-JSON-M-1-21` and `MSGENC-XML-M-1-21`, plus derived
  malformed-input, bounds, unknown-value, and semantic round-trip tests.
- 0057 MUST test fragmented raw TLS documents, document completion and
  trailing data, matching/mismatching formats, correct HTTPS content types,
  response byte caps, delivery state, zeroization, and no retry, then verify
  equivalent Rust/C/Java/Python behavior.
- No codec team may edit `specification/api/public-api.json` or generated
  outputs; 0057 updates that contract through the pinned generator.

The shared codec signatures, error categories, byte ownership, and per-file
branch ownership are fixed in [the common wire codec contract](contracts/common-wire-codec.md).

## Assumptions

- The present generic TTLV `Item` model remains the shared semantic substrate;
  `TtlvItem` in planning discussions refers to this model, not a new public
  type.
- The resolved catalog provides canonical tag names, assigned Enumeration
  values, and tag-to-Enumeration-family mappings. Hex or numeric forms
  preserve unknown values as permitted by each OASIS encoding profile.
- ADR-0017 records the approved project interpretation of the HTTPS source
  conflict, while formal profile conformance remains an evidence-driven claim.
