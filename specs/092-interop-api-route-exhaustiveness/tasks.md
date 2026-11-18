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

## Development evidence: release pin refresh after KMIPKIT-0120

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

KMIP 2.0 §6.1.25 and KMIP 2.1 §6.1.26 define Interop as test-state
control and prohibit its availability on production servers. The catalog now
requires the explicit dedicated-environment mutation opt-in for both versions.

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

## Development evidence: Java Boolean field route execution

- Red commit `4651a7c4c` required Java generic TTLV evidence for the catalogued
  Boolean message-field case; Red commit `a62eda031` added a focused fixture
  that failed because the controlled peer emitted no per-route Boolean
  construction or decode evidence.
- Green commit `98a4dabc9` executes all 12 selected Boolean message-field and
  structure-member cases through their applicable generic routes. It also
  invokes the typed Request/Response Message parsers for Criticality Indicator,
  checks Attestation Capable Indicator and Batch Order Option in a valid
  two-item request, and exercises Random IV through the Cryptographic
  Parameters builder, typed attribute converters, and AttributeSet.
- Refactor documentation updates the Java consumer README, FR-003 traceability,
  and the current ledger snapshot while leaving T006, T007, and T009 open.
- The route ledger was regenerated with the repository validator using
  `--selected-scope --write-ledger`, then checked with
  `tools/compare_route_ledger.py --refresh-reference` and the normal comparer;
  no generated file was edited by hand.
- Verification: the focused `LocalCatalogRouteChecksTest` and controlled TLS
  evidence test pass. `validate_route_evidence.py` accepts 26,745/26,745
  selected cells with zero failed cells, and the ledger comparison passes.
  The full Java matrix remains incomplete at 26,745/71,649 executed and 44,904
  pending cells across 1,266 case IDs; T006, T007, and T009 remain open and
  this work does not make the PR ready for review.
- The complete Java validator suite passes 56/56 tests with zero failures,
  errors, or skips. Temporary JVM files were placed under the validator's
  ignored `build/java-temp` directory because `/tmp` was nearly full.

## Development evidence: Java Item Type enum routes

- Red commit `8534357d6` requires every catalogued Item Type enum value to
  execute the four applicable generic TTLV routes; the focused test failed to
  compile because the dedicated recipe was absent. The general enum fixture
  correctly does not handle this family because it has no same-named Tag.
- Green commit `86887bae4` reads all 11 values from the approved catalog,
  verifies each code with `TtlvItemType.fromCode`, round-trips the value as a
  generic Enumeration Item, and emits the constructor, Item create, Item view,
  and codec decode route evidence for KMIP 2.0 and 2.1.
- The generated ledger was refreshed only after `validate_route_evidence.py`
  accepted the selected report; both the reference refresh and normal ledger
  comparison passed.
- Verification: `LocalCatalogRouteChecksTest` and the controlled TLS evidence
  test pass; the selected scope accepts 26,833/26,833 cells with zero failed
  cells. The full Java matrix remains incomplete at 26,833/71,649 executed and
  44,816 pending cells across 1,265 case IDs. T006, T007, and T009 remain open.
- Full-suite verification: the Java validator Gradle suite passes 57/57 tests
  with zero failures, errors, or skips. The selected-scope validator and both
  ledger comparison commands pass; `git diff --check` passes.

## Development evidence: Java Item Type individual value cases

- Red commit `875ccfa80` requires the 11 catalogued Item Type values to emit
  generic TTLV route evidence under their individual normative case IDs as
  well as the aggregate. The focused local test fails because only the
  aggregate has evidence.
- Green commit `57c6b2160` associates each catalog value with its unique
  normative test case, emits the four applicable routes for both case levels,
  and includes the individual IDs in the controlled report scope. The generated
  route ledger is refreshed from the accepted report, not edited by hand.
- Verification: the focused local and controlled-TLS tests pass; the
  selected-scope validator accepts 26,921/26,921 cells with zero failures and
  both ledger comparisons pass. The full Java validator suite passes 57/57
  tests with zero failures, errors, or skips; `git diff --check` passes.
- The matrix remains incomplete at 26,921/71,649 executed and 44,728 pending
  cells across 1,254 case IDs. T006, T007, and T009 remain open; the PR stays
  WIP.

## Development evidence: Java key-bearing managed-object root routes

- Red commit `a76832637` added the required test for the Symmetric Key,
  Private Key, and Public Key root structures; it failed to compile because
  the focused route recipe did not exist.
- Green commit `10afa5ac8` exercises each matching named factory plus
  `ManagedObject.fromFields` and `ManagedObject.fromItem` with a Key Block
  containing Raw Key Material. Report Red commit `f1552eebe` then proved that
  the selected evidence omitted those exact installed-package routes; Green
  commit `2457ade76` added them to the report and generated ledger. Refactor
  commit `178686b75` binds fixture identities to the exact catalog target.
- Verification: both focused Java route tests passed; the controlled report
  validates 34,696/34,696 selected cells with zero failures. The full ledger
  now has 71,641 planned, 34,696 executed, and 36,945 pending cells. Full
  Java matrix closure and exact-head CI remain open; T006, T007, and T009
  remain incomplete.

## Development evidence: deterministic Java evidence ledger

- Red commit `c5e78d52d` added a regression proving that unordered declared
  case IDs made otherwise equivalent evidence ledgers differ.
- Green commit `7d478fe82` sorts the case IDs when building the ledger;
  Refactor commit `0fb31cfa6` names the normalized input before assembling the
  output. The focused regression and all 109 shared route tests pass, and the
  capability audit accepts 1,507 routes.
- The selected Java report validates 34,696/34,696 cells with zero failures.
  Refreshing and rechecking the generated ledger now succeeds reproducibly;
  36,945 of 71,641 Java cells remain pending. T006, T007, and T009 remain
  open.

## Java Split Key managed-object root routes

- Red commit `45b9ab5c5` requires the Split Key root case to emit the specific
  factory and both generic `ManagedObject` factory routes; it fails to compile
  because the route recipe is absent.
- Green commit `e3b1bdfe7` constructs the Split Key with KMIP 2.0/2.1 §2.8
  Table 9 field order, the XOR method, two parts, threshold two, and a Raw Key
  Block. It invokes `ManagedObject.splitKey`, `fromFields`, and `fromItem`,
  checks the parsed fields, and records each route in the controlled report.
- Refactor commit `e8fdeb63d` uses the existing canonical scalar TTLV encoder
  and verifies that all emitted routes carry identical bytes.
- Verification: full Java validator suite 87/87, shared route suites 109/109,
  selected route evidence 34,702/34,702 with zero failures, generated ledger
  comparison, 1,507-route capability audit, and `git diff --check` pass. The
  full Java ledger remains incomplete at 36,939 pending of 71,641; the stack
  remains WIP and T006, T007, and T009 remain open.
