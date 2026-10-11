# T037 Report — Managed object retrieval test evidence

**Status:** T037 task scope complete. The repository-wide coverage gate remains open under T043; this report does not claim a full official OASIS case or profile pass.

## Scope and source integrity

The two pinned Get XML files were checked against `specification/oasis/kmip-2.1/SOURCES.md`:

| Source fixture | Verified SHA-256 |
| --- | --- |
| `TC-PKCS12-1-21.xml` | `7ff0578acce3937f157797938ee894dab2b40f80e8b3be054523eca229f8701d` |
| `TC-PKCS12-2-21.xml` | `aa3420f7ee43697988715d625cc133b9c04d88e81ad4426a044c728826720087` |

`python -B tools/normative_catalog/check_immutable_sources.py --base-sha 931a524acb729fd46ba3d160f99ac113f0395d94 --repo-root .` passed and confirmed that pinned OASIS upstream sources match the task baseline. No upstream source or existing OASIS fixture was modified.

## Selected OASIS Get items

The test-support unit tests parse the pinned XML, correlate request and response messages by `ClientCorrelationValue`, select only the in-scope Get operations, and retain each selected request/response pair as generic TTLV. They cover four pairs total:

| Catalog case | Selected source steps | Executable test | Checks |
| --- | --- | --- | --- |
| `KMIPKIT-TEST-CN01-2-68` / `TC-PKCS12-1-21` | 8 | `pkcs12_case_1_preserves_get_item_at_correlation_step_8` | Required Get request fields, explicit PKCS#12 Key Format Type, response Object Type/UID, complete opaque Any Object clone preservation |
| `KMIPKIT-TEST-CN01-2-69` / `TC-PKCS12-2-21` | 5, 7, 9 | `pkcs12_case_2_preserves_get_items_at_correlation_steps_5_7_and_9` | Per-step request fields, PKCS#12 Key Format Type where supplied, response object kind/UID, complete opaque Any Object clone preservation |

The checks cover 1 and 3 selected pairs respectively. They do not execute the complete OASIS case scripts and are not official-case passes. Source citations are Test Cases v2.1 CN01 §§2.68–2.69 and KMIP Specification v2.1 §6.1.19 Tables 220–222.

The pinned XML contains unresolved symbols for Get identifiers and private-key material. The unit-test-only parser maps `$UNIQUE_IDENTIFIER_1` and `$UNIQUE_IDENTIFIER_2` to stable local text identifiers and maps the unresolved `KeyMaterial` symbol to the opaque byte sentinel `0xA5`. This permits generic-tree shape/value comparison with a zeroizing clone without asserting that the sentinel equals official key bytes. Assertion messages are constant text; source nodes, TTLV values, and key material are never formatted or printed. The added synthetic BigInteger parser test uses only the local value `01020304` and asserts its type and byte length.

## Five derived local vectors

Each unavailable catalog mapping now has one local JSON vector marked `derived-local-not-official`, with `sample_values_synthetic: true`, and one executable test. Only client request evidence is asserted.

| Catalog ID / official label | Local vector | Test and supported evidence |
| --- | --- | --- |
| `KMIPKIT-TEST-PROF-5-12-6-3` / `TL-M-3-21` | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-PROF-5-12-6-3-TL-M-3-21-derived.json` | `derived_profile_tape_library_vector_covers_only_locate_and_get_payloads`; synthetic Locate Application Specific Information and Get UID payloads from Profile §§5.12.2/5.12.6.3 and the approved mapping. No profile-support claim. |
| `KMIPKIT-TEST-CN01-2-62` / `TC-MDO-2-21` | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-62-TC-MDO-2-21-derived.json` | `derived_metadata_only_case_vector_contains_a_locate_item_without_server_results`; empty Attributes Locate envelope only. It does not invent matching or which object the server returns. |
| `KMIPKIT-TEST-CN01-2-63` / `TC-MDO-3-21` | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-63-TC-MDO-3-21-derived.json` | `derived_key_value_location_case_vector_contains_url_locate_criteria`; synthetic URL-form Key Value Location criterion with URI type. |
| `KMIPKIT-TEST-CN01-2-64` / `TC-OFFSET-1-21` | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-64-TC-OFFSET-1-21-derived.json` | `derived_offset_case_vector_contains_locate_offset_input`; synthetic Locate Offset Items value 1 and empty Attributes. |
| `KMIPKIT-TEST-CN01-2-65` / `TC-OFFSET-2-21` | `crates/kmipkit-protocol/tests/fixtures/KMIPKIT-TEST-CN01-2-65-TC-OFFSET-2-21-derived.json` | `derived_symmetric_key_offset_case_vector_contains_locate_offset_input`; synthetic Locate Offset Items value 1; catalog Symmetric Key context only. |

The source relationship for the unavailable cases is limited to the pinned case descriptions and approved catalog/specification mapping named in the task brief. These files are not the missing official XML and provide no evidence of server matching, returned IDs, or Tape Library profile conformance.

## TDD commits and verification

All listed commits are DCO-signed by `NeverWe1come <vgarciaaguilar97@gmail.com>`.

| Phase | Commit | Evidence |
| --- | --- | --- |
| Red | `6755880a` — `test: add red managed-object fixture checks` | `cargo test -p kmipkit-test-support managed_object_fixture_tests -- --nocapture` exited 101 with `0 passed; 7 failed; 19 filtered out`. Exactly five failures reported the not-yet-added derived JSON vectors; the two pinned XML tests stopped at the existing adapter's `UnknownSymbol` for selected Get identifier placeholders. No pre-existing test failed. |
| Green | `cca568f5` — `test: support managed-object retrieval vectors` | Added test-only symbol handling and the five vectors. The same focused command passed `7 passed; 0 failed; 19 filtered out`. |
| Refactor | `594c3f7e` — `refactor(test): simplify opaque item assertions` | Simplified recursive opaque comparison to keep payload values out of assertion diagnostics; focused lint cleanup only, no product behavior/API change. |
| Coverage follow-up | `d575c453` — `test: cover BigInteger fixture parsing` | Added a synthetic `Modulus`/BigInteger parser check and kept test-symbol substitution explicitly limited to ByteString. No artificial failing test was introduced because the existing parser behavior was already correct. |

Final T037 focused command `cargo test -p kmipkit-test-support managed_object_fixture_tests -- --nocapture` passed `8 passed; 0 failed; 19 filtered out`. The standalone synthetic BigInteger check passed `1/1`. Final `cargo test --workspace --all-features` exited 0 with all workspace test targets and doctests passing, including 27 test-support unit tests, 17 test-support fixture integration tests, and 7 test-support transport integration tests.

Other final checks:

- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `python -B tools/normative_catalog/validate.py --repo-root .` — passed: `Catalog valid: sources=4 clauses=1411 records=4032`.
- `python -B tools/normative_catalog/check_immutable_sources.py --base-sha 931a524acb729fd46ba3d160f99ac113f0395d94 --repo-root .` — passed.
- `python -B -m unittest discover -s tools/normative_catalog/tests -v` — passed: 207 tests, 7 skipped. The symlink-creation test was skipped because this Windows host returned WinError 1314 for the required privilege.

## Coverage result and remaining gate

The last successful `cargo llvm-cov --workspace --all-features` run completed, but coverage was below the repository gates: workspace line coverage was **85.47%** (below 90%). In `crates/kmipkit-test-support/src/oasis_crypto_fixtures.rs`, the report showed **86.33% region coverage** (676/783 regions); JSON line summary was 444/485 (91.55%). No pre-T037 coverage profile is available, so these figures cannot be separated into baseline and T037 attribution.

For changed executable lines in the fixture-parser source file, the last successful full-workspace profile covered 42/42 (100%) after aligning source line numbers for the one-line BigInteger call compaction. The BigInteger line was already instrumented as covered by the existing fixture suite; the new synthetic test now directly exercises that branch. The LLVM export did not list `oasis_get_fixture_tests.rs` as a separate source file, so it provides no line denominator for that test module.

Two final attempts to refresh full-workspace coverage exited 101 before producing a new report: all three loopback-bind tests in `kmipkit-test-support/tests/transport_fixtures.rs` failed with Windows socket error 10013 (`PermissionDenied`). The normal full workspace test command passed immediately afterward. Therefore the workspace coverage gate is recorded as **not passed** at the last successful measurement, and the latest coverage rerun as environment-blocked; T043 remains open. No historic test cleanup or coverage exclusion was added.

## Scope boundary

The changes are confined to fixture JSON, the test-support parser/unit tests, task traceability, and this report. They add no production behavior/API, runtime dependency, catalog availability record, generated artifact, or modified OASIS source. No full OASIS case, Tape Library profile, live-server behavior, or KMIP profile support is claimed.
