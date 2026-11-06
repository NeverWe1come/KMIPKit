# KMIPKIT-0052 Traceability

Normative identifiers use `KMIPKIT-0052-R###`. Project API/ownership decisions use `KMIPKIT-0052-P###` and do not claim to be OASIS requirements.

## Normative Requirements

| ID | OASIS source | Requirement interpretation | Implementation | Executable verification |
|---|---|---|---|---|
| KMIPKIT-0052-R001–R056 | KMIP v2.1 §6.1 operation-specific Response Payload tables listed row-by-row in `gap-matrix.md`; §8.6 Table 399 for Response Payload presence | Preserve and expose each supported operation's response result and payload presence without assigning new protocol semantics. Poll has no standalone payload table and is governed by §6.1.38 and Table 399. Query Asynchronous Requests Table 286 retains its caption/content discrepancy and remains generic. | Pending operation readers and response tree owner | Pending: all-operation reader matrix, per-operation status and payload tests |
| KMIPKIT-0052-R057 | KMIP v2.1 §8.5 Table 398 | Expose the complete ordered response header structure to callers. | Pending: `ResponseHeaderView` raw access and client header surface | Pending: header unknown/repeated field tests in Rust and C consumers |
| KMIPKIT-0052-R058 | KMIP v2.1 §8.6 Table 399; §10.1.2 | Return the Response Payload only when present and preserve its decoded ordered Structure children. Do not fabricate a missing payload. | Pending: response-item raw view and payload-presence logic | Pending: absent/empty/present payload tests and order assertions |
| KMIPKIT-0052-R059 | KMIP v2.1 §9.13; §11.56 | Preserve generic vendor extension fields in the accepted `0x540000`–`0x54FFFF` tag range when they occur in a response tree. | Pending: nested TTLV traversal and raw response handles | Pending: `0x54xxxx` KeyBlock/ServerInformation tests in Rust/C/Java/Python |
| KMIPKIT-0052-R060 | KMIP v2.1 §3.1 Table 12; §7.37 Table 389; §6.1.19 Table 221 | Keep nested Key Block, Server Information, and Get Any Object descendants reachable as ordered structures in generic response data. | Pending: nested item-to-Structure access | Pending: descendant traversal tests with repeated and vendor children |

## Project Decisions

| ID | Decision | Evidence |
|---|---|---|
| KMIPKIT-0052-P001 | A high-level typed response item owns a shared `Arc<ResponseMessage>` and its batch index; raw views borrow from that owner. This preserves one source tree per decoded response message and avoids duplicating each payload. | Pending: ownership/lifetime tests and ADR-0018 |
| KMIPKIT-0052-P002 | `.raw()` represents the decoded ordered TTLV tree, not received bytes or padding. | Pending: docs and round-trip boundary test |
| KMIPKIT-0052-P003 | Failure or Pending outcomes with no Response Payload return `None`; an actual empty Structure remains `Some(empty_view)`. | Pending: status matrix tests |
| KMIPKIT-0052-P004 | Rust response views borrow; C/JNI/CFFI raw Structure handles own zeroizing copies and require explicit release/close. | Pending: lifetime/release tests |
| KMIPKIT-0052-P005 | The Table 286 query-async response remains generic until the separate approved discrepancy resolution; no interpretation is inferred from its caption. | Pending: generic payload preservation test and docs |
| KMIPKIT-0052-P006 | Generic raw traversal never formats payload values automatically; explicit caller access does not weaken redaction of Debug/errors. | Pending: cross-language canary tests |

## Verification Record

Implementation verification is appended here before the draft PR is marked ready. The approved scope remains Rust/C/Java/Python; no formal conformance or certification claim is made.
