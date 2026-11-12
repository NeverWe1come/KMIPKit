# Task 3 implementation report

## Scope and status

Worktree: `/home/ramp1953/.codex/worktrees/fluent-journeys/KMIPKit`.
Branch: `feature/KMIPKIT-0107-fluent-journeys`.
Owner: approved KMIPKIT-0093 FR-011/FR-013, F001.3; KMIP 2.1 sections
6.1.2 (Create Key Pair), 6.1.55 (Sign), and 6.1.56 (Signature Verify).

The four consumers execute Create Key Pair -> Sign -> Signature Verify over
the existing TLS 1.3 mutual-authentication listener. They explicitly select
ECDSA/256, private SIGN and public VERIFY usage, SHA-256, and ECDSA-with-SHA-256.
The listener decodes the parameters, operation, key identifiers, payload and
signature; the consumers inspect real encoded TTLV responses and valid results.
The new binding facades assemble existing typed Rust requests; they add no
local cryptographic implementation, retries or implicit algorithm selection.

Red commit: `93bf72e9523390667ad4985d5227ff99f02adac4`.
Green and Refactor commits: recorded after the verified implementation below.

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

## Self-review

The C input record uses fixed-width fields, explicit v1 metadata and reserved
bytes. Optional masks are not inferred. Attribute construction and validation
remain Rust-owned. Consumers keep the responses alive while using borrowed C
identifier/signature spans, release owned native handles, and inspect successful
typed outcomes. The fixture's connection count and required successful exit
prevent a consumer-only success from hiding missing or rejected requests.
The partial manifest mapping retains unresolved API behavior as blocked and
does not claim the five-journey requirement complete: F001.4/F001.5 remain open.

Refactor review and final exact local evidence follow after Green is committed.
