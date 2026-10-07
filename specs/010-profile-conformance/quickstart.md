# Quickstart: Profile Evidence and Validation Scenarios

This guide defines acceptance scenarios for the implementation phase. It does not claim any KMIP profile is supported.

## Prerequisites

- Use the checked-in KMIP 2.1 catalog, `profile-targets.json`, and generated profile metadata.
- Select a client profile explicitly when exercising runtime profile validation.
- Use the fake transport for deterministic request/no-request assertions.
- Use only local pinned source and fixture files; a missing official fixture is an evidence blocker.

## Scenario 1: Validate a profile-specific message constraint

1. Construct a test client without profile state and choose Baseline Client in a per-call `ProfileValidationSelection`.
2. Construct a request that violates a concrete mapped message/transport rule.
3. Submit it with `Client::execute_with_profiles(request, selection)`.
4. Confirm a stable error names the profile requirement ID and exact source section, and the fake transport records zero writes.
5. Submit a valid request through the existing `Client::execute` method and confirm it remains unprofiled.

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

## Scenario 11: Close inherited profile subclauses

1. Generate or validate every catalogued client profile against the pinned OASIS clause hierarchy and normative-inclusion references.
2. Confirm Baseline Client includes every KMIP Specification v2.1 clause and every client-applicable `KMIPKIT-REQ-SPEC-*` in the approved 1.0 scope through Profiles §6.1 item 1, while retaining server-only/out-of-scope dispositions.
3. Confirm §6.1 item 2 includes applicable §3.1.1 and §3.1.2 requirements `KMIPKIT-REQ-PROF-3.1.1-002`, `-004`, `-005`, `KMIPKIT-REQ-PROF-3.1.2-002`, and `-005`.
4. Remove one oracle-listed inherited clause or requirement from a test catalog and confirm validation fails with its exact IDs and readiness stays incomplete.

## Scenario 12: Reject untrusted, stale, or caller-asserted verification evidence

1. Run the required CI matrix for a pull request and confirm the run remains ineligible to certify evidence; after the reviewed change is present on the protected `release/1.0.0` branch, produce a manifest from its same-run `push` aggregate and fresh branch-protection API response.
2. Repeat with `pull_request`, `schedule`, or another event; an unprotected or wrong ref; a previous commit/run attempt; another repository or workflow; each missing/failing required job including `coverage-gate`; a duplicate verification ID; malformed metadata; missing token/API access; a protection response with too few reviews, stale approvals not dismissed, missing/malformed `bypass_pull_request_allowances`, any user/team/app bypass entry, admins not enforced, force pushes or deletions enabled; and a locally supplied `verified` status.
3. Confirm only the valid same-run `push` manifest for the protected release ref with sufficient current policy and all six required job IDs verifies results; every other input leaves evidence incomplete and names the stable verification/check ID without exposing logs or arbitrary paths.

## Scenario 13: Keep committed reports independent of CI run IDs

1. Generate the committed coverage report from the same catalog, profile-target, and assignment inputs under two different CI run IDs and attempts.
2. Confirm the committed report is byte-identical and contains no run ID, attempt, job conclusion, temporary manifest path, or raw workflow output.
3. Confirm the aggregate job may display separate ephemeral readiness results for each manifest without editing or uploading generated repository files.

## Expected result

Profile validation is deterministic and explicit. Reports show exact source-linked obligations and evidence. An absent mandatory fixture prevents readiness; no scenario publishes a profile claim.
