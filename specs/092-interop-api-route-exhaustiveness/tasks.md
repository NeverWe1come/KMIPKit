# KMIPKIT-0092 Tasks

- [x] T001 Audit pinned OASIS 2.0/2.1 inventories and normative value references
  and add red tests for missing operations, objects, attributes, TTLV types,
  assigned/reserved values, extension allocations, tag ranges, request/response
  branches, and normative boundary/error classes.
- [x] T002 Add red tests for route-capability manifest validation and missing
  case/version/route/encoding/transport matrix cells.
- [x] T003 Define the shared route-capability schema and authored capability
  manifest for Rust, C, Java, and Python public packages.
- [x] T004 Generate the exhaustive coverage matrix from the versioned catalog
  and per-language route capability manifest.
- [x] T005 Extend validation evidence and aggregation to retain every execution
  axis and compare normalized request/response results without overwriting
  repeated runs.
- [ ] T006 Add CI checks that reject missing/duplicate coverage, declared
  routes that do not call the named public API, incomplete generic TTLV
  roundtrips, and falsely positive cases.
- [ ] T007 Add cross-language fixture contracts for all matrix axes and
  deterministic byte comparison rules.
- [x] T008 Update documentation, requirement traceability, and generated
  artifacts.
- [ ] T009 Run common checks, review the scoped diff, commit Red/Green/Refactor
  evidence, push the feature branch, and remove `[WIP]` only after its exact-head
  CI passes.

T006 and T007 remain open until each separate language validator consumes the
generated matrix, invokes every named public route, exchanges requests and
responses with a controlled peer, and emits validated per-cell evidence. A
passing shared-contract job does not satisfy those language-level requirements.

## Development evidence: Secret Data constructor applicability

- Red commit `f7cb47c47`: `python3 -m unittest interop.shared.tests.test_route_capabilities.RouteCapabilitiesTests.test_secret_data_constructor_does_not_claim_inapplicable_key_block_crypto_fields -v` failed because the route included Cryptographic Algorithm and Cryptographic Length.
- Green commit `81a51297a`: the source-backed target mapping now applies the KMIP 2.0/2.1 §3.1 Table 11 exclusions to both `SecretData::try_new` and `SecretData::try_from_item`; focused route tests and `route_capabilities.py --check` passed.
- Refactor/documentation commit: records the applicability rule in requirement traceability and preserves the exact Red/Green evidence here.
- Verification: `python3 -m unittest discover -s interop/shared/tests -v` passed all 161 tests on 2026-11-12.

## Development evidence: Secret Data route boundary

- Red commit `8e4f3601c`: the focused parser-scope test failed because the Secret Data parser route also declared eight unrelated object roots.
- Green commit `ad88c763b`: constructor and parser target generation now use only the Secret Data subtree while excluding the Table 11 fields; the focused tests and route-manifest regeneration check passed.
- Full verification: `python3 -m unittest discover -s interop/shared/tests -v` passed all 161 tests on 2026-11-12.
- Refactor/documentation commit: records the object-local boundary in requirement traceability and keeps this Red/Green evidence with the feature.

## Development evidence: release pin refresh after KMIPKIT-0120

- The feature branch conflicted with the updated `release/1.0.0` after PR #204 (`KMIPKIT-0120`) merged as `489fafc79ef89db2524eafa7a6c6482f5cf75d97`. The release merge was preserved as a normal signed merge commit; two FFI coverage script conflicts were resolved by retaining both the isolated `--no-clean` run and the existing package-scoped report checks.
- Red commit `072f4081a`: the focused release-pin contract test failed because the generated route inventory still identified the old commit.
- Green commit `9d9dbb308`: the route generator now pins the exact current release commit and regenerates all 1,491 route records. The catalog semantics and route count are unchanged.
- Refactor/documentation: the approved feature spec now records the updated release base and this evidence.
- Verification: the focused Red was observed; `python3 -m unittest interop.shared.tests.test_route_capabilities -v` passed 63 tests, `python3 interop/shared/tools/route_capabilities.py --check` passed, and `python3 -m unittest scripts.tests.test_workflow -v` passed 95 tests after conflict resolution. Exact-head CI must still pass after this branch update.

## Development evidence: session mutation policy

KMIPKIT-0083-FR-004 requires explicit mutation enablement for mutating cases.
The primary Login, Logout, and Delegated Login cases alter authenticated
session state and therefore require the same explicit opt-in as Log.

- Red commit `c35acc2a8` added a catalog regression; its initial version also
  assumed Delegated Login applied to KMIP 2.0. The source-backed catalog shows
  that Delegated Login is KMIP 2.1-only. Red commit `7ccde004f` scoped the
  assertion to the three primary `protocol_and_session` cases and confirmed
  that each was incorrectly classified as `read_only`.
- Green commit `85f4095a2` classifies Log, Login, Logout, and Delegated Login as
  mutating and regenerates the shared catalog. Negative and boundary cases
  retain their individual classifications.
- Verification: `python3 interop/shared/catalog/generate_test_catalog.py
  --check` passed, and `python3 -m unittest discover -s
  interop/shared/tests -p 'test_*.py'` passed all 188 tests.
- Exact-head CI and language-level checks that a policy-restricted session
  operation makes no network request remain required.
