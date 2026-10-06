# Quickstart: Profile Evidence and Validation Scenarios

This guide defines acceptance scenarios for the implementation phase. It does not claim any KMIP profile is supported.

## Prerequisites

- Use the checked-in KMIP 2.1 catalog, `profile-targets.json`, and generated profile metadata.
- Select a client profile explicitly when exercising runtime profile validation.
- Use the fake transport for deterministic request/no-request assertions.
- Use only local pinned source and fixture files; a missing official fixture is an evidence blocker.

## Scenario 1: Validate a profile-specific message constraint

1. Configure a test client with Baseline Client explicitly selected for validation.
2. Construct a request that violates a concrete mapped message/transport rule.
3. Submit it through the client.
4. Confirm a stable error names the profile requirement ID and exact source section, and the fake transport records zero writes.

## Scenario 2: Preserve an additional valid operation

1. Configure a profile whose required operation list does not contain a base-protocol-conformant operation.
2. Construct that valid operation with explicit cryptographic parameters where relevant.
3. Validate and send through the fake transport.
4. Confirm the operation is not rejected solely for being outside the profile's minimum capability list, and the explicit values remain unchanged.

## Scenario 3: Report Baseline evidence gaps

1. Generate the profile readiness report from the checked-in catalog.
2. Inspect `KMIPKIT-PROFILE-BASELINE-CLIENT` and its 16 mandatory tests.
3. Confirm each unavailable fixture remains unavailable, the readiness state is incomplete, and no public conformance claim appears.

## Scenario 4: Fail a conditional profile closed

1. Select HTTPS Client without satisfying one of its exact catalogued conditions.
2. Confirm profile selection or readiness fails with the unsatisfied requirement ID.
3. Confirm the client makes no hidden Query/Discover Versions call and does not silently choose another profile.

## Scenario 5: Fail an incompletely mapped profile closed

1. Select a profile with at least one applicable requirement that has no explicit typed Rust rule mapping or executable test.
2. Confirm selection returns a deterministic incomplete-coverage error before the fake transport is written.
3. Confirm changing free-form catalog descriptions cannot enable the profile or change runtime validation behavior.

## Scenario 6: Reject an unsafe target manifest

1. Run profile generation with malformed target-manifest fixtures: input over 64 KiB, invalid UTF-8/JSON, duplicate JSON keys, missing/unknown fields, wrong JSON types, an empty profile list, empty/duplicate/unresolved IDs, a release line other than `1.0`, an empty or unresolved `decision_ref`, and a symlink/reparse point.
2. Confirm each input fails before generated output is written and no path outside the repository is read.

## Scenario 7: Reconcile readiness with current evidence

1. Set a profile's catalog lifecycle to `evidence_complete` while a required test is missing, failed, unavailable, or contradictory.
2. Confirm the effective result is `evidence_incomplete` with a stale-state blocker and no claim.
3. Give a profile passing evidence while its catalog lifecycle remains below `evidence_complete`.
4. Confirm readiness does not silently upgrade the catalog state and reports that a reviewed catalog update is required.

## Scenario 8: Escape untrusted catalog strings

1. Provide generator metadata containing quotes, backslashes, line breaks, tabs, control characters, and Unicode.
2. Confirm generated Rust compiles, the literal round-trips to the original string, and no injected syntax or additional Rust item is created.

## Scenario 9: Keep target reporting separate from runtime selection

1. Choose an in-scope client profile explicitly for runtime validation that is not in `profile-targets.json`.
2. Confirm runtime selection succeeds when its conditions, requirement mappings, and verification coverage are complete; changing target membership affects generated target/report metadata only.

## Scenario 10: Redact profile evidence reports

1. Render a report from malformed evidence/diagnostic inputs containing credential, private-key, and raw KMIP-body sentinel strings.
2. Confirm neither the report nor any returned error contains those sentinels or raw payload text; output uses only stable identifiers and validated repository-relative references.

## Expected result

Profile validation is deterministic and explicit. Reports show exact source-linked obligations and evidence. An absent mandatory fixture prevents readiness; no scenario publishes a profile claim.
