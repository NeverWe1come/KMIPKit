# T038 Implementer Report

## Status

The Get and Locate four-language mapping is implemented in the existing T076 worktree. T038 remains unchecked pending controller review. No push, merge, or PR-state change was made.

## TDD commits

- Red: `801f058f4608f7329ac771688cb2b17f4777a355` — added failing manifest, C, Java, Python, and FFI mapping tests.
- Green: `ba9eff84a88565400e9d4d6b4a0303e32343e95a` — implemented Rust FFI accessors, C/Java/JNI/Python/CFFI mappings, manifest records, and pinned generated outputs.
- Refactor: `f84026fa77a3150538ffd527f161b0c6a55b338d` — formatted the FFI tests and recorded traceability and dependency readiness.

## Focused verification

- Red: `python -B -m unittest tools.api_manifest.tests.test_get_locate_parity_manifest -v` — expected failure, 3 tests with 7 failures and 2 errors because verified mappings and generated C/parity entries were absent.
- Green: `python -B tools/api_manifest/generate.py --write` — PASS, generated 12 files; `python -B tools/api_manifest/generate.py --check` — PASS, all 12 outputs current.
- Green: `python -B -m unittest tools.api_manifest.tests.test_get_locate_parity_manifest -v` — PASS, 3 tests.
- Green: `cargo test -p kmipkit-ffi --all-features --lib` — PASS, 137 tests.
- Green: `cargo test -p kmipkit-ffi --all-features --test c_api_coverage generated_public_client_abi_consumer_compiles_and_runs -- --exact` — PASS, C consumer compiled and ran (1 test).
- Green: `mvn -f bindings/java/pom.xml -Dtest=GetLocateResponseMappingTest test` — PASS, 6 tests, 0 failures/errors; JNI built.
- Green: `$env:PYTHONPATH=(Resolve-Path 'bindings/python/src').Path; $env:KMIPKIT_NATIVE_LIBRARY=(Resolve-Path 'target/debug/kmipkit_ffi.dll').Path; uv run --isolated --with pytest --with cffi --python 3.12 --no-project python -c "import pytest,sys; raise SystemExit(pytest.main(['bindings/python/tests/test_get_locate_operation.py','-q']))"` — PASS, 4 tests.
- Refactor: `cargo fmt --all --check`, `python -B tools/api_manifest/generate.py --check`, `python -B -m unittest tools.api_manifest.tests.test_get_locate_parity_manifest -v`, and `git diff --check` — PASS.

## Concerns

Windows Rust/JNI builds emitted the existing `aws-lc-sys` MSVC probe and missing-NASM warnings; builds and focused tests succeeded. Full workspace tests, selected coverage gates, and current-head CI were not run in this continuation. The mapping does not claim complete OASIS conformance, server behavior, interoperability, profile support, or certification.
