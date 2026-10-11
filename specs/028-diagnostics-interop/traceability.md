# KMIPKIT-0042 Traceability: Interop, Log, Validate

Before review, link each source row, stable element, requirement, implementation location, and named executable test. Table fields are traced through catalog elements and approved FRs; do not invent client duties from server result tables.

| Stable catalog element / requirement | Exact source | Coverage | Planned implementation / verification |
|---|---|---|---|
| KMIPKIT-ELEM-OP-C2S-INTEROP; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-INTEROP-00000034 | §6.1.26 Tables 241–243; §11.36 | Interop payload and Operation value | interop.rs; protocol/client Interop suites |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-15-INTEROP-FUNCTION | §6.1.26 Table 241; §7.15 | Required open Enumeration | Required/type/unknown round trip |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-16-INTEROP-IDENTIFIER | §6.1.26 Table 241; §7.16 | Required Text String | Missing/type/round-trip tests |
| Table 242 empty success; Table 243 errors | §6.1.26 | Empty payload and server result | Empty success and common-result preservation |
| KMIPKIT-ELEM-OP-C2S-LOG; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOG-0000002C | §6.1.29 Tables 250–252; §11.36 | Log payload and Operation value | log.rs; protocol/client Log suites |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-19-LOG-MESSAGE | §6.1.29 Table 250; §7.19 | Required, redacted message | Required/type, secret-sentinel redaction |
| KMIPKIT-DISC-017 | §6.1.29.1 / Table 252 | Approved Query-caption correction for Log | Table 252 reason preservation; no Query mapping |
| KMIPKIT-ELEM-OP-C2S-VALIDATE; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-VALIDATE-00000017 | §6.1.57 Tables 340–342; §11.36 | Validate payload and Operation value | validate.rs; protocol/client Validate suites |
| KMIPKIT-ELEM-TAG-420013 | §6.1.57 Table 340 | Repeated Certificate material | Repeated field/type/order cases |
| KMIPKIT-ELEM-TAG-420094 | §6.1.57 Table 340 | Repeated Managed Certificate identifier | Repeated field/type/order cases |
| KMIPKIT-ELEM-TAG-42009A | §6.1.57 Table 340 | Optional Validity Date | Present/absent and server-default interpretation tests |
| KMIPKIT-ELEM-TAG-42009B | §6.1.57 Table 341 | Required open Validity Indicator | Missing/type/unknown value tests |
| KMIPKIT-REQ-SPEC-6.1.57-001 | §6.1.57 opening paragraph; SHALL | A request contains only one certificate chain | Model shape has one chain; rejects a second chain representation |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | §6.1.57 second paragraph; MAY | Chain may use Certificate objects, identifiers, or both | Certificate-only, identifier-only, mixed input cases |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | §6.1.57 second paragraph; MAY | Caller may specify evaluation date | Supplied Validity Date round trip |
| KMIPKIT-REQ-SPEC-6.1.57-004 | §6.1.57 Table 340 / repeated field; MAY | Repeated Certificates | Multiple Certificate values preserved |
| KMIPKIT-REQ-SPEC-6.1.57-005 | §6.1.57 Table 340 / repeated field; MAY | Repeated Unique Identifiers | Multiple identifiers preserved |
| KMIPKIT-REQ-SPEC-6.1.57-006 | §6.1.57 Validity Date rule; SHALL | Omitted date means server assumes current date/time | Omission remains absent; request test does not synthesize local time |

The Interop production-availability sentence is server-only; no KMIPKit client test claims a server implementation. There are no direct catalog Test Cases for these operations. FR-001–FR-008, SC-001–SC-004, all six Validate requirements, approved DISC-017, source-derived vectors, redaction, and required repository gates must be evidenced before review.
