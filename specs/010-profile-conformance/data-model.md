# Data Model: KMIP 2.1 Client Profile Conformance

## ClientProfile

A `ClientProfile` is the reviewed catalog record for a KMIP 2.1 profile.

| Field | Meaning | Constraint |
|---|---|---|
| `profile_id` | Stable catalog identity | Must resolve to one profile record. |
| `name`, `role` | OASIS profile label and actor role | Only role `client` may be selected by the 1.0 client. |
| `applicability` | 1.0 scope disposition | `client_1_0`, `conditional`, `server_only`, and `out_of_scope` remain distinct. |
| `source_clause_ids` | Exact profile clause records | Each ID resolves to a pinned OASIS source and section. |
| `requirement_ids` | Profile-specific normative records | Every ID resolves, has an exact source keyword/section, and links to the profile. |
| `dependency_profile_ids` | Profiles required for conformance | Resolve recursively; missing references and cycles fail closed. |
| `element_ids` | Operations and protocol elements associated with the profile | These express required capabilities and are not an allowlist unless an exact source clause says otherwise. |
| `transport_requirements`, `encoding_requirements` | Transport and encoding conditions | Must be evaluated against the accepted product boundary. |
| `test_case_ids` | Official conformance evidence IDs | Each ID preserves mandatory/optional status and fixture availability. |
| `claim_state` | Accepted catalog evidence/readiness lifecycle | `not_claimed`, `candidate`, `selected`, `evidence_incomplete`, or `evidence_complete`; it is not a published release claim and does not determine target membership or runtime selectability. |

For this feature, `not_claimed` means no readiness disposition has been recorded; `candidate` means a profile is under consideration but is not selected; `selected` means it is a target before evidence evaluation; `evidence_incomplete` means evaluation found one or more blockers; and `evidence_complete` means all applicable evidence gates pass. None of these catalog values publishes a claim. The target manifest is authoritative for target membership, and a published claim exists only through the separate release process.

## ProfileRuleMapping

A `ProfileRuleMapping` is an explicit Rust rule associated with one or more stable normative requirement IDs. It has a typed rule kind, exact source references, and verification test references. Every applicable requirement must have at least one mapping and passing executable test before its profile can be selected for runtime validation. A mapping may cover multiple requirements only when its tests independently exercise each linked requirement. Unmapped requirements and tests without a mapped requirement remain visible gaps.

Rule coverage is computed from the typed Rust mapping registry and test evidence; it is not a catalog input or generated profile field. Catalog-generated profile records remain source and applicability metadata.

## ProfileTarget

The 1.0 product target is the selected set of client profiles used for conformance planning, recorded in the project-owned `specification/catalog/profile-targets.json`.

The generator/report path reads only this fixed repository-relative file through the repository's safe-read helper, with a 64 KiB maximum. It rejects symlinks/reparse points, invalid UTF-8 or JSON, duplicate JSON keys, unknown fields, duplicate profile IDs, unresolved IDs, and profiles outside the approved client scope before generating output. The manifest schema is strict; no path is supplied by callers and no network source is consulted. Runtime profile selection uses only explicit caller-selected profile IDs and their requirement/dependency mappings; target membership is not a runtime allowlist or prerequisite.

The root JSON value MUST be an object with exactly these required fields: `release_line` (string), `profile_ids` (array of strings), and `decision_ref` (non-empty string). Unknown or missing fields and values of any other JSON type are rejected. Each `profile_ids` entry is a non-empty stable profile ID and entries are unique.

| Field | Meaning | Constraint |
|---|---|---|
| `release_line` | Product line the selection applies to | Must be `1.0`. |
| `profile_ids` | Selected profile IDs | Non-empty, unique, resolve to catalog records with client role, and stay within the approved product scope. |
| `decision_ref` | Stable project requirement, decision, or ADR establishing the selection | Must resolve to a checked-in decision/specification, for example `KMIPKIT-0010/FR-003`. |

The initial selected target is Baseline Client. Target selection is independent of profile applicability, runtime opt-in validation, catalog `claim_state`, evidence completion, and public claim state. Selecting a target does not mean that any profile is implemented, evidence-complete, or publicly claimed.

## RuntimeProfileSelection

An application may explicitly select one or more client profiles for validation. Selection records profile IDs and validated dependency closure. Unsupported, conditionally unsatisfied, server-only, out-of-scope, cyclic, conflicting, incompletely mapped, or incompletely tested selections return an error before network I/O. An empty selection does not cause hidden discovery calls.

## ProfileCondition

A `ProfileCondition` is a finite typed rule linked to one or more stable catalog requirement IDs and exact source clauses. It evaluates a declared product capability, transport/encoding choice, or protocol behavior. Catalog descriptions are not predicates. Unknown, untyped, unresolved, or untested conditions are not treated as satisfied.

## ProfileValidationResult

A validation result contains:

- selected profile IDs and dependency closure;
- required capability checks;
- message, transport, and response constraint outcomes where applicable;
- profile-authorized defaults applied, without replacing explicit caller choices;
- stable requirement IDs and source sections for failures;
- no raw KMIP body, credentials, private key, or secret value.

Required operation lists are minimum capability sets. Extra base-protocol-conformant operations remain allowed unless a mapped normative rule explicitly prohibits them.

## ProfileEvidence

A `ProfileEvidence` record links one obligation to its implementation and verification evidence.

| Field | Meaning |
|---|---|
| `requirement_id` / `clause_id` | Stable source obligation |
| `implementation_ref` | Code, generated metadata, or documented external behavior |
| `verification_ref` | Test ID, result artifact, or explicit unresolved marker |
| `status` | Unassigned, planned, implemented, verified, blocked, or deviated by an accepted decision |
| `deviation_id` | Required when a SHOULD/SHOULD NOT/RECOMMENDED deviation is accepted |

Every applicable MUST/SHALL is release-blocking. MUST NOT/SHALL NOT require negative tests. In-scope MAY/OPTIONAL capabilities remain representable and usable. No record becomes verified without passing evidence.

## OfficialTestCaseEvidence

An official test record preserves its catalog ID, OASIS case ID, profile links, requirement links, source section, mandatory/optional status, fixture path, and fixture availability. The test is evidence rather than an independent normative requirement. An unavailable or absent fixture cannot be passed or reconstructed.

## ProfileReadiness

`ProfileReadiness` is a deterministic view of a profile and all required dependencies. It contains separate `target_selected` and `runtime_validation_available` booleans, one effective evidence/readiness lifecycle value, a blocker list, and `published_claim=false` for this feature. The catalog validator checks catalog-internal evidence, links, and fixture availability. The readiness evaluator additionally recomputes status from current requirement mappings, implementation/verification assignments, test results, deviations, and dependencies before comparing it with the catalog `claim_state`.

If a catalog `claim_state` says `evidence_complete` but any required evidence is missing, failed, unavailable, or contradictory, the effective state is `evidence_incomplete` and the report names both the blocker and stale catalog state. If all evidence passes but the catalog state is below `evidence_complete`, the effective state stays at the catalog value and the report requires a reviewed catalog update; the generator never silently advances it. The evaluator does not create a second readiness enum or mutate the catalog. An independent release process controls public claim publication; `evidence_complete` alone does not publish or certify a profile.

## Relationships

- One client profile links to many clauses, requirements, protocol elements, dependencies, and test cases.
- One requirement can apply to multiple profiles and retains one stable requirement ID.
- One test case can support multiple requirements/profiles while remaining evidence only.
- One runtime selection expands to a dependency closure and a deterministic set of conditions.
- One profile readiness result aggregates its direct evidence and dependency results.
