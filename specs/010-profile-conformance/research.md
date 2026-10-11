# Research: KMIP 2.1 Client Profile Conformance

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

**Rationale**: `specs/002-normative-inventory` records absent fixtures as evidence gaps and forbids reconstruction from titles. `docs/compliance/conformance.md` requires every mandatory test for a named profile to pass. The current pinned tree marks all 16 Baseline Client mandatory fixtures unavailable.

**Alternatives considered**: Treating an absent fixture as a pass, reconstructing it from a description, or downloading mutable content in CI was rejected by the inventory contract and document hierarchy.

## Decision: Separate readiness calculation from release claim publication

**Decision**: The feature reports target selection and runtime-validation availability as separate booleans and exposes one effective evidence/readiness lifecycle value with blockers. It recomputes current evidence before comparing the result with catalog `claim_state`. The Baseline profile's state becomes `evidence_incomplete` because its 16 mandatory fixtures are unavailable; this lifecycle value is not a published claim. This feature never emits a public support claim. A separate reviewed release process may publish only after all applicable normative requirements, mandatory tests, and project release gates pass.

**Rationale**: The accepted catalog data model and ADR-0009 separate implementation, profile conformance, interoperability, and certification. Human review and release governance remain authoritative.

**Alternatives considered**: Publishing a claim as soon as a local readiness function returns `evidence_complete` was rejected because release approval and evidence review are separate gates.

## Current evidence baseline

- The catalog contains 18 client profile records; 15 are marked `client_1_0` or `conditional`.
- On 2026-10-07, the GitHub branch-protection API returned `Branch not protected` for `release/1.0.0`, and the repository ruleset listing returned no rulesets. Therefore the protected-push evidence gate is currently unsatisfied and MUST remain incomplete until an administrator-owned policy is configured and the read-only API check passes during each eligible run.
- Baseline Client's current catalog record links 10 Profile requirement IDs and 16 mandatory test IDs. All 16 fixture records have `fixture_availability: unavailable`.
- HTTPS Client is marked `conditional`, links 19 requirement IDs and 17 mandatory test IDs, and depends on Baseline Client.
- The source catalog currently marks profiles `not_claimed`; this feature records Baseline readiness as `evidence_incomplete` because its 16 mandatory fixtures are unavailable. No public profile claim is emitted.
- The 1.0 target remains TTLV, KMIP 2.1, TLS 1.3 with mutual TLS, raw TLS and HTTP/1.1 over HTTPS. JSON/XML and server-initiated operations remain outside this feature's scope.

## Pinned-source client-profile closure baseline (2026-10-11)

This is a review snapshot of the catalog records against the pinned KMIP Specification v2.1 and Profiles v2.1 HTML, not a conformance result. The exact identifier sets remain in each profile's `direct_source_clause_ids`, `source_clause_ids`, `requirement_ids`, `dependency_profile_ids`, and `test_case_ids` arrays in `specification/catalog/kmip-2.1.json`; the counts below are cross-checks. The independent pinned-source oracle required by T003 is still absent, so the catalog-computed sets are not yet independently certified.

Seventeen client-profile records directly include `KMIPKIT-CLAUSE-PROF-3.1-001`; HTTPS receives §3.1 transitively through its Baseline dependency, because its own §6.4 references §3.2 and Baseline §6.1. All 18 records directly include the shared Profiles §6.1 roots `KMIPKIT-CLAUSE-PROF-6.1-001` through `-004`; each also includes its profile-specific direct clause roots summarized in the table. Pinned Profiles §6.1 items 1 and 2 require support for KMIP-SPEC and the Basic Authentication Suite conditions in §3.1. The catalog's two normative inclusion edges attach to roots `KMIPKIT-CLAUSE-PROF-6.1-001` and `-002`, mapping respectively to the whole `KMIPKIT-SRC-spec` source and `KMIPKIT-SRC-profiles` §3.1 with its subclauses; these are not Baseline-only edges. The pinned Profiles text is at `specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html` lines 10479–10485; §§3.1, 3.1.1, and 3.1.2 begin at lines 3997, 4007, and 4058. Baseline's stored closure contains all 877 catalogued Specification clauses plus 20 Profiles clauses (897 total), and 344 Specification `client_1_0` requirements plus 10 Profile requirements (354 total). Its ten Profile requirement IDs include `KMIPKIT-REQ-PROF-3.1.1-002`, `-004`, `-005`, `KMIPKIT-REQ-PROF-3.1.2-002`, and `-005`; the earlier note that these five were missing is stale and corrected here.

| Client profile | Applicability | Direct Profiles sections | Stored clauses | Stored requirements | Dependency | Linked test IDs | Fixture status |
|---|---|---|---:|---:|---|---:|---|
| AES XTS | `client_1_0` | 3.1, 5.13.1, 6.1, 6.30 | 904 | 356 | Baseline | 18 | all unavailable |
| Asymmetric Key Lifecycle | `client_1_0` | 3.1, 5.8.1, 6.1, 6.16 | 904 | 357 | Baseline | 20 | all unavailable |
| Baseline | `client_1_0` | 3.1, 5.1.1, 6.1 | 897 | 354 | none | 16 | all unavailable |
| Cryptographic Advanced | `client_1_0` | 3.1, 5.9.2, 6.1, 6.19 | 905 | 358 | Baseline | 34 | all unavailable |
| Cryptographic Basic | `client_1_0` | 3.1, 5.9.1, 6.1, 6.18 | 905 | 358 | Baseline | 37 | all unavailable |
| Cryptographic RNG | `client_1_0` | 3.1, 5.9.3, 6.1, 6.20 | 905 | 358 | Baseline | 21 | all unavailable |
| HTTPS | `conditional` | 3.2, 5.3.1, 6.1, 6.4 | 918 | 363 | Baseline | 17 | all unavailable |
| JSON | `out_of_scope` | 3.1, 5.5.2, 6.1, 6.8 | 905 | 349 | Baseline | 17 | all unavailable |
| Opaque Managed Object Store | `client_1_0` | 3.1, 5.10.1, 6.1, 6.24 | 904 | 357 | Baseline | 18 | all unavailable |
| PKCS#11 | `out_of_scope` | 3.1, 5.18.3, 6.1, 6.34 | 906 | 358 | Baseline | 17 | all unavailable |
| Quantum Safe | `client_1_0` | 3.1, 5.15, 5.17, 6.1, 6.32 | 910 | 359 | Baseline | 18 | all unavailable |
| Storage Array with Self-Encrypting Drives | `client_1_0` | 3.1, 5.11.1, 6.1, 6.26 | 905 | 358 | Baseline | 19 | all unavailable |
| Symmetric Key Foundry Advanced | `client_1_0` | 3.1, 5.7.3, 6.1, 6.14 | 904 | 356 | Baseline | 20 | all unavailable |
| Symmetric Key Foundry Basic | `client_1_0` | 3.1, 5.7.1, 6.1, 6.12 | 904 | 357 | Baseline | 20 | all unavailable |
| Symmetric Key Foundry Intermediate | `client_1_0` | 3.1, 5.7.2, 6.1, 6.13 | 904 | 356 | Baseline | 20 | all unavailable |
| Symmetric Key Lifecycle | `client_1_0` | 3.1, 5.6.1, 6.1, 6.10 | 904 | 357 | Baseline | 17 | all unavailable |
| Tape Library | `client_1_0` | 3.1, 5.12.2, 5.12.3, 5.12.4, 6.1, 6.28 | 922 | 374 | Baseline | 19 | all unavailable |
| XML | `out_of_scope` | 3.1, 5.4.2, 6.1, 6.6 | 905 | 349 | Baseline | 17 | all unavailable |

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

All non-Baseline client-profile records depend on Baseline. All 18 records together have 365 profile-to-test-ID associations and 93 distinct case IDs; every linked fixture record is marked unavailable. The audit found the linked IDs, labels, and hrefs in the pinned Profiles HTML, but presence does not establish correct label-to-href pairing. The pinned source has recorded mismatches: §5.9.7.13 labels `CS-BC-M-13-21` but points to `CS-BC-M-14-21.xml` and displays `CS-BC-M-21-21.xml`; §5.9.7.14 labels `CS-BC-M-14-21` but points to that same `CS-BC-M-14-21.xml` and displays `CS-BC-M-20-21.xml`; §5.17.1 labels `QS-M-1-12` but points to and displays `QS-M-1-21.xml`. These source defects are tracked as `KMIPKIT-DISC-027` and `KMIPKIT-DISC-028` in the catalog. None of the 93 IDs occurs in the separately pinned CN01 Test Cases HTML. The case IDs are present in Profiles, while the separate Test Cases document supplies no cross-reference for them; fixture execution remains unavailable and is not inferred. The catalog's normative closure follows only the two explicit inclusion edges described above; CN01 case links are evidence, and the Usage Guide has informative authority, so neither adds normative clauses or requirements. The source link details are in `specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html` lines 7910–7921 and 9386–9397.

The catalog-level mapping audit found no unresolved profile clause or requirement IDs and no stored requirement whose `source_clause_ids` falls outside its profile's stored `source_clause_ids` closure. This confirms internal link consistency, not independent source closure. The applicability counts remain 14 `client_1_0`, one `conditional` (HTTPS), and three `out_of_scope` (JSON, PKCS#11, XML); server-only records remain outside client closures.

The whole-Spec closure of every client profile includes clause `KMIPKIT-CLAUSE-SPEC-14.1-002`, whose requirement `KMIPKIT-REQ-SPEC-14.1-002` is marked `profile_conditional` and says a claimed client profile SHALL conform to its normative clauses and subclauses (pinned Specification §14.1, lines 66041–66047). No profile's current `requirement_ids` includes it because the requirement record has no `profile_ids`; it is not one of the 344 `client_1_0` Specification requirements. **Disposition:** for any of the 18 client-profile closures evaluated for a possible conformance claim, treat this requirement as the claim-level normative gate and include its ID in that profile's expected requirement closure. T003's independent oracle will list it for all 18 records, and T005 will map its `profile_ids` to all 18 so generated closure carries the condition into readiness evaluation. This meta-requirement is evaluated by checking every applicable profile-clause statement, not by adding a stand-alone operation rule or test fixture. For JSON, PKCS#11, and XML the mapping is future/conditional metadata and does not change their `out_of_scope` status or create a 1.0 evidence obligation. Its inclusion does not make the current client or profile conformant and does not publish a claim. The expected Baseline requirement closure after that correction is 345 Specification requirements (344 `client_1_0` plus this conditional claim requirement) and 10 Profile requirements, 355 total. The table above intentionally records the current stored count of 354. No profile claim is made.

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
| H1 — Baseline target versus runtime eligibility | `profile-targets.json` records Baseline as the 1.0 product target only. It does not grant runtime availability. Each in-scope requirement in the complete profile closure must have a typed mapping and a passing category-appropriate executable test before runtime selection; a gap rejects selection before I/O. The unavailable mandatory fixtures independently block evidence completion and claims. | Target/runtime separation tests, full-closure typed-rule completeness tests, and a readiness report showing all 16 unavailable Baseline fixtures as blockers. |
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
