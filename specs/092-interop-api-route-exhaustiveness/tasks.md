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

## Development evidence: named managed-object Object Type routes

- Normative scope: KMIP 2.0 and 2.1 §§2.3–2.8 define Opaque Object, PGP Key, Private Key, Public Key, Secret Data, and Split Key; §11.34 defines their Object Type values. This implements KMIPKIT-0092-FR-003 for the installed C object constructors.
- Red commit `051647e3a80c95dfbb61fa478ed9f364a6882fe0`: `python3 -m unittest interop.c-kmip-validator.tests.test_route_matrix.RouteMatrixTests.test_named_object_type_cells_execute_their_typed_constructors -v` failed for all six Object Type targets because the matrix dispatcher returned no execution result.
- Green commit `120a29b5822e6de675d7a047a54109604412a94d`: dispatch each supported Object Type constructor to its matching catalogued object structure recipe; the focused test and all 92 C route-matrix contract tests passed.
- Refactor/documentation commit: retain the six-route evidence and exact normative sections with the implementation record.

## Development evidence: C Modify Attribute typed payload route

- Normative scope: OASIS KMIP v2.0 §6.1.33 Table 247 and v2.1 §6.1.34 Table 265 define the Modify Attribute Request Payload. This exercises the installed public `kmipkit_modify_attribute_request_create` and `kmipkit_modify_attribute_request_to_ttlv_payload` C symbols under KMIPKIT-0092-FR-003 and FR-009.
- Red commit `0045ca260`: `PYTHONPATH=interop/c-kmip-validator/tests:interop/c-kmip-validator/tools python3 -m unittest test_route_matrix.RouteMatrixTests.test_modify_attribute_typed_payload_route_is_inspected_from_installed_c_abi -v` failed at the expected missing-route assertion because `ModelCases.execute` returned `None` for the public payload route.
- Green commit `51fc4f620`: added the installed-ABI recipe to construct the typed request and inspect the serialized payload, including samples with both optional fields present and omitted. The focused test passed.
- Refactor/documentation commit: factored the repeated Attribute fixture shape and recorded the normative scope and Red/Green evidence here.
- Verification: the focused test passed; a targeted probe of the 14 generated local request cells for `test-op-modify-attribute` and its normative payload-field cases passed for KMIP 2.0 and 2.1; `ctest --test-dir interop/c-kmip-validator/build --output-on-failure -R '^test_c_validator$'` passed.
- The captured pre-change `route-execution-evidence.gaps.json` listed 72 network cells for `test-op-modify-attribute` as pending: 36 each for the typed request-to-batch-item and direct batch-item routes. This local payload slice does not address those transport cells; the full route matrix was not rerun here.

## Development evidence: C Delegated Login response accessors

- Normative scope: OASIS KMIP v2.0 §6.1.12 Tables 184–185 and v2.1 §6.1.12 Tables 199–200. The controlled peer supplies a deterministic Ticket, and the installed C consumer checks both `kmipkit_client_batch_item_response_delegated_login` and `kmipkit_client_batch_item_response_delegated_login_ttlv` under KMIPKIT-0092-FR-003, FR-005, and FR-008.
- The public client response accessors require a handle created by a client execution. A minimal Delegated Login request with the required Rights structure obtains that handle through the existing local mutual-TLS fixture; the standalone protocol response-message parser returns a different handle type. The fixture checks the request shape and does not substitute it for request-field variant coverage.
- Red commit `4df35394e`: the two focused response-accessor tests failed as expected because the missing Delegated Login recipe produced zero controlled-peer checks.
- Green commit `ae0702543`: the installed C consumer verifies the typed Ticket Type and exact Ticket Value, then reads the owned Ticket TTLV copy and checks both fields, their order, types, and values. The negative test changes Ticket Value bytes without changing their length and requires failure. Returned ticket data is never included in reports or diagnostics; the generic copy and all views are released on each path.
- Refactor/documentation commit: share the test catalog selection and record the normative scope and evidence here.
- Verification: `cmake --build interop/c-kmip-validator/build --target kmip_validator_c test_c_validator -j 2` passed; both focused tests and `ctest --test-dir interop/c-kmip-validator/build --output-on-failure -R '^test_c_validator$'` passed. A targeted Delegated Login execution passed all 40 previously pending operation cells: four local decode cells and 36 fixture transport cells across KMIP 2.0/2.1, TTLV/JSON/XML, and raw TLS/HTTPS, with 12 passing peer checks. The full route matrix and the operation's request/response-field variant cases remain outside this slice.

## Development evidence: C registered-key cryptographic batch

- Normative scope: OASIS KMIP v2.0 §§6.1.39 (Register), 6.1.1 (Activate), 6.1.17 (Encrypt), 6.1.11 (Decrypt), 6.1.40 (Revoke), and 6.1.15 (Destroy); v2.1 §§6.1.43, 6.1.1, 6.1.17, 6.1.11, 6.1.44, and 6.1.15 respectively. Both versions §§8.2–8.4 and §8.6 govern ordered batch processing. The ID Placeholder use is specified in §6.1, its Unique Identifier datatype is v2.0 §4.51/v2.1 §4.58, and its enumeration value is v2.0 §11.56/v2.1 §11.58. This implements KMIPKIT-0092-FR-003 through FR-006 and FR-008 for `test-batch-register-encrypt-decrypt-destroy`.
- A single request registers a raw AES-128 object with explicit Encrypt/Decrypt usage, activates it, encrypts and decrypts a deterministic block with explicit AES/ECB/None parameters, revokes it for Cessation of Operation, and destroys it. Every item after Register uses ID Placeholder; ordered execution and Continue are explicit so the cleanup items remain in the request after a preceding operation failure.
- The precomputed public vector is [FIPS 197 (2001), Appendix C.1](https://csrc.nist.gov/files/pubs/fips/197/final/docs/fips-197.pdf). The two crypto requests use its predetermined inputs so the complete batch can be constructed before dispatch. The controlled fixture checks exact request fields and returns that vector; it does not implement a local cryptographic provider or establish vendor compatibility. The installed consumer checks all numeric statuses and operation order, every response identifier, exact Encrypt/Decrypt Data, and Register/Revoke generic response TTLV. The peer observes pre-active, active, revoked, and deleted state and verifies that final state equals initial state.
- Red commit `83b3cfbe6`: the two focused tests failed at their expected peer-check counts (`0` versus `12`/`1`) because no batch recipe existed.
- Green commit `80b308a08`: the compiled installed C consumer, checked peer request/response recipe, and lifecycle state machine execute all 304 typed-route cells for the case. The positive test passes 12 combinations (KMIP 2.0/2.1 × TTLV/JSON/XML × raw TLS/HTTPS); the negative test mutates ciphertext bytes without changing length and requires consumer failure while peer state remains clean. Key material, plaintext, and ciphertext are absent from serialized evidence and diagnostics.
- Refactor/documentation: name the ordered operation constants and record scope, references, and verification here. The dedicated register, response-comparison, and batch-execution helpers keep this workflow separate from unrelated operation flags.
- Verification: C build, both focused batch tests, the existing Create/Activate/Get Attribute List/Revoke/Destroy batch regression, and `ctest -R '^test_c_validator$'` passed. Before this slice 304 typed cells were pending; the targeted execution covers all 304 with zero failures. The case's 24 existing generic TTLV cells and the full matrix were not rerun for this slice; other batch, cryptographic, request-field, and error cases remain pending separately.

## Development evidence: C controlled-peer registered-key batch regression

- Red commit `d3a276ed6`: the controlled TLS fixture was required to observe the six-operation Register/Activate/Encrypt/Decrypt/Revoke/Destroy batch; the focused KMIP 2.0 and 2.1 peer tests failed because that operation plan was not recognized.
- Green commit `150964b0a`: the fixture accepts the exact batch and requires each unique item identifier exactly once, then returns matching response identifiers. No request or response body is added to diagnostics.
- Refactor/documentation: this record distinguishes the earlier missing fixture recipe from a protocol or installed-library failure and records the verified regression gates.
- Verification: `ctest --test-dir interop/c-kmip-validator/build --output-on-failure` passed all 7 tests, including both controlled TLS peers; `python3 interop/c-kmip-validator/tests/test_route_matrix.py` passed all 109 contract tests; `git diff --check` passed. This verifies the controlled fixture and matrix contract, not external KMS interoperability.

## Development evidence: release pin refresh after KMIPKIT-0120

- The feature branch conflicted with the updated `release/1.0.0` after PR #204 (`KMIPKIT-0120`) merged as `489fafc79ef89db2524eafa7a6c6482f5cf75d97`. The release merge was preserved as a normal signed merge commit; two FFI coverage script conflicts were resolved by retaining both the isolated `--no-clean` run and the existing package-scoped report checks.
- Red commit `072f4081a`: the focused release-pin contract test failed because the generated route inventory still identified the old commit.
- Green commit `9d9dbb308`: the route generator now pins the exact current release commit and regenerates all 1,491 route records. The catalog semantics and route count are unchanged.
- Refactor/documentation: the approved feature spec now records the updated release base and this evidence.
- Verification: the focused Red was observed; `python3 -m unittest interop.shared.tests.test_route_capabilities -v` passed 63 tests, `python3 interop/shared/tools/route_capabilities.py --check` passed, and `python3 -m unittest scripts.tests.test_workflow -v` passed 95 tests after conflict resolution. Exact-head CI must still pass after this branch update.
