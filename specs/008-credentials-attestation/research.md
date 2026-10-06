# Research: KMIPKIT-0008 Credentials and Attestation

## Source and scope

- Primary source: pinned `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`; checksum is recorded in `specification/oasis/kmip-2.1/CHECKSUMS.sha256`. Source clauses are §9.3/Table 402, §9.4/Table 403, §9.11/Tables 410–416, §9.14/Table 419, and §11.11/Table 442. Only local immutable source copies were consulted.
- Applicable credentials: Username and Password (1), Device (2), Attestation (3), One Time Password (4), Hashed Password (5), Ticket (6), and Extensions (8XXXXXXX). All other raw values must remain round-trippable through generic TTLV.
- No requirement-specific official Test Case IDs are linked in the catalog; the 203 XML fixtures recorded in `KMIPKIT-DISC-036` are unavailable. Acceptance tests must be explicitly labeled derived.

## Decisions

### Credential and Authentication models

Use an ordered `Authentication` collection with at least one `Credential` when present. Represent every named variant from Tables 411–416 and table-required fields. Keep the Extensions value and unknown enum cases opaque. Generic TTLV remains the lossless fallback, including unknown children and vendor values.

**Rationale**: §9.4 makes Authentication optional and requires one or more Credentials when present; Table 403 permits Credential repetition. The public API policy requires future/vendor values to survive round trips. OASIS gives no schema for vendor Extensions credentials.

**Alternatives considered**: reject repeated entries (contradicts Table 403); reject unknown enum values (violates preservation policy); invent an Extensions schema (unsupported by source).

### Attestation and Nonce

Attestation Credential carries caller-provided Nonce, Attestation Type, and at least one of Attestation Measurement or Assertion. Permit both evidence fields because the prose requires data in either field but does not explicitly prohibit both. Preserve Nonce ID/Value exactly from the server's Nonce object. Do not generate or verify evidence.

**Rationale**: §9.11 and Table 413 require Nonce and Attestation Type; §9.14/Table 419 defines the server-assigned and server-created Nonce values. The library is not a local cryptographic provider.

**Alternatives considered**: generate a Nonce or attestation evidence locally (outside product boundary); make evidence exclusive (source does not prohibit both); drop unknown Attestation Type values (lossy).

### Hashed Password

Require and preserve caller-supplied hashed bytes, username, and Timestamp; allow an optional raw Hashing Algorithm value. Its effective default is SHA-256 while field absence remains intact. Do not compute the OASIS hash formula locally. Do not create an implicit clock or persistence layer, or implement/claim monotonicity checking or tests, until OD-003 review settles the owner, comparison scope, and clock behavior.

**Rationale**: §9.11 states the formula, monotonic timestamp requirement, and SHA-256 default; Table 415 marks Hashing Algorithm optional. Product definition excludes local cryptographic algorithms.

**Alternatives considered**: hash a supplied password locally (violates product boundary); always emit Hashing Algorithm (changes an optional field's wire presence); silently accept timestamps as monotonic without state/evidence (unverifiable).

### Secret handling and transmission

Redact all Credential formatting and diagnostics. Zeroize KMIPKit-owned secret allocations under the repository's explicit secret type/ownership rules, reusing the pinned workspace `zeroize` dependency if approved by the implementation review. OD-004 is resolved by scope: Credential and Authentication values remain in-memory; 0008 only adds the non-secret Attestation Capable Indicator to the existing request header builder. It adds no Credential writer or secret-bearing send path. Any later send path belongs to a separate approved client feature that owns the candidate callsite and owner-through-transport lifecycle test.

**Rationale**: `AGENTS.md` §8, constitution IV, and `docs/architecture/public-api.md` require redaction and zeroization; the existing architecture explicitly gates secret-bearing encoding.

**Alternatives considered**: allow secret-bearing generic writer use (violates the gate); log raw protocol trees for diagnostics (violates redaction); claim zeroization of foreign runtime copies (not controllable by Rust).

### Device and profile behavior

Represent and preserve all six Table 412 fields and model Password as optional. Section 9.11 says the client SHALL provide at least one field; this specification treats any Table 412 member as a qualifying present field. It does not infer a non-empty text rule. The separate uniqueness prose names Device Serial Number, Network Identifier, Machine Identifier, and Media Identifier. The caller must supply one or a combination of these identifiers that is unique as required by §9.11. The source does not state the comparison scope, and client-local data cannot establish it, so KMIPKit will not verify or enforce uniqueness. Authentication applicability and mechanism selection are deferred to KMIPKIT-0010.

**Rationale**: §9.11's final sentence requires at least one field after defining the Device Credential Value structure, whose six members are listed in Table 412. We apply that minimum to any of those six members and keep the separate four-field uniqueness rule distinct. Field presence does not establish global uniqueness. The source explicitly points mechanism choices to KMIP Profiles.

**Alternatives considered**: narrow the minimum-presence rule to the four fields named in the uniqueness sentence; impose a client-global identity registry; claim an authentication profile from base-model support.

## Inventory findings requiring review

1. The pinned §9.4 multiple-Credential sentence uses lowercase `must`; §1.2 defines uppercase RFC 2119 keywords. The accepted catalog retains the keyword-strength candidate under open `KMIPKIT-DISC-041`, but now assigns `KMIPKIT-REQ-SPEC-9.4-001-003` to `role=server`, `scope_state=server_only`, and `status=unassigned`, with no client implementation or verification references. Do not test or enforce “all Credentials satisfied” as a client duty.
2. The accepted catalog summary for `KMIPKIT-REQ-SPEC-9.11-001` separates general client identification from profile-indicated optional authentication and links Credential reciprocally.
3. KMIPKIT-0002 tasks T045–T047 completed the separate catalog correction, pinned-source review, report regeneration, and independent parent QA. Evidence is recorded in `specification/catalog/review-evidence.md`. No explicit catalog-owner sign-off is named there; preserve `KMIPKIT-DISC-041` and do not decide the lowercase word's force in this feature. No catalog input or generated output is changed by KMIPKIT-0008.
4. §9.11 says the client SHALL provide at least one field, and Table 412 enumerates six members of the Device Credential Value. Interpret this as at least one present Table 412 member; reject an empty Device without imposing non-empty text. Separately, the caller must supply a unique one or combination of the four named identifiers. The source does not define comparison scope, and KMIPKit does not check uniqueness from client-local data. The separate uniqueness prose does not narrow the minimum-presence rule. OD-002 is resolved with this bounded interpretation.
5. The lowercase OTP sentence says “may” and the catalog already classifies `KMIPKIT-CLAUSE-SPEC-9.11-008` as `informative_context`, with no requirement ID. OD-006 is resolved: do not add library-wide replay/single-use state or normative client-side enforcement. Request-scoped use follows architecture; request execution selection is OD-005. Keep KMIPKIT-0008 OD-006 distinct from KMIPKIT-0007 OD-006, which concerns secret-send test ownership.
6. §9.11 defines Hashed Password computation and monotonic Timestamp but not the state owner, comparison scope, or clock behavior. Require and preserve the caller's Timestamp and hashed bytes and expose effective SHA-256 when omitted; do not calculate hashes or claim a monotonicity check/test until OD-003 review.
7. OD-004 is resolved by scope: Credential and Authentication values remain in-memory; 0008's only execute change is the non-secret Attestation Capable Indicator in the existing header builder. It adds no Credential writer or secret-bearing send path. Any future path requires its own approved feature and candidate-callsite/owner-through-transport lifecycle evidence.
8. OD-005 remains open only for execution integration. An explicit KMIPKIT-0007 handoff must settle inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the entire batch. Standalone in-memory Credential and Authentication models do not need an execution API.

## Dependencies

- KMIPKIT-0005 owns TTLV wire encoding and its separately gated secret-memory/writer lifecycle contract.
- KMIPKIT-0006 owns common headers/messages and currently holds Authentication opaquely.
- KMIPKIT-0007 owns the request selection/execution boundary. Its accepted release does not define credential selection; OD-005 tracks the handoff needed only by a future Authentication integration, not by these in-memory models.
- KMIPKIT-0010 owns profile-specific authentication applicability and all profile claims.

OASIS §§8.1–8.3 (Tables 394–396) place one Request Header in a Request Message, place optional Authentication in that header, and allow repeated Request Batch Items. That layout implies message-wide scope for the header Authentication. This is a structural inference from the pinned source, not an OASIS rule for KMIPKit credential selection. Defaults, request/batch replacement, omission, and precedence remain project policy under OD-005.

## Dependency verification (2026-10-06)

The dependency audit was refreshed after PR #44 (KMIPKIT-0009 implementation)
merged into `release/1.0.0`. The resulting release commit is
`ae87b89d43957e4fc028e785dc181e69b0165dac`. KMIPKIT-0009 is not a prerequisite
for the in-memory credential models; the merge advances the base required by
T004.

| Dependency | Merged evidence | Accepted artifact evidence | Relevance to KMIPKIT-0008 |
|---|---|---|---|
| KMIPKIT-0005 TTLV wire codec | PR #30 merged as `548dda0e0e15abda0023264abf85f51179127987` | `specs/005-ttlv-wire-codec/approval-record.md`; `docs/adr/0012-caller-requested-wire-encoding-policy.md` is Accepted. ADR-0012 SHA-256: `130C4CAD7AFC9985BC816286F8652729B392425C5E510DCAB9285223E827F333`. | The accepted policy gates any production secret-bearing writer/callsite. KMIPKIT-0008 adds no new writer or secret-bearing path. |
| KMIPKIT-0006 message and batch model | PR #32 merged as `35a445f500d0ac0b55fe39cd95bf25984ea65216` | `specs/006-message-batch-model/approval-record.md` SHA-256: `910EE6B15EB35EBB40FF2457BB13F839FF07F9E8F63329C53C5B91D579B4AB33`. | The Request Header exposes Authentication only as a callback-scoped generic structure (`crates/kmipkit-protocol/src/message/header.rs`, `RequestHeaderView::with_authentication`). There is no typed credential-selection API in this dependency. |
| KMIPKIT-0007 typed client execution | PR #40 merged as `4126d62f16927753a6ae6fc3f5e628d29245683f` | `specs/007-client-execution/approval-record.md` SHA-256: `E2FFE0A1E3D4285E965F3E9B0EA7B60D6BD4EE941BEE60662EE985243A7F8B85`. | `Client::execute` owns the existing private Request Header builder and sole writer/permit path. It has no credential defaults, Authentication precedence, or Authentication selection API. 0008 reuses this path only to emit the non-secret Attestation Capable Indicator; OD-005 remains open for future Authentication selection. |

The 0008 feature branch has now been rebased onto the exact active release tip
`ae87b89d43957e4fc028e785dc181e69b0165dac`; the branch's merge base matches
that commit. The rebase includes the merged KMIPKIT-0009 implementation in PR
#44. The pinned OASIS source SHA-256 is
`8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF`.
Only this local immutable source copy was used. The release contains the
accepted 0005/0006/0007 implementation gates and the accepted ADR-0012
ownership rule. The separate OD-001 catalog correction is complete under
KMIPKIT-0002 T045–T047; the lowercase-keyword discrepancy remains open and
the evidence does not name a catalog-owner sign-off. The future Authentication
execution handoff remains open under OD-005.
