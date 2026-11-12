# Task 3 implementation report

## Scope and status

Worktree: `/home/ramp1953/.codex/worktrees/fluent-journeys/KMIPKit`.
Branch: `feature/KMIPKIT-0107-fluent-journeys`.
Owner: approved KMIPKIT-0093 FR-011/FR-013, F001.3; KMIP 2.1 sections
6.1.2 (Create Key Pair), 6.1.55 (Sign), and 6.1.56 (Signature Verify).

The four consumers execute Create Key Pair -> Sign -> Signature Verify over
the existing TLS 1.3 mutual-authentication listener. They explicitly select
ECDSA/256 with Recommended Curve P-256, private SIGN and public VERIFY usage,
SHA-256, and ECDSA-with-SHA-256.
The listener decodes the parameters, operation, key identifiers, payload and
signature; the consumers inspect real encoded TTLV responses and valid results.
The new binding facades assemble existing typed Rust requests; they add no
local cryptographic implementation, retries or implicit algorithm selection.

Red commit: `93bf72e9523390667ad4985d5227ff99f02adac4`.
Green commit: `3037a278ee2dd1a3fd52b5906e698d7bc581cf4a`.
Refactor commit: the commit containing the final ownership cleanup and this
report update, titled `refactor(KMIPKIT-0107): scope key-pair attribute ownership`.

## Existing dirty work and implementation review

The initial diff already contained the four journey consumers, a C v1 input
record and two entry points, Java/Python builders, manifest declarations,
generated output, and traceability/ledger updates. These Task 3 changes were
reviewed and retained. The untracked `bindings/python/uv.lock` was present at
handoff and is preserved without staging it.

Additional corrections:

- Local build/to-batch-item mappings now target
  `kmipkit_client_batch_item_create_key_pair`, not the C function that sends a
  request. A focused regression test failed on six incorrect mappings before
  this source-manifest fix, then passed after regeneration.
- The 0097 exact-path contract distinguishes the 12 supported journey paths
  from the 34 still-blocked Rust paths. Names, expert attributes and unimplemented
  binding behavior are not silently promoted.
- The shared declaration snapshot accounts for exactly 37 added binding
  declarations (16,359 -> 16,396).
- The C journey returns failure if an unexpected response shape/result is
  encountered with a successful accessor status, instead of accidentally
  reporting success during cleanup.
- Added invalid ABI-record cases from the compiled C consumer, Rust unit
  coverage for v1 record fields and absent usage groups, and Python/Java
  mandatory-field and builder tests.

## Verification evidence

Commands run from the worktree unless otherwise stated:

1. `sh tests/consumers/0093-journeys/verify.sh`
   - Passed at initial review and again after Green corrections.
   - Rust, C, Java and Python each passed lifecycle, AES-GCM and key-pair/signature
     journeys. The responder accepted all 40 ordered requests and exited 0.
   - C compiled with `-std=c11 -Wall -Wextra -Werror`; invalid ABI requests
     returned `KMIPKIT_ERROR_INVALID_INPUT` without consuming listener exchanges.
   - Logs: `/tmp/kmipkit-task3-journeys.log` and
     `/tmp/kmipkit-task3-journeys-green.log`.
2. `python3 tools/api_manifest/generate.py --write`, followed by
   `python3 tools/api_manifest/generate.py --check`
   - Passed; 12 generated outputs verified. All generated files were produced
     by this pinned repository generator, never edited manually.
   - Logs: `/tmp/kmipkit-task3-generate.log`,
     `/tmp/kmipkit-task3-manifest-check.log`.
3. `python3 tools/api_manifest/audit_binding_exports.py --check`
   - Passed for the C, Java and Python declaration/export surfaces.
   - Log: `/tmp/kmipkit-task3-export-audit.log`.
4. `uv run --python 3.12 --no-project --with jsonschema==4.26.0 python -B -m unittest tools.api_manifest.tests.test_0097_fluent_create_key_pair_manifest tools.api_manifest.tests.test_t102_crypto_operation_error_trait_exclusions tools.api_manifest.tests.test_t102_join_split_key_secret_data_type_mapping tools.api_manifest.tests.test_extension_fixtures -v`
   - 29 passed, including the exact CI extension-fixture contract command's
     17 tests and the relevant declaration/snapshot checks.
   - Log: `/tmp/kmipkit-task3-focused-py312.log`.
5. `uv run --locked --python 3.12 --project tests/consumers/0093-journeys/python --with pytest python -m pytest bindings/python/tests/test_fluent.py -q`
   - 27 passed. Log: `/tmp/kmipkit-task3-python-tests.log`.
6. `mvn -f bindings/java/pom.xml -Dtest=org.kmipkit.fluent.FluentFacadeShapeTest -Dexec.skip=true -Dmaven.compiler.useIncrementalCompilation=false test`
   - 10 passed, no failures/errors/skips.
   - Log: `/tmp/kmipkit-task3-java-tests-rerun.log`.
7. `cargo test -p kmipkit-ffi --lib --all-features key_pair_v1 -- --nocapture`
   - 2 passed, 475 unrelated tests filtered out.
   - Log: `/tmp/kmipkit-task3-ffi-tests.log`.
8. `cargo clippy -p kmipkit-ffi --lib --all-features -- -D warnings`
   and `cargo clippy -p kmipkit-ffi --lib --tests --all-features -- -D warnings`
   - Both passed. Logs: `/tmp/kmipkit-task3-clippy.log`,
     `/tmp/kmipkit-task3-clippy-tests.log`.
9. `cargo fmt --all --check` and `git diff --check`
   - Passed on the reviewed Green tree.

## Failure diagnosis and scope limits

The original Red journey failed when the listener expected the Rust key-pair
journey after AES-GCM and received the next language's lifecycle Create instead.
The implementation closes that missing three-operation sequence in all languages.

Initial local manifest discovery reproduced the stale 0097 blocked-path
expectation. Focused snapshot tests also confirmed the stale declaration total.
The broad `python3 -m unittest discover -s tools/api_manifest/tests` run used
host Python 3.14, reported failures while loading the old snapshots, and was
interrupted at the coordinator's request to concentrate on Task 3. It is not
reported as a passing full suite. The corrected focused tests were rerun under
Python 3.12. This does not establish that the local failures match the remote
Python-contract failures on PR #194/#197; that exact-head reproduction belongs
to the coordinator's separate CI investigation.

The interrupted discovery's failure positions were then resolved to exact
tests and reproduced separately. The C API documentation suite had four stale
report-count subtest failures after the Task 3 binding additions. A new dated
report is generated by the audit tool, and the current architecture link,
audit-guide snapshot and documentation test use it. The earlier dated report
is retained as historical evidence.

One unrelated baseline failure remains:
`test_0053_candidate_exclusion_manifest.CandidateExclusionManifestTests.test_candidate_exclusions_require_an_exact_verified_counterpart`.
It expects `ManifestError` after assigning a missing counterpart to the first
0053 exclusion. That exclusion is `kmipkit_test_client_batch_item_encoded_operation`
and has no `artifactKind`, so the validator does not inspect the injected
counterpart. Applying exactly that mutation to both the original Red manifest
(`93bf72e9523390667ad4985d5227ff99f02adac4`) and the current manifest produces the
same no-exception result; the generator source is unchanged by Task 3. This
pre-existing test/policy discrepancy is left for its owner instead of expanding
the journey task into generator-policy changes. Logs:
`/tmp/kmipkit-task3-extra-contracts-red.log` and
`/tmp/kmipkit-task3-preexisting-candidate-baseline.log`.

The first Java command (`mvn -q -f bindings/java/pom.xml
-Dtest=FluentFacadeShapeTest test`) found no matching tests with the existing
incremental build output. Recompiling the changed class with incremental
compilation disabled and an explicit fully-qualified selector ran all 10 tests
successfully. The four-language fixture also compiled and ran the Java consumer.

A preliminary Rust test assumed that the generic typed attribute layer rejects
negative Integer values. It showed that the existing shared protocol layer
validates TTLV types without that invented range rule. The test was narrowed
to the versioned ABI contract; no new protocol restriction was introduced.

The fixture signature is fixed synthetic wire data. This verifies request and
response handling and returned validity propagation, not ECDSA mathematical
correctness or external-KMS interoperability. Runtime-owned Java/Python byte
copies cannot be guaranteed erased. Full workspace coverage, Windows/macOS,
external KMS tests, and final exact-head CI remain Task 6/PR closeout work.
Default-feature builds emit existing dead-code warnings in
`kmipkit-client/src/execute.rs`; the all-feature FFI Clippy checks pass.

No push, PR metadata change, readiness action or unrelated cleanup was performed.

## Files reviewed and changed

- C ABI implementation: `crates/kmipkit-ffi/src/lib.rs`,
  `crates/kmipkit-ffi/src/extension_registry.rs`,
  `crates/kmipkit-ffi/src/extension_registry/client_abi/fluent.rs`.
- Java: `bindings/java/src/main/java/org/kmipkit/fluent/FluentRequest.java`,
  `bindings/java/src/main/java/org/kmipkit/fluent/CreateKeyPairBuilder.java`,
  `bindings/java/src/test/java/org/kmipkit/fluent/FluentFacadeShapeTest.java`.
- Python: `bindings/python/src/kmipkit/fluent.py`,
  `bindings/python/tests/test_fluent.py`.
- Manifest input and contract tests: `specification/api/public-api.json`,
  `tools/api_manifest/tests/test_0097_fluent_create_key_pair_manifest.py`,
  `tools/api_manifest/tests/t102_manifest_totals.py`.
- Generated: `bindings/c/include/kmipkit.h`,
  `bindings/java/src/main/java/org/kmipkit/generated/PublicApi.java`,
  `bindings/java/src/test/java/org/kmipkit/generated/PublicApiParityFixtures.java`,
  `bindings/python/src/kmipkit/_generated/public_api.py`,
  `crates/kmipkit-ffi/src/public_api_generated.rs`,
  `tests/fixtures/api/generated/kmipkit_internal.h`,
  `tests/fixtures/api/generated/public_api_parity.json`.
- Consumers: `tests/consumers/0093-journeys/c/main.c`,
  `tests/consumers/0093-journeys/java/src/main/java/Consumer.java`,
  `tests/consumers/0093-journeys/python/main.py`,
  `tests/consumers/0093-journeys/rust/src/main.rs`.
- Traceability/evidence: `specs/093-fluent-complete-api/tasks.md`,
  `specs/093-fluent-complete-api/traceability.md`,
  `.superpowers/sdd/journeys-implementation-plan/progress.md`, this report.
- Audit snapshot follow-through: `docs/architecture/ffi-and-bindings.md`,
  `tools/api_manifest/binding-export-audit.md`,
  `tools/api_manifest/tests/test_c_api_docs.py`, and the audit-generated
  `specs/043-public-api-bindings/reports/binding-export-report-2026-11-12-key-pair-journeys.json`.

## Self-review

The C input record uses fixed-width fields, explicit v1 metadata and reserved
bytes. Optional masks are not inferred. Attribute construction and validation
remain Rust-owned. Consumers keep the responses alive while using borrowed C
identifier/signature spans, release owned native handles, and inspect successful
typed outcomes. The fixture's connection count and required successful exit
prevent a consumer-only success from hiding missing or rejected requests.
The partial manifest mapping retains unresolved API behavior as blocked and
does not claim the five-journey requirement complete: F001.4/F001.5 remain open.

## Refactor evidence and final self-review

After Green, two regression cases injected failure while constructing the
second and third Python attribute groups. Both failed because the earlier
group handles remained open. `CreateKeyPairBuilder.build()` now scopes the
limits and each attribute structure with context managers, releasing earlier
groups if a later allocation fails. This also removes the manual cleanup block
without changing the public API or wire request.

- Red: `uv run --locked --python 3.12 --project tests/consumers/0093-journeys/python --with pytest python -m pytest bindings/python/tests/test_fluent.py -q -k releases_earlier_groups`
  failed both injected-allocation cases; 27 unrelated cases were deselected.
  Log: `/tmp/kmipkit-task3-refactor-red.log`.
- Green after cleanup: the full same Python fluent suite passed 29 tests.
  Log: `/tmp/kmipkit-task3-python-tests-refactor.log`.
- The complete four-language TLS journey passed again with all 40 expected
  exchanges and the listener's successful exit after the cleanup.
  Log: `/tmp/kmipkit-task3-journeys-refactor.log`.
- Signature contract review caught an inaccurate manifest description of the
  options execution route: Java/Python take options through `withOptions()` /
  `with_options()`, then call `execute(limits)`; neither has an
  `execute(limits, options)` overload. A regression failed twice, once per
  language, before fixing these two source-manifest parameter lists and
  regenerating. Log: `/tmp/kmipkit-task3-signature-contract-red.log`.
- Final focused Python 3.12 contracts passed 30 tests, including the signature
  regression. Log: `/tmp/kmipkit-task3-focused-refactor.log`.
- Final generated-output and binding-export checks, plus `git diff --check`,
  passed. Logs: `/tmp/kmipkit-task3-manifest-check-refactor.log`,
  `/tmp/kmipkit-task3-export-audit-refactor.log`.
- `python3 tools/api_manifest/audit_binding_exports.py --report specs/043-public-api-bindings/reports/binding-export-report-2026-11-12-key-pair-journeys.json --check`
  regenerated the current checked-in binding snapshot through the tool.
  `uv run --python 3.12 --no-project python -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`
  passed all 4 documentation/report contracts afterward. Logs:
  `/tmp/kmipkit-task3-export-report-generation.log`,
  `/tmp/kmipkit-task3-doc-contracts-final.log`.

The remaining concerns are the explicit scope limits and unrelated baseline
candidate-exclusion test failure above. The pre-existing
untracked Python lockfile is still preserved. Tasks 4, 5 and final PR closeout
were not included in either Task 3 commit.

## Review fix round 1: explicit P-256 and Java crypto ownership

Verified reviewer findings against the approved Task 3 ledger: ECDSA/256 does
not uniquely select P-256, and the Java Sign/Signature Verify builder handles
were not closed by the consumer. Recommended Curve P-256 is already catalogued
(KMIP 2.1 section 11.45, enumeration value 7) and belongs in the typed
Cryptographic Domain Parameters attribute (section 4.14). The existing Rust
common-attribute route supports this exact representation. The C v1 input layout
will remain unchanged.

Red: `sh tests/consumers/0093-journeys/verify.sh` exited 1. After the Rust
lifecycle and AES-GCM exchanges, the listener rejected the Rust Create Key
Pair because the explicit P-256 domain parameters were absent. Log:
`/tmp/kmipkit-task3-fix1-red.log`. The fixture also has two passing focused
contracts that require exact P-256 and reject missing, wrong-type, empty and
secp256k1 domain selections.

### Fix round 1 Green implementation and verification

- Rust uses its existing typed `common_attribute` method with
  `Attribute<CryptographicDomainParameters>` containing the catalogued
  Recommended Curve P-256 Enumeration. No new Rust fluent method is needed.
- Java/Python expose `commonAttribute(TypedAttribute)` /
  `common_attribute(TypedAttribute)`; those inputs are explicitly borrowed until
  build/execute finishes, while each built typed request owns its copy.
- C adds `kmipkit_client_create_key_pair_with_common_attributes_v1`, accepting
  the existing v1 record and an existing typed Attribute Set handle. It copies
  those attributes into the request, validates duplicate singleton attributes,
  and executes once. The original v1 fields, size (28 bytes), functions and
  behavior are unchanged. No implicit P-256 default is applied to other callers.
- All four consumers construct the same typed domain-parameter attribute and
  use the catalogued P-256 constant. The fixture requires its exact nested
  Enumeration encoding, not merely ECDSA plus a 256-bit length.
- Java SignBuilder and SignatureVerifyBuilder are declared as resources before
  their execution/response resources. They therefore close on successful paths
  and while unwinding an execution or response-inspection failure. The existing
  Java builder lifecycle test now also asserts that Sign/Signature Verify reject
  build calls after close; there is no native handle census for the consumer.
- The manifest adds exactly three binding declarations (16,396 -> 16,399),
  promotes the existing canonical Rust common-attribute declaration, and
  regenerates the API output and binding audit snapshot through pinned tools.
  The currently supported canonical key-pair subset has 13 declarations;
  33 other exact paths remain blocked.

Fresh Green commands and results:

1. `sh tests/consumers/0093-journeys/verify.sh` — exit 0; 40 ordered TLS 1.3/mTLS
   requests accepted, including explicit P-256 in Rust/C/Java/Python; C still
   compiles with warnings treated as errors.
   Log: `/tmp/kmipkit-task3-fix1-green.log`.
2. `uv run --locked --python 3.12 --project tests/consumers/0093-journeys/python --with pytest python -m pytest bindings/python/tests/test_fluent.py -q`
   — 30 passed. Log: `/tmp/kmipkit-task3-fix1-python.log`.
3. `mvn -f bindings/java/pom.xml -Dtest=org.kmipkit.fluent.FluentFacadeShapeTest,org.kmipkit.fluent.FluentCryptoApiShapeTest -Dexec.skip=true -Dmaven.compiler.useIncrementalCompilation=false test`
   — 15 passed, no failures/errors/skips.
   Log: `/tmp/kmipkit-task3-fix1-java.log`.
4. `cargo test -p kmipkit-ffi --lib --all-features key_pair_v1 -- --nocapture`
   — 3 passed, including unchanged v1 layout, additional attributes and rejection
   of duplicate singleton attributes. Log: `/tmp/kmipkit-task3-fix1-ffi.log`.
5. `uv run --python 3.12 --no-project --with jsonschema==4.26.0 python -B -m unittest tools.api_manifest.tests.test_0097_fluent_create_key_pair_manifest tools.api_manifest.tests.test_t102_crypto_operation_error_trait_exclusions tools.api_manifest.tests.test_t102_join_split_key_secret_data_type_mapping -v`
   — 13 passed. Log: `/tmp/kmipkit-task3-fix1-contracts.log`.
6. `uv run --python 3.12 --no-project python -B -m unittest discover -s tests/consumers/0093-journeys -p test_local_journey_responder.py -v`
   — 2 passed. Log: `/tmp/kmipkit-task3-fix1-fixture-contracts.log`.
7. `python3 tools/api_manifest/generate.py --write`, then
   `python3 tools/api_manifest/generate.py --check` — 12 outputs verified.
   Logs: `/tmp/kmipkit-task3-fix1-generate.log`,
   `/tmp/kmipkit-task3-fix1-manifest-check.log`.
8. `python3 tools/api_manifest/audit_binding_exports.py --report specs/043-public-api-bindings/reports/binding-export-report-2026-11-12-key-pair-journeys.json --check`
   — exit 0; all inventories complete, with 2,012 C, 2,402 Java and 2,611 Python
   exact manifest matches. Log: `/tmp/kmipkit-task3-fix1-export-report.log`.
9. `uv run --python 3.12 --no-project python -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`
   — 4 passed. Log: `/tmp/kmipkit-task3-fix1-docs.log`.
10. `cargo clippy -p kmipkit-ffi --lib --tests --all-features -- -D warnings`
    — passed. Log: `/tmp/kmipkit-task3-fix1-clippy.log`.
11. `cargo fmt --all --check`,
    `cargo fmt --manifest-path tests/consumers/0093-journeys/rust/Cargo.toml -- --check`
    and `git diff --check` — passed. Log: `/tmp/kmipkit-task3-fix1-fmt.log`.

Red review-fix commit: `f0bd6c7431a328a9a5ad99a6ff2ea7eebd1f0a9d`.
Green review-fix commit: `bd338ac06b57bdef2164dc9e158f2d218a9b47a1`.

Self-review: no invented algorithm/curve code, no C layout break, no curve
inference from key length, and no request retry. Native ownership of all newly
constructed domain-parameter values is scoped; C releases its helper handles,
and Java/Python build requests while borrowed attributes are still alive.
Only the listed worktree is modified, and the unrelated untracked Python
lockfile remains untouched. The prior unrelated candidate-exclusion baseline
failure and cross-platform/coverage scope limits remain as documented above.

### Fix round 1 Refactor and final status

The separate Refactor commit is titled
`refactor(KMIPKIT-0107): share Java signature parameters`. It centralizes the
identical SHA-256 / ECDSA-with-SHA-256 parameter construction in a private Java
consumer helper. The helper closes its parameter builder before returning the
owned parameter object. Sign/SignatureVerify builders remain explicit resources
in their operation scopes. No public API or wire-contract change is introduced
by the Refactor.

- `sh tests/consumers/0093-journeys/verify.sh` passed again, including all 40
  exchanges and the listener's successful exit. Log:
  `/tmp/kmipkit-task3-fix1-refactor.log`.
- `mvn -f bindings/java/pom.xml -Dtest=org.kmipkit.fluent.FluentFacadeShapeTest,org.kmipkit.fluent.FluentCryptoApiShapeTest -Dexec.skip=true -Dmaven.compiler.useIncrementalCompilation=false test`
  passed all 15 tests again. Log: `/tmp/kmipkit-task3-fix1-java-refactor.log`.
- `git diff --check` passed before committing.

Both reviewer findings are addressed. Red, Green and Refactor are separate
review-fix commits. The manifest generator, Java suites and export audit all
completed successfully before the Green commit; the Refactor changes only the
consumer helper and evidence. The worktree is ready for independent review,
with only the pre-existing untracked `bindings/python/uv.lock` left outside the
commits. There was no push, PR metadata change or readiness action.
