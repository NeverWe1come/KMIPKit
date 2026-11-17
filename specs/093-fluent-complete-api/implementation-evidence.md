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
## KMIPKIT-0093 Set Attribute bindings

### Red

- Commit `dcb129af4` extends the C consumer, Java facade test, Python consumer
  test, and API-manifest test to require a typed `SetAttributeRequest` from
  the fluent builder, inspect its payload, and convert it to the matching
  batch item. The earlier binding contract allowed only direct batch-item
  construction and did not prove this typed-request path.
- The manifest test first failed because
  `kmipkit_set_attribute_builder_build_request` was missing from the public C
  symbol declarations and the four-language capability mapping did not match
  the exact ready exports. After adding its public declaration and reconciling
  the three affected capability mappings, the generated header exposed the
  ABI function.

### Green

- Added the typed C ABI request constructor on the existing Rust
  `SetAttributeBuilder`, the Java `buildRequest()` JNI path, and Python's
  `build_request()` method. Each returns an independently owned typed request
  that can be inspected and converted to a batch item.
- `cargo fmt --all --check`,
  `cargo clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings`,
  and `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage`
  pass; the C consumer suite reports 61 tests passed, including payload
  inspection, batch conversion, and release of the request, payload, and item.
- `mvn --batch-mode --file bindings/java/pom.xml test` passes 718 tests with
  no failures; its JNI build and `SetAttributeBuilderTest` pass.
- Built the Python binding with the CI-pinned coverage requirements in a
  temporary Python 3.12 environment. The first collection run found that the
  new CFFI prototype referenced `kmipkit_set_attribute_request_t` before its
  forward declaration; moving the prototype below that declaration allowed
  collection to proceed. The focused request test then found that the two
  CFFI modules' opaque pointer types must be bridged by address before handing
  ownership to `NativeHandle`; the bridge now follows the existing fluent
  batch-item ownership pattern. The focused test passes and
  `python -m pytest -q bindings/python/tests` reports 1,359 passed.
- `python3 -B -m unittest
  tools.api_manifest.tests.test_0093_set_attribute_builder_manifest
  tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions -v`
  passes the exact Rust path, cross-language mapping, C-header, and T102
  snapshot assertions. `python3 -B tools/api_manifest/generate.py --check`
  confirms all 12 generated outputs are current.

### Refactor

- Updated the English and Spanish attribute-operation guides and FR-004,
  FR-006, and FR-011 traceability rows to describe the typed request path and
  its four-language bindings without implying complete operation-wide parity.
- The C, Java, and Python declarations remain generated or manifest-backed;
  the implementation adds no second transport or codec path.

### Rebased T102 snapshot follow-up

- Red commit `1616b8342` reproduced the post-rebase failure in
  `test_non_target_declarations_and_operation_rows_match_snapshot`: the new
  Set Attribute binding declarations changed the unrelated Certify fingerprint.
  A focused run failed with expected hash `cd0161cb...` and observed hash
  `a033de4a...`.
- Green commit `065717e70` scopes the non-target exclusion to the Set Attribute
  fluent parity and C ABI capability IDs. The C manifest rows use
  `fluent_set_attribute_c_abi`, while the Rust, Java, and Python rows use
  `fluent_set_attribute_parity`; both are already checked by the dedicated
  exact-surface manifest contract.
- `python3 -B -m unittest
  tools.api_manifest.tests.test_t102_certify_request_builder_mapping.T102CertifyRequestBuilderMappingTests.test_non_target_declarations_and_operation_rows_match_snapshot
  tools.api_manifest.tests.test_0093_set_attribute_builder_manifest -v`
  passes all four assertions, and `git diff --check` passes.

## KMIPKIT-0093 Add Attribute fluent request builders

### Red

- Commits `00f890771` and `3ffb824c8` added Rust consumer tests for typed
  request construction, validation, batch conversion, and one typed exchange;
  they failed first because `RequestBuilder::add_attribute` was absent.
- Commit `135859cf2` added a compiled C consumer for the opaque Add Attribute
  builder; it failed first because the public C type and symbols were absent.
- Commit `22f790cb8` added four-language API manifest assertions; they failed
  first because the builder declaration and operation functions were absent.
- Commit `57491df9f` added a Java consumer test that failed to compile because
  `AddAttributeBuilder` was not present.
- Commit `e14076b65` added a Python consumer test that failed because
  `Request.add_attribute` was absent.

### Green

- Rust commit `ffc82f876` added `AddAttributeBuilder` over the existing typed
  `AddAttributeRequest`, including detached construction, catalogued and
  explicit Items, Vendor Attribute construction, batch conversion, and one
  exchange through the existing client.
- C commit `07738867e` added the opaque ABI and generated public declarations;
  the compiled C consumer builds the typed request, inspects its batch
  operation, and exercises the pre-send null-client error path.
- Java commit `ae20d1018` and Python commit `5dd8d0de4` add the JNI/CFFI
  facades over the same Rust-owned builder and typed request handles.
  Their focused consumer tests exercise request ownership, validation, batch
  conversion, and release. The Python typed-request adapter owns the returned
  handle independently.
- `cargo test -p kmipkit-client --all-features add_attribute` passes six
  focused tests; the compiled C ABI consumer passes; the full Java Gradle test
  suite passes; and Python passes 1,362 tests with 119 subtests, including the
  built-wheel contract with `maturin` installed.
- The Add Attribute manifest tests pass and
  `python3 tools/api_manifest/generate.py --check` verifies the 12 generated
  outputs. The generated binding export report reconciles all C, Java, and
  Python declarations with zero missing, stale, or kind-mismatched exports.

### Refactor

- Commit `b4e4336cd` extracts the New Attribute selection, duplicate-call
  rejection, vendor construction, TTLV copying, and local error state shared
  by the Rust Add Attribute and Set Attribute builders.
- English and Spanish attribute-operation guides and the FR-004, FR-006, and
  FR-011 traceability rows now describe Add Attribute and its four binding
  paths with the KMIP v2.1 §6.1.2 and §5.7 references.
- Full required coverage and exact-head CI remain open. The dependent PR stays
  WIP until those checks pass.

### Add Attribute audit, metadata, and current-head verification

- Red commit `7066f415f` added a regression for duplicate public export paths;
  it failed on eight stale Rust v3.0 `object_class_mask` rows that were already
  covered by the approved KMIPKIT-0047 declarations. The manifest paths now
  point to the generated `bitmask` module, those eight blocked duplicates are
  removed, and the capability mapping list is sorted again.
- Red commit `129380d25` reproduced owner lookup selecting `0120-*` for
  KMIPKIT-0012 before the actual `012-*` directory. Green commit `72d0d60c5`
  requires an exact numeric directory boundary; all four owner-lookup tests
  pass.
- Red commit `4e95192a2` added the Add Attribute capability to the C runtime
  metadata expectation and exposed that the Rust facade still returned it as
  unavailable. Green commit `5af0f5190` exposes
  `fluent_add_attribute_parity` from `capability_is_available`; the focused FFI
  metadata test passes.
- `rust-export-report-2026-11-16.json` matches all 30,744 observed export paths: 9,779
  declarations, 20,965 exact exclusions, and zero missing, unobserved,
  mismatched, or unresolved rows. Its exact `--check` passes. The shared Rust
  inventory reports 5,292 mapped declarations and 20,965 exclusions; its
  generated report and `--check` pass.
- Public-manifest inventory, the Add Attribute contract, C header inventory,
  T102 snapshot, C API documentation, shared inventory, generator, and
  generated-output checks pass. The generator verifies all 12 outputs.
- Current local verification passes: `cargo fmt --all --check`, workspace
  Clippy with all targets and features, `cargo test --workspace --all-features`,
  and `cargo llvm-cov --workspace --all-features` (90.71% total line coverage).
  The compiled C ABI consumer suite passes 62 tests; Java `./gradlew test`
  succeeds; Python passes 1,362 tests and 119 subtests.
- Exact-head CI has not yet run. AAB004 remains open and the dependent PR stays
  WIP until all required CI jobs are green.

## Add Attribute exact-head CI repair

### Red

- PR #236 at `0147e68189aa3b85454f6b775f9d2158736322e5` failed the
  production execute-boundary audit because three item macros in
  `kmipkit-client/src/fluent.rs` have opaque expansions. Core and coverage
  jobs consequently failed their shared client test.
- Script contracts also exposed 76 KMIPKIT-0093-FR-011 declarations incorrectly
  tagged as KMIPKIT-0043 candidates. The checked-in T102 contract requires zero
  blocked KMIPKIT-0043 candidates.

### Green

- Replaced the opaque macro-generated Add/Set Attribute methods with explicit
  builder implementations, keeping the production AST audit fail-closed.
  The boundary inventory regression and
  `cargo test --workspace --all-features --locked` pass.
- Extended Rust export reconciliation so an explicit feature owner repairs
  unresolved blocked rows for that exact requirement. Reconciled the report
  with `--candidate-spec-id KMIPKIT-0093 --requirement-id KMIPKIT-0093-FR-011`
  and regenerated the 12 public API outputs. The T102 response-trait and Rust
  reconciliation suites pass all 102 tests; the KMIPKIT-0043 blocked count is
  restored to zero.
- `cargo fmt --all --check`, workspace Clippy with all targets/features and
  `-D warnings`, and `git diff --check` pass. The public API generator check
  passes.

### Refactor

- Documented the explicit owner and requirement flags for feature-specific
  Rust export reconciliation, including its narrow repair behavior. The public
  builder surface remains fully typed and the AST audit still rejects opaque
  macros.
- The exact-head CI rerun is pending; PR #236 remains WIP until every required
  check passes.

## Add Attribute SonarCloud duplication repair

### Red

- PR #236 at `2cc3ecd9be84a21380205d181558bc22c91ffcb3` failed the SonarCloud
  quality gate with 5.0% duplication on new code; the limit is 3%. The
  identical explicit Add/Set builder method bodies introduced to keep the
  production AST audit fail-closed exceeded that gate.

### Green

- Commit `7d715b26d` moves the shared Add/Set request state, validation,
  construction, and execution into private helpers. Each public builder keeps
  its existing named methods and typed request/result surface; method bodies
  delegate to the shared implementation without introducing a macro.
- The fluent request tests pass (31 tests), the complete Rust workspace suite
  passes, workspace Clippy with all targets/features and `-D warnings` passes,
  and `cargo fmt --all --check` passes.

### Refactor

- The shared helper accepts the operation-specific typed request constructor,
  validator, and `ClientRequest` variant. The builders keep their distinct
  missing-attribute errors and KMIP request types while execution still uses
  the existing single typed exchange path.
- A new exact-head SonarCloud analysis on PR #236 (`bb1a1a92`) still measured
  5.3435% duplication across 1,572 new lines. The duplication endpoint
  identified the common Add/Set builder methods in `fluent.rs`; the previous
  request-construction helper had not removed that source duplication.
- Replaced the repeated method bodies with short explicit wrappers over one
  private state-update helper. The public builder types and method names stay
  unchanged. An attempted macro-based extraction was rejected by the closed
  AST inventory, so the final form keeps every public method directly
  inspectable.
- The Add/Set parity and manifest suite passes all 26 tests,
  `cargo test -p kmipkit-client --all-features --lib` passes all 929 tests,
  the generator verifies 12 outputs, and workspace formatting passes. Exact
  head SonarCloud and CI results remain pending.
- The next exact-head SonarCloud result must confirm duplication is at or below
  the 3% gate. PR #236 remains WIP until every required check is green.

### Recheck Red

- The exact-head analysis for `8a878f1b4ba7d885fe10725a4503fe704a72c6d9`
  still fails SonarCloud: `new_duplicated_lines_density` is 5.4307% against a
  3% limit across 1,602 new lines.
- Sonar's duplication report locates a 60-line repeated Add/Set builder block
  at `fluent.rs` lines 3139 and 3201. Sharing only their state mutation and
  request execution therefore did not remove the duplicated public builder
  methods.

### Recheck Green

- Commit `4a183689f` moves shared Add/Set mutation methods into private trait
  defaults. Each concrete builder keeps its public inherent methods and
  signatures, with explicit type-specific delegation so the closed production
  AST can still inspect every public method. No macro-generated public code is
  introduced.
- The Set Attribute manifest audit now filters blocked duplicate module paths
  while requiring every declared Rust path to map across the four languages.
- `cargo test -p kmipkit-client --all-features --lib` passes all 929 tests;
  the Add/Set/parity manifest suite passes all 30 tests; the public API
  generator verifies all 12 outputs; workspace formatting and Clippy with all
  targets/features and `-D warnings` pass; `git diff --check` passes.

### Recheck Refactor

- This keeps the builder types and Rust paths as concrete nominal types, avoids
  adding a public generic helper type, and shares the repeated mutation logic
  without relying on macros that the source audit rejects.
- PR #236 remains WIP until a new exact-head SonarCloud analysis confirms the
  3% duplication gate and every other required CI check is green.

## KMIPKIT-0093 Locate and Revoke fluent bindings

### Red

- Commits `c4577c319`, `caa751a84`, and `6d01681d0` add failing consumer,
  option-boundary, and API-manifest checks before the implementation.
- Commit `8c1af13c4` pins the observed Java signatures for Locate and the two
  `execute` overloads. The test failed against the manifest: Locate carried
  Revoke signatures, and both execute overloads omitted
  `throws ClientOperationException`.
- The tests require typed Locate filters and Revoke reason/message/date,
  request options, pre-send rejection of unsupported versions and invalid
  dates, four-language declarations, and explicit public C ownership.

### Green

- Added typed Rust Locate and Revoke builders, opaque C builder handles and
  direct C conveniences, and Java/Python facades over typed request items.
  Builders preserve explicit filters and revocation data, expose execution
  options, and return the existing success, pending, failure, or delivery
  errors without polling or retrying.
- `python3 -B -m unittest tools.api_manifest.tests.test_parity tools.api_manifest.tests.test_0093_locate_revoke_fluent_manifest -v`
  passes 28 tests. `cargo test -p kmipkit-client --all-features --lib
  fluent_lifecycle_operation_tests` passes 23 tests. The C ABI coverage
  harness passes all 64 tests, including its compiled C consumer.
- The focused Java `LocateRevokeBuilderTest` passes. The Python fluent suite
  passes all 35 tests. `python3 -B tools/api_manifest/generate.py --write`
  regenerated the 12 manifest outputs and the subsequent `--check` verified
  them. `git diff --check` passes.
- The first full workspace run exposed a missing Rust readiness mapping for
  `fluent_locate_revoke_parity` and an incomplete closed C-export inventory
  for the new macro-generated Locate setters. Added the capability to the
  public readiness query and extended the inventory to verify the setter
  macro's panic boundary. The rerun `cargo test --workspace --all-features`
  exits 0; all 12 FFI metadata tests pass. Workspace Clippy
  (`cargo clippy --workspace --all-targets --all-features -- -D warnings`),
  `cargo fmt --all --check`, and `git diff --check` pass.

### Refactor

- The API parity audit initially treated the 19 C Locate/Revoke builder
  functions as one-to-one JNI methods. Java instead constructs typed
  `ClientBatchItem` values directly. Recorded that intentional binding shape
  in the parity test's explicit exception list; the 28 parity and manifest
  tests then pass.
- Updated English and Spanish guides, operation traceability, generated
  declarations, and the KMIPKIT-0093 task list with the new operation slice.
- The parity inventory records that Java Locate/Revoke facades construct typed
  batch items directly instead of dispatching each C builder function through
  one-to-one JNI methods; the focused 28-test suite passes.
- Corrected the Java signatures in the source manifest and declared the 26
  existing Rust root re-exports for Locate/Revoke with their four-language
  mappings. The C Locate handle now maps to `kmipkit_locate_builder_t`.
- Refreshed the T102 non-target declaration fingerprints and public declaration
  total for those manifest changes, then regenerated the 12 public API outputs.
  The binding audit now reconciles C 2,228/2,228, Java 2,640/2,640, and Python
  2,904/2,904 with no missing, stale, kind-mismatched, or signature-mismatched
  declarations.
- Added C consumer checks for invalid options, identifier alignment, message
  spans, and Revoke validation; Java checks every Locate/Revoke execute overload
  against a local peer that closes before TLS authentication; Python checks
  default and options execution plus invalid builder state and optional inputs.
  The 64-test C ABI coverage harness passes, `mvn --batch-mode --file
  bindings/java/pom.xml clean verify` passes 726 tests (one skipped), and the
  Python coverage suite passes 1,371 tests at 96.42%.
- Recomputed selected-scope coverage with the prior CI's Rust platform and JNI
  reports plus refreshed local C, Java, and Python reports. Changed production
  coverage is 423/458 (92.36%), above the 90% gate; workspace coverage is
  95.31%. New exact-head hosted CI remains required before this PR can leave
  WIP.
- Full workspace formatting, lint, coverage, and exact-head CI remain part of
  LRB004 and must pass before this PR can leave WIP.

### Complete Rust export reconciliation (KMIPKIT-0093)

- **Red — `6a91e562b`**: the manifest-limit regression expects the reviewed
  full 0093 inventory to fit a bounded 64 MiB input, but the generator still
  used the prior 56 MiB limit. The focused size test failed with
  `58720256 != 67108864`; the initial generator run also rejected the manifest.
  After temporarily allowing validation of the larger input, validation
  exposed eight stale KMIP 3.0 `enumeration::object_class_mask` mapping paths.
- **Green — `db9d720ca`**: set the strict input bound to 64 MiB, removed only
  the eight duplicate mapping paths identified by the Rust export audit, and
  refreshed the source manifest and all generated outputs with the pinned
  generator. Reconciled 419 exact Rust declaration candidates as blocked under
  KMIPKIT-0093 and recorded 666 exact Rust trait exclusions. No cross-language
  mapping was invented for a blocked candidate.
- The compiler audit reports 31,977 observed Rust paths, 10,346 manifest
  declarations and 21,631 approved exclusions, with zero missing, stale,
  duplicate, kind-mismatched, or conditional-mismatch paths. The shared Rust
  report is current at 5,440 mapped declarations and 21,631 approved
  exclusions. C, Java, and Python export audits match 2,191, 2,598, and 2,855
  declarations respectively, with no missing, stale, or kind-mismatched rows.
- The 64 MiB bound leaves the manifest reader strictly bounded while admitting
  the complete reviewed declaration and exclusion inventory. The KMIPKIT-0050
  reconciliation helper is not part of the current reproduction sequence: it
  enforces the old blocked `typed_cryptographic_parameters` contract that was
  superseded by the verified KMIPKIT-0053 capability.
- **Refactor**: updated the T102 count/fingerprint snapshots and the dated
  audit report guide. Focused API, T102, shared-inventory, size-bound and
  reconciliation tests pass (62 tests); three additional manifest-v2
  execution/result mapping tests pass. The generator verifies all 12 outputs,
  the Rust and shared-inventory report checks pass, and `git diff --check`
  passes. Exact-head hosted CI is still required before this PR can leave WIP.

### Delete Attribute stack rebase

- Rebased the Delete Attribute binding branch from its old Add Attribute base
  onto the updated KMIPKIT-0093 Add Attribute head `811062df3`. The rebase kept
  the feature commits and merged the source manifest by stable API identities;
  generated C, Rust, Java, Python, and fixture outputs were restored from the
  current base and regenerated from the merged source.
- The resulting inventory includes 77 ready Delete Attribute declarations,
  two blocked Rust candidates retained without cross-language mappings, and 18
  exact Rust trait exclusions. The Rust export audit reports 32,025 observed
  paths, 10,376 matched declarations, 21,649 approved exclusions, and zero
  missing, stale, kind-mismatched, or unresolved paths.
- The shared Rust inventory reports 5,468 mapped declarations and 21,649
  approved exclusions. Binding export audits match C 2,205/2,205, Java
  2,614/2,614, and Python 2,874/2,874 with zero missing, stale, or kind-
  mismatched rows. The generator verifies all 12 outputs.
- The Delete Attribute, shared-inventory, manifest-size, Rust reconciliation,
  and T102 regression modules pass (44 tests). Three additional manifest-v2
  execution/result mapping tests pass. Exact-head hosted CI for the rebased
  PR remains required before it can leave WIP.

## Catalog and code generation

### Red

- Added `tools/normative_catalog/tests/test_fluent_api_catalog_consistency.py`
  for the exact 59 distinct KMIP 2.1 operation codes across 62 direction
  routes, the seven assigned KMIP 3.0 codes, their client-to-server direction,
  the 60 exact KMIP 3.0 tag assignments, and cross-source tag name/value
  consistency.
- `python3 -B -m unittest
  tools.normative_catalog.tests.test_fluent_api_catalog_consistency -v` fails
  on 16 of 17 entries in `kmip-versioned-tags.json` `v2_1_tags`: the names at
  `0x420166`–`0x420171` and `0x420173`–`0x420176` disagree with the pinned
  KMIP Specification v2.1 §11.56, Table 487. The base normative catalog and
  pinned source agree; the derived supplement contains the mismatches. The
  other two checks pass, including the seven KMIP 3.0 operation values and 60
  assigned tag values.
- The reviewed sources are the pinned KMIP Specification v2.1 §11.36/Table
  470 and §11.56/Table 487, and the pinned KMIP 3.0 draft §11.37/Table 584
  and §11.58/Table 605. The pinned OASIS source files remain unchanged.
- Added a focused generation and manifest test for all 354 assigned KMIP 2.1
  tags and 60 KMIP 3.0 tags across Rust, C, Java, and Python. The first run
  exposed a test setup error (`StopIteration`) while locating the missing v3
  Tag group; after correcting the assertion to report the missing group, the
  focused test fails as expected: the existing KMIP 2.1 Tag enumeration has
  zero generated values instead of 354. Reproduced with:
  `python3 -B -m unittest tools.normative_catalog.tests.test_generate_named_values.NamedValueGeneratorTests.test_tag_enumeration_is_generated_for_all_supported_versions_and_languages -v`
- Reconciliation adds the generated tag declarations to the reviewed API
  manifest. A follow-up generation attempt exposed the existing 56 MiB manifest
  input bound: the reconciled file is 59,718,087 bytes. The manifest-size test
  now requires a bounded 72 MiB limit to accommodate this approved catalog
  growth and the remaining planned declarations; the unchanged generator fails
  that expectation. The precise validator check is in
  `tools/api_manifest/tests/test_t102_manifest_size_limit.py`.
- The first reconciled declaration inventory exposed six duplicate Rust module
  paths because the v3 Tag capability re-declared parent modules already owned
  by the versioned-value capability. The focused no-duplicate export assertion
  fails at `6,672 != 6,666`, and the manifest validator independently rejects
  the duplicate public export path. This test is included in
  `test_tag_enumeration_is_generated_for_all_supported_versions_and_languages`.
- The fresh Rust export audit also found eight previously unmapped KMIP 3.0
  `Object Class Mask` paths: the generated values are under `versions::v3_0::bitmask`,
  while the manifest generator recorded `versions::v3_0::enumeration`. The new
  regression test `test_versioned_bitmask_public_values_use_the_bitmask_namespace`
  fails on the expected `::bitmask::object_class_mask::USER` path. The audit
  reported these eight declarations as not observed at the manifest path.
- The generated KMIP 2.1 Tag name lookup exceeded Clippy's function-size gate:
  `cargo clippy --locked -p kmipkit-protocol --all-targets --all-features -- -D warnings`
  failed with `clippy::too_many_lines` at 357 lines. The new regression test
  `test_large_tag_lookup_is_split_into_clippy_sized_rust_functions` also fails
  because generated Rust has no chunked tag lookup helpers.
- Adding the assigned tag declarations made the existing full-manifest snapshot
  stale. `python3 -B -m unittest
  tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions.T102Pkcs11RequestTraitExclusionTests.test_t102_slice_and_manifest_snapshots_remain_consistent -v`
  fails at the declaration total (`18895 != 16816`). The manifest now contains
  2,077 KMIPKIT-0093 tag declarations; the approved exclusion total is unchanged.

### Green

- Generation now adds the 354 assigned KMIP 2.1 tags from the canonical
  catalog and the 60 assigned KMIP 3.0 tags from the checksum-validated
  §11.58/Table 605 catalog. The corrected 17-entry KMIP 2.1 supplement matches
  §11.56/Table 487; pinned upstream files are unchanged.
- The reconciled manifest adds 2,077 declared Rust, C, Java, and Python tag
  values and helper exports under `KMIPKIT-0093-FR-001` and `FR-002`. The public
  manifest input bound is 72 MiB, and its pinned snapshot now records 18,895
  declarations before the release rebase; it records 19,357 after merging the
  current release manifest. KMIP 3.0 tag names remain explicitly versioned.
- Rust lookups split the 354-value match into small private functions; the
  generated source passes Clippy with warnings denied. New C, Java, Python, and
  Rust consumer tests check exact 2.1/3.0 tag names and unknown-value behavior.
- The bilingual package guides document the new names and version boundary.
- Before the release rebase, local verification passed: 23 catalog tests; 114 API-manifest generator
  tests; 1,453 Rust protocol tests plus the tag integration test; C consumer
  tests 4/4; Java tests 714 with one pre-existing skip; Python tests 1,356;
  all four generated-output checks; and the C/Java/Python export audit. The
  pre-rebase Rust audit mapped every new tag path, but reported 193 other
  existing Rust paths missing from the manifest; none of those paths are tag exports.
  The full Rust export audit therefore remains incomplete outside this slice.
- Exact-head CI has not run yet. G004 remains open until the WIP PR's required
  checks pass and any failures are fixed.


### Refactor

- Consolidated the four language renderers' choice between the canonical
  Enumeration values and the separately sourced Tag assignments in one helper.
  `python3 -B -m unittest tools.normative_catalog.tests.test_generate_named_values
  tools.normative_catalog.tests.test_fluent_api_catalog_consistency -v` passes
  all 23 tests, and both the named-value and manifest reconciliation checks
  confirm generated outputs are unchanged.

## KMIPKIT-0093 named-tag capability runtime reporting

### Red

- CI run `43831588866`, Ubuntu coverage job `133178419627`, failed the FFI
  capability test: the generated C capability list included
  `fluent_named_tag_values`, but the Rust runtime query returned `Ok(false)`.
- After extending the expected C capability list, the focused Rust runtime
  metadata test reproduced the missing implementation at
  `crates/kmipkit/tests/runtime_metadata.rs`.

### Green

- Added the named-tag capability to the Rust runtime query and asserted it in
  both Rust and FFI capability tests. The focused Rust runtime and C capability
  tests now pass.
- `cargo fmt --all --check` and `git diff --check` pass.

### Refactor

- Kept the capability query's existing boundary and generated manifest output;
  only the runtime mapping and its parity tests changed. Re-ran both focused
  tests after the mapping change, with one test passing in each package.

## KMIPKIT-0093 release rebase contract follow-up

### Red

- After rebasing the catalog branch onto `release/1.0.0` at `e1b12188b`,
  `test_registry_parity_fixture_preserves_the_v1_registry_slice` failed because
  the current manifest ordered the preserved v1 `types` and `functions`
  differently from the historical fixture. Comparing records by ID showed the
  same record sets and contents; only array order differed.

### Green

- The fixture assertion now canonicalizes the identity-keyed `types` and
  `functions` arrays by ID before comparing them, while still comparing every
  field and rejecting missing or changed records.
- The manifest reconciliation, named-value generation, and API manifest
  generation checks pass on the merged source. The C, Java, and Python export
  audit also matches all declarations with no missing, stale, kind, or
  signature issues: C 2,580; Java 2,985; Python 3,236.
- `python3 -B -m unittest
  tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_registry_parity_fixture_preserves_the_v1_registry_slice -v`
  passes.

### Refactor

- This update changes only the test comparison; generated API output and runtime
  behavior are unchanged.

### Tag export allowlist CI repair

- **Red — `3cdc52712`**: added a regression that compares generated KMIP tag
  declarations with the canonical API manifest. It failed because all 415
  generated C tag declarations were missing from the manifest; the C ABI
  export audit reported the same 415 missing entries.
- **Red — `abf0795f8`**: added a regression for reconciling blocked observations
  after a named declaration is generated. It failed because two blocked Rust
  module observations retained the same export paths as the new versioned
  named-value declarations, which made API generation reject the duplicate
  paths.
- **Green — `209d6c919`**: the reconciler now removes blocked observations only
  when an approved generated declaration owns that exact package and export
  path. The canonical manifest was reconciled with
  `reconcile_named_value_manifest.py --write`, then all 12 API outputs were
  regenerated through `api_manifest/generate.py --write`.
- The 22 named-value generator tests pass. Reconciliation, named-value
  generation, and API-manifest checks pass. The Rust 1.94 C ABI build and export
  audit now match 2,643/2,643 declarations with zero missing, stale, or kind
  mismatches; the compiled C consumer suite passes 4/4 tests.
- Full exact-head CI remains required; G004 stays open and this PR stays WIP
  until the final head's applicable checks pass.

### Windows capability inventory regression

#### Red

- CI run `43974169961` failed on Windows Rust stable (job `133676924176`) and
  Rust 1.94 (job `133676924236`). The workspace test
  `capability_query_only_reports_c_mapped_nonblocked_capabilities` found that
  generated `C_AVAILABLE_CAPABILITIES` included the newly generated
  `fluent_named_tag_values` capability, while the test's explicit expected
  capability set omitted it. Both jobs passed formatting and Clippy; each ran
  476 FFI tests with this one failure.
- The focused Red is reproducible locally with
  `cargo +1.94 test --locked -p kmipkit-ffi --lib --features
  coverage-c-consumer
  extension_registry::metadata_tests::capability_query_only_reports_c_mapped_nonblocked_capabilities
  -- --exact --nocapture`.
- After adding that expected C capability locally, the same FFI test reaches its
  Rust parity check and fails because `kmipkit::capability_is_available` returns
  `Ok(false)`. A new assertion in
  `capability_query_matches_manifest_ready_capabilities` reproduces that missing
  runtime mapping with
  `cargo +1.94 test --locked -p kmipkit --test runtime_metadata
  capability_query_matches_manifest_ready_capabilities -- --exact --nocapture`.

#### Green

- Added `fluent_named_tag_values` to the Rust runtime capability query and to
  the FFI test's exact expected C capability set. The Rust runtime parity test
  and the C capability parity test now pass with Rust 1.94:
  `cargo +1.94 test --locked -p kmipkit --test runtime_metadata
  capability_query_matches_manifest_ready_capabilities -- --exact` and
  `cargo +1.94 test --locked -p kmipkit-ffi --lib --features
  coverage-c-consumer
  extension_registry::metadata_tests::capability_query_only_reports_c_mapped_nonblocked_capabilities
  -- --exact`.
- The full local workspace test reached doctests but could not link four
  `kmipkit-client` doctests: `rust-lld` terminated with signal 7 (Bus error).
  The focused runtime and FFI parity tests pass; exact-head hosted CI remains
  required before G004 can close or the PR can leave WIP.

#### Refactor

- Kept the existing match-based Rust capability query and generated C list.
  This fix only aligns their named-tag capability inventory and its parity
  assertions. `rustfmt +1.94 --edition 2024 --check` on the three changed Rust
  files and `git diff --check` pass.

### Current-head Script Contracts repair

- **Red — CI run `43978049870`, job `133693896521`**: the T102 contract suite
  could not import `T102_HISTORICAL_RUST_AUDIT_EXCLUDED` because the historical
  November 1 Rust audit totals were not exposed separately from the current
  November 16 totals. Restoring distinct historical constants keeps the old
  snapshot immutable while allowing current audit totals to advance.
- The full local API-manifest suite then identified stale binding-export
  counts for C, Java, and Python. Regenerated
  `binding-export-report-2026-11-16-kmipkit-0093-tag-values.json` through
  `audit_binding_exports.run` using the built Rust 1.94 C ABI, then refreshed
  the report table in `binding-export-audit.md`. The report now matches all
  declarations: C 2,643/2,643, Java 3,055/3,055, and Python 3,319/3,319, with
  no missing, stale, or kind-mismatched entries.
- **Green — local verification**: the 25 focused tests across
  `test_c_api_docs`, `test_t102_certify_request_builder_mapping`,
  `test_t102_pkcs11_request_trait_exclusions`, and
  `test_t102_usage_allocation_context_trait_exclusions` pass; `git diff
  --check` passes. Exact-head CI must still pass before this PR is ready for
  human review.

### Owner matrix evidence refresh

- **Red — `94c279c97`**: added a regression requiring the typed owner catalog's
  evidence list to match the current public operation parity evidence. The
  focused test failed for Delete Attribute because the owner catalog omitted
  five tests added by the Delete Attribute binding slice on `release/1.0.0`.
- **Green — `c4d98920c`**: added the five current C, Java, Python, Rust, and
  manifest test paths to the source owner catalog and regenerated the fluent
  operation inventory. The focused owner inventory suite passes all 10 tests;
  `generate_fluent_operation_inventory.py --check` and `git diff --check` pass.
- **Refactor**: kept the owner evidence synchronized with the canonical public
  API manifest. Exact-head hosted CI remains required before M005 is complete
  and this PR can leave WIP.
- **Post-merge inventory sync**: mirrored the five Delete Attribute builder
  evidence paths into the canonical public API operation's `parityEvidenceIds`
  and regenerated the public API projections. This keeps the owner catalog's
  exact-evidence check aligned with the operation inventory after the builder
  tests landed. The API and fluent inventory generators both pass `--check`.

## Rust Ping fluent builder slice

### Red

- Commit `801ae9b78` defines construction, local validation, typed build and
  batch conversion, one-exchange Success, explicit Pending correlation, and
  KMIP 2.0 rejection before exchange (OASIS KMIP v2.1 §6.1.36, Tables 271–272).
- Reproduced with `cargo test --locked -p kmipkit-client --lib
  ping_execution_tests`: compilation failed with five E0599 diagnostics because
  `RequestBuilder::ping` did not exist.

### Green

- Added the Rust-only `PingBuilder` around the existing empty `PingRequest`.
  Execution uses `ClientRequest::Ping` and the private shared typed exchange
  path, preserving typed Success/Pending and the existing version gate.
- `cargo test --locked -p kmipkit-client --lib ping_execution_tests` passes
  all eight Ping tests. No transport, decoder, or generic-send API was added.
- This slice remains WIP pending C, Java, and Python fluent builder parity and
  exact-head CI verification. It does not close operation-wide O001/O002 or
  FR-011 parity.

### Refactor

- Reused the existing private exchange-state helper for Ping instead of
  duplicating its client/options/asynchronous fields. Renamed that helper to
  `FluentBuilderExecution` to reflect its shared use; public cryptographic
  builder behavior is unchanged.
- `cargo test --locked -p kmipkit-client --lib ping_execution_tests` again
  passes all eight Ping tests after the refactor.

### Inventory, documentation, and local verification

- Reconciled the fresh Rust 1.94 compiler inventory with
  `tools/api_manifest/reconcile_rust_exports.py --report
  target/ping-rust-exports.json --candidate-spec-id KMIPKIT-0093
  --requirement-id KMIPKIT-0093-FR-011 --write`: nine blocked Ping declarations
  and nine exact standard-trait exclusions were added. Preserved the original
  order and exact contents of existing exclusions through the reconciler's
  serializer, then regenerated all outputs with
  `python3 -B tools/api_manifest/generate.py --write`.
- The manifest snapshot test first failed at `20303 != 20294`; its expected
  full-manifest totals now include exactly the nine new declarations and nine
  trait exclusions. All eight focused snapshot/PKCS#11 contracts pass.
- The independent Rust export report also found eight pre-existing stale
  `object_class_mask` module/value paths in the generated KMIP 3.0 namespace
  under `kmipkit_protocol` and `kmipkit::protocol`. No new Ping path remains
  missing after reconciliation; this is not a claim of a globally clean Rust
  audit. Historical checked-in audit snapshots remain unchanged.
- Passed `cargo fmt --all --check`,
  `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`,
  38 Rust export/reconciliation contracts, and 33 requirement/feature
  traceability contracts. Six marked examples in the English and Spanish
  Query/Ping guides compile through `scripts/test_user_guide_examples.py`.
- `cargo llvm-cov test --locked -p kmipkit-client --lib --all-features --lcov
  --output-path target/ping.lcov ping_execution_tests` passes eight tests.
  The Ping constructor/builder region has 29/29 covered executable lines.
- A first workspace run hit an infrastructure ENOENT in trybuild while coverage
  cleanup ran concurrently. Reran `cargo test --locked --workspace
  --all-features` sequentially after coverage completed: the full workspace
  suite passes, including compile-fail boundaries and doctests. Hosted CI must
  independently verify the PR head.

## Ping binding metadata and CI repair

### Red

- Commits `902894db8`, `544c142c7`, `42131fac4`, and `cc4d8e44f` added
  regression coverage for the C opaque builder kind, runtime capability
  availability, Java/Python declaration inventory, and capability mappings.
  The focused checks reproduced the missing/incorrect declarations and the
  capability query returning unavailable.
- Commit `308a0b662` added the exact Python constructor-kind regression. The
  focused test failed because the canonical manifest classified
  `PingBuilder.__init__` as `type` while the source audit reports
  `python-export`.

### Green

- Commit `80270c39e` corrected the canonical manifest and regenerated its
  outputs: C `kmipkit_ping_builder_t` is a `type`; Java and Python
  `PingBuilder` classes are `type`; Python `PingBuilder.__init__` is a
  `python-export`; Java execute overload signatures and `executeWithOptions`
  are declared; and the Ping capability mapping includes the ready exports.
  The runtime capability query now exposes `fluent_ping_builder`, and the
  direct C Ping function enters through the shared FFI panic boundary.
- `python3 -B -m unittest tools.api_manifest.tests.test_0093_ping_builder_manifest
  -v` passes all seven tests. `python3 -B tools/api_manifest/generate.py
  --check` verifies all 12 outputs. The complete binding audit passes with
  exact inventories: C 2,661/2,661, Java 3,086/3,086, Python 3,350/3,350;
  there are no missing declarations, kind mismatches, or unobserved entries.
- `cargo test --workspace --all-features`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo fmt --all --check`, and `git diff --check` pass. The C consumer
  rebuilds and CTest passes 4/4. The Python binding suite passes 1,384 tests
  after placing pytest's temporary wheel-build directory on the disk-backed
  task temp volume.

### Refactor

- Kept the new runtime capability assertion in its own integration test so
  the pre-existing capability test remains below the Clippy line-count limit.
  Regenerated Java, Python, and shared parity projections from the canonical
  manifest rather than editing generated output manually.
- `bash gradlew test --no-daemon` passes the full Java suite: 731 tests, zero
  failures, one skipped. The first local attempts exposed `/tmp` exhaustion;
  after removing only this turn's failed pytest scratch and rerunning without
  `JAVA_TOOL_OPTIONS` (which polluted subprocess stderr), the full suite passes.
  Exact-head hosted CI remains required before this draft can be marked ready
  for human review.
