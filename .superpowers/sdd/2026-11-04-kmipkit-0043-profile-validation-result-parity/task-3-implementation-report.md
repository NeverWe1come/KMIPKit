# Task 3: Java profile validation result implementation report

## Changes

- Added `ProfileValidationResult`, which owns/releases the opaque result handle, snapshots ordered profile and requirement IDs into immutable lists, rejects use after close, redacts `toString()`, and forwards transport/request/response validation with optional caller context.
- Added a package-private bridge seam for lifecycle and validation tests. This does not bypass native profile resolution in production.
- Added `ProfileValidationException` carrying the existing `ProfileValidationDetails` with a static redacted message.
- Added `ProfileValidationSelection.validate()` and the manual `NativeExtensionRegistry` declarations/JNI C++ exports for resolution, result ID access, five validators, and release.
- Added tests for the fail-closed Baseline details, immutable ordered snapshots, redacted result text, every validation method, unknown transport rejection before bridge calls, use-after-close, and release when snapshot construction fails.
- `ProfileTransport.java` already existed with closed raw values 1 and 2, so it was left unchanged.

## Red / Green evidence

- **Red commit:** `a652fe2c1` — `test(java): require fail-closed profile validation details`.
- **Red command:** `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=ProfileValidationMappingTest' test`.
- **Red result:** expected test-compilation failure because `ProfileValidationException`, `ProfileValidationResult`, and `ProfileValidationSelection.validate()` were not yet present.
- **Green commit:** `c0b70308b` — `feat(java): expose profile validation results`.
- **Compile:** `mvn --batch-mode --file bindings/java/pom.xml -DskipTests test-compile` — passed with Java release 17.
- **Wrapper tests:** `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=ProfileValidationMappingTest#resultSnapshotsOrderedIdsAndRedactsItsStringRepresentation+resultForwardsEveryValidationAndRejectsUnknownTransportBeforeNativeCall+resultUseAfterCloseFailsWithoutCallingNativeBridgeAgain+resultClosesHandleWhenIdentifierSnapshotFails' surefire:test` — 4 passed, 0 failed, 0 errors.
- **Focused suite and JNI build:** `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=ProfileValidationMappingTest' test` — JNI native build succeeded; 11 tests passed, 0 failed, 0 errors. The Rust build emitted AWS-LC/MSVC and NASM warnings but completed successfully.
- **Whitespace check:** `git diff --check` — passed.
- No separate Refactor commit was warranted after the focused Green run.

## Manual bridge declarations requiring manifest exclusion inventory

The following 11 public native methods were added to `NativeExtensionRegistry.java`. Per coordinator direction, inventory them as `KMIPKIT-0043-FR-008/021` internal profile-validation bridge exclusions; do not expose them as product API:

1. `profileValidationSelectionValidate`
2. `profileValidationResultProfileIdCount`
3. `profileValidationResultProfileIdAt`
4. `profileValidationResultRequirementIdCount`
5. `profileValidationResultRequirementIdAt`
6. `profileValidationResultValidateTransport`
7. `profileValidationResultValidateRequest`
8. `profileValidationResultValidateRequestWithContext`
9. `profileValidationResultValidateResponse`
10. `profileValidationResultValidateResponseWithContext`
11. `profileValidationResultRelease`

Matching JNI C++ exports were added in `bindings/java/native/kmipkit_jni.cpp` and call the fixed `kmipkit_profile_validation_*_v1` C ABI symbols. No native bridge methods remain missing in this Java scope.

## Remaining boundary / unresolved gap

The current catalog has no production-resolvable `client_1_0` selection because each candidate depends on Baseline, whose evidence state remains incomplete. The selection test therefore asserts the real fail-closed exception/details path. Successful result snapshots and method forwarding are covered through the injected Java adapter seam; the Rust FFI/C tests remain responsible for actual successful rule validation. No Baseline eligibility or conformance claim was added.

`ExtensionRegistryApi.java` and other generated outputs were not edited by hand. The coordinator owns manifest exclusion updates and pinned regeneration after the related contract tests are complete.
