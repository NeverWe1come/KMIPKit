# Protocol Review Checklist: Encrypt and Decrypt

**Purpose**: Reviewer-owned check of protocol requirement completeness and clarity.
**Feature**: ../spec.md
**Status**: Draft. Leave every box unchecked until the reviewer verifies the criterion.

## Normative scope

- [ ] CHK001 Are Encrypt and Decrypt bounded to exact KMIP 2.1 sections and request, response, and error tables?
- [ ] CHK002 Are all 21 applicable client catalog requirements and two operation elements mapped without assigning server-only ID Placeholder or Usage Limits work?
- [ ] CHK003 Are the six shared operation-structure elements, Cryptographic Parameters attribute element, and all three source-linked OASIS Test Case records assigned?
- [ ] CHK004 Are shared batching, client execution, and Pending requirements explicitly inherited from their existing owners?

## Wire and model correctness

- [ ] CHK005 Does request Data preserve all §7.9 Byte String, Enumeration, and Integer encodings while response Data remains Byte String?
- [ ] CHK006 Do request members preserve Table 196/214 order, singleton cardinality, and exact TTLV forms?
- [ ] CHK007 Does the multipart matrix require initial Init, later server Correlation Value, final Final, and middle-part Data?
- [ ] CHK008 Does the typed client accept Init=true/Final=true with Data, and gate only Data omission while KMIPKIT-DISC-045 remains open without selecting the disputed omission interpretation?
- [ ] CHK009 Are Cryptographic Parameters optional at the operation field level, unknown members preserved, and KMIPKIT-CLAUSE-SPEC-4.16-004 excluded with rationale and no requirement link because “REQUIRED” is a Table 59 column heading?
- [ ] CHK010 Are §4.16 IV Length and GCM Tag Length obligations tested where caller-supplied parameters expose the mode?
- [ ] CHK011 Do successful, failed, and Pending response shapes match Tables 197/215 and the shared ClientBatchOutcome contract?
- [ ] CHK012 Are response Data fields optional even in single-part success payloads?
- [ ] CHK013 Are all Message Data Structures Result Reasons supported, including known reasons absent from Tables 198/216 and unknown future codes?

## Security and behavior

- [ ] CHK014 Are Enumeration and Integer request Data redacted like Byte String values?
- [ ] CHK015 Are request/response bytes, AAD, AEAD Tag, Correlation Value, and raw bodies excluded from diagnostics?
- [ ] CHK016 Are zeroization ownership limits clear for KMIPKit-owned buffers and foreign-language copies?
- [ ] CHK017 Does every invocation perform at most one exchange and never retry or poll automatically?
- [ ] CHK018 Does the operation model preserve Unique Identifier optionality while client-layer tests reject locally detectable ineligible shapes and preserve per-item server results after a prior operation fails?
- [ ] CHK019 Does a client test and example explicitly execute Recover before Encrypt/Decrypt for a caller-known archived object after the KMIPKIT-0018 dependency is available?
- [ ] CHK020 Are delivery classification and decoder limits inherited from the existing client contract?

## Acceptance and evidence

- [ ] CHK021 Can every feature requirement be exercised with deterministic protocol or fake-transport tests?
- [ ] CHK022 Are all in-scope Encrypt/Decrypt items from the three pinned XML fixtures tested and labeled as fixture-derived evidence, without claiming complete-case passes?
- [ ] CHK023 Are English and Spanish examples consistent with caller-driven multipart behavior?
- [ ] CHK024 Does the spec, plan, model, payload contract, traceability, tasks, and catalog agree after speckit-analyze?
