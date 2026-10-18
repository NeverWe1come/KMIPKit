# T038 Implementer Report

## Status

The Get and Locate four-language mapping is implemented in the existing T076 worktree. T038 remains unchecked pending final-head CI and controller review. PR #104 remains a draft and has not been merged.

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

Windows Rust/JNI builds emitted the existing `aws-lc-sys` MSVC probe and missing-NASM warnings; builds and focused tests succeeded. The mapping does not claim complete OASIS conformance, server behavior, interoperability, profile support, or certification.

## Reviewer correction: Locate identifier-count preservation

The Java and Python adapters no longer impose the decoder's default 100,000-element limit on Locate response identifiers. Java rejects counts above `Integer.MAX_VALUE`, its array representation limit; Python rejects counts above `sys.maxsize`, its sequence representation limit. Both preserve all accepted identifiers in order, including repeats. The original T038 Red/Green/Refactor commits above are unchanged; this focused correction uses separate Red and Green commits:

- Red: `07a211a2bdc73a5d11b9786b504941e7cab69d9e` — added Java and Python tests for 100,001 repeated identifiers.
- Green: `9d2c3a4e191b045b9f88ef8e9def6b03b7588573` — replaced the binding-only 100,000 cap with each language's representation bound.

### Correction verification

- Red, Java (`cwd: bindings/java`): `mvn -Dtest=GetLocateResponseMappingTest test` — expected failure in the new test at the former Java 100,000 cap; 7 tests ran, 1 error.
- Red, Python (`cwd: bindings/python`): `$env:UV_PROJECT_ENVIRONMENT='.venv-t038-fix'; $env:KMIPKIT_NATIVE_LIBRARY='C:\Users\ramp1953\.codex\worktrees\0043-t076-client-mapping\KMIPKit\target\debug\kmipkit_ffi.dll'; uv run --offline --python 'C:\Users\ramp1953\AppData\Roaming\uv\python\cpython-3.12-windows-x86_64-none\python.exe' --with pytest python -m pytest -q tests/test_get_locate_operation.py -k beyond_default_decoder_limit` — expected failure with `ResourceLimitError` from the former cap.
- Green, Java (`cwd: bindings/java`): `mvn -Dtest=GetLocateResponseMappingTest test` — PASS, 7 tests, 0 failures/errors.
- Green, Python (`cwd: bindings/python`): `$env:UV_PROJECT_ENVIRONMENT='.venv-t038-fix'; $env:KMIPKIT_NATIVE_LIBRARY='C:\Users\ramp1953\.codex\worktrees\0043-t076-client-mapping\KMIPKit\target\debug\kmipkit_ffi.dll'; uv run --offline --python 'C:\Users\ramp1953\AppData\Roaming\uv\python\cpython-3.12-windows-x86_64-none\python.exe' --with pytest python -m pytest -q tests/test_get_locate_operation.py` — PASS, 5 tests.
- Green: `git diff --check` — PASS.

T038 remains unchecked pending final-head CI and controller review. PR #104 remains a draft.

## Workspace verification continuation

The focused lint correction replaces three redundant closures in the FFI test
with direct method references; it changes no runtime behavior. Initial
workspace Clippy failed on those three test expressions. After the correction,
the following checks passed on the current implementation tree:

- `cargo fmt --all --check` — PASS.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS.
- `cargo test --workspace --all-features` — PASS.
- `cargo llvm-cov --workspace --all-features` — PASS; Rust workspace line
  coverage was 92.55 percent, above the 90 percent workspace minimum.
- `python -B tools/api_manifest/generate.py --check` — PASS; all 12 generated
  outputs are current.
- `git diff --check` — PASS.

These local checks do not substitute for the selected-scope 95 percent changed
code gate or cross-platform checks on the final pushed head. Those remain
dependent on PR CI after the continuation commits are pushed.
