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
