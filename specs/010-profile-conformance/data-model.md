# Data Model: KMIP 2.1 Client Profile Conformance

## ClientProfile

A `ClientProfile` is the reviewed catalog record for a KMIP 2.1 profile.

| Field | Meaning | Constraint |
|---|---|---|
| `profile_id` | Stable catalog identity | Must resolve to one profile record. |
| `name`, `role` | OASIS profile label and actor role | Only role `client` may be selected by the 1.0 client. |
| `applicability` | 1.0 scope disposition | `client_1_0`, `conditional`, `server_only`, and `out_of_scope` remain distinct. |
| `direct_source_clause_ids` | Clauses directly specified by the profile | Each ID resolves to a pinned OASIS source and section; these are the closure roots. |
| `source_clause_ids` | Computed normative clause closure for the profile | Includes direct clauses, clauses normatively incorporated by them recursively, and every same-source subclause; informative references do not expand the closure. The stored list MUST equal the validated closure. |
| `requirement_ids` | Complete profile-applicable normative records | Every applicable normative statement in the clause closure has an ID that resolves, retains its exact source keyword/section, and links to the profile. |
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

Runtime selection is per exchange through `Client::execute_with_profiles` and a `ProfileValidationSelection`. The existing `Client::execute` path remains unprofiled and unchanged. The selection contains explicit profile IDs only; the project target manifest is never consulted. This feature adds no production constructor or implicit client-wide selection state.

## NormativeInclusion

A `NormativeInclusion` is a checked-in catalog edge that records a source-backed normative incorporation without parsing source prose during builds. It may target a complete pinned source document or a section within one.

| Field | Meaning |
|---|---|
| `inclusion_id` | Stable identity for the source-backed edge |
| `source_clause_id` | Clause containing the normative incorporation statement |
| `included_source_id` | Pinned source document incorporated by reference |
| `included_section` | Exact included section; `null` means every clause in `included_source_id` |

`source_clause_id` and `included_source_id` resolve to catalog records. The edge is normative by definition; informative references are not recorded in this relation. Closure expansion starts with each profile's `direct_source_clause_ids`, includes every clause whose section is that exact section or a dot-delimited subclause in the same source, follows inclusion edges from every reached clause, and repeats until no new clauses are reached. A whole-document target includes every catalogued clause from that source, not only clauses with a currently applicable requirement. The computed requirement closure then includes every linked requirement applicable to the profile's role and approved product scope while preserving server-only and out-of-scope dispositions. Unknown IDs, cycles, or invalid source/section resolution fail catalog validation. The profile's stored `source_clause_ids` and `requirement_ids` must match this expansion and all applicable linked requirement records.

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
| `verification_ref` | Stable test/verification ID; an ephemeral readiness result may associate it with a current manifest check ID, but catalog assignments store no run-specific ID or arbitrary artifact path |
| `status` | Unassigned, planned, implemented, verified, blocked, or deviated by an accepted decision |
| `deviation_id` | Required when a SHOULD/SHOULD NOT/RECOMMENDED deviation is accepted |

Every applicable MUST/SHALL is release-blocking. MUST NOT/SHALL NOT require negative tests. In-scope MAY/OPTIONAL capabilities remain representable and usable. The `verified` status is a current-run readiness result only; catalog assignments retain stable test/check identifiers and are never rewritten by CI run results.

## ProfileValidationError

A profile validation error contains a stable error code, profile ID, requirement ID, exact source clause ID, and request delivery state. It contains no free-form source error, arbitrary diagnostic text, secret data, credentials, or raw KMIP body. Implementation references are repository-relative paths validated beneath the canonical checkout root; verification references are stable IDs, not caller-provided paths.

## CurrentVerificationManifest

The readiness evaluator accepts current test-result evidence only from a manifest created by the final aggregate job in `.github/workflows/ci.yml` during the same workflow run. The aggregate job derives results directly from its GitHub Actions `needs` context and current run context; it does not download or trust previously uploaded artifacts.

The strict versioned manifest contains:

| Field | Meaning |
|---|---|
| `schema_version` | Exact supported manifest schema version |
| `repository` | Canonical repository identity |
| `event_name` | GitHub Actions event that produced the manifest; only `push` to the protected release ref is eligible |
| `ref` | Fully qualified Git ref; must be `refs/heads/release/1.0.0` |
| `ref_protected` | GitHub context boolean; must be `true` for the run to be eligible |
| `branch_protection` | Normalized result of a fresh same-run GitHub branch-protection API query; must show the required review and push controls and an explicitly present empty pull-request bypass policy |
| `checked_out_commit` | Commit SHA tested by this workflow run |
| `workflow_ref` | Workflow path and ref for the aggregate job |
| `run_id`, `run_attempt` | Current GitHub Actions run and retry attempt |
| `checks` | Unique stable verification IDs, required job names, and job conclusions |

The evaluator requires event name `push`, ref `refs/heads/release/1.0.0`, `ref_protected: true`, the canonical repository and expected CI workflow, equality between the manifest SHA, checked-out `HEAD`, and `GITHUB_SHA`, equality between manifest run ID/attempt and current GitHub context, unique check IDs, and success for exactly the required jobs `core`, `script-contracts`, `normative-inventory`, `coverage`, `coverage-gate`, and `dependency-policy`. Every eligible run re-queries `GET /repos/{owner}/{repo}/branches/release%2F1.0.0/protection` using a short-lived GitHub App installation token with read-only Administration permission. The normalized response must show the branch is protected, at least one required approving review, stale-review dismissal, an explicitly present `bypass_pull_request_allowances` object with empty `users`, `teams`, and `apps` arrays, admin enforcement, force pushes disabled, and deletions disabled. A missing object, missing array, malformed value, or any bypass entry fails closed because GitHub can grant these identities an exception from required pull-request reviews. The API response is current-run evidence, not a cached setup check; errors, missing fields, insufficient policy, or unavailable token fail closed. A pull-request or `schedule` run, or any other event/ref/protection state, is unverified even if its aggregate job succeeds. Missing, malformed, stale, failed, duplicate, mismatched, locally asserted, or caller-supplied evidence is unverified and blocks `evidence_complete`. A local report may show catalog facts and blockers but cannot mark test results verified without current same-run protected-release-push CI evidence. The manifest contains no raw test output, secrets, credentials, private keys, or KMIP payloads.

The manifest and any readiness result derived from it are ephemeral outputs of that workflow run. They are not copied into the committed `specification/catalog/coverage-report.md` or used to rewrite catalog state. The committed coverage report is regenerated only from stable checked-in inputs and therefore remains byte-identical across workflow runs; a CI job may separately expose its current readiness evaluation as run-scoped output.

## OfficialTestCaseEvidence

An official test record preserves its catalog ID, OASIS case ID, profile links, requirement links, source section, mandatory/optional status, fixture path, and fixture availability. The test is evidence rather than an independent normative requirement. An unavailable or absent fixture cannot be passed or reconstructed.

## ProfileReadiness

`ProfileReadiness` is a deterministic view of a profile and all required dependencies for a fixed set of inputs. It contains separate `target_selected` and `runtime_validation_available` booleans, one effective evidence/readiness lifecycle value, a blocker list, and `published_claim=false` for this feature. The committed coverage report is a deterministic catalog view without run-specific evidence. The final CI aggregate job may separately compute an ephemeral readiness result using the current same-run manifest; it does not alter the committed report or catalog. The catalog validator checks catalog-internal evidence, complete normative clause closure, links, and fixture availability. The readiness evaluator additionally recomputes status from current requirement mappings, implementation assignments, same-run CI manifest results, deviations, and dependencies before comparing it with the catalog `claim_state`.

If a catalog `claim_state` says `evidence_complete` but any required evidence is missing, failed, unavailable, or contradictory, the effective state is `evidence_incomplete` and the report names both the blocker and stale catalog state. If all evidence passes but the catalog state is below `evidence_complete`, the effective state stays at the catalog value and the report requires a reviewed catalog update; the generator never silently advances it. The evaluator does not create a second readiness enum or mutate the catalog. An independent release process controls public claim publication; `evidence_complete` alone does not publish or certify a profile.

## Relationships

- One client profile links to many clauses, requirements, protocol elements, dependencies, and test cases.
- A profile's clause and requirement links equal the closure computed from direct source clauses, `NormativeInclusion` whole-source/section edges, and same-source subclauses; omission blocks readiness.
- One requirement can apply to multiple profiles and retains one stable requirement ID.
- One test case can support multiple requirements/profiles while remaining evidence only.
- One runtime selection expands to a dependency closure and a deterministic set of conditions.
- One profile readiness result aggregates its direct evidence and dependency results.
- One ephemeral CI verification manifest binds verification IDs and job conclusions to exactly one repository, commit, workflow, run, and attempt; it never becomes part of committed generated output.
