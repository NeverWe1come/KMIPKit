# Research: KMIP 2.1 Interop and Diagnostics

## Decision 1: Add typed client operations to the common execution path

Interop, Log, and Validate become distinct protocol request/response models and explicit typed Rust client calls. Reuse the common single exchange, result, Pending, delivery, TTLV, and generic response contracts.

## Decision 2: Keep Interop server restrictions on the server side

§6.1.26 says Interop SHALL NOT be available in a production server. KMIPKit is a client library and does not implement or control a server. Expose the client request defined by Tables 241–243 without claiming a target server offers it.

## Decision 3: Use approved KMIPKIT-DISC-017 for Log error handling

The §6.1.29 error prose mistakenly says Query Operation. Accepted KMIPKIT-DEC-006 records the approved disposition assigning it to Log, and the approved spec cites Table 252. Preserve Table 252 result reasons. The caller must not submit secrets; redact all Log Message diagnostics regardless.

## Decision 4: Validate forwards one chain without local certificate policy

Model repeated Certificate and Unique Identifier inputs, including mixed use, as the supplied chain material. Read §6.1.57 paragraph 1 together with paragraph 2 and Table 340: each Validate request has one chain with at least one total member across the two lists; either individual list may be omitted, and either may repeat. Validity Date alone does not count as chain material. Preserve repeated-field order. Support optional Validity Date and preserve its absence when omitted. Section 6.1.57 states passively that the current date and time SHALL be assumed if the date is omitted, but it does not identify an actor; the source actor remains unspecified. The `client_1_0` scope denotes KMIPKit feature/API serialization applicability only. Client verification covers preserving the omission in request serialization only; date evaluation is unverified. Expose the server Validity Indicator as an open value. This is request-representation evidence only; do not cryptographically validate the chain, choose trust anchors, or predict server policy.

## Decision 5: Gate only cross-language binding work on KMIPKIT-0043

Complete Rust protocol/client models independently. Wait for the approved 0043 capability mapping before changing the public API manifest, generated outputs, adapters, or parity tests.

## Evidence reviewed

Pinned OASIS Specification v2.1 §§6.1.26, 6.1.29, 6.1.57 and Tables 241–243, 250–252, 340–342; approved spec 028; normative catalog; existing client/protocol conventions. The spec links no direct official Test Case.


## Decision 6: Apply the existing normative Structure description order

Pinned section 10.1.2, unnumbered Type table Structure row requires defined Structure fields in their
description order (KMIPKIT-REQ-SPEC-10.1.2-001). Table 340 therefore encodes
Certificates, then Unique Identifiers, then optional Validity Date. Preserve
relative order within the two lists, including when builders are called in
mixed order. Section 2.1, Table 2 likewise places Certificate Type before
Certificate Value; reversed input is a local structural error/NotSent.
This supersedes the earlier mixed insertion/wire-order design interpretation,
using the existing normative rule rather than inventing a client requirement.
The coordinator directed this correction under the authorized pinned-source
workflow on 2026-10-12. DER contents and trust/server policy remain opaque.
The global common-rule record is not claimed complete by this scoped proof.
