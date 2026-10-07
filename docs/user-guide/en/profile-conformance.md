# KMIP 2.1 client profile evidence

This guide explains how to read KMIPKit's client profile evidence. It does not
provide a currently usable profile-validation API. The typed profile rule
registry is empty, so no profile is available for runtime validation.

## Current status

KMIP Specification v2.1 §14.1 requires a client to conform to one or more
client profiles. KMIPKit records Baseline Client as the initial 1.0 target for
future evidence work. Target membership is a project planning choice; it does
not establish profile support or satisfy the specification requirement.

The Baseline Client catalog state is `evidence_incomplete`. Its 16 mandatory
official fixture records are unavailable in the pinned repository. They remain
evidence gaps and are not counted as passing tests. KMIPKit does not recreate
or infer the missing fixture contents.

## Reading profile status

The [generated coverage report](../../../specification/catalog/coverage-report.md)
shows profile applicability, selected target, runtime rule coverage, catalog
evidence state, linked tests, and fixture availability as separate facts:

- **Applicability** describes the profile's scope in KMIPKit 1.0. A profile
  outside `client_1_0` may be server-only, conditional, or deferred.
- **Target selection** identifies a profile whose evidence the project plans
  to establish. It does not turn the profile into a runtime option.
- **Runtime validation** requires a complete typed mapping and an
  executable, keyword-appropriate test for every applicable requirement.
  Missing or incomplete mappings keep selection unavailable.
- **Evidence state** reflects the catalog's recorded evidence. A local report
  cannot upgrade it, and stale `evidence_complete` data is treated as
  incomplete.
- **Public claim** is a separate release decision. This feature does not
  publish a conformance claim.

Use the linked clause and requirement IDs to inspect the source obligations.
Catalog summaries are descriptive metadata; they are never interpreted as
executable validation rules. A server's Query or Discover Versions response
also does not establish profile support.

## Claim readiness

A profile can become eligible for a later claim only after every applicable
requirement and mandatory official test has current evidence. Unavailable
fixtures, missing implementation or verification links, incomplete typed
rules, unsatisfied profile conditions, and unmet profile dependencies keep the
profile ineligible. An eligible evidence result still does not publish a
claim; publication belongs to the separately reviewed release process.

See the [conformance strategy](../../compliance/conformance.md), the
[KMIP 2.1 inventory](../../../specification/catalog/README.md), and the
[KMIPKIT-0010 specification](../../../specs/010-profile-conformance/spec.md)
for the evidence model and its scope.
