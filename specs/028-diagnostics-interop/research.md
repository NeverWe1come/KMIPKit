# Research: KMIP 2.1 Interop and Diagnostics

## Decision 1: Add typed client operations to the common execution path

Interop, Log, and Validate become distinct protocol request/response models and explicit typed Rust client calls. Reuse the common single exchange, result, Pending, delivery, TTLV, and generic response contracts.

## Decision 2: Keep Interop server restrictions on the server side

§6.1.26 says Interop SHALL NOT be available in a production server. KMIPKit is a client library and does not implement or control a server. Expose the client request defined by Tables 241–243 without claiming a target server offers it.

## Decision 3: Use approved KMIPKIT-DISC-017 for Log error handling

The §6.1.29 error prose mistakenly says Query Operation. The approved disposition assigns it to Log, and the approved spec cites Table 252. Preserve Table 252 result reasons. The caller must not submit secrets; redact all Log Message diagnostics regardless.

## Decision 4: Validate forwards one chain without local certificate policy

Model repeated Certificate and Unique Identifier inputs, including mixed use, as the supplied chain material. Preserve repeated-field order. Support optional Validity Date and leave it absent when omitted; server behavior assumes current date/time. Expose the server Validity Indicator as an open value. Do not validate the chain, choose trust anchors, or predict server policy.

## Decision 5: Gate only cross-language binding work on KMIPKIT-0043

Complete Rust protocol/client models independently. Wait for the approved 0043 capability mapping before changing the public API manifest, generated outputs, adapters, or parity tests.

## Evidence reviewed

Pinned OASIS Specification v2.1 §§6.1.26, 6.1.29, 6.1.57 and Tables 241–243, 250–252, 340–342; approved spec 028; normative catalog; existing client/protocol conventions. The spec links no direct official Test Case.
