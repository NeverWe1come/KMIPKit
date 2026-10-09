# Research: KMIP 2.1 Get and Locate Operations

**Date**: 2026-10-09<br>
**Normative source**: Pinned OASIS KMIP Specification v2.1, especially §§6.1.19 and 6.1.28 and Tables 220–222 and 247–249.<br>
**Catalog baseline**: `release/1.0.0` at `227e3f9104f14494810598d078c013c389c24f8f`.

## Decision 1: Keep Get and Locate in one operation-family specification

**Decision**: Specify Get and Locate together as KMIPKIT-0017 with independently testable user stories and operation modules.

**Rationale**: Both expose server-held object state, depend on the existing Unique Identifier and AttributeSet models, and share the same typed client result, batch, decoder, secret-payload, and transport gates. They do not share request semantics, so their payload models and conformance tests remain distinct.

**Alternatives considered**:

- Separate specifications for Get and Locate: would repeat catalog, result, secret-handling, and client-dispatch preparation without removing a dependency.
- One generic retrieval request: rejected because Tables 220 and 247 define materially different selectors and response payloads.

## Decision 2: Preserve Get's object as opaque, secret-bearing TTLV

OASIS §6.1.19 Tables 220–221 define five optional request fields and three required success-response fields. The response's Any Object varies with Object Type. The existing generic TTLV model preserves nested item order and unknown allocated values, redacts payload contents in its metadata-only formatting, and zeroizes owned payload allocations when dropped. Reusing that representation avoids both a second object model and accidental local key processing.

**Key-format behavior**: §6.1.19 requires the client to account for key-format capabilities and restrictions. A server is required to return a registered key in the format used at registration; other conversions are optional. When PKCS#12 is requested, OASIS defines the response container, linked Secret Data password, certificate-chain selection, and the error for multiple valid chains. KMIPKit will preserve explicit Key Format Type, Key Wrap Type, Key Compression Type, Key Wrapping Specification, and response values. It will not perform a conversion, construct PKCS#12, follow object links, unwrap a key, or assert that a server supports an optional conversion. Derived tests verify field presence/order and opaque response preservation; they do not claim PKCS#12 cryptographic interoperability.

**Response errors**: Table 222 includes the operation-specific reasons. The shared result model retains Result Status, applicable Result Reason, and Result Message without copying raw server payload into diagnostics.

## Decision 3: Keep Locate matching and placeholder state server-authoritative

OASIS §6.1.28 and Tables 247–248 define a required Attributes Structure, optional Maximum Items, Offset Items, Storage Status Mask, and Object Group Member, then optional Located Items and zero or more repeated Unique Identifiers. An empty Attributes Structure matches all objects. The client must preserve caller criteria; it has no authoritative candidate-object set against which to evaluate them.

The request/response and acceptance criteria preserve these source rules:

- Returned objects match all requested attributes. Structured attributes may contain only fields needed for the desired match.
- One Date attribute instance means an equal date; two instances specify an inclusive range; the maximum representable Date value acts as undefined.
- Cryptographic Usage Mask matching is a requested-bit subset comparison. Usage Limits Count and Total must be at least the requested values.
- Group Member Fresh and Group Member Default invoke server policy semantics; KMIPKit does not create or select group members.
- An omitted Storage Status Mask searches online objects. The server must not return archived or destroyed object identifiers unless the respective indicator is included.
- Offset Items zero is equivalent to omission. With multiple results, the server returns descending object-creation order. The client retains any returned order and does not sort or deduplicate.
- The server updates ID Placeholder for one result and clears it for qualifying multiple-result cases. The client sends batches as requested and does not predict that state.
- Archived objects require Recover followed by Get. Recover is outside KMIPKIT-0017, so no end-to-end archived-payload retrieval claim is made here.

Table 249 errors use the shared result contract.

## Decision 4: Use existing release foundations without depending on KMIPKIT-0016

The release baseline contains the generic TTLV values, the Create feature's `UniqueIdentifier` and `AttributeSet`, batch/message models, common result and Pending handling, typed client dispatch, decoder limits, and TLS/HTTPS transports. Locate can encode its required Attributes Structure with the existing direct-item AttributeSet. Get does not use Attribute Reference. Therefore this feature can be designed from the release branch without importing the unmerged KMIPKIT-0016 implementation. Both branches may touch shared dispatch files; rebase/update and full CI after intervening release merges are required.

No additional crate or runtime dependency is justified.

## Normative inventory and official test evidence

The canonical catalog has two operation elements and 15 linked requirement rows (2 Get and 13 Locate). Six Locate rows have server-action summaries under client actor metadata and are tracked as an open actor-allocation issue; they are not treated as duties a client library can enforce. The source-linked cases are:

- Get: `TC-PKCS12-1-21` and `TC-PKCS12-2-21` (`KMIPKIT-TEST-CN01-2-68/-69`), plus Profile case `TL-M-3-21` (`KMIPKIT-TEST-PROF-5-12-6-3`).
- Locate: `TC-MDO-2-21`, `TC-MDO-3-21`, `TC-OFFSET-1-21`, and `TC-OFFSET-2-21` (`KMIPKIT-TEST-CN01-2-62` through `-65`).

The pinned catalog marks these seven XML fixtures unavailable. Catalog test IDs are distinct from official case labels. KMIPKIT-DISC-032 leaves the two PKCS#12 label-to-fixture mappings unresolved, with weak mapping confidence. Local derived vectors may cover observable client request/response behavior, but MUST NOT be reported as official-case passes or profile conformance.

## Clarification pass

The pinned normative text, catalog, constitution, and approved roadmap resolve the feature's scope, field forms, matching behavior, archived-object boundary, language stage, and secret handling. Open inventory questions remain: KMIPKIT-DISC-015 covers the lowercase shall in PKCS#12 output guidance; KMIPKIT-DISC-032 covers the PKCS#12 case-label/fixture mismatch; and six Locate requirement rows describe server actions while catalog actor metadata says client. The feature can specify lossless client forwarding and response preservation, but cannot claim those server-side outcomes are implemented or conformant. Resolve these catalog questions before implementing affected conformance assertions. Human approval of the written specification remains an implementation gate.
