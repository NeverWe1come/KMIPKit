# KMIPKIT-0096 FR-003 review follow-up

Base: `7c4518ff0`, after the parent rebased T006 onto KMIPKIT-0095
`5791bcaddc60062d69310d37ad396e97ed30b69c`.

The parent authorized closing the shared delivery-metadata gap in the two
native symmetric-key insertion/build endpoints and their JNI bridges, with
no new ABI. Other argument errors and other native operations must preserve
their existing behavior.

## Red

```sh
CARGO_BUILD_JOBS=2 cargo test -p kmipkit-ffi --all-features --locked --test c_api_coverage public_c_fluent_builder_inserts_explicit_attributes_at_a_final_position -- --exact
bash bindings/java/gradlew -p bindings/java --offline --console=plain --no-daemon '-Dorg.gradle.jvmargs=-Xmx2g -XX:MaxMetaspaceSize=768m' test -x buildNativeLibrary --tests org.kmipkit.fluent.FluentFacadeShapeTest
```

C: exit 101; one failed, 55 filtered out. The compiled C consumer returns 2
when the existing out-of-range build error still reports delivery `NONE`.
The consumer also checks duplicate positions, rejected insertion, null
output, preserved typed attribute mismatch codes, and a subsequent unrelated
generic ABI error retaining delivery `NONE`.

Java: exit 1; nine tests, four failed because build errors are
`InvalidInputException` and catalogued-type insertion errors are
`TypedAttributeValueTypeMismatchException`, whereas the required
`ClientOperationException` must expose `NOT_SENT` and the original native
status/category/cause. Tests also require no connection to a real listening
socket and preserve argument/non-builder exception behavior.

The first Java compile attempt exhausted the default 512 MiB daemon heap
before executing tests. Repeating with the documented 2 GiB JVM setting ran
the expected Red tests. The generated `java_pid10703.hprof` was deleted.
For Red only, the baseline 0095 JNI binary was copied into the local ignored
Java target directory and `buildNativeLibrary` was skipped. Green rebuilds
the current JNI/Rust sources through the repository script.

The repository `target` is temporarily linked to the prior stack's Cargo
target cache to avoid duplicating build outputs; Cargo recompiles this
worktree's source. This symlink will be removed after verification without
removing the shared target directory.

## Green

The two C entry points now classify only their own validation failures as
`NOT_SENT`, retaining the original status/category/cause. JNI's insertion and
typed-build bridges use the existing client-operation exception conversion;
generic argument checks and all other JNI conversion paths are unchanged.
There is no new ABI, native symbol, exception class, or manifest declaration.

```sh
CARGO_BUILD_JOBS=2 cargo test -p kmipkit-ffi --all-features --locked --test c_api_coverage public_c_fluent_builder_inserts_explicit_attributes_at_a_final_position -- --exact
CARGO_BUILD_JOBS=2 bash bindings/java/native/build.sh
bash bindings/java/gradlew -p bindings/java --offline --console=plain --no-daemon '-Dorg.gradle.jvmargs=-Xmx2g -XX:MaxMetaspaceSize=768m' test --tests org.kmipkit.fluent.FluentFacadeShapeTest
PYTHONPATH=bindings/python/src KMIPKIT_NATIVE_LIBRARY=/home/ramp1953/.codex/worktrees/fluent-python-attributes/KMIPKit-fluent-dx-worktree/target/debug/libkmipkit_ffi.so /tmp/kmipkit-0095-py312/bin/python -m pytest -q bindings/python/tests/test_fluent_expert_attributes.py bindings/python/tests/test_fluent.py --tb=short
```

All exited 0. Compiled C: 1 passed, 55 filtered out. Java/JNI: nine tests,
zero failures, with native library rebuilt from this worktree. Python:
46 passed. The Python override ensures the newly built shared library is
used; only the generated CFFI declarations are reused from the prior stack.
Normal native builds emit two pre-existing default-feature `dead_code`
warnings in `kmipkit-client::execute` and aws-lc build diagnostics; there are
no newly introduced warnings.

## Refactor and final verification

Red: `86716c1ce`. Green: `92c5000b2`.

Consolidated native classification in a private helper called inside the
existing `ffi_entrypoint` boundary. Removed the provisional Python-only
normalization; Python now consumes the shared native metadata unchanged.
The first full FFI run caught an adapter arrangement outside the required
first panic boundary; moving the helper inside corrected it. The existing
metadata/panic-boundary test was retained unchanged.

Added a focused Rust ABI test for corrupt/empty owners and an isolated
poisoned lock. It verifies invalid-input delivery is `NOT_SENT` while an
internal lock failure retains its original internal category/cause and
delivery `NONE`.

Final test commands (the explicit TMPDIR is on the workspace filesystem):

```sh
TMPDIR=/home/ramp1953/.codex/worktrees/fluent-python-attributes/KMIPKit-fluent-dx-worktree/target/tmp-0096-fr003 CARGO_BUILD_JOBS=2 cargo test -p kmipkit-ffi --all-features --locked --quiet
TMPDIR=/home/ramp1953/.codex/worktrees/fluent-python-attributes/KMIPKit-fluent-dx-worktree/target/tmp-0096-fr003 CARGO_BUILD_JOBS=1 cargo clippy -p kmipkit-ffi --all-targets --all-features --locked -- -D warnings
LLVM_PROFILE_FILE=/tmp/KMIPKIT-0096-fr003-jni/profraw/%m-%p.profraw bash bindings/java/gradlew -p bindings/java --offline --console=plain --no-daemon '-Dorg.gradle.jvmargs=-Xmx2g -XX:MaxMetaspaceSize=768m' test -x buildNativeLibrary
TMPDIR=/home/ramp1953/.codex/worktrees/fluent-python-attributes/KMIPKit-fluent-dx-worktree/target/tmp-0096-fr003 PYTHONPATH=bindings/python/src KMIPKIT_NATIVE_LIBRARY=/home/ramp1953/.codex/worktrees/fluent-python-attributes/KMIPKit-fluent-dx-worktree/target/debug/libkmipkit_ffi.so COVERAGE_FILE=/tmp/KMIPKIT-0096-fr003-python.coverage /tmp/kmipkit-0095-py312/bin/python -m coverage run --rcfile=bindings/python/pyproject.toml -m pytest -q bindings/python/tests --tb=short
python3 -B tools/api_manifest/generate.py --check
python3 -B tools/api_manifest/audit_jni_public_exports.py --binary bindings/java/target/native/libkmipkit_jni.so
```

All exited 0. FFI: 474 unit tests, 56 compiled C tests, the 56-operation
parity integration test and doc tests pass. Java: 708 tests across 108 suites,
one skipped, no failures/errors. Python: 1,344 tests, 119 subtests pass.
Python aggregate coverage is 96.80%, and `fluent.py` is 94.86%.
Clippy passes with warnings denied. All 12 generated outputs remain current.
The JNI public-export check confirms both required symbols.

`cargo fmt --all --check`, targeted `rustfmt --edition 2024 --check` for
both changed Rust files, and `git diff --check` pass on the final source.
`python3 -B tools/normative_catalog/validate.py --structural-only` passes:
4 sources, 1,411 clauses and 4,143 records.

Full binding inventory was checked with the repository's
`audit_binding_exports.run(Path.cwd(), c_binary=Path.cwd() / 'target/debug/libkmipkit_ffi.so')`:
C, Java and Python are complete and match the manifest with zero issues.
Exact declaration matches are respectively 1,998, 2,390 and 2,597; observed
exports are respectively 2,058, 3,090 and 3,606. The JSON artifact is
`/tmp/KMIPKIT-0096-fr003-binding-export-audit.json`. No generated output or
manifest declaration changed in this follow-up.

### Focused changed-line coverage

Rust LLVM coverage was collected with `cargo llvm-cov --no-clean` for the
compiled C positioned-attribute probe and the new internal-metadata unit
test. JNI was built with the pinned LLVM 20.1.8 tools using the repository
build script (`CXX=clang++-20 KMIPKIT_JNI_COVERAGE=llvm`), then the nine focused
Java tests were rerun with profiles on the workspace filesystem. Profiles
were merged with `llvm-profdata-20` and exported with `llvm-cov-20`.

```sh
CARGO_BUILD_JOBS=2 cargo llvm-cov --no-clean -p kmipkit-ffi --all-features --locked --test c_api_coverage --json --output-path /tmp/KMIPKIT-0096-fr003-ffi-coverage.json -- public_c_fluent_builder_inserts_explicit_attributes_at_a_final_position --exact
TMPDIR=/home/ramp1953/.codex/worktrees/fluent-python-attributes/KMIPKit-fluent-dx-worktree/target/tmp-0096-fr003 CARGO_BUILD_JOBS=2 cargo llvm-cov --no-clean -p kmipkit-ffi --all-features --locked --lib --json --output-path /tmp/KMIPKIT-0096-fr003-ffi-coverage.json -- ffi_symmetric_builder_validation_delivery_preserves_internal_failure_metadata
python3 /tmp/KMIPKIT-0096-fr003-coverage-summary.py
```

The local summary projects the LLVM exports onto exact current-worktree
source paths (excluding older cache paths), normalizes external macro/header
filenames through `scripts/coverage_gate.py`, and intersects its executable
line counters with `git diff 7c4518ff0 --unified=0` using that script's
production-line and executable-line filters. It combines Rust/JNI counters
with coverage.py's Python counters. Results: Rust 38/39, JNI 13/15, Python
2/2; aggregate 53/56 (94.64%). The uncovered lines are the defensive
u64-to-usize overflow branch on this 64-bit host and the JNI success-with-null
owner guard. This is focused changed-line evidence; complete platform and
component coverage gates remain required in hosted CI.

### Resolved environment issue and cleanup

One full Python run passed 1,343 tests but failed its wheel build when the
3.8 GiB `/tmp` tmpfs filled. A concurrent C rebuild failed for the same
reason, and the JNI profiles written at JVM exit were incomplete. Only the
wheel-build target from this task's failed pytest directory was moved to
`target/tmp-0096-fr003/failed-wheel-attempt`. Repeating the complete Python
and FFI suites with TMPDIR on the workspace filesystem passed. JNI profiles
were recollected successfully there. No test was skipped to work around it.

`java_pid10703.hprof` is removed. Temporary `target` and `_native_ffi`
symlinks are removed after verification without deleting the shared cache.
Only the expected parent-owned SDD brief/review artifacts remain untracked.
No push, PR, or further rebase was performed. Exact-head CI and independent
review remain with the parent; no implementation blocker remains.
