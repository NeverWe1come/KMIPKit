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

## Development evidence: scoped Rust route evidence union

- Red commit `351d9ccfd` added tests that require a route report to validate an
  exact planned subset without claiming the full matrix. The focused tests
  failed before scoped validation was implemented. Red commits `1b894a648` and
  `80ade8a8a` then reproduced incomplete evidence unions, valid events outside
  a declared scope, and the local runner selecting response routes that need a
  live peer.
- Green commit `045f86298` added exact scoped-report validation and collection
  union checks, rejected events outside each scope, and isolated local
  constructors from response accessors served by the controlled mTLS fixture.
- The gate now measures pending local and network cells against the canonical
  full Rust matrix even when a test supplies a smaller `--matrix` input.
- Verification on 2026-11-12: all 25 tests in
  `python3 -m unittest discover -s interop/shared/tests -p test_route_execution_evidence.py -v`
  passed, and the focused response evidence gate test passed.
- Full Rust route-gate execution reports 48,792 local cells and 287 controlled
  response cells invoked, with zero failed invocations. It still correctly
  exits nonzero with 26,376 network cells pending. T006 and T007 remain open.

## Development evidence: stable controlled-peer identity

- Red commit `5d1d764c9` added an assertion that the controlled mTLS peer has
  the same configuration fingerprint for KMIP 2.0 and 2.1. It failed because
  hashing the complete connection document included the selected protocol
  version and ephemeral endpoint port.
- Green commit `354da0bdf` fingerprints the server identity, selected endpoint
  host, TLS trust certificate, client certificate, and SNI. It omits connection
  details that vary per matrix cell, allowing response evidence from the same
  peer configuration to be combined safely.
- Verification on 2026-11-12: Rust validator Clippy passed; all 67 Rust unit
  tests and 3 CLI tests passed; the controlled mTLS response-accessor test
  passed for KMIP 2.0 and 2.1 with one shared fingerprint.

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

## Development evidence: C asynchronous Process then Poll route

- Normative scope: OASIS KMIP 2.1 §6.1.39 and Tables 278–279 define Process correlation, the requirement that the next Poll not be Pending, and the empty Process response payload. This implements KMIPKIT-0092-FR-003 and FR-005 for `test-async-process-then-poll-state`.
- Red commit `de923f2f`: the focused test failed because the C driver did not map the Process case to an isolated route.
- Green commits `3e0d4a0f` and `bb4aca1d`: the installed C consumer executes Create(Pending) → Process(correlation, Prohibited) → Poll(Success with original Create payload) → authenticated Destroy. The controlled peer checks the exact correlation, response shape, fixture state, and cleanup; the general TLS peer also resets state before the next independent async case.
- Refactor/documentation commit: records the normative source and per-cell evidence in KMIPKIT-0085 implementation evidence and this traceability record.
- Verification: the focused Process/Poll test passed; the complete CTest suite passed 7/7. The matching matrix recorded 40,931/66,736 invoked cells, all passed, and 25,805 pending recipes. The installed artifact SHA-256 is `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. Completeness remains open at `test-async-query-outstanding-filter`; no pending recipe is classified as an API limitation.

## Development evidence: C asynchronous Query outstanding filters

- Normative scope: OASIS KMIP 2.1 §6.1.41 Tables 285–287 defines the Query
  Asynchronous Requests filter fields and response structure; §7.2 Table 353
  defines the returned Asynchronous Request structure. This implements
  KMIPKIT-0092-FR-003 and FR-005 for
  `test-async-query-outstanding-filter`.
- Red commit `62e393db`: the focused route test failed because the installed C
  consumer had no Query outstanding-filter recipe.
- Green commit `31fb6298`: added asynchronous Create/Create/Get setup, completed
  one Create before Query, intersecting operation/correlation filters,
  controlled response assertions, terminal Polls, and authenticated cleanup
  across all six KMIP 2.1 encoding/transport pairs.
- Supplemental Red commit `8b250b0d`: the focused test failed because the
  recipe did not invoke `kmipkit_client_batch_item_response_create` for Pending
  Create responses.
- Supplemental Green commit `42a26232`: the consumer now uses
  `read_pending_create_correlation()` for those responses, checking the typed
  Create accessor's absent result payload and the exact server correlation.
- Verification: the focused test passed, including six controlled peer checks;
  CTest passed 7/7. The full matrix recorded 40,971/66,736 invoked cells, all
  passed, and 25,765 pending `validator_recipe_pending` cells. The installed
  artifact SHA-256 is
  `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. The
  completeness validator next reports
  `test-async-unknown-correlation-rejected`; coverage remains incomplete and
  no pending recipe is treated as an API limitation.## Development evidence: release pin refresh after KMIPKIT-0120

- The feature branch conflicted with the updated `release/1.0.0` after PR #204 (`KMIPKIT-0120`) merged as `489fafc79ef89db2524eafa7a6c6482f5cf75d97`. The release merge was preserved as a normal signed merge commit; two FFI coverage script conflicts were resolved by retaining both the isolated `--no-clean` run and the existing package-scoped report checks.
- Red commit `072f4081a`: the focused release-pin contract test failed because the generated route inventory still identified the old commit.
- Green commit `9d9dbb308`: the route generator now pins the exact current release commit and regenerates all 1,491 route records. The catalog semantics and route count are unchanged.
- Refactor/documentation: the approved feature spec now records the updated release base and this evidence.
- Verification: the focused Red was observed; `python3 -m unittest interop.shared.tests.test_route_capabilities -v` passed 63 tests, `python3 interop/shared/tools/route_capabilities.py --check` passed, and `python3 -m unittest scripts.tests.test_workflow -v` passed 95 tests after conflict resolution. Exact-head CI must still pass after this branch update.

## Development evidence: complete Rust route matrix CI budget

- Exact-head run `43470863653`, attempt 1, spent its full 30-minute job budget.
  All preceding checks passed; the final canonical route-matrix step was
  cancelled at timeout before it produced a completeness result.
- Red commit `cf6ef367` added
  `test_rust_interop_job_reserves_time_for_the_full_route_matrix`; the focused
  test failed because the workflow allowed only 30 minutes.
- Green commit `843be907` raised the bounded job timeout to 60 minutes. The
  focused test and all 96 tests in `scripts/tests/test_workflow.py` pass.
- Refactor/documentation commit records the CI time-budget reason in the
  validator README. Exact-head interop CI is pending after this correction.

## Development evidence: session mutation policy

KMIPKIT-0083-FR-004 requires explicit mutation enablement for mutating cases.
The primary Login, Logout, and Delegated Login cases alter authenticated
session state and therefore require the same explicit opt-in as Log.

- Red commit `c35acc2a8` added a catalog regression; its initial version also
  assumed Delegated Login was unavailable in KMIP 2.0. That assumption was
  incorrect: the pinned KMIP 2.0 source defines §6.1.12, Tables 184–186, as
  does KMIP 2.1 §6.1.12, Tables 199–201. Red commit `7ccde004f` scoped the
  assertion to the primary session cases and confirmed that each was
  incorrectly classified as `read_only`; the catalog retains both versions.
- Green commit `85f4095a2` classifies Log, Login, Logout, and Delegated Login as
  mutating and regenerates the shared catalog. Negative and boundary cases
  retain their individual classifications.
- Verification: `python3 interop/shared/catalog/generate_test_catalog.py
  --check` passed, and `python3 -m unittest discover -s
  interop/shared/tests -p 'test_*.py'` passed all 188 tests.
- Exact-head CI and language-level checks that a policy-restricted session
  operation makes no network request remain required.

KMIP 2.0 §6.1.25 and KMIP 2.1 §6.1.26 define Interop as test-state
control and prohibit its availability on production servers. The catalog now
requires the explicit dedicated-environment mutation opt-in for both versions.

## Development evidence: C Delegated Login payload fields

The installed C consumer now constructs the required Rights structure and the
Lease Time, Request Count, and Usage Limits variants from the Delegated Login
catalog cases. The controlled peer checks their exact KMIP field order and
values and returns a deterministic Ticket checked by the typed and TTLV
response accessors. Generic TTLV route fixtures cover Rights and Ticket
structures as well as the optional scalar and Usage Limits fields. This follows
KMIP 2.0 §6.1.12 Tables 184–185 and KMIP 2.1 §6.1.12 Tables 199–200.

- Red: the route recipe and fixture tests failed for all ten request/response
  field cases. The installed-consumer test then reported all ten as `not_run`
  with `fixture_peer_accept_timeout`, because the C driver recognized only the
  primary operation case and skipped the payload variants.
- Green: the recipe, local TTLV fixtures, and typed request construction now
  cover all ten field cases. The focused route tests passed all ten cases over
  TTLV/raw TLS for both protocol versions.
- `cmake --build interop/c-kmip-validator/build --target kmip_validator_c
  test_c_validator -j 2` passed. Exact-head CI and the full route matrix remain
  open; this slice does not claim the overall C route gate is complete.
- Full CTest passed 7/7. The full installed C route matrix now records 41,730
  of 66,736 cells invoked, zero failed, and 25,006 pending. The 304-cell
  increase from the preceding matrix includes all Delegated Login field
  variants; no Delegated Login cells remain pending. The completeness gate is
  still open.

## Development evidence: C Login under KMIP 2.0

The earlier C route evidence treated Login as KMIP 2.1-only and skipped Login
and Logout in the KMIP 2.0 controlled fixture. The pinned OASIS KMIP 2.0
Login payload tables (§6.1.29, Tables 235–236) have the same Lease Time,
Request Count, Usage Limits, and Ticket fields exercised by the C typed
constructor/readers in KMIP 2.1 (§6.1.30, Tables 253–254). The driver now
executes both direct Login and the ticket-backed Logout flow under KMIP 2.0;
the peer checks typed and generic Ticket access and never records the Ticket.

- Red: the focused KMIP 2.0 TLS contract failed on the prior pre-transport
  Login skip and then exposed an omitted expected Login exchange.
- Green: `c_validator_controlled_tls_peer_kmip20` passes with the Login
  constructor, typed/TTLV response readers, and Logout exchange exercised.
- Exact-head CI and the complete C route matrix remain outstanding.

- Red commit `8d1b1b938` added the catalog regression and observed that Interop
  was incorrectly classified as `read_only` for KMIP 2.0 and 2.1.
- Green commit `7d3ef6ac7` classifies Interop as `mutating` and regenerates the
  checked-in catalog. Its exact normative references are asserted by the test.
- Verification: `python3 interop/shared/catalog/generate_test_catalog.py
  --check` passed; the focused executable-contract tests passed 12/12; the
  complete shared test suite passed 189/189.
- This is test policy and catalog evidence. It does not claim server support,
  and the language validators still need checks that policy-restricted runs
  make no network request.

## Development evidence: Ephemeral message field encoding

- Red commit `31d996ec6`: the shared inventory regression failed because the
  source catalog and generated route catalog did not carry the Boolean type
  declared for Ephemeral.
- Green commit `315c2931a`: the normative inventory now records the source
  encoding and regenerates the executable catalog. The pinned source-table
  check ties the value to KMIP 2.0 §8.3/Table 358 and KMIP 2.1 §8.3/Table 396.
- Verification: all 297 normative catalog validation tests passed; all 23
  shared normative inventory tests passed; the complete catalog validator
  passed with 4,143 records; and `generate_test_catalog.py --check` passed.
- This closes the shared recipe-data gap only. Language-level generic TTLV and
  typed-message routes remain open until each validator emits passing evidence.

## Development evidence: regenerated route targets for Boolean TTLV

- Red: exact-head CI runs `43484370343` and `43484597700` failed the route
  inventory check because the generated Boolean value-constructor routes did
  not include the newly catalogued `KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL`
  target.
- Green: `python3 interop/shared/tools/route_capabilities.py --write`
  regenerated the route manifest from pinned public API source metadata,
  adding Ephemeral to the Rust, C, Java, and Python Boolean constructor routes.
- Verification: `python3 interop/shared/tools/route_capabilities.py --check`
  passed and reported 1,491 routes for library commit
  `489fafc79ef89db2524eafa7a6c6482f5cf75d97`.
- This updates generated route metadata only. The broader route execution and
  completeness gates T006, T007, and T009 remain open.

## Development evidence: New Attribute request structure type

- Red commit `5bfdcca01` added a pinned-source inventory regression for the Add
  Attribute request payloads. It failed for KMIP 2.0 and 2.1 because generated
  payload targets did not retain the `New Attribute` TTLV Structure type.
- Green commit `d23965263` records the Structure encoding and adds OASIS KMIP
  2.0 §5.7/Table 148 and KMIP 2.1 §5.7/Table 163 as source references. It
  regenerates the test catalog and shared route manifest, adding the six
  versioned New Attribute targets to generic TTLV Structure routes for all
  four language validators.
- Verification: the focused regression passed; all 24 normative inventory
  tests passed; `generate_test_catalog.py --check` passed; and
  `route_capabilities.py --check` passed with 1,491 routes pinned to library
  commit `489fafc79ef89db2524eafa7a6c6482f5cf75d97`.
- Language-level execution remains required; T006, T007, and T009 stay open.

## Development evidence: C asynchronous Poll and Cancel interleaving

- Normative scope: OASIS KMIP 2.1 §§6.1.5 and 6.1.38 define the Cancel and Poll
  operations; §§8.2–8.3, 8.6, and 9.1–9.2 define the request and response
  message shapes. The deterministic scenario sends asynchronous Create,
  Pending Poll, then Cancel and requires one terminal result with the same
  server-issued Asynchronous Correlation Value.
- Red commit `632e64300`: the focused route-matrix regression failed because
  `test-async-cancel-vs-poll-race` had no controlled C peer recipe.
- Green commit `a2f8da437`: the installed C ABI creates and inspects typed Poll
  and Cancel payloads, executes both operations after an asynchronous Create,
  checks both typed response views, and makes the controlled peer verify the
  exact three-request ordering and Pending/Pending/Success statuses.
- Refactor commit `0e34f770c`: asynchronous Create seed authentication is
  shared between the Cancel and Poll/Cancel routes; payload correlation checks
  use a common helper.
- Route-evidence merge regression: Red commit `b4fce5944` reproduced duplicate
  local Cancel cells already emitted by the offline model executor; Green
  commit `99f23c5e8` passes the existing cell keys into the network peer so one
  valid receipt covers each cell. Refactor commit `b44ebf347` documents the
  combined local and network coverage-key contract.
- Verification: the focused asynchronous race test passed for all six KMIP
  2.1 encoding/transport combinations. Both controlled TLS CTest peers passed.
  The full C route matrix recorded 40,250 invoked cells, zero failed, and
  26,486 pending, all `validator_recipe_pending`. The matching evidence
  validator exits unsuccessfully at the remaining missing-route check because
  the matrix is incomplete; it no longer reports duplicate route cells.

## Development evidence: C asynchronous Create, Query, and Poll lifecycle

- Normative scope: OASIS KMIP 2.1 §§6.1.8, 6.1.11, 6.1.38, and 6.1.41 define
  Create, Destroy, Poll, and Query Asynchronous Requests; §§8.2–8.3, 8.6,
  9.1–9.2, and 11.3 define the request/response and asynchronous message
  shapes. The route correlates Query and Poll to the same server-issued value,
  accepts only terminal Poll Success with the original Create payload, then
  cleans up the created object using authenticated Destroy.
- Red commit `5a1e59d84`: the focused regression failed because
  `test-async-lifecycle-poll` had no C route implementation.
- Green commit `9ee886593`: the installed C ABI performs Create, Query, Poll,
  and authenticated Destroy; the peer verifies exact request ordering,
  correlation filtering, terminal payload, and cleanup state.
- Refactor commit `02ae1ba91`: consolidate asynchronous peer step and exchange
  checks.
- Verification: six focused asynchronous route tests, the CMake build, Python
  syntax checks, `test_c_validator`, and both controlled TLS CTest peers passed.
  The full C matrix recorded 40,655/66,736 invoked cells, zero failed, and
  26,081 `validator_recipe_pending`. The exact completeness validator still
  fails on missing routes, next reporting
  `test-async-no-recursive-async-poll-cancel`; the matrix gate remains open.

## Development evidence: C mandatory asynchronous Create

- Normative scope: OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, and 11.3;
  Create is §6.1.8 Table 171 (2.0) and Table 186 (2.1). Response Batch Item
  structure is §8.6 Table 361 (2.0) and Table 399 (2.1). A Mandatory
  Asynchronous Indicator requires the server to process every batch item
  asynchronously and return a correlation value; a Pending response includes
  the required Response Payload structure.
- Red commit `bbab6bb04` added the accepted-Mandatory route regression. It failed
  because the C peer recipe did not yet implement the case.
- Green commit `01a3ea25d` constructs a typed Create request with
  `Asynchronous Indicator=Mandatory`, accepts only Pending with the exact
  server-issued correlation, and exercises KMIP 2.0/2.1 over all six supported
  encoding/transport pairs. The peer checks the Pending result, correlation,
  and empty Response Payload structure. A no-payload fixture was rejected by
  the installed client because only Pending Poll has the no-payload exception.
- Refactor commits `4551ce1ec` and `f1eccd192` share the Pending Create payload
  shape check and name the asynchronous Create seed assertion by its operation.
- Verification: the focused Mandatory Create route passed all 12 network
  cells; the five Cancel-result routes, Poll-before-Cancel route, and Cancel
  response payload roundtrip also passed. Both controlled TLS CTest peers,
  Python syntax checks, and the CMake build passed.
- The matching full route matrix recorded 40,336 invoked cells, zero failed,
  and 26,400 `validator_recipe_pending` cells. The exact completeness validator
  still exits unsuccessfully, next reporting `test-async-indicator-omitted-default`.
  This route advances coverage without closing the matrix gate.

## Development evidence: C omitted asynchronous indicator default

- Normative scope: OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, and 11.3;
  Create is §6.1.8 Table 171 (2.0) and Table 186 (2.1). When the optional
  Asynchronous Indicator is omitted, its default is Prohibited and Create
  completes synchronously without a correlation value.
- Red commit `3ba8b1a5a`: the focused route test failed because no C peer recipe
  existed for `test-async-indicator-omitted-default`.
- Green commit `c22343d5f`: the installed C ABI sends Create with the indicator
  omitted and checks synchronous Success, the typed response, and absent
  correlation over all six encoding/transport combinations and both versions.
  The peer also omits Unique Batch Item ID to match the request and permit
  correct response association.
- Refactor commit `cdd9f7d45`: share the Create request builder and typed
  response assertions with Mandatory asynchronous Create.
- Verification: the focused omitted-default route passed 12 network cells;
  the C build, Python syntax checks, five Cancel-result routes, Poll-before-
  Cancel route, Cancel response payload roundtrip, and both controlled TLS
  CTest peers passed. The full matrix recorded 40,422 invoked, zero failed,
  and 26,314 pending cells. The matching validator exits unsuccessfully at the
  missing-route check, now beginning with
  `test-async-indicator-optional-completes-or-pends`; T006, T007, and T009
  remain open.

## Development evidence: C Optional asynchronous Create completion

- Normative scope: OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, and 11.3;
  Create is §6.1.8 Table 171 (2.0) and Table 186 (2.1). For an Optional
  indicator, the server may complete synchronously or return Pending; this
  fixture selects synchronous completion, so no Poll is sent.
- Red commit `7eda42010`: the focused test failed because no controlled C peer
  recipe existed for `test-async-indicator-optional-completes-or-pends`.
- Green commit `8d6e93641`: the installed C ABI sends Create with the Optional
  indicator and validates synchronous Success with no correlation across both
  versions and all six encoding/transport pairs. The peer confirms the exact
  indicator and a single exchange.
- Refactor commit `8ab2db743`: share the synchronous Create response check
  between omitted and Optional indicator recipes.
- Verification: all three focused asynchronous Create tests, the CMake build,
  Python syntax checks, `test_c_validator`, and both controlled TLS CTest peers
  passed. The full matrix recorded 40,508 invoked, zero failed, and 26,228
  pending cells. The exact validator exits unsuccessfully on missing routes,
  now beginning with `test-async-indicator-prohibited-synchronous`; T006, T007,
  and T009 remain open.

## Development evidence: C explicit Prohibited asynchronous Create

- Normative scope: OASIS KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, and 11.3;
  Create is §6.1.8 Table 171 (2.0) and Table 186 (2.1). An explicitly
  Prohibited indicator requires synchronous operation without a correlation
  value.
- Red commit `7472db396`: the focused test failed because the C peer had no
  route recipe for `test-async-indicator-prohibited-synchronous`.
- Green commit `f204ff0c7`: the installed C ABI sets Prohibited and validates
  synchronous Success with no correlation over both versions and all six
  encoding/transport pairs.
- Refactor commit `161155dbd`: share the synchronous indicator request,
  response, and peer evidence checks with Optional completion.
- Verification: four focused async Create route tests, the C build, Python
  syntax checks, `test_c_validator`, and both controlled TLS CTest peers passed.
  The full matrix recorded 40,594 invoked, zero failed, and 26,142 pending
  cells. The exact validator exits unsuccessfully on missing routes, now
  beginning with `test-async-lifecycle-poll`; T006, T007, and T009 remain open.

## Development evidence: C request-based Get Attribute List batch route

- Normative scope: OASIS KMIP 2.0/2.1 §§6.1.19, 6.1.21, 8.2–8.6, and 9.5–9.7.
  The `test-batch-continue-after-failure` scenario contains Get followed by Get
  Attribute List and must retain both item results when the first operation
  fails under Batch Error Continuation Option Continue.
- Red commit `3e00fdb7`: extended the controlled-peer test to require the
  request-based Get Attribute List route across the case's version, encoding,
  and transport matrix. Before the recipe change, the test failed with 48
  checks instead of the required 60.
- Green commit `a9c08eb7`: run the batch case through both direct and
  `GetAttributeListRequest` constructors; the peer checks the same ordered
  operations and per-item outcomes for each variant.
- Refactor/documentation commit: cite the governing OASIS clauses on the
  conformance test and link this evidence to FR-003 and FR-009.
- Verification: the focused test passed all 60 peer checks, including 36
  request-based route events for two versions × three encodings × two
  transports × construct/send/decode. The full matrix and exact-head CI remain
  open; T006, T007, and T009 are not complete.

## CI follow-up: continued-batch route state assertion

- Red evidence: exact-head C installed-consumer run `43569674395` on
  `56b1c32a4a944c5d13047ce11b8441229335314e` failed because
  `test_continued_batch_evidence_has_observed_unchanged_fixture_state` still
  expected one check after the direct and standalone variants were both added.
  The same focused test reproduced `AssertionError: 2 != 1` locally.
- Green commit `d76df104`: require the two expected variants, verify each
  passed, bind each variant to its public API trace symbols, and confirm the
  controlled peer observed unchanged fixture state for every send event.
- Refactor/documentation commit: record the exact CI regression and its
  conformance-test coverage here and in FR-003 traceability.
- Verification: the focused test passed for both constructor variants, and
  `ctest --test-dir interop/c-kmip-validator/build -R
  '^c_validator_route_matrix_contracts$' --output-on-failure` passed all route
  matrix contract tests in 72.58 seconds. The full installed C route matrix
  and exact-head CI remain open; T006, T007, and T009 are not complete.

## Development evidence: C Adjust Attribute routes

- Normative scope: OASIS KMIP 2.0 §6.1.3 Tables 155–156 and KMIP 2.1 §6.1.3
  Tables 170–171 define the request and response. Sections 8.2–8.6 and 9.5–9.7
  govern the client batch exchange and decoded result. Increment by five on an
  absent integer attribute exercises §6.1.3's zero starting value.
- Red commit `c994fa3b`: the focused installed-consumer test failed because the
  C route registry produced zero checks instead of the required 24 direct and
  standalone route checks.
- Green commit `26d5f047`: build both `kmipkit_client_batch_item_adjust_attribute`
  and `kmipkit_client_batch_item_adjust_attribute_from_request` requests, encode
  the standalone payload, decode the typed response, and model the controlled
  peer's numeric adjustment and cleanup.
- Refactor/documentation commit: record the normative scope, local evidence,
  and remaining full-matrix gate here and in FR-003 traceability; extend the
  general TLS fixture to validate Adjust Attribute and its request fields.
- Verification: the focused test passed all 24 checks across KMIP 2.0/2.1,
  TTLV/JSON/XML, and raw TLS/HTTPS, with matching direct and standalone request
  trees and successful response decoding. `cmake --build
  interop/c-kmip-validator/build --parallel 2` passed. Full CTest passed 7/7
  tests in 77.18 seconds, including the route matrix contracts and the
  KMIP 2.0/2.1 controlled TLS peers. Feature traceability passed 31/31 tests.
  Exact-head CI and the full C route matrix remain open.

## C typed batch count evidence

The C controlled-peer case `test-batch-count-mismatch-rejected` now exercises
the public typed batch API as a read-only network route. Across KMIP 2.0/2.1,
TTLV/JSON/XML, and raw TLS/HTTPS, it sends Get plus Get Attribute List and the
peer validates Batch Count 2 against the two Batch Items. This demonstrates
that the typed send path derives the count from its items and exposes no count
override. Its 12 focused checks pass; this does not complete T006/T007/T009 or
the cross-language matrix.
## Development evidence: C Export request and response routes

- Normative scope: OASIS KMIP v2.0 §6.1.18 Tables 202–203 and v2.1 §6.1.18
  Tables 217–218 define the Export Request and Response Payloads. The response
  requires Object Type, Unique Identifier, Attributes, and Any Object. The
  ordered Create/Export/Revoke/Destroy exchange exercises ID Placeholder use
  and cleanup under §6.1 and §§8.2–8.6.
- Red commit `76f402452`: the focused test failed because `test-op-export` had
  no controlled-peer recipe.
- Green commit `f4aa1a690`: the installed C consumer builds the typed Export request,
  follows its Create with Export and cleanup, checks the typed response and
  complete response TTLV, and returns the expected payload from a controlled
  peer. The peer validates the four required response fields and the AES-128
  attributes without including key bytes in diagnostics or evidence.
- Refactor/documentation commit: record the scoped Export evidence, normative
  payload fields, and remaining matrix gate in C implementation evidence and
  requirement traceability.
- The operation slice passes all 44 C cells for `test-op-export`: four local
  generic TTLV cells, 36 constructor/send/decode cells across KMIP 2.0/2.1 and
  TTLV/JSON/XML over raw TLS/HTTPS, and four response-accessor cells. All 12
  controlled-peer checks pass and observe Create → Export → Revoke → Destroy
  with cleanup returning fixture state to its initial value.
- Request payload Red `4d4174e79` failed with zero checks; Green `367006d57`
  passes all 120 present/omitted peer checks for the ten catalogued request
  fields across both protocol versions and all six encoding/transport pairs.
  Response payload Red `42f8238e4` failed with zero checks; Green `488fdf799`
  passes all 96 checks for eight field/version cases, including 48 expected
  rejections after a required response field is removed.
- Generic TTLV Red `8a69dc91b` found four local Key Wrap Type/Any Object
  cases without fixtures; Green `208380cb0` added catalog-backed enumeration
  and Symmetric Key fixtures and passed the installed-ABI roundtrip test. The
  full matrix now has no pending Export cells. Exact report `c-route-19dc4d35-1e4d-44d0-8d38-c7573e1226fc`
  records 66,736 expected, 41,683 invoked, 0 failed and 25,053 pending overall.
- Verification: the focused request and response tests pass 120/120 and 96/96;
  the generic roundtrip test passes; full CTest passes 7/7 in 90.64 seconds,
  including 162 route-matrix contract tests. Python compilation and
  `git diff --check` pass. The full installed C route matrix and exact-head CI
  remain open; these operation/payload slices do not close
  KMIPKIT-0092-FR-003 or FR-009 globally.

## Development evidence: JNI sanitizer workflow contract after release refresh

- Red: rebasing the route-matrix branch onto the refreshed KMIPKIT-0092 base
  exposed `test_ffi_sanitizer_job_runs_the_c_consumer_under_address_sanitizer`:
  it required the literal `bindings/java/native/tests/zeroizing_bytes_test.cpp`
  in the workflow job, while the refreshed job runs every
  `bindings/java/native/tests/*.cpp` file. The file is therefore executed, but
  the test contract did not recognize the wildcard route.
- Green commit `6f43b17a4` accepts either the explicit source path or the
  wildcard source set and separately verifies that the zeroization test exists.
- Verification: `python3 -B -m unittest scripts.tests.test_workflow` passed
  all 96 tests and `git diff --check` passed.
- Refactor/documentation: the contract checks which test is covered without
  depending on whether the workflow names it directly or through a wildcard.

## Development evidence: route manifest after KMIPKIT-0092 refresh

- Red: after updating this stack to the refreshed KMIPKIT-0092 base, the
  existing route-manifest check and `test_route_capabilities` failed because
  the checked-in generated mapping no longer matched `public-api.json` in
  `normative_target_ids`. The route tests reported 6 failures and 1 error; the
  validation mismatch also masked several negative-test assertions.
- Green commit `35884ea32` regenerated `route-capabilities.json` using the
  repository generator, restoring the mapping for all 1,491 routes pinned to
  library commit `489fafc79ef89db2524eafa7a6c6482f5cf75d97`.
- Refactor/documentation commit: records the generated-file refresh and fixes
  the JNI sanitizer Green commit reference after rebasing the feature branch.
- Verification: route-manifest validation passed; focused route tests passed
  65/65; shared interop tests passed 189/189; workflow contract tests passed
  96/96; Rust 1.94 validator tests passed 72/72; and `git diff --check` passed.

## Development evidence: C Batch Order and ID Placeholder cases

- Normative scope: OASIS KMIP 2.0 and 2.1 §§8.1–8.6, 9.5–9.8, 9.21, 6.1.1,
  and 6.1.8, as recorded by the two catalog entries. §9.8 gives an omitted
  Batch Order Option its effective value of true.
- Red commits `f371958ab` and `46d77c92c` require exact route recipes and
  controlled-peer exchanges for `test-batch-order-required-placeholder-resolution`
  and `test-batch-order-omitted-defaults-true`. Both focused tests failed before
  the route was implemented.
- Green commit `b692631bf` adds a two-item Create/Activate batch to the C
  installed consumer. It verifies the typed result identifiers, checks that
  Activate resolves to the identifier returned by Create, and cleans up in a
  second exchange. The peer checks order, response correlation, option presence
  or omission, placeholder resolution, and fixture-state restoration.
- Verification: the focused peer tests pass for all 24 KMIP 2.0/2.1,
  TTLV/JSON/XML, and raw TLS/HTTPS combinations. CTest passes 7/7 in 115.95
  seconds, including both standalone controlled TLS peers. Exact-head CI and
  the global FR-003/FR-005 matrix remain open.

## Development evidence: C two-item batch read constructors

- Normative scope: OASIS KMIP 2.0/2.1 §§6.1.19 (Get), 6.1.21 (Get Attribute
  List), 8.2–8.6, and 9.5–9.8, as recorded in the batch cases.
- Red commit `73c9e66da` adds a test requiring direct and
  `GetAttributeListRequest`-based constructors for both
  `test-batch-count-matches-items` and
  `test-batch-order-false-correlates-response-items`; it failed with only the
  direct constructor's 24 checks.
- Green commit `37cb057b7` adds the two cases to the standalone constructor
  variants. The installed batch path verifies request and response counts and
  correlates typed responses by Unique Batch Item ID; the peer reverses the
  responses for the Batch Order Option false case.
- Verification: both focused tests pass across KMIP 2.0/2.1 and all six
  encoding/transport pairs, with direct and standalone variants. The full C
  matrix confirms zero pending cells for both target cases.

## Development evidence: C batch response view decoders

- Normative scope: OASIS KMIP 2.0/2.1 §§6.1.19, 6.1.21, 8.3, 8.6, and 9.7.
- Red commit `8669cbb9b` requires response metadata v1, response header TTLV,
  pending-correlation, and raw payload accessors on both batch read cases; the
  focused route test failed because these symbols were not invoked.
- Green commit `292f0c5fe` reads those accessors for each successful response
  item and checks the returned structures, metadata status, and synchronous
  absence of pending correlation and Result Reason. Continue's failed Get
  uses the existing accessor checks for its absent payload.
- Verification: focused route tests pass for both cases, both protocol
  versions, and all six encoding/transport pairs. CTest passes 7/7, including
  175 route-matrix contract tests. The exhaustive matrix reports 42,349/66,736
  cells invoked, zero failed, and 24,387 pending, with zero pending cells for
  both cases. The global completeness gate remains open.

## Development evidence: C MAC and Sign response TTLV accessors

- Normative scope: OASIS KMIP v2.0 §6.1.31 MAC and §6.1.49 Sign; v2.1
  §6.1.32 MAC and §6.1.55 Sign. KMIPKIT-0092-FR-003 and FR-005 require the
  applicable response decode routes to be invoked and checked.
- Red commit `29a3e1867` extends the focused route test and fails because the
  installed consumer does not invoke the MAC and Sign generic response TTLV
  accessors for either protocol version.
- Green commit `3bbfc5180` invokes both accessors after the existing typed
  response checks and verifies each returned complete response item has four
  outer fields. It does not read or record the MAC Data or Signature Data
  bytes.
- Refactor commit `01b12cbb7` names the complete response item field count and
  adds the normative references to the route test.
- Verification: the focused route test passes for both operations and both
  versions. Full CTest passes 7/7 in 139.15 seconds. The full matrix report at
  `01b12cbb7` records 66,736 expected, 42,067 invoked, zero failed, and 24,669
  pending. It adds the four expected MAC/Sign TTLV decode cells; FR-003/FR-005
  and the completeness gate remain open.
