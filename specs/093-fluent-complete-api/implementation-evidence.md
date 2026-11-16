# KMIPKIT-0093 Implementation Evidence

## KMIPKIT-0122 binding audit report regeneration

### Red

- On PR #221 head `3595d0c8cf416b9bb9b024cac9f86fe5c938de87`, ran:
  `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`
- The four tests failed because the checked-in binding export report predates
  the versioned catalog additions. Its per-package manifest counts were short
  by 62 declarations: C `2,002` versus `2,064`, Java `2,406` versus `2,468`,
  and Python `2,617` versus `2,679`. The dedicated C report check failed on
  the same stale count.
- The finding is a stale generated report and documentation snapshot; the
  manifest and binding declarations are already present on this branch.

### Green

- Regenerated the current C, Java, and Python export audit with:
  `python3 -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-11-15-versioned-catalogs.json`
- The generated report is complete for every package, with zero missing,
  stale, kind-mismatched, or otherwise unclassified exports. Its manifest
  counts are C `2,064`, Java `2,468`, and Python `2,679`.
- Updated the current-report references and documentation snapshot to the
  generated inventory. The previous dated report remains unchanged as history.
- `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v` passes
  all 4 tests. Regenerating the inventory with the documented `--check`
  command also passes without changing the checked-in report.

### Refactor

- Extracted the shared manifest-to-package declaration grouping used by both
  report assertions, keeping the blocked C declaration rule in one place.
- Re-ran `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`;
  all 4 tests pass.

## KMIPKIT-0124 manifest snapshot repair

### Red

- On PR #223 head `f57660fa60be9d9c86161204333169e24d602136`, CI run
  `43700578979` failed `Script contracts / windows-latest` in
  `test_t102_slice_and_manifest_snapshots_remain_consistent`.
- Reproduced with:
  `python3 -B -m unittest tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions.T102Pkcs11RequestTraitExclusionTests.test_t102_slice_and_manifest_snapshots_remain_consistent`
- The test reported `AssertionError: 16816 != 16772`. Comparing the public API
  manifest against the PR base identified 44 added declarations: 28 KMIP 3.0
  fluent operations and 16 asynchronous batch bindings. The inherited T102
  public-declaration total had not been refreshed for those approved additions.

## KMIPKIT-0124 binding audit report regeneration

### Red

- After rebasing PR #223 on updated PR #221 at local head
  `adf5eaecc3a3c4554a8906d340fc353726fe7f1f`, ran:
  `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`
- The four tests failed because the inherited report contains the KMIPKIT-0122
  base counts but omits 11 KMIPKIT-0124 declarations in each binding package:
  C `2,075` versus `2,064`, Java `2,479` versus `2,468`, and Python `2,690`
  versus `2,679`.
- The generated audit then identified 11 unclassified public JNI bridge methods
  and four Java facade mappings that name `createGroupFromAttributes`,
  `createUserFromAttributes`, `deactivateFromFields`, and
  `obliterateFromIdentifier`, while the source exposes those typed operations
  as overloaded draft method names. The exact bridge symbols need approved
  internal exclusions, and the semantic facade names need to match the
  approved manifest mappings.
- Added manifest/documentation assertions and changed the Java consumer test to
  call the manifested method names. Red evidence:
  `python3 -B -m unittest tools.api_manifest.tests.test_c_api_docs -v` fails
  with the stale Java audit and the new method/exclusion assertions;
  `bash ./gradlew test --tests org.kmipkit.extensions.Kmip30SemanticBatchItemTest`
  fails compilation because those four manifested semantic methods are absent.
- The expanded T102 run then exposed a traceability mismatch on the public
  `kmipkit::protocol::LocateResponse::try_from_response_item` re-export:
  its exact `rust-kmipkit` alias omits KMIPKIT-0093-FR-005 and FR-008 present
  on the canonical `rust-kmipkit-protocol` declaration. Reproduction:
  `python3 -B -m unittest discover --failfast -s tools/api_manifest/tests -p 'test_t102*.py'`
  fails `test_locate_client_declarations_exclude_server_only_size_requirement`.
- The same alias test subsequently exposed KMIPKIT-0093-FR-008 missing from
  the public `kmipkit::protocol::LocateResponse::result` alias while present
  on its canonical protocol declaration.
- Continuing the same T102 suite exposed a second response traceability gap:
  `kmipkit::DelegatedLoginResponse::result` omits KMIPKIT-0093-FR-008, while
  its shared `ClientResponseView` carries it. The suite fails
  `test_exact_operation_result_declarations_map_to_shared_metadata` for
  `KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN`.
- The focused response-mapping test confirms the same missing FR-008 on 23
  additional operation-result declarations; each paired response view has
  that requirement and no other requirement-set difference. Reproduction:
  `python3 -B -m unittest tools.api_manifest.tests.test_t102_operation_result_mappings -v`.
- The full T102 run also found one historical non-target manifest fingerprint
  that must include KMIPKIT-0124's 44 approved declarations and the response
  traceability corrections. It fails
  `test_all_non_target_declarations_and_operations_match_the_pinned_snapshot`
  in `test_t102_optional_request_structure_mapping.py`.

### Green

- Updated the shared T102 declaration snapshot to 16,816 and recorded the
  KMIPKIT-0124 additions that account for the increase.
- The focused reproduction command above passes (`Ran 1 test`, `OK`).

### Refactor

- Split the current public-manifest count assertions from the historical Rust
  audit totals so future approved public API additions do not obscure the
  unchanged Rust audit snapshot.
- `python3 -B -m unittest tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions`
  passes all 7 tests.

### Green follow-up after the response and export corrections

- Regenerated the 12 public API manifest outputs with
  `python3 -B tools/api_manifest/generate.py --write`; the subsequent
  `python3 -B tools/api_manifest/generate.py --check` verified all 12 outputs.
- Reconciled `LocateResponse::try_from_response_item` and
  `LocateResponse::result` alias requirements with their canonical protocol
  declarations. Added KMIPKIT-0093-FR-008 to the 25 operation-result
  declarations whose paired response views already expose it.
- Updated the non-target T102 fingerprints and exclusion count for the
  KMIPKIT-0124 additions. The exact focused regression run
  `python3 -B -m unittest tools.api_manifest.tests.test_t102_locate_request_builder_mapping tools.api_manifest.tests.test_t102_certify_request_builder_mapping tools.api_manifest.tests.test_t102_optional_request_structure_mapping.T102OptionalRequestStructureMappingTests.test_all_non_target_declarations_and_operations_match_the_pinned_snapshot tools.api_manifest.tests.test_t102_operation_result_mappings`
  passes all 17 tests.
- Renamed the Java semantic factories to the manifest names while retaining
  the separate TTLV overloads. The focused Java test
  `bash bindings/java/gradlew --project-dir bindings/java --no-daemon --console=plain --quiet test --tests org.kmipkit.extensions.Kmip30SemanticBatchItemTest`
  passes.
- The updated binding report records complete scans with no issues or missing,
  stale, or kind-mismatched exports: C 2,135 observed / 2,075 matched, Java
  3,190 / 2,479, and Python 3,699 / 2,690. The documented C and binding audit
  consistency tests pass (`python3 -B -m unittest
  tools.api_manifest.tests.test_c_api_docs -v`, 6 tests).
- `git diff --check` passes. The full T102 discovery run was interrupted before
  it returned a result; it is not counted as passing evidence.

### Refactor

- Reviewed the Java semantic factories and manifest renderer after the green
  run. The factories already share `systemObjectFromAttributes`, and the
  renderer remains the single source for generated outputs, so no additional
  production refactor was warranted.
- Re-ran the 17 focused T102/manifest regression tests, the six binding report
  tests, and the focused Java semantic batch-item test after that review; all
  pass.

## KMIPKIT-0124 follow-up after exact-head CI run 43721677012

### Red

- The exact PR #223 head `f882f8d73d89ac81eeacddd115606e10552de0e5` failed
  `Script contracts / ubuntu-latest` in
  `test_internal_bridge_and_dataclass_artifacts_use_exact_exclusions` because
  the expected JNI bridge inventory omitted 11 KMIPKIT-0124 bridge symbols.
  Reproduction before the fix:
  `python3 -m unittest tools.api_manifest.tests.test_binding_exports.BindingExportAuditTests.test_internal_bridge_and_dataclass_artifacts_use_exact_exclusions`.
- Extending the expected inventory then exposed that the audit accepted the
  older bridge requirements but not the approved KMIPKIT-0093-FR-011 ownership
  evidence. The test failed at its exact evidence check until that requirement
  was included.
- The same run failed the changed-production coverage gate at 502/578 lines
  (86.85%). Its macOS and Windows script-contract jobs were cancelled after the
  Ubuntu matrix failure; they are not independent product failures.

### Green

- The exact bridge inventory now includes the 11 KMIPKIT-0124 JNI dispatch
  methods, and the binding audit accepts their KMIPKIT-0093-FR-011 evidence.
  The focused binding-export audit passes.
- Async response tests now exercise typed response views, result access, and
  Debug/Display rendering for Cancel, Poll, Process, and Query Async Requests;
  pending async responses also exercise their typed borrowed views.
  `cargo test -p kmipkit-client --lib --all-features async_batch_item_` passes
  all 5 matching tests.
- The Java semantic batch-item suite now checks rejection and messages for
  missing inputs before JNI loading. The focused Gradle test
  `bash bindings/java/gradlew --project-dir bindings/java --no-daemon --console=plain --quiet test --tests org.kmipkit.extensions.Kmip30SemanticBatchItemTest`
  passes.
- The exact-head CI coverage gate has not yet run on these updates; its result
  remains pending and will be recorded before the PR leaves WIP.

## KMIPKIT-0093 HTTPS surplus-response coverage follow-up

### Red

- CI for PR #226 at `b9ea45b53` failed the selected-scope coverage gate:
  `Changed Rust code coverage 0/3 is below 90%`. The only changed production
  path was a `#[cfg(test)]` helper in `crates/kmipkit-transport/src/https.rs`;
  the source-included integration test passed but its helper lines were not
  credited by the workspace coverage report.

### Green

- Commit `77867e943` replaces the source helper with an observable peer-side
  signal: after sending the surplus body, the HTTPS fixture reports when that
  connection closes. The client waits for this signal before issuing the third
  exchange, then proves the third request used a new TLS connection and got its
  own response. No production Rust lines change.
- `cargo fmt --all --check` passes;
  `cargo test -p kmipkit-transport --test https` passes all 122 tests; and
  `cargo clippy -p kmipkit-transport --all-targets --all-features -- -D warnings`
  passes.

### Refactor

- The integration test now observes the protocol effect through the peer
  socket instead of exposing internal connection state solely for a test.
  The exact-head CI coverage gate is pending on the updated PR head.

## KMIPKIT-0093 C Create Key Pair builder

### Red

- Added a compiled C consumer probe for the opaque builder and a second Red
  check requiring local validation to reject missing algorithm and length.
  The generated C header initially lacked the new declarations.

### Green

- Added the C opaque builder ABI for explicit algorithm, length, separate
  private/public usage masks and names, grouped catalogued/expert/vendor
  attributes, asynchronous indicator, local validation, typed batch-item
  conversion, execution with exchange options, and release.
- The Rust builder can be detached for FFI use; detached execution returns a
  payload-free `NotSent` validation error. It uses the existing request
  validation and typed execution paths.
- Java now exposes `FluentRequest.createKeyPair()` and an `AutoCloseable`
  builder with explicit choices, grouped attributes, local validation, typed
  batch conversion, and execution through the existing client. Python exposes
  the same request-facade flow and a context-managed builder backed by the C ABI.
- Java and Python preserve the three attribute groups and provide catalogued
  and vendor-specific attribute methods; unknown attributes require an
  explicitly typed TTLV item.
- `cargo +1.94 test -p kmipkit-client --all-features fluent_create_key_pair`
  passes all 21 matching tests.
- `cargo +1.94 test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage`
  passes all 60 C ABI tests, including the incomplete/complete builder
  validation and typed batch-item probe.
- `bash bindings/java/gradlew test` succeeds, and the full Python suite passes
  all 1,357 tests. The focused Java and Python builder tests exercise explicit
  choices, validation, typed batch construction, and request-facade access.
- `python3 -B -m unittest tools.api_manifest.tests.test_0093_c_key_pair_builder_manifest -v`
  passes both tests. `python3 tools/api_manifest/generate.py --write` generated
  12 outputs and the subsequent `--check` verified them.
- Regenerated the current binding export report. All package inventories are
  complete with zero missing, stale, kind-mismatched, or otherwise unclassified
  exports: C 2,158 observed / 2,098 matched, Java 3,214 / 2,503, and Python
  3,726 / 2,717. The Rust export audit found no Create Key Pair builder export
  missing from the manifest; its repository-wide inventory still reports 180
  other missing declarations and is not represented as complete here.
- `cargo +1.94 fmt --all --check` passes. After correcting one
  `manual_let_else` reported by Clippy, `cargo +1.94 clippy -p kmipkit-ffi
  --all-targets --all-features -- -D warnings` passes.

### Refactor

- The C ABI remains separately declared as a C-only surface, while the shared
  Create Key Pair builder capability maps exact declared exports in Rust, C,
  Java, and Python. The binding audit enforces exact public export names.
- Added English and Spanish user-guide coverage for C validation, ownership,
  grouped attributes, typed batch construction, and Java/Python request-facade
  usage. The API manifest generator remains the source for all checked-in
  generated outputs.

## KMIPKIT-0093 Create Key Pair and signing TLS journey

### Red

- Extended the external-consumer acceptance check to require an ECDSA
  Create-Key-Pair/Sign/Destroy journey in all four languages. Before the
  fixture response and consumer paths were added, each consumer was missing the
  required journey marker.
- Added a focused fixture test requiring Destroy to echo the Unique Identifier
  from the request. It failed before the responder accepted and returned the
  request identifier.

### Green

- The loopback fixture now validates ECDSA, the explicit 256-bit length, the
  private Sign and public Verify masks, and the Sign request's SHA-256/ECDSA
  parameters. It returns distinct private/public identifiers and fixture
  Signature Data, and echoes each Destroy request identifier.
- Each language creates the pair, reads both typed identifiers, signs the same
  explicit input, checks the typed Signature Data, and destroys both keys.
  C initializes the versioned output records before reading typed response
  fields; Python reads the typed Destroy response and wipes its signature copy
  even when an assertion fails.
- The fixture consumer runner checks the full 41-request operation sequence
  over TLS 1.3/mTLS and rebuilds the editable Python CFFI package against the
  current workspace before execution. `bash tests/consumers/0053-parity/verify.sh`
  passes on the current worktree, with all four languages completing the key
  pair journey.

### Refactor

- Updated the English and Spanish user guides to distinguish these controlled
  TLS exchanges from synthetic response parsing and external-KMS
  interoperability. The same docs record that the expert-attribute and batch
  ID Placeholder journeys remain for the five-journey acceptance criterion.
- Updated FR-004, FR-011, and FR-013 traceability to reflect the exercised
  four-language path while preserving the remaining operation-matrix and
  closeout work.

## KMIPKIT-0093 expert Vendor Attribute TLS journey

### Red

- Added a fixture contract test that requires the named expert key's exact
  Vendor Attribute structure and rejects a Create request that omits it:
  `python3 -B -m unittest discover -s tests/consumers/0053-parity -p test_execute_ping.py -k fixture_requires_the_expert_vendor_attribute_on_the_named_key -v`.
- Before the fixture validator enforced the contract, the test failed because
  no `RuntimeError` was raised for the missing Vendor Attribute.
- Running the new Java journey against the old 41-operation sequence also
  failed at connection 12 because the fixture expected the old Destroy request
  instead of the expert Create request.

### Green

- All four language consumers now create an AES-256 key with explicit
  Encrypt/Decrypt usage and the Vendor Attribute (`KMIPKit.Example`, `purpose`,
  TextString `fluent expert journey`), then verify typed Create, Get, and
  Destroy responses for the returned `fluent-expert-key` identifier.
- The TLS fixture validates the decoded Attribute beneath the Object
  Attributes structure and echoes the request identifier in Get and Destroy
  responses. The expected runner sequence now covers 53 TLS exchanges.
- The focused fixture contract passes, and an independent run of
  `bash tests/consumers/0053-parity/verify.sh` passed for Rust, C, Java, and
  Python, including the new four-language Vendor Attribute journey.

### Refactor

- Updated the English and Spanish user guides, consumer README, and FR-013
  traceability to describe four completed journeys and the remaining batch ID
  Placeholder journey. The guide continues to distinguish controlled fixture
  responses from external-KMS interoperability evidence.

## KMIPKIT-0093 ordered batch ID Placeholder TLS journey

### Red

- Added fixture tests for grouped operation sequences, Create→Get request
  decoding, Batch Order Option validation, and response batches that retain
  all typed results and their Unique Batch Item IDs.
- Before the consumers had a batch journey, the end-to-end runner reached the
  next Ping where it expected `Create, Get`. The first implementation attempt
  was rejected locally with `RequestDeliveryState::NotSent`; tracing
  `validate_batch` showed that every item in a multi-item batch must carry a
  distinct Unique Batch Item ID.
- The fixture initially looked for Batch Order Option under the wrong tag and
  rejected the real request. The pinned typed model identifies this field as
  `0x420010`; the corrected fixture now validates its eight-byte Boolean value.

### Green

- All four consumers now create a named AES-256 key and Get it in the same
  ordered batch. Get omits Unique Identifier so the server resolves the
  preceding ID Placeholder. Each item has a distinct Unique Batch Item ID;
  the fixture echoes both IDs and returns typed Create and Get results.
- The fixture requires the exact Create→Get order, Batch Order Option true,
  the Create Name, omitted Get Unique Identifier, non-empty distinct batch
  IDs, and both corresponding response IDs. The response Batch Count is two.
- `python3 -B -m unittest discover -s tests/consumers/0053-parity -p test_execute_ping.py -v`
  passes all 11 tests, including the 57-exchange TLS 1.3/mTLS run across Rust,
  C, Java, and Python. An independent
  `bash tests/consumers/0053-parity/verify.sh` run also passes all five journeys
  in all four consumers.
- `cargo +1.94 check --locked --manifest-path tests/consumers/0053-parity/rust/Cargo.toml`,
  C11 `-Wall -Wextra -Werror -fsyntax-only`, Java consumer compilation,
  Python `py_compile`, and `git diff --check` pass.

### Refactor

- Updated English and Spanish user guides, the consumer README, F001 task
  status, and FR-013 traceability to describe all five completed progressive
  journeys and the fixture's limits. The evidence remains limited to a
  controlled fixture and does not claim external-KMS interoperability.

## PR #226 macOS HTTPS surplus-response CI follow-up

### Red

- Exact-head CI run `43758665557` failed on macOS stable in
  `https_never_attributes_a_surplus_response_to_a_later_exchange`: the third
  request received the server's extra response. The peer's completion signal
  followed its local TLS flush, which did not prove the client had processed
  those bytes before the test started the next exchange.

### Green

- Added a test-only observation of whether the retained HTTPS connection is
  still reusable. The integration test now waits, with a bounded condition
  wait, for the idle driver to reject the surplus input before sending the
  third exchange. It also requires the third request to reach the peer on a
  different connection and receive its own exact response body.
- `cargo test -p kmipkit-transport --test https
  https_never_attributes_a_surplus_response_to_a_later_exchange -- --exact`
  passed 12 consecutive runs after the synchronization change.

## KMIPKIT-0093 Rust Set Attribute fluent builder

### Red

- Commit `2cc3411a1` added a focused consumer test for the fluent constructor;
  `cargo test -p kmipkit-client --lib fluent_request_tests::set_attribute_builder_constructs_the_existing_typed_request -- --nocapture`
  failed because `RequestBuilder` had no `set_attribute` method.
- Commit `0163959f4` added negative validation coverage. The focused command
  remained red because the builder and its missing-attribute error did not yet
  exist.
- Commit `882a5c7da` added the manifest path contract; its focused Python test
  failed because the 26 public Rust paths were absent from `public-api.json`.

### Green

- Commit `86a38a25e` adds the Rust `SetAttributeBuilder` on the existing typed
  request and client execution paths. It supports catalogued typed attributes,
  explicitly typed TTLV Items, the KMIP Vendor Attribute constructor, optional
  identifiers, validation, batch conversion, exchange options, and one-exchange
  execution.
- `cargo test -p kmipkit-client --lib fluent_request_tests::set_attribute_builder -- --nocapture`
  passes all five focused tests, including local rejection with zero exchanges
  and successful typed response extraction after one exchange.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
  passes.
- `python3 -B -m unittest tools.api_manifest.tests.test_0093_set_attribute_builder_manifest -v`
  passes the exact-path and pending binding-parity checks.
- `python3 tools/api_manifest/generate.py --write` generated 12 outputs;
  `python3 tools/api_manifest/generate.py --check` confirms they are current.
- The Rust export inventory reports no missing or unobserved Set Attribute
  builder paths after reconciliation. Its full-repository report still has 205
  unrelated missing declarations and 8 unobserved manifest declarations; this
  feature does not claim that broader audit is complete.

### Refactor

- Added Rust usage guidance to the English and Spanish attribute-operation
  guides, updated the FR-004/FR-011 traceability notes, and declared the fluent
  builder's Rust exports and exact standard-trait exclusions in the canonical
  API manifest. C, Java, and Python Set Attribute fluent bindings remain an
  explicit open part of the global parity objective.
- The first manifest generation exposed that FR-006 was not included in the
  manifest requirement registry. Added the existing approved requirement ID
  before regeneration; full manifest validation and generation then passed.

### T102 snapshot follow-up

#### Red

- The exact-head CI for PR #231 failed
  `test_non_target_declarations_and_operation_rows_match_snapshot` because
  the new Set Attribute builder declarations were included in an unrelated
  Certify non-target fingerprint. The focused test reproduced the mismatch:
  expected `cd0161cb...`, observed `1ee37489...`.

#### Green

- Excluded only the KMIPKIT-0093 Set Attribute builder paths from the Certify
  non-target fingerprint. The dedicated Set Attribute manifest test continues
  to assert the complete exact public Rust path set and the explicit binding
  parity gap.
- `python3 -B -m unittest
  tools.api_manifest.tests.test_t102_certify_request_builder_mapping
  tools.api_manifest.tests.test_0093_set_attribute_builder_manifest -v`
  passes all 9 tests, and `git diff --check` passes.

## Rebase after Create Key Pair merged

- PR #227 has merged into `release/1.0.0`; this branch was rebased onto the
  release branch. On the resulting pre-follow-up head `e8ecb0f08`,
  `fluent_request_tests::` passes 24 tests, the focused Set Attribute manifest
  test passes, the manifest generator check passes, and `cargo fmt --all
  --check` passes.
- The exact-head check list on `e8ecb0f08` contained only SonarCloud success;
  the repository CI workflows had not produced runs for that SHA. PR #231
  remains `[WIP]` until full CI reports every required result on its updated
  head.

