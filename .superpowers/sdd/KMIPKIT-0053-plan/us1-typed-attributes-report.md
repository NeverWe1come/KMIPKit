# US1 Typed Attribute Implementation Report

Branch: `feature/KMIPKIT-0053-parity-facade`

## Result

Implemented value-level typed direct attributes and typed `AttributeSet` operations in C, Java, and Python, using the existing Rust TTLV and protocol catalog validation. All 62 generated standard attributes support construction from a TTLV value, copied value readback, and item conversion. Readers preserve each source value, including ordered Structure children, unknown child tags, repeated items, and raw open Enumeration values. `UniqueIdentifier` additionally retains its catalogued TextString, Enumeration, and Integer representations.

Typed set APIs now append a validated occurrence, count occurrences, and read an owned occurrence by zero-based position. They preserve order and repeats. Generic insertion and export continue to retain unknown allocated Items. Caller ownership is preserved for failed and successful copy operations. Public diagnostics and wrapper representations omit values.

## Red, Green, Refactor

- Red: `e7f23eec9` (`test(0053): assert typed attribute values and set access`). The focused tests were added before implementation. They failed because the C ABI value and typed-set exports were absent; Java compilation lacked the typed construction/read methods; Python's previously built native extension lacked the new symbols.
- Green: `b489baf4a` (`feat(0053): implement typed attribute parity`). Added the validated Rust C ABI, compiled C consumer routes, Java JNI and owners, Python CFFI wrappers, route evidence, and regenerated public API outputs. Focused suites passed.
- Refactor: this commit extracts language-test value comparison helpers and records the implementation and verification evidence here; behavior is unchanged from Green.

The C consumer was compiled and executed: `build.rs` builds `bindings/c/tests/typed_attribute_parity_consumer.c` as a static library; the focused Rust FFI tests link and invoke its wrappers for construction, readback, and typed set operations. The catalog loop exercises every one of the 62 entries through that compiled consumer.

## Manifest and generated outputs

Added five C ABI routes for value construction/readback and typed set append/count/indexed read. Added the matching Java and Python routes. Fifteen public declarations cover the C, Java, and Python symbols, with Rust model mappings; all routes carry requirement and parity-evidence IDs. `tools/api_manifest/reconcile_0053_api_parity.py --write` reconciled the reviewed routes, and the pinned API generator regenerated the manifest snapshots, C header, Java/Python API inventories, and parity fixture. The normative attribute generator was run in check mode only; its generated catalog sources were not edited.

## Focused verification

- `cargo fmt --all --check` — passed.
- `cargo test -p kmipkit-ttlv --lib` — 54 passed.
- `cargo test -p kmipkit-protocol attribute -- --quiet` — 235 passed; selected integration examples/redaction tests also passed.
- `cargo test -p kmipkit-ffi --lib --features coverage-c-consumer typed_attribute_0053_tests` — 8 passed; includes compiled C consumer execution, all 62 value roundtrips, caller ownership, Structure values, repeated order, open values, and redaction.
- `mvn -q -f bindings/java/pom.xml -Dtest=TypedAttributeParityTest test` — 9 passed; the build rebuilt and loaded the JNI library.
- `uv run --python 3.12 --project bindings/python --with pytest==8.4.2 --with cffi==1.17.1 python -m pytest -q bindings/python/tests/test_typed_attribute_parity.py` — 11 passed, using the rebuilt native library.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0053_api_parity -v` — 15 passed.
- `python -B tools/normative_catalog/generate_attribute_types.py --check` — passed for all five generated catalog files.
- `python -B tools/api_manifest/generate.py --check` — passed for all 12 outputs.
- `git diff --check` — passed.

## Caveats

Rust builds on this Windows host emit non-fatal AWS-LC `stdalign_check.c` and missing NASM warnings. Cargo, Maven/JNI, and maturin complete successfully despite those diagnostics. Full workspace suites and cross-platform runs were outside this focused task group. The worktree also contains pre-existing/unrelated untracked `bindings/java/target/` and `bindings/python/uv.lock`; neither is included in these commits.

## Review round 1

### Changes

- Added additive, stable C statuses 12 (`KMIPKIT_ERROR_TYPED_ATTRIBUTE_TAG_MISMATCH`) and 13 (`KMIPKIT_ERROR_TYPED_ATTRIBUTE_VALUE_TYPE_MISMATCH`). The FFI now distinguishes tag mismatch from catalogued TTLV value-type mismatch while retaining status 4 for unknown/non-catalogued tags and other invalid input.
- Added Java `TypedAttributeTagMismatchException` and `TypedAttributeValueTypeMismatchException` mappings through JNI. Both remain subclasses of `InvalidInputException`; JNI supplies fixed, redacted messages. Added Python error types with matching status IDs that subclass `InvalidInputError`.
- Corrected Rust routes for construction, readback, typed insertion, and typed set iteration. The count and indexed-read adapters now map to the exact `AttributeSet::typed` iterator route (the Rust API used to count occurrences or select `nth`). Marked the exact Rust model and error declarations ready with four-language evidence and synchronized the typed-attribute capability inventory.
- Updated the approved 0053 route input to carry the new error categories and corrected Rust routes, keeping reconciliation repeatable. Added explicit preservation assertions for unknown child tags and their original order within an Application Specific Information Structure.

### Red evidence

- The new manifest regression expected typed Rust routes and two error categories; the pre-fix inventory still pointed to generic constructors and exposed no typed mismatch classifications.
- The new FFI assertions expected distinct statuses 12/13 while the old path returned 4. A fixture initially used the wrong raw Name tag (`0x42000a`); it was corrected to the catalogued Name tag `0x420053` before final verification.
- Python and Java parity tests initially lacked the two typed exception mappings. The first JNI run also exposed that `ThrowNew` requires a `(String)` constructor; the exception classes were updated to the native bridge signature.

### Green verification

- `cargo test -p kmipkit-ffi --lib --features coverage-c-consumer typed_attribute_0053_tests` — 9 passed, including mismatch classification, redaction, Structure child order, 62 catalogued attribute constructions/readbacks, and `compiled_c_consumer_accepts_all_sixty_two_catalogued_attribute_types`. This links and executes the compiled C consumer.
- `cargo test -p kmipkit-protocol attribute -- --quiet` — 235 protocol unit tests, 2 guide examples, and 3 redaction tests passed.
- From `bindings/python`: `uv run --python 3.12 --project . --with pytest==8.4.2 --with cffi==1.17.1 --reinstall-package kmipkit python -m pytest -q tests/test_typed_attribute_parity.py` — 11 passed. Rebuilt the release native library first with `cargo build --release -p kmipkit-ffi`; pytest loads `bindings/python/src/kmipkit/_native_ffi/_native_ffi.dll`, whose SHA-256 matched `target/release/kmipkit_ffi.dll` (`3d00c03e3b043f1a2d4bb645410e8203fe4f07906a9b9611137f6ae616cead7c`).
- `mvn -q -f bindings/java/pom.xml -Dtest=TypedAttributeParityTest test` — 9 passed; Maven rebuilt and loaded the JNI library.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0053_api_parity -v` — 16 passed.
- `python -B tools/api_manifest/reconcile_0053_api_parity.py --write` — reconciled the approved route source (`+0 functions`); `python -B tools/api_manifest/generate.py --write` — generated 12 public API outputs.
- `python -B tools/api_manifest/generate.py --check` — all 12 generated outputs verified.
- `python -B tools/normative_catalog/generate_attribute_types.py --check` — all five catalog outputs verified.
- `cargo fmt --all --check` and `git diff --check` — passed.

The Windows AWS-LC build emits non-fatal `stdalign_check.c` and missing NASM warnings; all focused builds and test suites listed above completed successfully.
