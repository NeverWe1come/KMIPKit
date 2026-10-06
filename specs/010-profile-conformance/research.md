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
- Baseline Client links 5 profile requirement IDs and 16 mandatory test IDs. All 16 fixture records currently have `fixture_availability: unavailable`.
- HTTPS Client is marked `conditional`, links 19 requirement IDs and 17 mandatory test IDs, and depends on Baseline Client.
- The source catalog currently marks profiles `not_claimed`; this feature records Baseline readiness as `evidence_incomplete` because its 16 mandatory fixtures are unavailable. No public profile claim is emitted.
- The 1.0 target remains TTLV, KMIP 2.1, TLS 1.3 with mutual TLS, raw TLS and HTTP/1.1 over HTTPS. JSON/XML and server-initiated operations remain outside this feature's scope.

## Security decisions from independent review

- Read only the fixed `specification/catalog/profile-targets.json` path with the existing safe-read helper and a 64 KiB maximum. Reject symlinks/reparse points, invalid UTF-8 or JSON, duplicate object keys, unknown fields, duplicate IDs, unresolved IDs, and IDs outside the approved client scope before generating output.
- Read the source catalog with repository safe I/O and `load_validated_catalog`, retaining its existing size and schema limits. The target manifest is only for target generation/reporting; runtime selection remains an explicit caller choice and never consults target membership.
- Generate Rust string literals with a dedicated encoder. Quotes, backslashes, control characters, and Unicode must round-trip as string data; tests must verify generated source compiles and cannot gain injected syntax from metadata.
- Validate catalog-internal links and fixture availability, then recompute readiness using current implementation and test evidence before trusting catalog `claim_state`. If the catalog says `evidence_complete` despite a missing, failing, unavailable, or contradictory gate, report effective `evidence_incomplete` and the stale-state blocker. If evidence passes but catalog state is lower, retain that lower state until an explicit reviewed catalog update; never auto-upgrade catalog data.

## Normative sources

- OASIS KMIP Specification v2.1, §14.1: client implementation conformance and the complete normative-clause requirement for any claimed client profile.
- OASIS KMIP Profiles v2.1, §§3.1, 5.1.1, 5.1.3, 5.6.3, and 6.1: Baseline Client and its profile conditions/tests.
- OASIS KMIP Profiles v2.1, §§3.2, 5.3.1, 5.3.3, and 6.4: HTTPS profile conditions and tests, retained as conditional.
- Additional in-scope profile clauses are cited by each stable `KMIPKIT-PROFILE-*`, `KMIPKIT-CLAUSE-PROF-*`, and `KMIPKIT-REQ-PROF-*` catalog record; the generator must preserve those exact source references.
- Test Cases v2.1 case IDs are evidence only; the pinned Profiles document and inventory record determine their applicability and fixture status.

## Implementation boundary

The runtime validator belongs in the Rust protocol/client boundary and consumes generated immutable profile metadata. The generator and report extension use the repository's pinned Python tooling and the catalog as input. Public C, Java, and Python selection surfaces and their parity belong to the later API/bindings work, which must expose equivalent behavior. No dependency on network access, dynamic OASIS parsing, or executable plug-ins is introduced.
