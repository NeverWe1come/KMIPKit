# Feature Specification: KMIP 2.1 Client Profile Conformance

**Feature Branch**: `feature/KMIPKIT-0010-profile-conformance`
**Created**: 2026-10-07
**Status**: Draft
**Input**: Roadmap item `KMIPKIT-0010-profile-conformance`; approved KMIPKit product boundaries, ADR-0009, and the KMIP 2.1 normative catalog.

## Scope and normative sources

This feature defines how KMIPKit identifies applicable KMIP 2.1 client profiles, applies selected profile rules, records evidence, and determines whether a profile is eligible for a later release claim. Specification §14.1 requires a KMIP client to conform to one or more client profiles and requires every normative statement in a claimed profile's clauses and their subclauses to be met. The initial KMIPKit 1.0 target selected by this draft is the Baseline Client profile, giving the product a concrete candidate for §14.1. Selecting that target does not assert conformance or make a public claim.

The feature uses the checked-in catalog as the canonical crosswalk to the pinned OASIS sources. It covers every client profile that the catalog marks `client_1_0` or `conditional`; server-only and out-of-scope client profiles remain visible as exclusions. Profile requirements are derived from the Specification and Profiles standards, not from informative Usage Guide text or test-case examples. Test cases are evidence. The feature does not edit the immutable OASIS source copies or fetch OASIS material during builds.

| Stable requirement or source | Treatment |
|---|---|
| `KMIPKIT-REQ-SPEC-14.1-001` and `KMIPKIT-REQ-SPEC-14.1-002`; KMIP Specification v2.1 §14.1 | Enforce the client profile prerequisite and require all normative statements in a claimed profile's clauses and subclauses. |
| `KMIPKIT-PROFILE-BASELINE-CLIENT`; Profiles v2.1 §§3.1, 5.1.1, 5.1.3, 5.6.3, and 6.1; linked requirement IDs in the catalog | Record Baseline Client as the initial 1.0 target and evaluate its exact catalogued requirements, dependencies, and test cases. |
| `KMIPKIT-PROFILE-HTTPS-CLIENT`; Profiles v2.1 §§3.2, 5.3.1, 5.3.3, and 6.4; linked requirement IDs in the catalog | Preserve its conditional applicability and evaluate each exact condition before selection or any claim. HTTPS transport availability alone does not prove profile conformance. |
| Other client profiles in `specification/catalog/kmip-2.1.json` | Preserve profile-specific clauses, requirements, dependencies, transport/encoding conditions, test IDs, and applicability without inferring support from catalog presence. |
| `docs/compliance/conformance.md`, ADR-0009, and `docs/compliance/document-hierarchy.md` | Keep applicability, selected target, evidence state, and release claim separate; require traceable evidence and do not claim certification. |

At specification time the pinned catalog lists 18 client profiles, of which 15 are marked `client_1_0` or `conditional`. All 16 mandatory test cases linked to the Baseline Client currently have unavailable fixtures in the pinned Git tree. That is an explicit evidence gap: this feature must report it and keep the Baseline claim ineligible until the gap is resolved through an approved source process. It must not reconstruct missing official fixtures or silently mark them as passed.

## User Scenarios & Testing

### User Story 1 - Select and apply a client profile (Priority: P1)

A KMIPKit application explicitly selects a client profile for validation so that the client can verify required capabilities and any profile-specific message or transport constraints before relying on it.

**Why this priority**: The client must conform to at least one profile, and profile-specific rules can constrain supported operations, authentication, transport, encoding, and defaults.

**Independent Test**: Given a selected Baseline Client profile with complete typed rule mappings and a request that violates one mapped applicable rule, validation rejects it before transport; an incomplete profile selection fails before transport; a valid request reaches the fake transport unchanged except for profile-authorized normative defaults.

**Acceptance Scenarios**:

1. **Given** a client with Baseline Client selected, **When** an outbound request violates an explicit profile-specific message or transport constraint, **Then** validation returns a stable profile error and the transport records no request.
2. **Given** a valid request and explicitly selected applicable profile, **When** validation runs, **Then** it applies only requirements and defaults linked to that profile and preserves explicit application choices.
3. **Given** a profile with unmet conditional prerequisites, **When** an application selects it, **Then** selection fails with the unmet conditions and does not fall back to another profile.
4. **Given** a profile with an applicable requirement lacking an explicit typed rule mapping or passing rule test, **When** an application selects it, **Then** selection fails with the uncovered requirement IDs before transport and does not interpret catalog prose.

### User Story 2 - Inspect profile evidence and gaps (Priority: P1)

A maintainer or application developer can inspect which clauses, normative requirements, operations, transport/encoding rules, and official test cases determine a profile's status.

**Why this priority**: A profile name or operation list alone cannot prove conformance; every obligation and evidence gap must be visible.

**Independent Test**: Given the checked-in catalog, generate the profile matrix and confirm each in-scope client profile links to resolvable clauses, requirement IDs, profile dependencies, and test IDs with fixture availability.

**Acceptance Scenarios**:

1. **Given** any client profile recorded in the catalog, **When** its evidence is inspected, **Then** the exact source sections, applicable requirement IDs, dependency profiles, and mandatory/optional test IDs are shown.
2. **Given** an official test case whose fixture is unavailable, **When** evidence is reported, **Then** it is marked unavailable and is not counted as a passing test.
3. **Given** a profile requirement linked to an unresolved source discrepancy or unassigned implementation/test, **When** readiness is evaluated, **Then** the profile remains evidence-incomplete and names the blocker.

### User Story 3 - Determine claim eligibility without making a claim (Priority: P1)

A release maintainer can determine whether a named profile is eligible for a later public claim without conflating selection, implementation, verification, and publication.

**Why this priority**: False conformance claims mislead users and undermine interoperability decisions.

**Independent Test**: Feed complete, incomplete, and contradictory evidence records to the readiness evaluator and verify that only complete, consistent evidence reaches `evidence_complete`; no feature-level evaluation emits a public claim.

**Acceptance Scenarios**:

1. **Given** a selected profile with any applicable normative clause or mandatory test unresolved, **When** readiness is evaluated, **Then** it is ineligible and lists each unresolved record.
2. **Given** all applicable clauses, linked implementation requirements, and required official tests have passing evidence, **When** readiness is evaluated, **Then** the profile may become `evidence_complete`, while its public claim remains a separate release decision.
3. **Given** the current Baseline Client evidence, **When** readiness is evaluated, **Then** its 16 unavailable mandatory fixtures keep the profile ineligible.

### Edge Cases

- A selected profile depends on another profile; the dependency must be satisfied recursively and cycles or missing profile IDs fail closed.
- Two selected profiles impose incompatible requirements or defaults; validation reports the conflict without using selection order to choose a winner.
- An applicable requirement has no feature, implementation, or verification assignment; readiness remains incomplete.
- A profile is conditional, server-only, or out of scope; it cannot silently become a 1.0 client claim.
- A mandatory test ID is present but its pinned fixture is missing, malformed, or linked to a different source section; it is an evidence gap, not a pass.
- A profile's required operation list is a minimum capability set, not an allowlist: an additional base-protocol-conformant operation is not rejected solely because it is absent from the profile list. Profile validation must not issue hidden Query or Discover Versions requests; these remain explicit operations.
- A profile rule conflicts with an unresolved OASIS source discrepancy; the affected behavior remains gated until a reviewed decision resolves it.

## Requirements

### Functional Requirements

- **FR-001**: The profile matrix MUST be derived from the checked-in normative catalog and cite the exact pinned KMIP Specification or Profiles document and section for every applicable requirement. It MUST NOT modify upstream OASIS source copies or download standards during builds.
- **FR-002**: The matrix MUST include every catalogued client profile marked `client_1_0` or `conditional`, with its stable profile ID, source clauses, normative requirements, dependency profiles, required operations/data, transport and encoding conditions, defaults, mandatory and optional test IDs, fixture availability, and applicability rationale.
- **FR-003**: KMIPKit MUST keep profile applicability, the selected 1.0 target, runtime profile validation selection, catalog evidence/readiness lifecycle (`claim_state`), and a published release claim as separate values. The selected 1.0 target MUST be recorded in the project-owned `specification/catalog/profile-targets.json`, independently of `claim_state`. The Baseline Client profile is the initial target whose conformance evidence KMIPKit will establish against the Specification §14.1 one-or-more-profile requirement; selecting it MUST NOT imply that the requirement is already satisfied or emit a conformance claim.
- **FR-004**: Before a profile is eligible for a claim, KMIPKit MUST account for every normative statement in every clause and subclause specified for that profile, including inherited profile dependencies, as required by Specification §14.1. Missing, unknown, contradictory, or unassigned records MUST fail closed.
- **FR-005**: When profile validation is explicitly selected, KMIPKit MUST verify required client capabilities and applicable message, transport, and response constraints against the selected profile before or during the corresponding exchange. A failed outbound message or transport constraint MUST prevent sending that request. Profile-required capabilities are minimum requirements, not an allowlist: KMIPKit MUST NOT reject an otherwise valid additional operation solely because the profile does not list it. Profile validation MUST NOT add hidden protocol exchanges or automatic retries.
- **FR-006**: Profile-authorized defaults MUST be applied only when their exact normative conditions hold. They MUST NOT override explicit application values or choose cryptographic algorithms, key sizes, usage, or protection policy implicitly.
- **FR-007**: A profile marked `conditional` MUST remain ineligible until all catalogued conditions are satisfied. Dependency profiles MUST be evaluated recursively. Profiles marked server-only or out of scope MUST NOT be reported as 1.0 client support.
- **FR-008**: Each official test-case record MUST preserve its pinned case ID, mandatory/optional status, source section, related profile and requirements, and fixture availability. Test-case descriptions and Usage Guide examples MUST remain evidence or informative material, not independent requirements.
- **FR-009**: Missing official fixtures MUST be reported as unavailable evidence and MUST block profile eligibility whenever required by the profile or the project conformance policy. KMIPKit MUST NOT reconstruct missing fixture contents from titles or wire examples.
- **FR-010**: The profile readiness report MUST show the selected target, applicability, exact clause/requirement/test coverage, typed rule mapping and rule-test coverage, implementation and verification links, dependencies, deviations, server prerequisites, fixture gaps, blockers, and evidence state. Identical reviewed inputs MUST produce deterministic report output.
- **FR-011**: This feature MUST NOT publish a public conformance claim. A separate reviewed release process may make a claim only after all applicable clauses and required tests pass and the profile reaches `evidence_complete`.
- **FR-012**: Profile validation errors and reports MUST NOT disclose credentials, private keys, secret material, raw KMIP bodies, or TLS private keys. Evidence references MUST use validated repository-relative paths or stable identifiers, never arbitrary sensitive payload or diagnostic text.
- **FR-013**: Profile semantics MUST be defined once in the Rust protocol core and remain suitable for equivalent Rust, C, Java, and Python capabilities in the later 1.0 public API parity work. This feature MUST NOT create language-specific conformance behavior.
- **FR-014**: A profile MUST NOT be selectable for runtime validation until every applicable normative requirement has an explicit typed rule mapping and executable verification coverage. Catalog descriptions and free-form summaries MUST remain metadata and MUST NOT be interpreted as executable rules. Missing mappings or verification coverage MUST fail closed before network I/O.
- **FR-015**: The profile-target manifest MUST be loaded only from its fixed repository-relative path using a bounded read of at most 64 KiB. The loader MUST reject symlinks or reparse points, invalid UTF-8 or JSON, duplicate object keys, unknown or missing fields, wrong JSON types, duplicate profile IDs, empty profile lists, release lines other than `1.0`, unresolved decision references, and IDs that do not resolve to in-scope client profiles. The profile generator MUST load the normative catalog through repository safe I/O and the existing `load_validated_catalog` path, enforcing the catalog's existing size and schema limits; generator tests MUST reject invalid and over-limit catalog inputs. Neither input loader may accept caller-controlled paths or fetch data from a network. The target manifest MUST supply target generation/reporting only and MUST NOT constrain explicit runtime profile selection.
- **FR-016**: The metadata generator MUST encode every catalog-derived Rust string with a Rust-compatible string-literal encoder. It MUST safely handle quotes, backslashes, control characters, and Unicode without allowing catalog text to alter generated Rust syntax. Generator tests MUST include adversarial strings.
- **FR-017**: The catalog validator MUST reject catalog-internal inconsistencies and MUST reject `claim_state: evidence_complete` when a required fixture is missing or unavailable; an unavailable fixture record itself MUST remain valid catalog evidence and an explicit blocker. The readiness evaluator MUST verify current linked normative, implementation, and test evidence. A catalog `claim_state` of `evidence_complete` MUST NOT be trusted when any required evidence is missing, failed, unavailable, or contradictory. Such a mismatch MUST yield effective `evidence_incomplete` with a blocker. If evidence passes but the catalog state has not been reviewed and advanced, readiness MUST remain at the catalog state and report that disposition as a blocker; generation MUST NOT silently upgrade the catalog.

## Key Entities

- **Client profile**: A named KMIP selection with stable identity, role, source clauses, requirements, dependencies, transport/encoding constraints, and official tests.
- **Selected target**: A profile chosen as KMIPKit's 1.0 conformance target, distinct from runtime validation configuration and a public claim.
- **Applicability condition**: A catalogued condition that determines whether a profile can be used within the 1.0 product boundary.
- **Evidence record**: A clause, requirement, operation, implementation link, verification result, test-case ID, or fixture status used to assess readiness.
- **Profile readiness**: The result of evaluating a selected profile and its dependencies, including blockers and evidence state.
- **Public conformance claim**: A release-level assertion made only by the separate reviewed release process after the evidence gate passes.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 18 client profiles in the reviewed catalog are represented; every one of the 15 `client_1_0` or `conditional` profiles has a traceable applicability disposition and every other client profile is visibly excluded with its catalog reason.
- **SC-002**: 100% of the selected Baseline Client profile's source clauses, requirement IDs, dependencies, and official test IDs resolve to catalog records and exact pinned OASIS sections.
- **SC-003**: For every selected profile, a single missing or failing applicable normative requirement or required test makes claim eligibility false and identifies the blocker.
- **SC-004**: Baseline Client is recorded as the initial 1.0 target, but no public claim is emitted by this feature; the 16 currently unavailable Baseline mandatory fixtures remain visible and block eligibility.
- **SC-005**: A rejected profile-invalid request produces zero transport writes; a valid request preserves explicit values and follows only profile-authorized defaults.
- **SC-006**: Profile and evidence reports are deterministic for identical catalog and verification inputs and contain no environment-specific paths or unsupported claims.
- **SC-007**: Every profile's mandatory and optional official test IDs and fixture status are visible, with unavailable fixtures never counted as passed.
- **SC-008**: No profile status is inferred from a server's Query or Discover Versions response, and no validation path issues those operations implicitly.
- **SC-009**: Selecting a profile with any unmapped applicable requirement or missing rule test fails deterministically before transport I/O; generated catalog metadata and free-form descriptions are never treated as executable validation rules.
- **SC-010**: The feature is ready for review only after required formatting, lint, tests, coverage, security, independent QA, generated-output, and cross-platform CI checks pass on a branch updated from `release/1.0.0`; the agent opens a draft PR and does not approve, merge, or publish it.
- **SC-011**: Oversized, malformed, duplicate-key, schema-invalid, symlinked, or out-of-scope target manifests are rejected before profile generation; adversarial catalog strings cannot alter Rust source syntax; and stale or contradictory `claim_state` never produces `evidence_complete`.

## Assumptions

- The checked-in catalog remains the canonical map from profile records to pinned KMIP 2.1 Specification, Profiles, and Test Cases sources.
- This draft selects Baseline Client as the initial target because it is a client profile applicable to the 1.0 scope; §14.1 requires one or more client profiles but does not mandate Baseline specifically. This is a target selection, not a conformance or certification claim.
- Runtime profile validation remains explicit, and Query and Discover Versions remain explicit operations, as recorded in the accepted API architecture.
- The 1.0 product boundary remains KMIP 2.1, TTLV, TLS 1.3 with mutual TLS, raw TLS and HTTP/1.1 over HTTPS. JSON/XML and server-initiated operations remain outside 1.0.
- All currently missing Baseline profile test fixtures remain evidence gaps until obtained and pinned through a separately reviewed source process. No current profile claim is considered complete.
- The existing claim-state vocabulary and report rules in `specs/002-normative-inventory/data-model.md` and `docs/compliance/conformance.md` remain authoritative.
- Catalog presence, a profile's in-scope applicability, or its selection as the 1.0 target does not make it runtime-selectable. Runtime selection requires explicit typed mappings and tests for all applicable normative requirements.

## Out of Scope

- Implementing KMIP server behavior or server profiles.
- Adding JSON or XML encodings or server-initiated operations to 1.0.
- Making a formal certification or FIPS claim.
- Publishing profile claims from this feature PR.
- Reconstructing or silently fetching missing OASIS fixtures.
- Replacing the canonical catalog or changing the document hierarchy.
