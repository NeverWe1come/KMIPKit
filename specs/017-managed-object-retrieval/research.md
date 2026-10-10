# Research: KMIP 2.1 Get and Locate Operations

**Date**: 2026-10-10<br>
**Normative source**: Pinned OASIS KMIP Specification v2.1, especially §§6.1.19 and 6.1.28 and Tables 220–222 and 247–249.<br>
**Release-contract baseline**: `release/1.0.0` at `4e15a8f15c4c8529426b004737aaf16961d1c073`; current catalog includes the KMIPKIT-0017 dispositions merged by PR #79.

## Decision 1: Keep Get and Locate in one operation-family specification

**Decision**: Specify Get and Locate together as KMIPKIT-0017 with independently testable user stories and operation modules.

**Rationale**: Both expose server-held object state, depend on the existing Unique Identifier and AttributeSet models, and share the same typed client result, batch, decoder, secret-payload, and transport gates. They do not share request semantics, so their payload models and conformance tests remain distinct.

**Alternatives considered**:

- Separate specifications for Get and Locate: would repeat catalog, result, secret-handling, and client-dispatch preparation without removing a dependency.
- One generic retrieval request: rejected because Tables 220 and 247 define materially different selectors and response payloads.

## Decision 2: Preserve Get's object as opaque, secret-bearing TTLV

OASIS §6.1.19 Tables 220–221 define five optional request fields and three required success-response fields. The response's Any Object varies with Object Type. The existing generic TTLV model preserves nested item order and unknown allocated values, redacts payload contents in its metadata-only formatting, and zeroizes owned payload allocations when dropped. Reusing that representation avoids both a second object model and accidental local key processing.

**Key-format behavior**: §6.1.19 requires the client to account for key-format capabilities and restrictions. A server is required to return a registered key in the format used at registration; other conversions are optional. When PKCS#12 is requested, the source describes a response container, linked Secret Data password, certificate-chain selection, and the error for multiple valid chains. §1.2 defines uppercase normative keywords; the maintainer chose in KMIPKIT-DEC-003 to treat the lowercase “shall” paragraph as descriptive server guidance, closing KMIPKIT-DISC-015. KMIPKit preserves explicit Key Format Type, Key Wrap Type, Key Compression Type, Key Wrapping Specification, and response values. It does not perform conversion, PKCS#12 construction, link traversal, unwrapping, or local container validation. Tests must verify client field presence/order and opaque response preservation without claiming PKCS#12 cryptographic interoperability solely from those checks.

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

## Decision 4: Use the current validated release foundations

The reviewed release baseline contains generic TTLV values, the Create feature's `UniqueIdentifier` and current validated `AttributeSet`, batch/message models, common result and Pending handling, typed client dispatch, decoder limits, and TLS/HTTPS transports. KMIPKIT-0021 has since extended the non-exhaustive `ClientRequest`, `ClientOperation`, and `ClientBatchOutcome` surfaces for Hash, MAC, and signature operations. It also added `ResponseBatchItemView::with_ttlv`, which lends the complete ordered response item within a callback. The callback does not permit retaining references. This feature adds a fallible `Item::try_clone` operation so Get can retain exactly the secret-bearing Any Object item rather than cloning the whole response tree. The `AttributeSet` source and contract are unchanged across the immediately reviewed release interval, though the current validation contract was added after its initial KMIPKIT-0014 introduction. Locate uses that current direct-item AttributeSet and its validation; Get does not use Attribute Reference. Both implementation branches touch shared dispatch files; start from the current release and rerun full CI after intervening merges.

No additional crate or runtime dependency is justified.

## Normative inventory and official test evidence

The canonical catalog has two operation elements and 15 linked inventory rows (2 Get and 13 Locate). KMIPKIT-DEC-005 assigns five Locate rows to the server and retires one false Group Member Default extraction; seven Locate rows remain client-applicable. The source-linked cases are:

- Get: `TC-PKCS12-1-21` and `TC-PKCS12-2-21` (`KMIPKIT-TEST-CN01-2-68/-69`), plus Profile case `TL-M-3-21` (`KMIPKIT-TEST-PROF-5-12-6-3`).
- Locate: `TC-MDO-2-21`, `TC-MDO-3-21`, `TC-OFFSET-1-21`, and `TC-OFFSET-2-21` (`KMIPKIT-TEST-CN01-2-62` through `-65`).

The catalog now marks two Get PKCS#12 XML fixtures available and the other five linked fixtures unavailable. Catalog test IDs are distinct from official case labels. In Test Cases §§2.68–2.69, the headings and hyperlink targets use `TC-PKCS12-…`, while visible link text inserts an extra hyphen as `TC-PKCS-12-…`. KMIPKIT-DEC-004 selects the heading/target identifiers after retrieving and parsing both official XML documents; their canonical URLs and SHA-256 values are pinned in SOURCES.md. This closes KMIPKIT-DISC-032. Fixture presence does not mean an official-case pass or profile conformance. Derived vectors for the remaining unavailable cases must be labeled separately.

## Clarification pass

The pinned normative text, catalog, constitution, and approved roadmap resolve the feature's scope, field forms, matching behavior, archived-object boundary, language stage, and secret handling. KMIPKIT-DEC-003, -004, and -005 dispose of the three catalog questions for PKCS#12 wording, official case mapping, and Locate actor ownership. The original feature specification and inventory correction were human-merged in PRs #63 and #79. This refresh aligns the design to release `4e15a8f1`; its review and merge remain required before implementation tests because #78 changed shared dispatch and response access. The feature specifies lossless client forwarding and response preservation without claiming server-side outcomes as client conformance.
