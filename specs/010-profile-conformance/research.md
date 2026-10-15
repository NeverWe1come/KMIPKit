# Research: KMIP 2.1 Client Profile Conformance

**Snapshot note:** The dated 2026-10-13 sections below are historical. The source-review audit dated 2026-10-14 at the end of this file supersedes their current counts and task status.

**Date**: 2026-10-07
**Feature**: `KMIPKIT-0010-profile-conformance`

## Decision: Use the reviewed catalog as the only profile-to-source map

**Decision**: Read profile identities, applicability, clauses, stable requirement IDs, dependencies, transport/encoding constraints, test IDs, and fixture availability from `specification/catalog/kmip-2.1.json`. Read the project-selected target set separately from `specification/catalog/profile-targets.json`; it is a project decision, not a second normative profile/source map. Runtime profile metadata is generated from these checked-in inputs; reports are produced by the existing offline catalog/report tooling. Do not parse live OASIS pages.

**Rationale**: The accepted catalog records every client and server profile and preserves source identity and evidence gaps. It already distinguishes `client_1_0`, `conditional`, `server_only`, and `out_of_scope`, and separates profile applicability from `claim_state`. The feature must not create a parallel source of truth.

**Alternatives considered**: Runtime HTML parsing or network lookups were rejected by ADR-0004, ADR-0009, and the document hierarchy. A separate profile spreadsheet was rejected because it would drift from the catalog.

## Decision: Keep executable profile rules explicitly typed and fail closed on incomplete mappings

**Decision**: Treat generated profile records as source and evidence metadata only. Runtime validation rules are explicit typed Rust mappings keyed by stable catalog requirement IDs, with executable tests for each mapped rule. A profile is not runtime-selectable until every applicable normative requirement has a typed mapping and test coverage. Never parse free-form catalog summaries or clause descriptions as predicates.

**Rationale**: Catalog records preserve normative traceability but do not define a safe executable rule language. Interpreting prose at runtime would make behavior ambiguous, difficult to test, and vulnerable to catalog wording changes. Failing closed prevents a partial rule set from being presented as profile validation.

**Alternatives considered**: Treating catalog presence as runtime support, applying every profile based on its summary text, or selecting profiles with partial rule coverage were rejected because they can silently omit normative behavior.

## Decision: Select Baseline Client as the initial 1.0 target, without making a claim

**Decision**: This draft selects `KMIPKIT-PROFILE-BASELINE-CLIENT` as KMIPKit's initial 1.0 target for demonstrating the one-or-more-client-profile requirement in Specification §14.1; that section does not mandate Baseline specifically. It remains only a target until evidence is complete and a reviewed release process makes any public claim. HTTPS Client stays conditional and is evaluated against each exact catalogued condition; supporting HTTPS transport alone is not evidence of HTTPS profile conformance.

**Rationale**: Specification §14.1 requires a KMIP client to conform to one or more client profiles but does not dictate which profile. Baseline Client is an applicable base client profile and is the narrowest target consistent with the 1.0 product boundary. The accepted inventory decision explicitly separates target selection, applicability, evidence completion, and public claims.

**Alternatives considered**: Claiming Baseline from operation coverage or selecting every applicable profile was rejected because a profile claim requires every normative statement in its clauses and subclauses plus the project evidence gate. Selecting no target was rejected because §14.1 requires one or more profiles.

The selected target is stored in the project-owned `specification/catalog/profile-targets.json`. The catalog profile `claim_state` remains an independent evidence-lifecycle field; it is not used to derive target membership or a public claim.

## Decision: Treat profile operation lists as minimum capabilities, not allowlists

**Decision**: Profile validation checks that the client provides required capabilities and applies any explicit profile-specific message, transport, or response constraints. An otherwise valid operation is not rejected merely because it is absent from a profile's required-operation list. Profile validation is explicit and adds no hidden Query/Discover Versions exchanges or retries.

**Rationale**: The profile records describe requirements and selections for a conforming implementation; the source does not make their operation lists prohibitions on extra valid capabilities. The accepted public API architecture says profile validation is explicit and Query/Discover Versions remain explicit operations. The Constitution also requires in-scope MAY/OPTIONAL capabilities to be represented and usable.

**Alternatives considered**: Treating each profile operation list as a closed allowlist was rejected because it would invent prohibitions and could reduce valid KMIP 2.1 client capabilities.

## Decision: Keep profile requirements, defaults, and secret choices distinct

**Decision**: Apply only profile-specific defaults supported by an exact normative source and applicable condition. Preserve explicit application values. Never select cryptographic algorithms, key sizes, usage, or protection policy implicitly. Report incompatible selected profile requirements as a deterministic validation error.

**Rationale**: The public API architecture permits normative mechanical defaults and requires cryptographic choices to remain explicit. Profile selections may add conditions but cannot silently override user intent or weaken base validation.

**Alternatives considered**: Applying every profile default unconditionally or resolving conflicts by selection order was rejected as unpredictable and unsafe.

## Decision: Missing official profile fixtures block evidence; do not reconstruct them

**Decision**: Treat a profile test case as passing only when its source-backed evidence is available and actually verified. An unavailable fixture remains unavailable and blocks eligibility when required by the profile or project conformance policy. Do not recreate missing fixture bytes from descriptions or informative wire examples, and do not fetch fixtures during builds.

**Rationale**: `specs/002-normative-inventory` records absent fixtures as evidence gaps and forbids reconstruction from titles. `docs/compliance/conformance.md` requires every mandatory test for a named profile to pass. At the time this decision was recorded, all 16 Baseline Client mandatory fixtures were unavailable; they were pinned on 2026-10-13, but remain unexecuted evidence.

**Alternatives considered**: Treating an absent fixture as a pass, reconstructing it from a description, or downloading mutable content in CI was rejected by the inventory contract and document hierarchy.

## Decision: Separate readiness calculation from release claim publication

**Decision**: The feature reports target selection and runtime-validation availability as separate booleans and exposes one effective evidence/readiness lifecycle value with blockers. It recomputes current evidence before comparing the result with catalog `claim_state`. The Baseline profile remains `evidence_incomplete` because execution results for its 16 mandatory fixtures are absent; this lifecycle value is not a published claim. This feature never emits a public support claim. A separate reviewed release process may publish only after all applicable normative requirements, mandatory tests, and project release gates pass.

**Rationale**: The accepted catalog data model and ADR-0009 separate implementation, profile conformance, interoperability, and certification. Human review and release governance remain authoritative.

**Alternatives considered**: Publishing a claim as soon as a local readiness function returns `evidence_complete` was rejected because release approval and evidence review are separate gates.

## Historical evidence baseline recorded on 2026-10-07

- The catalog contains 18 client profile records; 15 are marked `client_1_0` or `conditional`.
- On 2026-10-07, the GitHub branch-protection API returned `Branch not protected` for `release/1.0.0`, and the repository ruleset listing returned no rulesets. At that snapshot the protected-push evidence gate was unsatisfied; the live state must be checked again by the readiness workflow and at T043.
- At that snapshot, Baseline Client's catalog record linked 10 Profile requirement IDs and 16 mandatory test IDs, and all 16 fixture records had `fixture_availability: unavailable`. The fixture availability state was superseded on 2026-10-13; see the current snapshot below.
- HTTPS Client is marked `conditional`, links 19 requirement IDs and 17 mandatory test IDs, and depends on Baseline Client.
- The source catalog at that snapshot marked profiles `not_claimed`; this feature recorded Baseline readiness as `evidence_incomplete` because its 16 mandatory fixtures were unavailable. No public profile claim was emitted.
- The 1.0 target remains TTLV, KMIP 2.1, TLS 1.3 with mutual TLS, raw TLS and HTTP/1.1 over HTTPS. JSON/XML and server-initiated operations remain outside this feature's scope.

## Pinned-source client-profile closure baseline (2026-10-13 historical snapshot; superseded below)

This is a current-state audit of the checked-in catalog against the pinned KMIP Specification v2.1 and Profiles v2.1 HTML, not a conformance result. The exact catalog sets remain in each profile's `direct_source_clause_ids`, `source_clause_ids`, `requirement_ids`, `dependency_profile_ids`, and `test_case_ids` arrays in `specification/catalog/kmip-2.1.json`. The pinned-source oracle at `tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json` records 18 profiles and `source_closure_status: complete`, but that status is not yet independently validated: the Tape Library inclusion-root defect found by T001 is corrected and tested, while the broader clause and requirement review remains open. The requirement crosswalk is explicitly incomplete (`requirement_crosswalk_complete: false`, 104 source-only requirement gaps, and two gap proposals). T003's source review and regression tests remain open; the current oracle status is not proof of a complete source or requirement mapping. Counts below distinguish actual stored profile arrays from normative closure counts.

Seventeen client-profile records directly include `KMIPKIT-CLAUSE-PROF-3.1-001`; HTTPS receives §3.1 transitively through its Baseline dependency, because its own §6.4 references §3.2 and Baseline §6.1. All 18 records directly include the shared Profiles §6.1 roots `KMIPKIT-CLAUSE-PROF-6.1-001` through `-004`; each also includes its profile-specific direct clause roots summarized below. Pinned Profiles §6.1 items 1 and 2 require support for KMIP-SPEC and the Basic Authentication Suite conditions in §3.1. Four normative inclusion edges are catalogued: the shared §6.1 roots `-001` and `-002` map respectively to the whole `KMIPKIT-SRC-spec` source and `KMIPKIT-SRC-profiles` §3.1 with its subclauses; PKCS#11 Client §5.18.3 item 2 maps to Profiles §5.18.1; Tape Library Client §6.28 item 6 maps to Profiles §5.12.6. The Tape edge is corrected in the catalog and independent oracle to `KMIPKIT-CLAUSE-PROF-6.28-006` (ordinal 6, the one-or-more Mandatory Test Cases clause); item 7 is Baseline Client conformance. T003's regression test checks this source clause and the pinned locator. The shared §6.1 edges apply to all profiles and are not Baseline-only. The pinned Profiles text is at `specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html` lines 10479–10485, 10399–10412, and 11129–11142; §§3.1, 3.1.1, and 3.1.2 begin at lines 3997, 4007, and 4058.

The Baseline profile currently stores 897 clause IDs and 277 `requirement_ids`; those raw catalog fields are not a proof of complete requirement closure. `KMIPKIT-REQ-SPEC-6.1.57-006` is deliberately absent from Baseline's client requirement closure: although its candidate source clause is in the clause closure and the catalog scope says `client_1_0`, the pinned-source review records the actor as unresolved and `expected_client_requirement_ids` as empty. Do not count scope alone as a client actor assignment. At the earlier snapshot represented here, 54 catalog requirement records marked `client_1_0` had an empty `source_clause_ids` array. That count is historical and superseded: the current row-level inventory has 55 such records in total, with 25 marked `client_1_0` and 30 `profile_conditional` (see the current source-review section below). The 25 client 1.0 rows appear in Baseline's stored `requirement_ids`; the remaining gap is source-clause linkage and independently reviewed row-level closure, not Baseline list membership. The earlier 2026-10-11 claim of 344 Specification requirements plus 10 Profile requirements (354 total) was not a count of the stored Baseline `requirement_ids` and is withdrawn pending the independent closure calculation. The ten stored Profile requirement IDs include `KMIPKIT-REQ-PROF-3.1.1-002`, `-004`, `-005`, `KMIPKIT-REQ-PROF-3.1.2-002`, and `-005`.

| Client profile | Applicability | Direct Profiles sections | Stored clauses | Stored `requirement_ids` | Dependency | Linked test IDs | Fixture records available / unavailable |
|---|---|---|---:|---:|---|---:|---:|
| AES XTS | `client_1_0` | 3.1, 5.13.1, 6.1, 6.30 | 904 | 281 | Baseline | 18 | 16 / 2 |
| Asymmetric Key Lifecycle | `client_1_0` | 3.1, 5.8.1, 6.1, 6.16 | 904 | 281 | Baseline | 20 | 16 / 4 |
| Baseline | `client_1_0` | 3.1, 5.1.1, 6.1 | 897 | 277 | none | 16 | 16 / 0 |
| Cryptographic Advanced | `client_1_0` | 3.1, 5.9.2, 6.1, 6.19 | 905 | 282 | Baseline | 34 | 16 / 18 |
| Cryptographic Basic | `client_1_0` | 3.1, 5.9.1, 6.1, 6.18 | 905 | 282 | Baseline | 37 | 16 / 21 |
| Cryptographic RNG | `client_1_0` | 3.1, 5.9.3, 6.1, 6.20 | 905 | 282 | Baseline | 21 | 16 / 5 |
| HTTPS | `conditional` | 3.2, 5.3.1, 6.1, 6.4 | 918 | 291 | Baseline | 17 | 16 / 1 |
| JSON | `out_of_scope` | 3.1, 5.5.2, 6.1, 6.8 | 905 | 273 | Baseline | 17 | 16 / 1 |
| Opaque Managed Object Store | `client_1_0` | 3.1, 5.10.1, 6.1, 6.24 | 904 | 281 | Baseline | 18 | 16 / 2 |
| PKCS#11 | `out_of_scope` | 3.1, 5.18.3, 6.1, 6.34 | 911 | 278 | Baseline | 17 | 16 / 1 |
| Quantum Safe | `client_1_0` | 3.1, 5.15, 5.17, 6.1, 6.32 | 910 | 283 | Baseline | 18 | 16 / 2 |
| Storage Array with Self-Encrypting Drives | `client_1_0` | 3.1, 5.11.1, 6.1, 6.26 | 905 | 282 | Baseline | 19 | 16 / 3 |
| Symmetric Key Foundry Advanced | `client_1_0` | 3.1, 5.7.3, 6.1, 6.14 | 904 | 280 | Baseline | 20 | 16 / 4 |
| Symmetric Key Foundry Basic | `client_1_0` | 3.1, 5.7.1, 6.1, 6.12 | 904 | 281 | Baseline | 20 | 16 / 4 |
| Symmetric Key Foundry Intermediate | `client_1_0` | 3.1, 5.7.2, 6.1, 6.13 | 904 | 280 | Baseline | 20 | 16 / 4 |
| Symmetric Key Lifecycle | `client_1_0` | 3.1, 5.6.1, 6.1, 6.10 | 904 | 281 | Baseline | 17 | 16 / 1 |
| Tape Library | `client_1_0` | 3.1, 5.12.2, 5.12.3, 5.12.4, 6.1, 6.28 | 931 | 305 | Baseline | 19 | 16 / 3 |
| XML | `out_of_scope` | 3.1, 5.4.2, 6.1, 6.6 | 905 | 273 | Baseline | 17 | 16 / 1 |

These values were recomputed read-only from the catalog: the client profiles have 365 profile-to-test associations and 93 distinct test IDs; the distinct linked fixtures comprise 16 available and 77 unavailable records. All 18 profiles link the same 16 mandatory Baseline cases, and all 16 corresponding fixtures are now pinned. Fixture presence is not execution evidence. JSON, PKCS#11, and XML remain `out_of_scope`; HTTPS remains `conditional`; 14 profiles remain `client_1_0`. Server-only records are outside client closures. The table is an auditable snapshot of stored metadata, while T002/T003 still need to certify source-derived clauses and requirements.

PKCS#11 Client closure includes Profiles §5.18.1 through the explicit inclusion in §5.18.3 item 2. The five clauses stay in its source closure, but their profile-specific requirements remain out of scope for the TTLV-only 1.0 line; the lowercase `may`/`required` wording in §5.18.1 ordinals 1–3 does not create normative requirement records. Its current catalog record stores 911 clause IDs and 278 `requirement_ids`; these raw counts do not independently certify requirement closure.

## Normative-reference cross-check for T002 (2026-10-13 historical snapshot; superseded below)

The pinned Profiles HTML supports the two shared Baseline roots and the profile sections in the 18-row table above. Profiles §6.1 item 1 says client implementations SHALL support `[KMIP-SPEC]` (lines 10479–84); the corresponding whole-source inclusion means all 877 catalogued Specification clauses are in the Baseline clause closure, including clauses that later need a client/server/out-of-scope disposition. Profiles §6.1 item 2 says SHALL support the Basic Authentication Suite conditions in §3.1 (lines 10484–85); the applicable §3.1, §3.1.1, and §3.1.2 records are included through the section edge. This source wording and closure do not imply that every Specification clause becomes a client 1.0 implementation obligation: the actor, direction, applicability, conditions, and source keyword remain attached to each reviewed requirement.

The current oracle has `source_requirement_reviews` for 890 source clauses, but still records 104 source-only requirement gaps and two proposals; `requirement_crosswalk_complete` is false. Separately, 55 catalog requirements backed by table-body rows have no `source_clause_ids`. All 55 have `table_body_row` locators, stable `source_gap_id` values, OASIS sections and row labels, and HTML line ranges; 24 are marked `verified`, six `scoped_verified`, and 25 `unassigned`. Their current scope is 25 `client_1_0` rows, all present in Baseline's stored `requirement_ids`, and 30 `profile_conditional` rows, excluded from every profile closure while applicability is unassigned. The five §5.5–§5.7 attribute rows have an approved feature owner; the 25 key-format/material rows still lack one. Row-level citations alone do not establish actor assignment, approved scope, or category-specific executable coverage. The independent source crosswalk must adjudicate those properties without minting clause IDs for rows that lack an existing source-clause candidate. Their stable IDs are: `KMIPKIT-REQ-SPEC-6.1.12-003`, `KMIPKIT-REQ-SPEC-6.1.12-005`, `KMIPKIT-REQ-SPEC-6.1.12-006`, `KMIPKIT-REQ-SPEC-6.1.12-007`, `KMIPKIT-REQ-SPEC-6.1.13-005`, `KMIPKIT-REQ-SPEC-6.1.13-006`, `KMIPKIT-REQ-SPEC-6.1.14-007-001`, `KMIPKIT-REQ-SPEC-6.1.14-009`, `KMIPKIT-REQ-SPEC-6.1.14-010`, `KMIPKIT-REQ-SPEC-6.1.14-011`, `KMIPKIT-REQ-SPEC-6.1.14-012`, `KMIPKIT-REQ-SPEC-6.1.18-005`, `KMIPKIT-REQ-SPEC-6.1.18-006`, `KMIPKIT-REQ-SPEC-6.1.18-007`, `KMIPKIT-REQ-SPEC-6.1.18-008`, `KMIPKIT-REQ-SPEC-6.1.19-003`, `KMIPKIT-REQ-SPEC-6.1.19-004`, `KMIPKIT-REQ-SPEC-6.1.19-005`, `KMIPKIT-REQ-SPEC-6.1.19-006`, `KMIPKIT-REQ-SPEC-6.1.23-002-001`, `KMIPKIT-REQ-SPEC-6.1.30-001`, `KMIPKIT-REQ-SPEC-6.1.30-002`, `KMIPKIT-REQ-SPEC-6.1.30-003`, `KMIPKIT-REQ-SPEC-6.1.31-001`, `KMIPKIT-REQ-SPEC-3.1-007`, `KMIPKIT-REQ-SPEC-3.2-003`, `KMIPKIT-REQ-SPEC-3.3-010`, `KMIPKIT-REQ-SPEC-3.3-011`, `KMIPKIT-REQ-SPEC-3.4-002`, `KMIPKIT-REQ-SPEC-3.5-002`, `KMIPKIT-REQ-SPEC-3.5-003`, `KMIPKIT-REQ-SPEC-3.5-004`, `KMIPKIT-REQ-SPEC-3.5-005`, `KMIPKIT-REQ-SPEC-3.6-002`, `KMIPKIT-REQ-SPEC-3.6-004`, `KMIPKIT-REQ-SPEC-3.6-005`, `KMIPKIT-REQ-SPEC-3.7-003`, `KMIPKIT-REQ-SPEC-3.8-002`, `KMIPKIT-REQ-SPEC-3.8-003`, `KMIPKIT-REQ-SPEC-3.9-002`, `KMIPKIT-REQ-SPEC-3.9-003`, `KMIPKIT-REQ-SPEC-3.9-004`, `KMIPKIT-REQ-SPEC-3.10-002`, `KMIPKIT-REQ-SPEC-3.10-003`, `KMIPKIT-REQ-SPEC-3.10-004`, `KMIPKIT-REQ-SPEC-3.11-002`, `KMIPKIT-REQ-SPEC-3.11-003`, `KMIPKIT-REQ-SPEC-3.12-002`, `KMIPKIT-REQ-SPEC-3.12-003`, `KMIPKIT-REQ-SPEC-4.31-004`, `KMIPKIT-REQ-SPEC-4.31-005`, `KMIPKIT-REQ-SPEC-5.5-003`, `KMIPKIT-REQ-SPEC-5.5-004`, `KMIPKIT-REQ-SPEC-5.5-005`, `KMIPKIT-REQ-SPEC-5.6-002`, and `KMIPKIT-REQ-SPEC-5.7-002`.

Profiles lists the 93 distinct test-case IDs across 365 profile associations. The pinned Test Cases CN01 HTML at `specification/oasis/kmip-2.1/upstream/kmip-testcases-v2.1-cn01.html` contains none of those 93 test-case IDs; their applicability and links originate in the Profiles HTML. Therefore the IDs are not independently cross-referenced by CN01, and only the fixed fixture records, exact Profiles source links, and execution results may establish their evidence state. The three known source href/display-label mismatches are catalogued as `KMIPKIT-DISC-027` and `KMIPKIT-DISC-028`; they must remain explicit source discrepancies, not be silently repaired by inference. The fixture status is 16 available shared mandatory Baseline files and 77 unavailable linked files; no execution result is implied by availability.

The reviewed source-backed inclusion set now has seven normative edges: shared §6.1 items 1–3 → the whole Specification, Profiles §3.1, and Profiles §5.1.1; PKCS#11 Client §5.18.3 item 2 → Profiles §5.18.1; Tape Library Client §6.28 item 6 → Profiles §5.12.6; and Foundry Intermediate/Advanced Client §§6.13/6.14 item 3 → literal Profiles §5.7.1. The §5.1.1 edge is newly recorded from pinned Profiles §6.1 item 3 (line 10486). Its separate condition inventory covers all eight top-level conditions and their 30 nested values across Profiles lines 4529–4636, including the sentence that introduces them as Baseline Client conformance conditions. It assigns no standalone REQ IDs; feature and evidence mappings remain pending. The catalog's existing §5.1.1 item 1 candidate contains the lowercase descriptive word “required” and has no standalone requirement ID. The Tape edge remains rooted at `KMIPKIT-CLAUSE-PROF-6.28-006`, ordinal 6; item 7 is the separate Baseline Client conformance clause. T002 remains open for the broader closure and requirement crosswalk audit. The pinned Profiles source lines are 10399–10412 and 11129–11142; the Specification §14.1 client-profile gate is at `kmip-spec-v2.1-os.html` lines 66044–66047.

## Fixture inventory update - 2026-10-13

The 16 Baseline mandatory XML fixtures linked from OASIS Profiles §§5.1.3 and 5.6.3 have since been pinned byte-for-byte from the OASIS Standard URLs. The catalog now records them as available, but no complete official-case execution result is recorded. Of 203 linked test-case XML fixtures, 23 are pinned and 180 remain absent. Fixture availability is not a passing result, and Baseline remains `evidence_incomplete`; see `implementation-evidence.md` for hashes, test commands, and the T036 execution gap.

Every profile links the shared official IDs `BL-M-{1..13}-21` and `SKLC-M-{1..3}-21`. The following are the additional official IDs linked by each profile; numeric ranges are inclusive:

| Client profile | Additional official case IDs |
|---|---|
| AES XTS | `AX-M-{1..2}-21` |
| Asymmetric Key Lifecycle | `AKLC-M-{1..3}-21`, `AKLC-O-1-21` |
| Baseline | none |
| Cryptographic Advanced | `CS-AC-M-{1..8}-21`, `CS-AC-M-OAEP-{1..10}-21` |
| Cryptographic Basic | `CS-BC-M-{1..14}-21`, `CS-BC-M-CHACHA20-{1..3}-21`, `CS-BC-M-CHACHA20POLY1305-1-21`, `CS-BC-M-GCM-{1..3}-21` |
| Cryptographic RNG | `CS-RNG-M-1-21`, `CS-RNG-O-{1..4}-21` |
| HTTPS | `MSGENC-HTTPS-M-1-21` |
| JSON | `MSGENC-JSON-M-1-21` |
| Opaque Managed Object Store | `OMOS-M-1-21`, `OMOS-O-1-21` |
| PKCS#11 | `PKCS11-M-1-21` |
| Quantum Safe | `QS-M-1-12`, `QS-M-2-21` |
| Storage Array with Self-Encrypting Drives | `SASED-M-{1..3}-21` |
| Symmetric Key Foundry Advanced | `SKFF-M-{9..12}-21` |
| Symmetric Key Foundry Basic | `SKFF-M-{1..4}-21` |
| Symmetric Key Foundry Intermediate | `SKFF-M-{5..8}-21` |
| Symmetric Key Lifecycle | `SKLC-O-1-21` |
| Tape Library | `TL-M-{1..3}-21` |
| XML | `MSGENC-XML-M-1-21` |

All non-Baseline client-profile records depend on Baseline. The 93 distinct linked case IDs currently resolve to 16 available fixtures (the shared mandatory Baseline cases) and 77 unavailable fixtures; availability does not establish execution or correct label-to-href pairing. The audit found source mismatches: §5.9.7.13 labels `CS-BC-M-13-21` but points to `CS-BC-M-14-21.xml` and displays `CS-BC-M-21-21.xml`; §5.9.7.14 labels `CS-BC-M-14-21` but points to that same `CS-BC-M-14-21.xml` and displays `CS-BC-M-20-21.xml`; §5.17.1 labels `QS-M-1-12` but points to and displays `QS-M-1-21.xml`. These source defects are tracked as `KMIPKIT-DISC-027` and `KMIPKIT-DISC-028` in the catalog. None of the 93 IDs occurs in the separately pinned CN01 Test Cases HTML. The case IDs are present in Profiles, while the separate Test Cases document supplies no cross-reference for them; no execution result is inferred. Four inclusion edges are present in the catalog, but the Tape edge root is incorrectly linked as described above; the source closure attribution must be corrected before calling the catalog closure complete. CN01 case links are evidence, and the Usage Guide has informative authority, so neither adds normative clauses or requirements. The client-source evidence for PKCS#11 and Tape Library is in the pinned Profiles HTML at lines 10399–10412 and 11129–11142; the source test links are at lines 7910–7921 and 9386–9397.

The catalog-level mapping audit found no unresolved profile clause or requirement IDs and no stored requirement whose `source_clause_ids` falls outside its profile's stored `source_clause_ids` closure. This confirms internal link consistency, not independent source closure. The applicability counts remain 14 `client_1_0`, one `conditional` (HTTPS), and three `out_of_scope` (JSON, PKCS#11, XML); server-only records remain outside client closures.

The whole-Spec closure of every client profile includes clause `KMIPKIT-CLAUSE-SPEC-14.1-002`, whose requirement `KMIPKIT-REQ-SPEC-14.1-002` is marked `profile_conditional` and says a claimed client profile SHALL conform to its normative clauses and subclauses (pinned Specification §14.1, lines 66041–66047). Its requirement record already lists all 18 client IDs in `profile_ids`, but no profile's `requirement_ids` currently includes it. **Disposition:** for any of the 18 client-profile closures evaluated for a possible conformance claim, treat this requirement as the claim-level normative gate and include its ID in that profile's expected requirement closure. The oracle and T005 must retain the existing `profile_ids` and ensure the requirement appears in each profile's `requirement_ids`/expected closure so generated readiness carries the condition. This meta-requirement is evaluated by checking every applicable profile-clause statement, not by adding a stand-alone operation rule or test fixture. For JSON, PKCS#11, and XML the mapping is future/conditional metadata and does not change their `out_of_scope` status or create a 1.0 evidence obligation. Its inclusion does not make the current client or profile conformant and does not publish a claim. The exact expected requirement closure must be derived from the pinned source by T003; earlier closure totals of 354 and 355 are withdrawn. No profile claim is made.

## Security decisions from independent review

- Read only the fixed `specification/catalog/profile-targets.json` path with the existing safe-read helper and a 64 KiB maximum. Reject symlinks/reparse points, invalid UTF-8 or JSON, duplicate object keys, unknown fields, duplicate IDs, unresolved IDs, and IDs outside the approved client scope before generating output.
- Read the source catalog with repository safe I/O and `load_validated_catalog`, retaining its existing size and schema limits. The target manifest is only for target generation/reporting; runtime selection remains an explicit caller choice and never consults target membership.
- Generate Rust string literals with a dedicated encoder. Quotes, backslashes, control characters, and Unicode must round-trip as string data; tests must verify generated source compiles and cannot gain injected syntax from metadata.
- Validate catalog-internal links and fixture availability, then recompute readiness using current implementation and test evidence before trusting catalog `claim_state`. If the catalog says `evidence_complete` despite a missing, failing, unavailable, or contradictory gate, report effective `evidence_incomplete` and the stale-state blocker. If evidence passes but catalog state is lower, retain that lower state until an explicit reviewed catalog update; never auto-upgrade catalog data.

## Decisions from independent design review

- Represent explicit normative references as stable `normative_inclusions` catalog edges (`inclusion_id`, `source_clause_id`, `included_source_id`, nullable `included_section`). A null section includes every clause in the target source; a non-null section includes that section and its dot-delimited subclauses. Keep each profile's `direct_source_clause_ids` separate from its computed `source_clause_ids`; compute closure over included source sections and same-source subclauses. Keep informative references outside that closure, reject unknown/cyclic edges, and verify all 18 client profiles against the pinned sources before assigning feature ownership or generating Rust metadata.
- Keep an independently reviewed, pinned-source closure oracle in `tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json` for all 18 client profiles. Tests compare catalog direct clauses, normative inclusion edges, closure clauses, and requirements against this oracle; the generator and runtime never read the oracle, so it is verification evidence rather than a second production source of truth.
- All 18 client profiles directly include the shared §6.1 roots and their two catalogued normative inclusions; current closure counts and Baseline-specific inherited Profile requirements are recorded above. The five inherited Profile requirement IDs from §§3.1.1 and 3.1.2 are already present in the current catalog. T003/T005 must compare all stored IDs against the independent pinned-source oracle and apply the §14.1-002 claim-level disposition above. Retain server-only and out-of-scope dispositions and existing protocol-spec ownership; preserve each source keyword and conditional disposition. TLS 1.3-only product scope does not erase a normative source obligation.
- A PR workflow cannot certify its own results: GitHub documents that `pull_request` executes the workflow file from the PR merge commit, while `push` runs against the pushed ref ([workflow event docs](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows), [securely using `pull_request_target`](https://docs.github.com/en/actions/reference/security/securely-using-pull_request_target)). Accept evidence only from a `push` run on `refs/heads/release/1.0.0` with GitHub context `ref_protected: true`; this boolean alone does not prove adequate rules. Query branch protection on every run using `GET /repos/{owner}/{repo}/branches/release%2F1.0.0/protection` and a short-lived GitHub App installation token scoped to read-only Administration permission. GitHub documents that this endpoint exposes branch protection details and requires Administration:read ([branch protection API](https://docs.github.com/en/rest/branches/branch-protection)). Validate the live response for required human review, stale-review dismissal, an explicitly present `bypass_pull_request_allowances` object with empty `users`, `teams`, and `apps` arrays, admin enforcement, and disabled force pushes/deletions. GitHub documents that configured bypass actors can bypass required-pull-request-review rules, so missing, malformed, or non-empty allowances fail closed; fail closed also if the token, API, or any required field is unavailable. The API token is never available to PR/schedule jobs and is not logged. PR and `schedule` runs remain useful for checks but cannot produce eligible evidence.
- Make verification manifests and current-run readiness ephemeral. The committed coverage report is regenerated only from stable checked-in inputs; no run ID, attempt, job conclusion, or temporary manifest is written into committed generated files or catalog lifecycle state. The aggregate CI job may expose its separate same-run result for the exact commit under evaluation.

## Independent implementation audit rulings (2026-10-07)

The independent audit reported H1/H2/M1/M2 before runtime implementation. These rulings refine the already approved fail-closed contract; they do not expand product or protocol scope.

| Finding | Ruling | Required evidence |
|---|---|---|
| H1 — Baseline target versus runtime eligibility | `profile-targets.json` records Baseline as the 1.0 product target only. It does not grant runtime availability. Each in-scope requirement in the complete profile closure must have a typed mapping and a passing category-appropriate executable test before runtime selection; a gap rejects selection before I/O. Missing, unexecuted, or failing mandatory fixtures independently block evidence completion and claims. | Target/runtime separation tests, full-closure typed-rule completeness tests, and a readiness report showing all 16 mandatory Baseline cases as blockers until their exact official IDs have passing execution evidence. |
| H2 — Normative verification categories | Preserve each exact `source_keyword` and require category-appropriate tests: positive behavior for MUST/SHALL/REQUIRED; negative behavior for MUST NOT/SHALL NOT; behavior or approved deviation for SHOULD/RECOMMENDED; absent discouraged behavior or approved deviation for SHOULD NOT; and usable positive capability coverage for each in-scope MAY/OPTIONAL. A generic “tested” marker cannot satisfy another category. | Tests that reject missing or wrong-category coverage for each category; exact catalog source keyword and stable requirement ID remain traceable. |
| M1 — Response-validation delivery state | Outbound profile failures are rejected before writes with `NotSent`. A profile failure after response receipt preserves the exchange delivery state, including `ResponseStarted`, in the stable client error. | Fake-transport response cases asserting no write for preflight errors and asserting `ResponseStarted` after an invalid response; no secret or free-form cause appears. |
| M2 — Traceability-test evidence | Traceability regression tests require a distinct signed Red commit with the observed expected failure and a separate signed Green commit with the minimal linkage/implementation and observed pass. Refactor evidence remains a third distinct commit when refactoring is needed. Reviewer-owned checklists remain unchecked by agents. | `implementation-evidence.md` records commit IDs, exact commands, and outcomes for Red/Green/Refactor; checklist ownership remains unchanged. |

The Baseline target and runtime-availability statuses must be exposed separately in generated evidence. No public claim is created by this implementation. Live branch-protection evidence remains unavailable when the GitHub API reports that `release/1.0.0` is not protected; protected Actions-secret availability also remains an external prerequisite.

## Normative sources

- OASIS KMIP Specification v2.1, §14.1: client implementation conformance and the complete normative-clause requirement for any claimed client profile.
- OASIS KMIP Profiles v2.1, §§3.1, 3.1.1, 3.1.2, 5.1.1, 5.1.3, 5.6.3, and 6.1: Basic Authentication Suite requirements inherited by Baseline Client and its profile conditions/tests.
- OASIS KMIP Profiles v2.1, §§3.2, 5.3.1, 5.3.3, and 6.4: HTTPS profile conditions and tests, retained as conditional.
- Additional in-scope profile clauses are cited by each stable `KMIPKIT-PROFILE-*`, `KMIPKIT-CLAUSE-PROF-*`, and `KMIPKIT-REQ-PROF-*` catalog record; the generator must preserve those exact source references.
- Test Cases v2.1 case IDs are evidence only; the pinned Profiles document and inventory record determine their applicability and fixture status.

## Implementation boundary

The runtime validator belongs in the Rust protocol/client boundary and consumes generated immutable profile metadata. The generator and report extension use the repository's pinned Python tooling and the catalog as input. Public C, Java, and Python selection surfaces and their parity belong to the later API/bindings work, which must expose equivalent behavior. No dependency on network access, dynamic OASIS parsing, or executable plug-ins is introduced.


## Source-review audit — 2026-10-14 historical snapshot

This is the T001–T002 source-review snapshot as of 2026-10-14; the later §4.31 update below supersedes its counts. It uses only the pinned Specification, Profiles, and Test Cases files under `specification/oasis/kmip-2.1/upstream/`, the checked-in catalog, and the independently maintained closure fixture. No upstream source or generated file was changed.

### Task disposition

- **T001 complete:** all 18 client-profile inventory records were compared with the source-reviewed closure fixture for direct roots, computed source-clause closure, requirement IDs, dependencies, test IDs, and fixture status. The full transitive Baseline clause baseline and its known requirement-mapping gaps are recorded below. This records the requested baseline; it does not certify a complete requirement crosswalk or profile conformance.
- **T002 remains open:** all direct roots and clause closures match the pinned-source oracle, and the source text supports the four normative inclusion edges. Exact requirement mapping is not complete: the oracle sets `requirement_crosswalk_complete: false` and still contains source-only review rows. Of 55 catalog table-body requirements without `source_clause_ids`, 25 are `client_1_0` and 30 are `profile_conditional`. The official case source audit is complete for the 203 catalogued case rows; the broader profile closure, applicability, and crosswalk review remains open. T003 was not started.

### Baseline Client closure and source rulings

KMIP Profiles v2.1 §6.1 item 1 says client implementations SHALL support `[KMIP-SPEC]` (pinned Profiles HTML lines 10479–10484). Its whole-source edge is `KMIPKIT-INCLUSION-PROF-BASELINE-SPEC`, from `KMIPKIT-CLAUSE-PROF-6.1-001` to `KMIPKIT-SRC-spec` with `included_section: null`. All 877 catalogued Specification clauses are in Baseline Client's computed closure. This includes source clauses later dispositioned as server-only or out of scope; source inclusion does not make those client 1.0 requirements.

Profiles §6.1 item 2 says clients SHALL support Basic Authentication Suite conditions in §3.1 (lines 10484–10485). Its edge is `KMIPKIT-INCLUSION-PROF-BASELINE-BASIC-AUTH`, from `KMIPKIT-CLAUSE-PROF-6.1-002` to `KMIPKIT-SRC-profiles` §3.1. Same-source subclause expansion includes §§3.1.1–3.1.4. Baseline's full 897-keyword-candidate clause closure is 877 Specification clauses plus 20 Profiles clauses: §3.1 (1), §3.1.1 (5), §3.1.2 (5), §3.1.3 (3), §3.1.4 (1), §5.1.1 (1), and §6.1 (4). The non-modal §5.1.1 conditions are inventoried separately and are not included in that clause count. Baseline has six direct Profiles roots across §§3.1, 5.1.1, and 6.1. The 278 stored requirement IDs are 268 Specification IDs plus 10 Profiles IDs; this stored count is not proof that the underlying source-to-requirement mapping is complete.

The source-reviewed §3.1.1/§3.1.2 records preserve actor, keyword, condition, and scope. `KMIPKIT-REQ-PROF-3.1.1-002` is a conditional client SHOULD; `KMIPKIT-REQ-PROF-3.1.1-004` is a client MAY marked `out_of_scope` under the approved TLS 1.3-only product boundary; `KMIPKIT-REQ-PROF-3.1.1-005` is a client/server SHALL NOT. `KMIPKIT-REQ-PROF-3.1.2-002` is a conditional client SHALL; `KMIPKIT-REQ-PROF-3.1.2-005` is a client/server SHALL NOT. The client-specific records are in Baseline's stored requirement set. The §3.1.3/§3.1.4 source clauses remain in closure with their source actor and applicability dispositions; they are not silently converted to client obligations.

The seven reviewed normative inclusions are:

| Inclusion | Pinned Profiles reference | Included source scope |
|---|---|---|
| `KMIPKIT-INCLUSION-PROF-BASELINE-SPEC` | §6.1 item 1 | Entire KMIP Specification v2.1 |
| `KMIPKIT-INCLUSION-PROF-BASELINE-BASIC-AUTH` | §6.1 item 2 | Profiles §3.1 and subclauses |
| `KMIPKIT-INCLUSION-PROF-BASELINE-CONDITIONS` | §6.1 item 3 | Profiles §5.1.1 |
| `KMIPKIT-INCLUSION-PROF-PKCS11-ENCODING` | §5.18.3 item 2 | Profiles §5.18.1 |
| `KMIPKIT-INCLUSION-PROF-TAPE-LIBRARY-TEST-CASES` | §6.28 item 6 | Profiles §5.12.6 |
| `KMIPKIT-INCLUSION-PROF-613-BASIC-SYMMETRIC-CLIENT` | §6.13 item 3 | Profiles §5.7.1 |
| `KMIPKIT-INCLUSION-PROF-614-BASIC-SYMMETRIC-CLIENT` | §6.14 item 3 | Profiles §5.7.1 |

The Tape Library edge is rooted at `KMIPKIT-CLAUSE-PROF-6.28-006`, item 6; item 7 is the separate Baseline conformance clause. PKCS#11 §5.18.1 remains in source closure while the profile is out of scope for the TTLV-only 1.0 line. The Usage Guide and Test Cases documents add no normative inclusion edges. Focused checks confirm informative references do not expand closure and server-only requirements do not enter the client requirement set.

The whole-Spec closure includes `KMIPKIT-REQ-SPEC-14.1-002`, the claim-level SHALL concerning every normative statement in a claimed profile's clauses and subclauses. The current catalog and oracle both include this requirement ID in all 18 client profile requirement arrays. It is a readiness gate, not a stand-alone operation rule, test case, conformance result, or public claim.

### Current client-profile inventory

The oracle contains 18 client profiles: 14 `client_1_0`, one `conditional` (HTTPS), and three `out_of_scope` (JSON, PKCS#11, XML). The following stored counts and case records were recomputed from the current catalog and compared with the pinned-source oracle. The section column shows Profiles sections attributed to a closure, including normative inclusion targets; dependency-derived Baseline membership is shown separately. Counts describe current metadata, not a complete source-requirement mapping.

| Client profile | Applicability | Profiles sections attributed to closure | Clauses | Requirement IDs | Dependency | Linked test IDs | Fixtures available / unavailable |
|---|---|---|---:|---:|---|---:|---:|
| AES XTS Client | `client_1_0` | 3.1, 5.13.1, 6.1, 6.30 | 904 | 282 | Baseline | 18 | 16 / 2 |
| Asymmetric Key Lifecycle Client | `client_1_0` | 3.1, 5.8.1, 6.1, 6.16 | 904 | 282 | Baseline | 20 | 16 / 4 |
| Baseline Client | `client_1_0` | 3.1, 5.1.1, 6.1 | 897 | 278 | none | 16 | 16 / 0 |
| Advanced Cryptographic Client | `client_1_0` | 3.1, 5.9.2, 6.1, 6.19 | 905 | 283 | Baseline | 34 | 16 / 18 |
| Basic Cryptographic Client | `client_1_0` | 3.1, 5.9.1, 6.1, 6.18 | 905 | 283 | Baseline | 37 | 16 / 21 |
| RNG Cryptographic Client | `client_1_0` | 3.1, 5.9.3, 6.1, 6.20 | 905 | 283 | Baseline | 21 | 16 / 5 |
| HTTPS Client | `conditional` | 3.2, 5.3.1, 6.1, 6.4 | 918 | 292 | Baseline | 17 | 16 / 1 |
| JSON Client | `out_of_scope` | 3.1, 5.5.2, 6.1, 6.8 | 905 | 274 | Baseline | 17 | 16 / 1 |
| Opaque Managed Object Store Client | `client_1_0` | 3.1, 5.10.1, 6.1, 6.24 | 904 | 282 | Baseline | 18 | 16 / 2 |
| PKCS#11 Client | `out_of_scope` | 3.1, 5.18.1, 5.18.3, 6.1, 6.34 | 911 | 279 | Baseline | 17 | 16 / 1 |
| Quantum Safe Client | `client_1_0` | 3.1, 5.15, 5.17, 6.1, 6.32 | 910 | 284 | Baseline | 18 | 16 / 2 |
| Storage Array with Self-Encrypting Drives Client | `client_1_0` | 3.1, 5.11.1, 6.1, 6.26 | 905 | 283 | Baseline | 19 | 16 / 3 |
| Advanced Symmetric Key Foundry Client | `client_1_0` | 3.1, 5.7.3, 6.1, 6.14 | 904 | 281 | Baseline | 20 | 16 / 4 |
| Basic Symmetric Key Foundry Client | `client_1_0` | 3.1, 5.7.1, 6.1, 6.12 | 904 | 282 | Baseline | 20 | 16 / 4 |
| Intermediate Symmetric Key Foundry Client | `client_1_0` | 3.1, 5.7.2, 6.1, 6.13 | 904 | 281 | Baseline | 20 | 16 / 4 |
| Symmetric Key Lifecycle Client | `client_1_0` | 3.1, 5.6.1, 6.1, 6.10 | 904 | 282 | Baseline | 17 | 16 / 1 |
| Tape Library Client | `client_1_0` | 3.1, 5.12.2, 5.12.3, 5.12.4, 5.12.6, 6.1, 6.28 | 931 | 306 | Baseline | 19 | 16 / 3 |
| XML Client | `out_of_scope` | 3.1, 5.4.2, 6.1, 6.6 | 905 | 274 | Baseline | 17 | 16 / 1 |

All 17 non-Baseline client profiles depend on Baseline. There are 365 profile-to-test associations and 93 distinct official profile test IDs. All share the 16 mandatory Baseline IDs (`BL-M-1-21` through `BL-M-13-21` and `SKLC-M-1-21` through `SKLC-M-3-21`); their XML fixtures are pinned. Seventy-seven other distinct linked fixture records remain unavailable. Fixture presence is not execution evidence, and no profile claim is made. The Profiles v2.1 source supplies these profile-case IDs and links. The separate pinned Test Cases v2.1 CN01 HTML contains none of the 93 IDs, so it does not independently cross-reference these Profiles test cases.

Three source label/href/display mismatches remain recorded as catalog discrepancies and are not repaired by inference: Profiles §5.9.7.13 labels `CS-BC-M-13-21` but points to `CS-BC-M-14-21.xml` and displays `CS-BC-M-21-21.xml`; §5.9.7.14 labels `CS-BC-M-14-21`, points to the same `CS-BC-M-14-21.xml`, and displays `CS-BC-M-20-21.xml`; §5.17.1 labels `QS-M-1-12` but points to and displays `QS-M-1-21.xml`. The catalog records these under `KMIPKIT-DISC-027` and `KMIPKIT-DISC-028`.

### Exact requirement-mapping gaps

The oracle records 505 requirement-crosswalk rows, 890 source-requirement review rows, and `requirement_crosswalk_complete: false`. Its 107 remaining `source_only_requirement_gaps` are source-review records, not 107 confirmed client requirements: 85 cite the pinned Specification and 22 cite the pinned Profiles document. Status counts are 62 source obligations covered by an existing stable label, 20 unlabeled source statements or boundary items, 10 reviewed procedure-context rows, 3 reviewed generic-parameter-context rows, two scope-unresolved rows, and one each for unresolved force/actor, missing stable label, unresolved source interpretation, unresolved keyword force, unresolved actor/capability classification, server-actor disposition, mapping to attribute policy, unresolved actor/client-capability, unresolved actor/scope, and a stable-label proposal not yet catalogued. Exact `gap_id`, source section, source line span, excerpt, and disposition are preserved in `tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json`.

Separately, 57 catalog requirements have no `source_clause_ids`: 27 have `scope_state: client_1_0` and 30 have `scope_state: profile_conditional`. All 57 point to row-level Specification references and carry a `source_gap_id`; the 27 client 1.0 rows appear in Baseline's stored requirement array. The current catalog statuses are 24 `verified`, six `scoped_verified`, and 27 `unassigned`. A verified row locator is not a source-clause mapping or proof of actor, scope, or category coverage. The affected stable IDs are:

`KMIPKIT-REQ-SPEC-2.3-003`, `KMIPKIT-REQ-SPEC-3.1-007`, `KMIPKIT-REQ-SPEC-3.10-002`, `KMIPKIT-REQ-SPEC-3.10-003`, `KMIPKIT-REQ-SPEC-3.10-004`, `KMIPKIT-REQ-SPEC-3.11-002`, `KMIPKIT-REQ-SPEC-3.11-003`, `KMIPKIT-REQ-SPEC-3.12-002`, `KMIPKIT-REQ-SPEC-3.12-003`, `KMIPKIT-REQ-SPEC-3.2-003`, `KMIPKIT-REQ-SPEC-3.3-010`, `KMIPKIT-REQ-SPEC-3.3-011`, `KMIPKIT-REQ-SPEC-3.4-002`, `KMIPKIT-REQ-SPEC-3.5-002`, `KMIPKIT-REQ-SPEC-3.5-003`, `KMIPKIT-REQ-SPEC-3.5-004`, `KMIPKIT-REQ-SPEC-3.5-005`, `KMIPKIT-REQ-SPEC-3.6-002`, `KMIPKIT-REQ-SPEC-3.6-004`, `KMIPKIT-REQ-SPEC-3.6-005`, `KMIPKIT-REQ-SPEC-3.7-003`, `KMIPKIT-REQ-SPEC-3.8-002`, `KMIPKIT-REQ-SPEC-3.8-003`, `KMIPKIT-REQ-SPEC-3.9-002`, `KMIPKIT-REQ-SPEC-3.9-003`, `KMIPKIT-REQ-SPEC-3.9-004`, `KMIPKIT-REQ-SPEC-5.5-003`, `KMIPKIT-REQ-SPEC-5.5-004`, `KMIPKIT-REQ-SPEC-5.5-005`, `KMIPKIT-REQ-SPEC-5.6-002`, `KMIPKIT-REQ-SPEC-5.7-002`, `KMIPKIT-REQ-SPEC-6.1.12-003`, `KMIPKIT-REQ-SPEC-6.1.12-005`, `KMIPKIT-REQ-SPEC-6.1.12-006`, `KMIPKIT-REQ-SPEC-6.1.12-007`, `KMIPKIT-REQ-SPEC-6.1.13-005`, `KMIPKIT-REQ-SPEC-6.1.13-006`, `KMIPKIT-REQ-SPEC-6.1.14-007-001`, `KMIPKIT-REQ-SPEC-6.1.14-009`, `KMIPKIT-REQ-SPEC-6.1.14-010`, `KMIPKIT-REQ-SPEC-6.1.14-011`, `KMIPKIT-REQ-SPEC-6.1.14-012`, `KMIPKIT-REQ-SPEC-6.1.18-005`, `KMIPKIT-REQ-SPEC-6.1.18-006`, `KMIPKIT-REQ-SPEC-6.1.18-007`, `KMIPKIT-REQ-SPEC-6.1.18-008`, `KMIPKIT-REQ-SPEC-6.1.19-003`, `KMIPKIT-REQ-SPEC-6.1.19-004`, `KMIPKIT-REQ-SPEC-6.1.19-005`, `KMIPKIT-REQ-SPEC-6.1.19-006`, `KMIPKIT-REQ-SPEC-6.1.23-002-001`, `KMIPKIT-REQ-SPEC-6.1.30-001`, `KMIPKIT-REQ-SPEC-6.1.30-002`, `KMIPKIT-REQ-SPEC-6.1.30-003`, and `KMIPKIT-REQ-SPEC-6.1.31-001`.

The oracle preserves two proposed stable IDs without adding them to the catalog: `KMIPKIT-REQ-PROF-5.18.3-004` for a Profiles §5.18.3 MAY statement (outside the TTLV-only 1.0 scope), and `KMIPKIT-REQ-SPEC-6.1.54-001` for the Specification §6.1.54 REQUIRED Endpoint Role row. These proposals and remaining gap dispositions need the separate T003 review/test tranche; this audit does not assign ownership or invent requirement mappings.

### Scoped source-consistency checks

Focused source checks passed: all-profile direct roots and closure match the independently reviewed oracle; Baseline includes every Specification clause plus the Profiles §3.1 inherited clauses; §14.1-002 appears in each client closure; test IDs, hrefs, mandatory status, dependencies, and fixture records match the source fixture; informative references do not expand closure; and server-only requirements remain excluded. These checks establish equality with the current closure fixture, not completion of the requirement crosswalk.

## Independent row-level source audit — 2026-10-14

An independent pass compared the 55 `client_1_0` requirement records without `source_clause_ids` and the then-current 106 oracle source-gap records with the exact pinned Specification and Profiles HTML. A read-only locator check resolved every stored HTML line range to the specified source; nested ranges matched. Counts were recomputed from the catalog and oracle rather than inferred from this narrative. The later bounded Usage Limits Count mapping moved that row from the gap inventory to `source_table_contracts`, retaining the exact locator and excerpt without minting a clause or requirement ID; the pre-existing catalog label remains linked through the shared `source_gap_id`. No upstream source copy changed.

### Disposition of the 55 source-link gaps

All 55 requirement IDs are stable existing catalog labels and all are present in Baseline Client’s stored requirement list. All 55 lack a formal `source_clause_ids` link because their locators are table-body rows (39 `required_table_body_value`, 16 `optional_table_body_value`); a table-body `Yes`/`No` value has no formal RFC keyword and does not itself produce a clause ID in the current source-clause extraction. Fifty-three have a direct `requirement_crosswalk` row. The remaining two have exact source-row evidence through the separate source-gap review or source-table contract layer, as detailed below; they are not unmapped table rows. Do not link a neighboring `REQUIRED` table-header clause to a body row or mint a clause ID for a table row.

Among the 55 catalog requirement rows, 24 are operation-request table rows in §§6.1.12–6.1.31 and the source-gap oracle classifies them `client_1_0` (8 required, 16 optional). One additional `client_1_0` requirement row, `KMIPKIT-REQ-SPEC-2.3-003`, covers the Opaque Data Type and Opaque Data Value rows in §2.3. The other 30 are object/attribute-definition rows in §§3.1–3.12 and 5.5–5.7. Their exact enclosing structures and alternatives are enumerated after the table. The source-gap oracle marks these 30 rows `profile_conditional`; this tranche aligns the catalog `scope_state` and both stored profile closures with that reviewed disposition. Their stable IDs remain catalogued, but no profile includes them in its mandatory requirement set while `profile_ids` is empty. The pinned rows identify when a field is required within its structure, but do not establish which 1.0 features or client profiles must support every enclosing key format or attribute domain. Applicability by profile remains unresolved, and all 25 key-format/material rows remain without a feature owner.

| Requirement ID | Pinned table-body locator and field | Oracle source-gap scope | Force | Catalog status | Oracle row-level crosswalk or source-row evidence |
|---|---|---|---|---|---|
| `KMIPKIT-REQ-SPEC-2.3-003` | §2.3, lines 6927–6945: Opaque Data Type; lines 6946–6964: Opaque Data Value | `client_1_0` | `required_table_body_value` | `verified` | Direct crosswalk absent; two exact source-gap review records reuse this stable ID and link the Type and Value elements, respectively. |
| `KMIPKIT-REQ-SPEC-3.1-007` | §3.1, lines 7698–7716: Key Format Type | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.10-002` | §3.10, lines 9283–9301: P | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.10-003` | §3.10, lines 9321–9339: G | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.10-004` | §3.10, lines 9359–9377: Y | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.11-002` | §3.11, lines 9438–9457: Recommended Curve | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.11-003` | §3.11, lines 9458–9476: D | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.12-002` | §3.12, lines 9533–9552: Recommended Curve | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.12-003` | §3.12, lines 9553–9571: Q String | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.2-003` | §3.2, lines 7893–7914: Key Material | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.3-010` | §3.3, lines 8114–8132: Wrapping Method | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.3-011` | §3.3, lines 8284–8302: Unique Identifier | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.4-002` | §3.4, lines 8468–8486: Key | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.5-002` | §3.5, lines 8544–8562: P | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.5-003` | §3.5, lines 8563–8581: Q | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.5-004` | §3.5, lines 8582–8600: G | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.5-005` | §3.5, lines 8601–8619: X | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.6-002` | §3.6, lines 8677–8695: P | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.6-004` | §3.6, lines 8715–8733: G | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.6-005` | §3.6, lines 8734–8752: Y | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.7-003` | §3.7, lines 8810–8828: Modulus | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.8-002` | §3.8, lines 9035–9053: Modulus | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.8-003` | §3.8, lines 9054–9072: Public Exponent | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.9-002` | §3.9, lines 9130–9148: P | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.9-003` | §3.9, lines 9168–9186: G | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-3.9-004` | §3.9, lines 9206–9224: X | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-5.5-003` | §5.5, lines 22117–22137: Vendor Identification | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-5.5-004` | §5.5, lines 22138–22156: Attribute Name | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-5.5-005` | §5.5, lines 22184–22202: Attribute Reference | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-5.6-002` | §5.6, lines 22260–22279: Any attribute in §4 - Object Attributes | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-5.7-002` | §5.7, lines 22337–22356: Any attribute in §4 - Object Attributes | `profile_conditional` | `required_table_body_value` | `unassigned` | present |
| `KMIPKIT-REQ-SPEC-6.1.12-003` | §6.1.12, lines 25489–25507: Rights | `client_1_0` | `required_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.12-005` | §6.1.12, lines 25434–25451: Lease Time | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.12-006` | §6.1.12, lines 25452–25470: Request Count | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.12-007` | §6.1.12, lines 25471–25488: Usage Limits | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.13-005` | §6.1.13, lines 25681–25699: Current Attribute | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.13-006` | §6.1.13, lines 25700–25718: Attribute Reference | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.14-007-001` | §6.1.14, lines 25980–26001: Attributes | `client_1_0` | `required_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.14-009` | §6.1.14, lines 25900–25917: Object Type | `client_1_0` | `required_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.14-010` | §6.1.14, lines 25919–25938: Unique Identifier | `client_1_0` | `required_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.14-011` | §6.1.14, lines 25940–25958: Derivation Method | `client_1_0` | `required_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.14-012` | §6.1.14, lines 25960–25978: Derivation Parameters | `client_1_0` | `required_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.18-005` | §6.1.18, lines 26902–26917: Key Format Type | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.18-006` | §6.1.18, lines 26918–26933: Key Wrap Type | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.18-007` | §6.1.18, lines 26934–26950: Key Compression Type | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.18-008` | §6.1.18, lines 26951–26966: Key Wrapping Specification | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.19-003` | §6.1.19, lines 27211–27228: Key Format Type | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.19-004` | §6.1.19, lines 27229–27247: Key Wrap Type | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.19-005` | §6.1.19, lines 27248–27266: Key Compression Type | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.19-006` | §6.1.19, lines 27267–27285: Key Wrapping Specification | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.23-002-001` | §6.1.23, lines 28080–28097: Usage Limits Count | `client_1_0` | `required_table_body_value` | `scoped_verified` | Direct crosswalk absent; Table 232 source-table contract links the row to approved KMIPKIT-0039 FR-001, implementation, and tests. |
| `KMIPKIT-REQ-SPEC-6.1.30-001` | §6.1.30, lines 29673–29688: Lease Time | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.30-002` | §6.1.30, lines 29689–29704: Request Count | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.30-003` | §6.1.30, lines 29705–29720: Usage Limits | `client_1_0` | `optional_table_body_value` | `verified` | present |
| `KMIPKIT-REQ-SPEC-6.1.31-001` | §6.1.31, lines 29866–29881: Ticket | `client_1_0` | `required_table_body_value` | `verified` | present |

#### Enclosing conditions for the 30 definition rows

The 25 key-format/material rows are required within their named representation: `KMIPKIT-REQ-SPEC-3.1-007` is the Key Format Type in a Key Block; `3.2-003` is Key Material in a Key Value; `3.3-010` is Wrapping Method in Key Wrapping Data; `3.3-011` is Unique Identifier in Encryption Key Information; `3.4-002` is the Key field in Transparent Symmetric Key Material; `3.5-002` through `3.5-005` are fields of Transparent DSA Private Key Material; `3.6-002`, `3.6-004`, and `3.6-005` are fields of Transparent DSA Public Key Material; `3.7-003` is in Transparent RSA Private Key Material; `3.8-002` and `3.8-003` are in Transparent RSA Public Key Material; `3.9-002` through `3.9-004` are in Transparent DH Private Key Material; `3.10-002` through `3.10-004` are in Transparent DH Public Key Material; `3.11-002` and `3.11-003` are in Transparent EC Private Key Material; and `3.12-002` and `3.12-003` are in Transparent EC Public Key Material. In each case the field requirement applies when that enclosing representation is used.

The five attribute rows are form-specific. In §5.5 Table 161, `KMIPKIT-REQ-SPEC-5.5-003` (Vendor Identification) and `5.5-004` (Attribute Name) belong to the Attribute Reference name form; `5.5-005` (Attribute Reference) belongs to its tag form. `KMIPKIT-REQ-SPEC-5.6-002` applies when the Current Attribute structure carries an attribute from §4, and `5.7-002` applies when the New Attribute structure carries one. These source conditions do not decide the owning feature or the set of client profiles required to support each form.

#### Independent profile-applicability and owner audit (2026-10-14)

A separate read-only comparison checked all 30 pinned table-body rows against all 18 client profile definitions. For the 25 key-format/material rows, `Required=Yes` applies inside the selected enclosing representation; the source does not require every client profile to support every representation. Specification §4.1.1 permits choices of Key Value/Key Material and Key Format Type. Baseline Client §5.1.1 permits support for unenumerated KMIP-SPEC clauses but does not require every such clause or representation; other profiles inherit Baseline without turning that permission into a mandate. Explicit Transparent-key references found in server profiles do not assign client requirements. No approved feature contract was found that assigns the 25 general key-format/material row IDs, so their owner and per-profile applicability remain open.

For the five §5.5–§5.7 attribute rows, KMIP Specification §5.5 Table 161 defines two alternative forms: Vendor Identification plus Attribute Name, or an Attribute Reference Enumeration. The §5.6/§5.7 rows apply only when their respective Current/New Attribute wrappers are present. Baseline requires Get Attributes but makes its Attribute Reference selector optional; Tape Library permits the relevant attribute operations optionally. Neither profile, nor another reviewed client profile, requires both forms or mandates Current/New wrappers. The approved KMIPKIT-0016 contract covers both Attribute Reference forms and the Current/New wrappers, so these five existing stable requirement IDs now map to KMIPKIT-0016, their typed encoders/decoder, and local protocol tests. Their catalog rows have `scope_state: profile_conditional` and `profile_ids: []`; they are excluded from each profile's mandatory requirement set. No profile obligation or official OASIS Test Case pass is inferred.

The catalog and source-gap oracle now both record these five rows as `profile_conditional`; the catalog's 18 profile requirement arrays and the independent oracle arrays no longer include them. The 25 key-format/material rows have the same conditional profile scope but remain unassigned to a feature. T002 remains open for their ownership and applicability, the broader profile-conditional source closure, and the remaining actor/force/source-mapping audit.

Two direct `requirement_crosswalk` entries remain absent, but both source rows have exact evidence in the approved table-row model. For `KMIPKIT-REQ-SPEC-2.3-003`, the two Table 4 source-gap records cover Opaque Data Type (lines 6927–6945) and Opaque Data Value (lines 6946–6964), reuse the same stable requirement ID, and link their respective typed elements; the catalog’s singular `source_gap_id` points to the Type row. For `KMIPKIT-REQ-SPEC-6.1.23-002-001`, the Table 232 source-table contract links Usage Limits Count (lines 28080–28097) to the approved `KMIPKIT-0039` FR-001 contract, implementation, and tests. The current direct crosswalk schema models requirement-to-clause links and does not represent multiple body-row links for one requirement; the separate source-gap and source-table records preserve those rows without inventing clause IDs. The absence of direct crosswalk entries is not itself a row-mapping blocker under the approved 010 model. The other 53 requirement IDs have a direct row-level oracle crosswalk record; none of these table-row records creates a formal source-clause ID. The oracle’s `requirement_crosswalk_complete: false` remains accurate for the broader unresolved inventory.

**Table 232 profile-closure test gap resolved:** the source-table contract marks the Usage Limits Count request row as client-applicable in §6.1.23, and the independently reviewed source-clause closure for each of the 18 client profiles includes that section. The existing catalog label `KMIPKIT-REQ-SPEC-6.1.23-002-001` links to the contract through the shared `source_gap_id`. CI run `38608003089` reported this label as the sole mismatch for all 18 profiles because the expected-set test calculation omitted `source_table_contracts`; the profile records already contained the label. The calculation now resolves labels through the reviewed source-gap link and includes client table contracts when their exact source section is in the profile closure. A focused run passes for all 18 profiles. This corrects the test oracle and does not mint a clause ID or alter profile membership. T002 remains open for the 30 unresolved structure applicability/ownership rows and other source, reference, and test-ID adjudications below.

### Review of all 107 remaining source-gap records

The following compact inventory gives each remaining oracle gap’s exact section and pinned HTML line range, source-derived scope disposition, review status, and current reused/proposed/candidate ID. Full normalized source excerpts, force, actor, direction, condition, locator kind, and review note remain in the oracle record keyed by the same gap ID. Of 107 records, 18 have a `reused_requirement_id`, 2 have a `proposed_requirement_id`, and 62 have a `candidate_requirement_id`; 18 of the candidate IDs overlap a reused ID, while 43 records have none of these fields. Candidate IDs in the inventory are not approved stable owners. `—` means no reused, proposed, or candidate ID is recorded. This audit leaves unresolved actors and candidates as recorded.

| Source-gap ID | Pinned source locator | Scope disposition | Review status | Reused ID / proposal / candidate |
|---|---|---|---|---|
| `KMIPKIT-SOURCE-GAP-PROF-3.1.1-4009` | profiles, §3.1.1 4009–4014 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.1-4023` | profiles, §3.1.1 4023–4028 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.2-4060` | profiles, §3.1.2 4060–4065 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.2-4076` | profiles, §3.1.2 4076–4087 | `mixed_server_and_out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.2-4089` | profiles, §3.1.2 4089–4185 | `mixed_server_and_out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.3-4196` | profiles, §3.1.3 4196–4198 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.3-4200` | profiles, §3.1.3 4200–4202 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.3-4204` | profiles, §3.1.3 4204–4206 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.1.4-4217` | profiles, §3.1.4 4217–4218 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-3.2.4-4260` | profiles, §3.2.4 4260–4262 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-5.12.3-8658` | profiles, §5.12.3 8658–8663 | `server_only` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-5.13.1-8990` | profiles, §5.13.1 8990–8991 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-PROF-5.13.1-002` |
| `KMIPKIT-SOURCE-GAP-PROF-5.18.3-10411` | profiles, §5.18.3 10411–10412 | `out_of_scope` | `stable_requirement_label_missing` | `KMIPKIT-REQ-PROF-5.18.3-004` |
| `KMIPKIT-SOURCE-GAP-PROF-5.3.1-5139` | profiles, §5.3.1 5139–5140 | `out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-5.3.1-5141` | profiles, §5.3.1 5141–5142 | `out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-5.3.1-5149` | profiles, §5.3.1 5149–5150 | `out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-5.4.2-5980` | profiles, §5.4.2 5980–5986 | `out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-5.5.2-6613` | profiles, §5.5.2 6613–6619 | `out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-6.13-10848` | profiles, §6.13 10848–10849 | `profile_conditional` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-6.14-10867` | profiles, §6.14 10867–10868 | `profile_conditional` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-6.6-10708` | profiles, §6.6 10708–10714 | `out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-PROF-6.8-10746` | profiles, §6.8 10746–10753 | `out_of_scope` | `unlabeled_source_statement_or_boundary_item` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-11.46-005` | spec, §11.46 56537–56549 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED` | spec, §2.3 6927–6945 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-2.3-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-VALUE-REQUIRED` | spec, §2.3 6946–6964 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-2.3-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.1-KEY-FORMAT-TYPE` | spec, §3.1 7698–7716 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.1-007` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.10-P` | spec, §3.10 9283–9301 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.10-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.10-G` | spec, §3.10 9321–9339 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.10-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.10-Y` | spec, §3.10 9359–9377 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.10-004` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.11-RECOMMENDED-CURVE` | spec, §3.11 9438–9457 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.11-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.11-D` | spec, §3.11 9458–9476 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.11-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.12-RECOMMENDED-CURVE` | spec, §3.12 9533–9552 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.12-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.12-Q-STRING` | spec, §3.12 9553–9571 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.12-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.2-KEY-MATERIAL` | spec, §3.2 7893–7914 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.2-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.3-WRAPPING-METHOD` | spec, §3.3 8114–8132 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.3-010` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.3-UNIQUE-IDENTIFIER` | spec, §3.3 8284–8302 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.3-011` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.4-KEY` | spec, §3.4 8468–8486 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.4-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.5-P` | spec, §3.5 8544–8562 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.5-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.5-Q` | spec, §3.5 8563–8581 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.5-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.5-G` | spec, §3.5 8582–8600 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.5-004` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.5-X` | spec, §3.5 8601–8619 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.5-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.6-P` | spec, §3.6 8677–8695 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.6-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.6-G` | spec, §3.6 8715–8733 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.6-004` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.6-Y` | spec, §3.6 8734–8752 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.6-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.7-MODULUS` | spec, §3.7 8810–8828 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.7-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.8-MODULUS` | spec, §3.8 9035–9053 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.8-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.8-PUBLIC-EXPONENT` | spec, §3.8 9054–9072 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.8-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.9-P` | spec, §3.9 9130–9148 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.9-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.9-G` | spec, §3.9 9168–9186 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.9-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-3.9-X` | spec, §3.9 9206–9224 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-3.9-004` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.21-CLIENT-LOWERCASE-MAY` | spec, §4.21 13896–13897 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-4.26-001` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.24-CLIENT-FRESH-INPUT-EXCEPTION` | spec, §4.24 14444–14449 | `unresolved` | `actor-capability-classification-unresolved` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.28-001-SERVER-NONMUTATION` | spec, §4.28 15308–15313 | `server_only` | `server_actor_disposition_recorded` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.30-CLIENT-LEASE-READ-ONLY` | spec, §4.30 15638–15646 | `client_1_0` | `mapped_to_attribute_policy` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.31-MAY-ABSENT-LINKS` | spec, §4.31 15815–15819 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-4.31-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.31-LINK-TYPE-REQUIRED` | spec, §4.31 15923–15940 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-4.31-004` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.31-LINKED-OBJECT-IDENTIFIER-REQUIRED` | spec, §4.31 15942–15961 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-4.31-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-4.39-LOWERCASE-FRIENDLY-NAME` | spec, §4.39 17251–17256 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-4.39-002-001` |
| `KMIPKIT-SOURCE-GAP-SPEC-5.5-VENDOR-IDENTIFICATION` | spec, §5.5 22117–22137 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-5.5-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-NAME` | spec, §5.5 22138–22156 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-5.5-004` |
| `KMIPKIT-SOURCE-GAP-SPEC-5.5-ATTRIBUTE-REFERENCE-TAG` | spec, §5.5 22184–22202 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-5.5-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-5.6-CURRENT-ATTRIBUTE` | spec, §5.6 22260–22279 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-5.6-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-5.7-NEW-ATTRIBUTE` | spec, §5.7 22337–22356 | `profile_conditional` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-5.7-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-MULTIPART-OPTIONAL-PREFACE` | spec, §6.1 22417–22420 | `unresolved` | `force_and_actor_unresolved` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-011` | spec, §6.1 22429–22431 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-006` | spec, §6.1 22433–22435 | `unresolved` | `reviewed_generic_parameter_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-012` | spec, §6.1 22437–22439 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-013` | spec, §6.1 22441–22443 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-014` | spec, §6.1 22449–22451 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-007` | spec, §6.1 22453–22455 | `unresolved` | `reviewed_generic_parameter_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-015` | spec, §6.1 22457–22459 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-016` | spec, §6.1 22465–22467 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-017` | spec, §6.1 22469–22471 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-018` | spec, §6.1 22473–22475 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-019` | spec, §6.1 22485–22488 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-008` | spec, §6.1 22490–22492 | `unresolved` | `reviewed_generic_parameter_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-020` | spec, §6.1 22494–22496 | `unresolved` | `reviewed_procedure_context` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009` | spec, §6.1 22498–22499 | `unresolved` | `unresolved_source_interpretation` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.12-LEASE-TIME-OPTIONAL` | spec, §6.1.12 25434–25451 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.12-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.12-REQUEST-COUNT-OPTIONAL` | spec, §6.1.12 25452–25470 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.12-006` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.12-USAGE-LIMITS-OPTIONAL` | spec, §6.1.12 25471–25488 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.12-007` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.12-RIGHTS-REQUIRED` | spec, §6.1.12 25489–25507 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.12-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.13-CURRENT-ATTRIBUTE-OPTIONAL` | spec, §6.1.13 25681–25699 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.13-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.13-ATTRIBUTE-REFERENCE-OPTIONAL` | spec, §6.1.13 25700–25718 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.13-006` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.14-OBJECT-TYPE-REQUIRED` | spec, §6.1.14 25900–25917 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.14-009` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.14-UNIQUE-IDENTIFIER-REQUIRED` | spec, §6.1.14 25919–25938 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.14-010` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.14-DERIVATION-METHOD-REQUIRED` | spec, §6.1.14 25940–25958 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.14-011` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.14-DERIVATION-PARAMETERS-REQUIRED` | spec, §6.1.14 25960–25978 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.14-012` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.14-ATTRIBUTES-REQUIRED` | spec, §6.1.14 25980–26001 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.14-007-001` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.18-GET-SEMANTICS` | spec, §6.1.18 26846–26849 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.18-001-001` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-FORMAT-TYPE-OPTIONAL` | spec, §6.1.18 26902–26917 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.18-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-WRAP-TYPE-OPTIONAL` | spec, §6.1.18 26918–26933 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.18-006` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-COMPRESSION-TYPE-OPTIONAL` | spec, §6.1.18 26934–26950 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.18-007` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.18-KEY-WRAPPING-SPECIFICATION-OPTIONAL` | spec, §6.1.18 26951–26966 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.18-008` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-FORMAT-TYPE-OPTIONAL` | spec, §6.1.19 27211–27228 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.19-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-WRAP-TYPE-OPTIONAL` | spec, §6.1.19 27229–27247 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.19-004` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-COMPRESSION-TYPE-OPTIONAL` | spec, §6.1.19 27248–27266 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.19-005` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.19-KEY-WRAPPING-SPECIFICATION-OPTIONAL` | spec, §6.1.19 27267–27285 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.19-006` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.30-LEASE-TIME-OPTIONAL` | spec, §6.1.30 29673–29688 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.30-001` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.30-REQUEST-COUNT-OPTIONAL` | spec, §6.1.30 29689–29704 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.30-002` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.30-USAGE-LIMITS-OPTIONAL` | spec, §6.1.30 29705–29720 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.30-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.31-TICKET-REQUIRED` | spec, §6.1.31 29866–29881 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.31-001` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.42-RECOVER-AUTHENTICATION-SHOULD-ACTOR-UNRESOLVED` | spec, §6.1.42 32613–32618 | `unresolved` | `actor_and_scope_unresolved` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.45-CERTIFICATE-REQUEST-UID-ONLY` | spec, §6.1.45 33348–33353 | `client_1_0` | `source_obligation_covered_by_existing_stable_label` | `KMIPKIT-REQ-SPEC-6.1.45-002-003` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.54-ENDPOINT-ROLE-REQUIRED` | spec, §6.1.54 36165–36187 | `client_1_1` | `stable_label_proposed_not_in_catalog` | `KMIPKIT-REQ-SPEC-6.1.54-001` |
| `KMIPKIT-SOURCE-GAP-SPEC-8.4-002` | spec, §8.4 43045–43062 | `server_only` | `scope_unresolved` | `—` |
| `KMIPKIT-SOURCE-GAP-SPEC-9.17-001` | spec, §9.17 45156–45158 | `unresolved` | `scope_unresolved` | `—` |

### Approved feature-scope cross-check for conditional key-format rows (2026-10-15)

An independent read-only review compared the 25 §3.1–§3.12 table-body requirements with the approved feature specifications and traceability records. It found no direct feature-owned stable-ID assignment for these rows in KMIPKIT-0016, -0017, -0018, -0022, or -0023. The roadmap entries for managed objects and client operations do not select key representations and therefore do not establish profile applicability.

The generic TTLV model and wire codec preserve data but do not define or validate the §3 key schemas: KMIPKIT-0004 FR-006 excludes operation-specific ordering, cardinality, and semantic validation, and KMIPKIT-0005 FR-003 excludes schema-level validity. KMIPKIT-0017 preserves an opaque Any Object and caller-supplied Key Format Type without parsing key material. KMIPKIT-0023 covers the Export Key Format Type selector and opaque Any Object handling. These are transport or operation-level contracts, not support for every key representation.

There is one narrow overlap: KMIPKIT-0022 Register US8 explicitly preserves Key Material in a plaintext Raw Key Value. This overlaps the §3.2 Key Material row (`KMIPKIT-REQ-SPEC-3.2-003`) for that Register path only. Its approved FR-008 and traceability map operation payload rows, not all nested §3 structures or all client profiles. Do not generalize that scenario to the remaining 24 rows or to every format/profile.

**Ruling:** retain all 25 rows as `profile_conditional`, with `applies_to=[]` and no client-profile requirement membership until an approved feature/profile contract selects the corresponding representation. This records conditional applicability when a structure is used; it does not permit violating a required member within that structure. The ruling preserves the existing source-row evidence and does not create source-clause IDs or broaden feature ownership. The current regression in `tools/normative_catalog/tests/test_source_gap_adjudication.py::test_profile_conditional_structural_rows_are_not_client_profile_requirements` remains valid for this reviewed state; a later feature contract selecting a specific representation must add a narrower mapping and update the regression accordingly.

### §4.21 lowercase Key Format request atom (2026-10-15)

The pinned KMIP Specification v2.1 paragraph is located at lines 13894–13901; the source-gap entry uses the exact narrow span 13896–13897 for the lowercase client request atom. The helper-produced excerpt for that span is exact and ends at `and the`; the separate server `SHALL` begins on line 13898 and remains source obligation 6. The §4.21 source atom points to its source-gap record for traceability only; its `candidate_requirement_id` is null and the review expects no new §4.21 client requirement. The formal client capability comes from §4.26 and is limited to Create, Create Key Pair, Create Split Key, Re-key, Re-key Key Pair, or Derive Key. Register is not included. The existing requirement source clause remains §4.26 only. The only linked local executable test covers Create request serialization; it does not verify the other five operations or a server outcome, and no official OASIS Test Case ID is linked.

### Remaining T002 blockers and exact dispositions

- **30 conditional definition rows:** the exact enclosing structures and alternative attribute forms are listed above. The source-gap oracle marks every row `profile_conditional`, while the previous catalog marked them `client_1_0` and inserted them into every client profile closure. This tranche aligns the catalog requirement scope and both stored closure sets with the reviewed source disposition; a regression now rejects universal membership. The 25 key-format/material rows still lack an owning feature and profile-specific applicability. The five §5.5–§5.7 attribute rows are owned by approved KMIPKIT-0016 for their typed models and operations, but their profile applicability remains conditional. Pinned table rows require their member only when the enclosing representation is used; they do not establish that every profile must support that representation. These 25 rows no longer block T002 profile mapping. Broader implementation ownership belongs in the feature-level roadmap/spec assignment before every representation is implemented; it does not change the reviewed absence of profile membership.
- **Procedure and generic-parameter context:** KMIP Specification §6.1 procedure records `-011` through `-018` describe the multi-part sequence; the pinned preface at lines 22417–22423 makes multipart support optional for cryptographic operations whose requests define Correlation Value, Init Indicator, and Final Indicator. Records `-019` and `-020` describe the separate single-part sequence at lines 22477–22496. The current gap records identify the client actor, the combined client request/response-processing direction, and the applicable cryptographic sequence condition; their force remains an imperative procedure without a formal modal, their scope disposition remains unresolved, and they receive no standalone requirement IDs. Records `-012`, `-014`, and `-016` link the existing §7.8 requirement only as context: Five `KMIPKIT-0021` request-table contracts cover Correlation Value fields, but do not establish response-to-follow-up correlation round-trip behavior. KMIPKIT-0019 Encrypt/Decrypt request Correlation Value rows still lack `source_table_contracts`. Records `-013`, `-015`, `-018`, and `-020` remain unresolved because response Data consumption remains unmapped without approved response-field contracts and executable tests. These links do not turn the optional procedure into a profile-wide obligation. Generic-parameter records `KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-006` through `-008` identify the client actor, client cryptographic-operation request-parameter direction, and applicable multi-part or single-part request condition. They refer to other required operation-specific parameters rather than introducing independent fields or requirement IDs; their scope disposition remains unresolved.

Paragraph `-009` at lines 22498–22499 has the same client request-parameter actor, direction, and condition, with unresolved scope and no candidate requirement ID. Its `source_force` is `structural_requirement_statement_lowercase_required`; the pinned sentence uses lowercase `required`, not a formal `REQUIRED` keyword. The independent source-gap oracle records `source_keywords: []`; the catalog's case-insensitive lexical `REQUIRED` hit is only a candidate token and does not establish RFC keyword force. The single-part procedure permits both Init Indicator and Final Indicator to be true without Correlation Value, after which paragraph 009 permits Data omission. `KMIPKIT-DISC-045` retains the conflicting source references for Encrypt, Decrypt, Hash, MAC, and Sign (Tables 196, 214, 235, 259, and 334) without selecting either interpretation or creating requirement IDs. MAC Verify and Signature Verify remain outside the affected set because their payloads use different data fields. Whether the general statement also conflicts with digest-only single-part Sign without either indicator remains a separate open source question. T002 remains open for the broader source-gap crosswalk; these procedures and unresolved conflicts are not treated as completed profile requirements.
- **Other unresolved actor/scope/force records:** the source-gap inventory preserves the exact rows for the §4.24 client Fresh=False input exception, §6.1.42 authentication SHOULD, §8.4-002 response MAY, §9.17-001 response/end-user MAY, and server/attribute-policy boundary rows in §§4.28 and 4.30. The §4.31 formal MAY now maps to `KMIPKIT-REQ-SPEC-4.31-002` for a client-visible absent-Link object state only when associated key material is unavailable to the server or client. The source does not name a message direction; this does not override operation-specific Link requirements. The separate server SHOULD remains server-only. Table 91 Link Type and Linked Object Identifier now map to `KMIPKIT-REQ-SPEC-4.31-004` and `KMIPKIT-REQ-SPEC-4.31-005`; both remain unassigned, with no local typed Link validation or verification evidence. The official Link case TC-PGP-1-21 is mapped but its fixture is unavailable. §4.31-003 remains a non-applicable table-header record. The §4.21 client atom traces to the existing §4.26 capability while retaining its lowercase force and Create-only evidence limit as documented above. Other ambiguous actors remain unresolved.
- **Profiles §§6.13 and §6.14:** the pinned text is literal and unambiguous about its references. Section 6.13 lines 10845–10853 says clients SHALL support [`KMIP-SPEC`], Basic Authentication Suite §3.1, Basic Symmetric Key Foundry Client conditions §5.7.1, one or more Intermediate mandatory test cases §5.7.6, and Baseline Client §6.1. Section 6.14 lines 10864–10872 gives the same first three references, one or more Advanced mandatory test cases §5.7.7, and Baseline Client §6.1. The literal §5.7.1 inclusions are already encoded in the catalog and independent oracle as `KMIPKIT-INCLUSION-PROF-613-BASIC-SYMMETRIC-CLIENT` and `KMIPKIT-INCLUSION-PROF-614-BASIC-SYMMETRIC-CLIENT`; focused generator tests cover both edges and their inherited requirement IDs. Do not substitute §§5.7.2 or 5.7.3 for the source's explicit reference. Keep `KMIPKIT-DISC-011` and `KMIPKIT-DISC-012` open as source-defect observations about authorial intent; the literal source mapping itself is not ambiguous. This correction closes the specific missing-edge note, while broader T002/T003 crosswalk and closure review remains open.
- **Official test IDs:** the 93 distinct profile test IDs are present in the pinned Profiles source, which is the source that defines profile cases and links their fixture hrefs. None occurs literally in pinned Test Cases CN01. `SOURCES.md` lines 24–30 inventories CN01 `TC-*` operation examples separately from Profiles fixtures at lines 34–49; CN01 §2 lines 1526–1529 says its request/response examples use the XML format specified in Profiles. Record the 0/93 CN01 literal match result and verify profile IDs/hrefs against Profiles; do not fabricate CN01 cross-references. Fixture availability and execution evidence remain separate gates.
- **Closure basis:** Baseline’s current clause closure is 897 clauses: all 877 Specification clauses plus 20 inherited/direct Profiles clauses. The 877 count is the full Specification source inventory; it includes server-only and out-of-scope clauses due to Profiles §6.1 item 1’s whole-Spec inclusion. Those clauses remain excluded from the client requirement closure where the source actors/applicability require it.

T002 remains unchecked. The 30 `profile_conditional` scope classifications have been reconciled and are not missing client-profile requirement mappings. The latest audit reconciles the TLS 1.2 mixed client/server boundary, the conditional scope of the §§6.13/6.14 clauses, and the missing §6.1 item 3 inclusion; broader actor, scope, normative-force, and direct/inherited requirement review remains open across all 18 profiles. The §4.21 client atom traces only to the existing §4.26 capability; its lowercase `may` remains non-normative and its linked test covers Create request serialization only. The §4.31 client MAY is positively represented in all 18 client closures as `KMIPKIT-REQ-SPEC-4.31-002`, conditioned on unavailable associated key material; message direction remains unspecified, and implementation/test evidence remains unverified. The separate server SHOULD remains server-only. Table 91 Link Type and Linked Object Identifier now appear in all 18 client closures; they apply to complete Link object-state values and preserve Locate's partial-Link criteria. Both requirements remain unassigned; the mapped TC-PGP-1-21 fixture is unavailable, so no official execution evidence exists. The §6.1 paragraph-009 source-keyword/disposition classification and broader procedure mapping remain open; no requirement IDs or interpretation are inferred from that source context. The Table 4 and Table 232 body rows have source-row evidence under the existing model and are not counted as missing row mappings. The literal §§6.13/6.14 references, client actors, profile-conditional scopes, and §5.7.1 inclusion edges are recorded for T002; T003 must retain their independent regression coverage while `KMIPKIT-DISC-011`/`012` remain open for authorial intent. Profile case IDs are verified against their defining Profiles source; their absence from Test Cases CN01 is an expected source partition, not a missing link. T003 remains separately tracked in `tasks.md`; this T002 disposition does not claim that T003 is complete.

## Current pinned official case source audit — 2026-10-14

The offline utility `python -B tools/normative_catalog/audit_case_sources.py --repo-root .` reads the local paths and SHA-256 values from `specification/catalog/kmip-2.1.json`, confines reads to the repository, and uses only the Python standard library. The Profiles source is `specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html` (`KMIPKIT-SRC-profiles`, SHA-256 `f11144aa793335f38dbd1864ec4709ca83ac80f588bbb790ba37f6c53be207a9`); the Test Cases CN01 source is `specification/oasis/kmip-2.1/upstream/kmip-testcases-v2.1-cn01.html` (`KMIPKIT-SRC-testcases`, SHA-256 `305e0638af25aa8163a2b4a67a1a617a0207af67b3b3032fa499966b22d42b88`).

The audit matched all 93/93 `profile_test_case` rows to a heading and exact `raw_href` in the declared Profiles `source_section`, and all 110/110 `official_conformance_test` rows to the corresponding CN01 section. The evidence category is required to bind to its designated source ID. Result: zero catalog-to-source mismatches. This checks case evidence locations only; it does not complete the broader source-clause, applicability, requirement-mapping, or executable-conformance review.

Ten source-internal official-ID versus href-basename divergences are retained and reported as source defects, not catalog mismatches. They are not normalized or interpreted as intended corrections:

| Source | Section | Official ID | Exact href basename |
|---|---:|---|---|
| Profiles v2.1 | 5.9.7.13 | `CS-BC-M-13-21` | `CS-BC-M-14-21.xml` |
| Profiles v2.1 | 5.17.1 | `QS-M-1-12` | `QS-M-1-21.xml` |
| Test Cases CN01 | 2.38 | `TC-ECDSA-SIGN-DIGESTEDDATA` | `TC-ECDSA-SIGN-DIGESTEDDATA-1-21.xml` |
| Test Cases CN01 | 2.48 | `TC-LOGIN-1-21` | `TC-LOGIN-1-21xml` |
| Test Cases CN01 | 2.60 | `TC-MD-24-21` | `TC-MD241-21.xml` |
| Test Cases CN01 | 2.90 | `TC-RSA-SIGN-DIGESTEDDATA` | `TC-RSA-SIGN-DIGESTEDDATA-1-21xml` |
| Test Cases CN01 | 2.92 | `TC-SETTATTR-1-21` | `TC-SETATTR-1-21.xml` |
| Test Cases CN01 | 2.93 | `TC-SETTATTR-2-21` | `TC-SETATTR-2-21.xml` |
| Test Cases CN01 | 2.94 | `TC-SETTATTR-3-21` | `TC-SETATTR-3-21.xml` |
| Test Cases CN01 | 2.97 | `TC-SJ-3-21` | `TC-SJ-2-21.xml` |

Existing source-defect records `KMIPKIT-DISC-027`, `KMIPKIT-DISC-028`, and `KMIPKIT-DISC-034` remain open and unchanged. The audit does not infer authorial intent, create a conformance claim, or alter any pinned OASIS file.

The current no-`source_clause_ids` row count was independently recalculated from the catalog with `python -B -c "import json,pathlib,collections; catalog=json.loads(pathlib.Path('specification/catalog/kmip-2.1.json').read_text(encoding='utf-8')); rows=[r for r in catalog['requirements'] if not r.get('source_clause_ids')]; print('requirements_without_source_clause_ids=',len(rows)); print('scope_state=',dict(sorted(collections.Counter(r.get('scope_state') for r in rows).items()))); print('status=',dict(sorted(collections.Counter(r.get('status') for r in rows).items())))"`. Historical 2026-10-14 result: 55 total; 25 `client_1_0`, 30 `profile_conditional`; status 24 `verified`, 6 `scoped_verified`, 25 `unassigned`. The 2026-10-15 Table 91 update below recalculates the current counts. T002 remains open for the broader closure/applicability/crosswalk review.

## Table 91 Link member mapping update — 2026-10-15

The pinned KMIP Specification v2.1 §4.31 Table 91 has two body rows with Required=Yes: Link Type (Enumeration), HTML lines 15923–15940, and Linked Object Identifier (Text String/Enumeration/Integer), lines 15942–15961. They now have separate stable client requirement IDs, `KMIPKIT-REQ-SPEC-4.31-004` and `KMIPKIT-REQ-SPEC-4.31-005`, with exact row locators and links to the existing structure-member element records. The IDs appear in all 18 client profile closures and the independently maintained closure oracle.

These requirements apply to complete Link values represented as object state. KMIP Specification v2.1 §6.1.28 explicitly permits partial structured Locate criteria, including Linked Object Identifier without Link Type; that search form remains valid and is excluded from the Table 91 condition. The rows do not create a server-side rejection duty. The existing official Test Case `TC-PGP-1-21` (`KMIPKIT-TEST-CN01-2-66`, §2.66) describes adding links between PGP key objects; its fixture is unavailable, so no official execution evidence is claimed.

Both new requirements remain `unassigned`, with no typed Link implementation or local verification references. The §4.31 absent-Link MAY (`KMIPKIT-REQ-SPEC-4.31-002`) remains separate, and `KMIPKIT-REQ-SPEC-4.31-003` remains the non-applicable table-header record. Current catalog totals are 505 requirement-crosswalk records, 890 source-review records, 107 source-only gap records, and 66 operation-table contracts. The current no-`source_clause_ids` count is 57 requirements: 27 `client_1_0` and 30 `profile_conditional`; 24 are `verified`, six `scoped_verified`, and 27 `unassigned`. `requirement_crosswalk_complete` remains false; T002 and T003 remain open. No runtime behavior or pinned OASIS source changed.

## T002 pinned-source re-audit correction — 2026-10-15

The follow-up audit found that matching the old oracle did not prove complete closure. Profiles v2.1 §6.1 item 3 (pinned HTML line 10486) says conformant clients SHALL support the Baseline Client conditions in §5.1.1. The catalog now records `KMIPKIT-INCLUSION-PROF-BASELINE-CONDITIONS` from `KMIPKIT-CLAUSE-PROF-6.1-003` to Profiles §5.1.1. The independently reviewed oracle records the same edge, and every client profile closure includes `KMIPKIT-CLAUSE-PROF-5.1.1-001`; the Baseline profile already included it as a direct root. That catalog clause is anchored at §5.1.1 item 1; its `REQUIRED` candidate is the lowercase word “required” in descriptive text, and it has no standalone requirement ID.

An independent condition inventory now records all eight top-level §5.1.1 conditions and their 30 nested values across pinned Profiles lines 4529–4636, including the introductory sentence identifying them as Baseline Client conformance conditions. The first six are required profile conditions; the final two are explicitly optional capabilities. Each has a stable `KMIPKIT-PROF-COND-5.1.1-*` source-condition ID, the governing §6.1 item 3 requirement link, and `pending_t002_crosswalk` status. No separate SHALL/REQ IDs are inferred from the title-case “Supports” or “Optionally supports” text. This source inventory does not yet map every named condition to its applicable KMIP-SPEC requirements, typed implementation rule, and test evidence, so a matching section edge and clause-set closure alone still do not establish complete semantic closure.

The actor/scope reconciliation is also recorded in both the source catalog and oracle. Profiles §3.1.2 lines 4076–4087 assigns the TLS 1.2 cipher-suite SHALL to clients or servers when TLS 1.2 is supported. The client branch is outside KMIPKit 1.0's TLS 1.3-only boundary; the server branch is outside this client library, so the oracle scope is `mixed_server_and_out_of_scope` and no client requirement ID is added. Profiles §§6.13/6.14 item 3 explicitly assigns a SHALL to conformant clients; its scope is `profile_conditional` on claiming the respective Foundry profile. Their literal §5.7.1 inclusions remain unchanged, no duplicate requirement IDs are assigned, and `KMIPKIT-DISC-011`/`012` remain open only for authorial intent.

The cross-platform Script Contracts run found that the two §6.13/§6.14 source-gap records lacked an explicit resolution even though their inclusion edges were already present. The independent oracle now marks each as `normative_inclusion`, tying the source-gap record to its exact source clause, profile, and existing `KMIPKIT-INCLUSION-PROF-613-BASIC-SYMMETRIC-CLIENT` or `KMIPKIT-INCLUSION-PROF-614-BASIC-SYMMETRIC-CLIENT` edge to §5.7.1. The review explicitly assigns no standalone requirement ID; the literal §5.7.1 references and open authorial-intent discrepancies remain unchanged. The source-membership regression verifies the edge resolves to the matching client profile and source section.

T002 remains unchecked. The independent source-case audit still matches 93/93 Profiles cases and 110/110 CN01 cases with zero catalog/source mismatches, but the requirement crosswalk remains incomplete and other actor, scope, normative-force, and inherited-mapping gaps remain. This correction changes source metadata and closure evidence only; it does not claim profile conformance or complete T003.

## Baseline §5.1.1 item 4 — Query crosswalk (2026-10-15)

Profiles v2.1 §5.1.1 item 4, pinned HTML lines 4563–4574, lists Get, Get Attributes, Locate, and Query as Baseline Client operations. Query is now independently crosswalked to client operation element `KMIPKIT-ELEM-OP-C2S-QUERY` and exactly two client request requirements from KMIP Specification v2.1 §6.1.40: `KMIPKIT-REQ-SPEC-6.1.40-014` (`SHALL`, clause `KMIPKIT-CLAUSE-SPEC-6.1.40-014`, paragraph locator 14, pinned HTML lines 31992–31993) and `KMIPKIT-REQ-SPEC-6.1.40-016` (`MAY`, clause `KMIPKIT-CLAUSE-SPEC-6.1.40-016`, table-row locator 16, pinned HTML lines 32025–32042). The independent oracle regression verifies these catalog relationships and source locators and rejects non-client or unrelated requirements.

This maps only the Query item. Requirement `KMIPKIT-REQ-SPEC-6.1.40-016` is a `MAY`/optional capability to repeat Query Function; listing it in the item crosswalk preserves that capability's traceability and category, but does not make repetition mandatory to satisfy Baseline item 4. Get, Get Attributes, and Locate remain unmapped, so the parent `KMIPKIT-PROF-COND-5.1.1-004` stays without a standalone requirement ID and retains `pending_t002_crosswalk`. The local regression is repository evidence, not an official OASIS test case; this partial crosswalk does not establish profile readiness or support any conformance claim. T002 remains open.


## Baseline §5.1.1 item 4 — Get, Get Attributes, and Locate crosswalk (2026-10-15)

Profiles v2.1 §5.1.1 item 4, pinned HTML lines 4563–4574, identifies the client-to-server operations Get, Get Attributes, Locate, and Query. The independent source inventory now maps the three previously unmapped items alongside the existing reviewed Query mapping:

| Profiles item | Operation element | Exact client requirements |
|---|---|---|
| Get | `KMIPKIT-ELEM-OP-C2S-GET` | `KMIPKIT-REQ-SPEC-6.1.19-002` (`SHALL`, Specification §6.1.19, paragraph 2, lines 27135–27136) |
| Get Attributes | `KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES` | `KMIPKIT-REQ-SPEC-6.1.20-001-001` (`SHALL NOT`, §6.1.20 paragraph 1, lines 27440–27447); `KMIPKIT-REQ-SPEC-6.1.20-004` (`MAY`, §6.1.20 table row 4, lines 27506–27515) |
| Locate | `KMIPKIT-ELEM-OP-C2S-LOCATE` | `KMIPKIT-REQ-SPEC-6.1.28-001` (`MAY`, paragraph 1, lines 29143–29148); `-002` (`MAY`, paragraph 2, lines 29150–29155); `-004-001` (conditional `REQUIRED`, paragraph 4, lines 29168–29170); `-007` (`REQUIRED` source keyword, paragraph 7, lines 29206–29210); `-013-001` (`REQUIRED`, table row 13, lines 29341–29362); `-013-002` (`MAY`, the same row and lines) |
| Query | `KMIPKIT-ELEM-OP-C2S-QUERY` | Existing reviewed mapping to `KMIPKIT-REQ-SPEC-6.1.40-014` (`SHALL`) and `KMIPKIT-REQ-SPEC-6.1.40-016` (`MAY`), preserved unchanged |

Each mapping links the item label, client-to-server operation element, and only its reviewed client-side requirement IDs. The Get Attributes response predicates are server-only; Locate response, matching, ID Placeholder, and policy obligations remain excluded. The regression checks each requirement's role, request direction, scope, source keyword and strength, source clause locator, source review excerpt and expected client obligation IDs against the pinned Specification HTML. It separately verifies that the reviewed server-only IDs do not enter these client mappings.

### Locate §6.1.28-007 strength adjudication

The exact source says that when a structured attribute is specified, all structure fields are “not REQUIRED to be specified”; the Link example permits Linked Object Identifier without Link Type (Specification v2.1 §6.1.28 paragraph 7, pinned lines 29206–29210). The word `not` negates a client obligation. It therefore describes client permission to supply a partial criterion, classified as `permission_or_optional`, while the literal `source_keyword: REQUIRED` and stable requirement ID remain unchanged. The example's statement about matching candidate objects irrespective of Link Type describes server-side matching behavior and creates no client obligation to omit fields. No new requirement ID is introduced.

The local Locate request regression checks that a partial structured Link criterion is accepted and preserved; it does not establish server matching behavior or an official OASIS test pass. The condition remains `requirement_id: null` and `pending_t002_crosswalk` in this bounded slice. These item crosswalks are traceability evidence only; T002 remains open and no profile readiness or conformance claim is made.

### Baseline Client §5.1.1(2) Attributes crosswalk (2026-10-15)

The read-only audit used the pinned Profiles v2.1 source `specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html` (SHA-256 `f11144aa793335f38dbd1864ec4709ca83ac80f588bbb790ba37f6c53be207a9`) and KMIP Specification v2.1 source `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` (SHA-256 `8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf`). Profiles §5.1.1 lines 4535–4540 identifies the second condition's nested `Attributes` item. It maps to `KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES`. The condition itself remains `requirement_id: null` and `mapping_status: pending_t002_crosswalk`; the item crosswalk does not close its parent.

Two optional client capabilities apply at different scopes. Specification §5.1 Table 157, pinned lines 21800–21820, says any §4 Object Attribute MAY be repeated and its REQUIRED column is `No`. `KMIPKIT-REQ-SPEC-5.1-002` therefore represents the optional ability to encode and decode repeated Attributes, not an obligation to repeat a value. Its source review states that client encode and decode direction; the catalog direction is `both`, and the source clause remains the general §5.1 table row. Separately, Specification §3.2 lines 7837–7845 and Table 13's Attributes row at lines 7919–7930 describe OPTIONAL attribute information inside Key Value associated with key material. `KMIPKIT-REQ-SPEC-3.2-001` remains limited to the Register request path and `client_to_server`; it does not generalize to ordinary Managed Object Attributes. Both optional requirement IDs map to the Attributes element while retaining these distinct source scopes and their existing Register/attribute implementation evidence.

The §4 cardinality rows `KMIPKIT-REQ-SPEC-4-001-001`, `-002`, and `-003` remain outside this mapping. Their actor and applicability to the Baseline Client condition are unresolved, so this crosswalk assigns them no client duties.

BL-M-4-21 is evidence that the pinned profile fixture contains repeated Attributes: fixture lines 105–109 show the Locate request's `RequestPayload.Attributes` with two direct `ProtectStopDate` children. Profiles §5.1.3.4 lines 5018–5021 only names the case and links its XML fixture; it does not name the Attributes element. Accordingly, the catalog keeps BL-M-4-21's `element_ids` and the Attributes element's `test_case_ids` unlinked, consistent with the rule that catalog case-element links require an explicit source naming the protocol element. The regression parses the pinned XML to verify the two entries as fixture evidence. It does not create a requirement-specific official-case link or claim that the runner executes/asserts the fixture.

### §5.1-002 fixture link correction (2026-10-15)

The first Green draft added symmetric catalog links between BL-M-4-21 and the Attributes element because the XML fixture contains the repeated structure. The full normative test suite exposed that this exceeded the source basis: `test_official_cases_link_only_html_explicitly_named_protocol_elements` requires catalog case-element links to match protocol elements explicitly named in the source HTML, and Profiles §5.1.3.4 only says “See” the XML href. The fixture remains useful evidence, but does not justify that catalog relationship. The correction removed both links and keeps the source distinction in the crosswalk regression. No requirement mapping, direction, source interpretation, or other existing profile-item mapping changed.

## Baseline §5.1.1 item 3 — Object Attribute element crosswalk (2026-10-15)

Profiles v2.1 §5.1.1 item 3, pinned HTML lines 4541–4562, names eight Object Attributes. The independent profile-closure fixture now maps each exact name to its existing attribute element:

| Profiles item | KMIP Specification element | Element source sections | Encoding | Tag |
|---|---|---|---|---|
| Activation Date | `KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE` | §§4.1, 11.56 | Date-Time | `0x420001` |
| Deactivation Date | `KMIPKIT-ELEM-ATTRIBUTE-DEACTIVATION-DATE` | §§4.18, 11.56 | Date-Time | `0x42002F` |
| Digest | `KMIPKIT-ELEM-ATTRIBUTE-DIGEST` | §§4.21, 11.56 | Structure | `0x420034` |
| Initial Date | `KMIPKIT-ELEM-ATTRIBUTE-INITIAL-DATE` | §§4.25, 11.56 | Date-Time | `0x420039` |
| Last Change Date | `KMIPKIT-ELEM-ATTRIBUTE-LAST-CHANGE-DATE` | §§4.29, 11.56 | Date-Time | `0x420048` |
| Object Type | `KMIPKIT-ELEM-ATTRIBUTE-OBJECT-TYPE` | §§4.36, 11.56 | Enumeration | `0x420057` |
| State | `KMIPKIT-ELEM-ATTRIBUTE-STATE` | §§4.57, 11.56 | Enumeration | `0x42008D` |
| Unique Identifier | `KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER` | §§4.58, 11.56 | Text String, Enumeration, or Integer | `0x420094` |

Every mapping has `requirement_ids: []`, `source_clause_ids: []`, and `mapping_status: reviewed`. Here, `reviewed` records only the exact profile-item-to-element relation. The profile list does not adjudicate client ownership of the attributes' §4 rules. Current source reviews leave actor or applicability unresolved for candidate requirements concerning Activation Date, Deactivation Date, Initial Date, State, and Unique Identifier. Digest's digest-setting rule is server-only, and its lowercase “Clients may request” sentence is not a formal modal requirement. These candidates remain excluded; no source review or catalog requirement classification changes in this tranche.

The regression binds the mapping list to the exact pinned Profiles span and checks each element's name, §4 and §11.56 source references, direction, scope, encoding, and parent tag record. Exact ordered-list equality rejects missing, extra, duplicate, reordered, or fabricated mappings and any requirement or clause IDs. The parent condition `KMIPKIT-PROF-COND-5.1.1-003` remains `requirement_id: null` / `pending_t002_crosswalk`. No official case link, profile readiness, or conformance claim is implied.

### Baseline Client §5.1.1 item 5 — message structure crosswalk (2026-10-15)

Profiles v2.1 §5.1.1 item 5, pinned HTML lines 4575–4613, lists 14 Message Data Structures. The reviewed mappings now cover `Batch Count` (line 4582), `Batch Error Continuation Option` (lines 4584–4585), and the nested `Batch Item` (lines 4586–4587), in that order. The first two map to `KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT` and `KMIPKIT-ELEM-MESSAGE-FIELD-9-6-BATCH-ERROR-CONTINUATION-OPTION`; the third maps structurally to `KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM`, with `KMIPKIT-CLAUSE-SPEC-9.7-001` as its source link and no client requirement IDs.

Specification v2.1 §9.5, clause `KMIPKIT-CLAUSE-SPEC-9.5-001`, is the paragraph at pinned lines 43728–43731. Its source review assigns two client-to-server, `client_1_0` obligations to the Batch Count element:

| Requirement | Source keyword | Client rule |
|---|---|---|
| `KMIPKIT-REQ-SPEC-9.5-001-001` | `REQUIRED` | Set Batch Count to the number of Batch Items in the message. |
| `KMIPKIT-REQ-SPEC-9.5-001-002` | `SHALL` | Set Batch Count to one for a single requested operation. |

Specification v2.1 §9.6, clause `KMIPKIT-CLAUSE-SPEC-9.6-001`, is the paragraph at pinned lines 43773–43775. The source review assigns three client-to-server, `client_1_0` requirements to the Batch Error Continuation Option element:

| Requirement | Source keyword | Client rule |
|---|---|---|
| `KMIPKIT-REQ-SPEC-9.6-001-001` | `SHALL` | Include the option only when Batch Count is greater than one. |
| `KMIPKIT-REQ-SPEC-9.6-001-002` | `SHALL` | Use Undo, Stop, or Continue as the option value. |
| `KMIPKIT-REQ-SPEC-9.6-001-003` | `SHALL` | If omitted, the option defaults to Stop. |

The third source anchor includes the preceding SHALL value constraint and the following “Stop is assumed” statement. The default phrase has no independent uppercase modal. The separate `KMIPKIT-REQ-SPEC-6.1.7-002-002` SHOULD is specific to a batch containing Check and is not linked to this general item-5 field mapping.

The §9.7 paragraph at pinned Specification lines 43819–43821 describes a structure holding individual requests or responses and says it is `REQUIRED`. The source does not name an actor or direction; the independent review therefore retains actor/direction `unspecified`, client-scope disposition `unresolved`, and an empty `expected_client_requirement_ids` list. Although the catalog has a candidate `KMIPKIT-REQ-SPEC-9.7-001`, its catalog role is `unspecified`, while its direction and scope are `unclear`; it has no linked element. The profile crosswalk identifies only the structure, so its mapping has an empty `requirement_ids` list and does not attribute that candidate obligation to the client.

The exact element represents the client message path: Specification §8.3/Table 396 defines Request Batch Item and §8.6/Table 399 defines Response Batch Item. The catalog element has `direction: both`, `scope_state: client_1_0`, and links to the request and response Batch Item parents. This mapping is limited to constructing/serializing requests and parsing/correlating responses for operations the client supports. Specification §6.1 says clients need not issue every operation, while a client that issues a request SHALL understand its response; §14.1 binds a client to the clauses of a profile it claims. This row does not claim support for every KMIP operation or for server-initiated requests.

Specification §9.7 says that the contents are described in “Section 12,” but the pinned §12 heading is “Bit Masks” (lines 64883–64890). The request and response Batch Item structures are instead directly specified in §§8.3 and 8.6. Record this apparent source cross-reference defect as written; do not modify the pinned OASIS source or silently treat the reference as corrected.

### Batch Order Option

Profiles v2.1 §5.1.1 item 5 lists `Batch Order Option` at pinned lines 4588–4590. Its structural mapping targets `KMIPKIT-ELEM-MESSAGE-FIELD-9-8-BATCH-ORDER-OPTION`, sourced to Specification §9.8, encoded as Boolean, directed client-to-server, and scoped to `client_1_0`; its Request Header parent is the §8.2 field. Specification §8.2 Table 395 (pinned lines 42762–42778) marks the field `Required=No` and states that omission assumes True. Table 407 in §9.8 (lines 43884–43901) confirms the Boolean representation. This mapping records support for the request field; it does not require sending the optional field in every multi-item request.

The §9.8 paragraph at pinned lines 43862–43866 starts with a descriptive sentence and then assigns a SHALL and MAY to server execution behavior. The candidate `KMIPKIT-REQ-SPEC-9.8-001-001`, which had claimed a client SHALL to use the field only when Batch Count exceeds one, remains explicitly `retired`, `not_applicable`, and `out_of_scope`; the source review rejects it as not a normative client requirement. The sibling `KMIPKIT-REQ-SPEC-9.8-001-002` SHALL and `-003` MAY remain server-only. Accordingly, the profile item mapping links the §9.8 clause but has `requirement_ids: []`, and the independent source review retains `expected_client_requirement_ids: []`. The catalog's historical element relationship to the retired candidate does not revive it as a profile or client obligation.

The separate §6.1.7-002-001 SHOULD to set this option True when a batch contains Check remains conditional on Check; Check is not in the Baseline required-operation list in Profiles §5.1.1 item 4. It is not attached to this general structural mapping. Specification §6.1 paragraph 3's client MAY about omitting Unique Identifier and using ID Placeholder when Batch Order Option is True or unspecified describes a separate conditional capability and does not require the client to set this option.

The four reviewed item-5 mappings remain in source order: Batch Count, Batch Error Continuation Option, Batch Item, and Batch Order Option. The other 10 values stay unmapped for later source-reviewed slices. Parent condition `KMIPKIT-PROF-COND-5.1.1-005` remains `requirement_id: null` / `pending_t002_crosswalk`; this partial crosswalk does not close it. These repository regressions are not official OASIS test-case executions and establish no profile readiness or conformance claim. T002 remains open.

### Maximum Response Size

Profiles v2.1 §5.1.1 item 5 lists `Maximum Response Size` as a supported Message Data Structure (pinned lines 4592–4593); Profiles §6.1 item 3 binds a conformant client to Baseline Client conditions (pinned lines 10481–10486). The reviewed item mapping targets `KMIPKIT-ELEM-MESSAGE-FIELD-9-12-MAXIMUM-RESPONSE-SIZE` and `KMIPKIT-CLAUSE-SPEC-9.12-001`. The catalog element is a client-to-server Integer field in `client_1_0`, parented to the §8.2 Request Header field.

Specification v2.1 §8.2's Request Header row marks the field `Required=No` and has an empty comment cell (pinned lines 42609–42625). Section 9.12 (lines 44811–44814) says it is OPTIONAL in a request, the requester SHALL be able to handle the response size it declares, and it SHOULD only be sent for requests that possibly return large replies. The source review assigns all three existing IDs—`KMIPKIT-REQ-SPEC-9.12-001-001` (OPTIONAL), `-002` (SHALL), and `-003` (SHOULD)—to the requester/client, client-to-server direction, and `client_1_0` scope. This permits the capability without requiring the field on every request.

Preserve the catalog's current ownership: the element and its header parent link only `KMIPKIT-REQ-SPEC-9.12-001-001`; the SHALL and SHOULD records keep empty `element_ids`. The independent profile crosswalk includes all three reviewed IDs without changing those catalog links. Section 9.12 Table 417 identifies the field as Integer (pinned lines 44832–44850), and §11.56 assigns tag `0x420050` (lines 59962–59973). The reviewed pinned text establishes no numeric default, numeric range, or server behavior when a declared limit is exceeded. Do not infer any of those, or a server-only/Locate-specific obligation, from this mapping.

The five reviewed item-5 mappings remain source-ordered through `Maximum Response Size`; the other nine values stay unmapped. Parent condition `KMIPKIT-PROF-COND-5.1.1-005` remains `requirement_id: null` / `pending_t002_crosswalk`. This partial crosswalk and its repository regression are not an official OASIS test-case execution and establish no profile readiness or conformance claim. T002 remains open.
