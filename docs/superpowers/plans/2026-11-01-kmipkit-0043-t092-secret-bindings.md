# KMIPKit T092 Secret Binding Plan

> **For agentic workers:** Execute this plan inline in the active `feature/KMIPKIT-0043-t076-shared-client-mapping` worktree. Keep Red, Green, and Refactor evidence in separate signed commits.

**Goal:** Complete KMIPKIT-0043 T092 parity for Rust `SecretBytes` and `SecretText` across C, Java, and Python. KMIPKIT-0043 owns the bindings; their secret lifecycle behavior follows the approved KMIPKIT-0008 contract.

**Architecture:** Rust remains the owner of the zeroizing secret allocation. C receives opaque handles and callback-scoped borrowed spans; Java and Python receive temporary copies for callbacks and clear mutable copies when their runtime permits. Converting a secret to TTLV moves its allocation to the existing zeroizing `kmipkit-ttlv::Value`; no credential send path is added.

**Tech Stack:** Rust 2024, C ABI, JNI/C++, Java 17, Python 3.12/CFFI, API manifest generator, cargo tests, Gradle, pytest.

**Spec:** `specs/043-public-api-bindings/spec.md` FR-004/005/010/013/017/021/022 owns and approves this binding work. `specs/008-credentials-attestation/spec.md` FR-010 and its secret ownership model provide the behavior contract; KMIPKIT-0008 explicitly excludes language bindings. OASIS KMIP 2.1 §9.11 Tables 410–416 and §11.11 Table 442 provide the credential context, while the Secret wrapper lifecycle itself is the approved KMIPKit owner contract.

**Manifest ownership:** T092 in KMIPKIT-0043 owns the cross-language mapping requirements and implementation. The semantic capability remains owned by KMIPKIT-0008, following the existing credential manifest pattern, so secret wrapper declarations use `ownerSpecId: KMIPKIT-0008` and approval evidence from `specs/008-credentials-attestation/spec.md`. They trace KMIPKIT-0043's parity requirements as well. The root re-export maps to the same foreign API as its `kmipkit_protocol` source declaration.

## Global Constraints

- Preserve Rust `SecretBytes::new`, `with_bytes`, and `into_ttlv_value`, and `SecretText::new`, `with_str`, and `into_ttlv_value` semantics.
- Redact secret content from formatting, errors, and logs; zeroize every KMIPKit-owned current allocation.
- C callback pointers are borrowed only for the synchronous callback duration; no registry lock is held, the handle owner remains alive through callback return, and callback code must not unwind or nonlocally jump across the ABI. Foreign code can deliberately copy a pointer, outside KMIPKit's zeroization guarantee.
- Java byte/text callbacks receive temporary `byte[]`/`char[]` copies that are cleared in `finally`; never expose a native-backed `ByteBuffer`. Python byte callbacks receive a read-only view over a temporary mutable buffer that is released and cleared in `finally`; text callbacks receive a `str` whose runtime copy cannot be cleared.
- Validate explicit lengths with checked `u64`-to-`usize` conversion, reject lengths above `isize::MAX`, and use fallible reservation before copying. Do not introduce a separate KMIP payload size limit for these generic secret owners.
- Preserve strict UTF-8 text with no normalization, reject unpaired Java/Python surrogates, and define null-pointer/zero-length behavior, handle ownership, and close/release behavior.
- Use only the `kmipkit-ffi` unsafe boundary, document each `SAFETY` block, and add no credential selection or transmission path.
- Update the manifest input and regenerate all derived outputs with the pinned generator; never edit generated files by hand.

---

### Task 1: Red contract for exact four-language mapping

**Files:**
- Modify: `tools/api_manifest/tests/test_t092_credentials_parity_manifest.py`
- Modify: `crates/kmipkit-ffi/tests/c_api_coverage.rs`
- Create: `bindings/java/src/test/java/org/kmipkit/credentials/SecretCredentialFacadeTest.java`
- Create: `bindings/python/tests/test_credential_secrets.py`
- Modify: `bindings/c/tests/credential_abi_consumer.c`

**Interfaces:** Assert exactly four Rust declarations per wrapper and four-language parity. Define observable behavior for redacted formatting, valid UTF-8 text, exact byte/text callback contents, callback-scoped access, TTLV value type/content, rejection of invalid UTF-8 and invalid/null inputs, closed-handle behavior, and temporary-copy cleanup after callbacks.

- [ ] Add manifest tests for the eight protocol-crate Rust exports (`SecretBytes`, `SecretText`, and their six inherent methods) plus the four blocked root `kmipkit::SecretBytes` type/method reexports; require the approved owner IDs and exact C/Java/Python mappings for both package paths.
- [ ] Add compiled C cases for empty/binary/embedded-NUL inputs, null/length combinations, strict UTF-8 rejection, source-buffer preservation, reentrant release, callback-only access, TTLV conversion, and redacted diagnostics.
- [ ] Add Java tests for Unicode and strict surrogate rejection, scoped `char[]`/`byte[]` callbacks, closed state, and temporary-array clearing after normal and exceptional callback completion.
- [ ] Add Python tests for Unicode and surrogate rejection, scoped bytes `memoryview`, closed state, redacted `repr`, mutable-buffer clearing, and exception re-raise after callback cleanup.
- [ ] Run the focused manifest and consumer tests and record expected Red failures before implementation.

### Task 2: Green C ABI and native ownership

**Files:**
- Modify: `crates/kmipkit-ffi/src/lib.rs`
- Modify: `crates/kmipkit-ffi/src/extension_registry.rs`
- Create: `crates/kmipkit-ffi/src/extension_registry/credential_secrets.rs`
- Modify: `bindings/c/include/kmipkit.h`
- Modify: `bindings/c/tests/credential_abi_consumer.c`
- Modify: `crates/kmipkit-ffi/tests/c_api_coverage.rs`

- [ ] Add opaque secret-handle types, `Kind` and `HandleValue` entries, constructors, callback-scoped access, consuming TTLV conversion, and release functions in the focused credential-secret module.
- [ ] Check `u64` lengths against `usize` and `isize::MAX`, reserve fallibly before copying, reject invalid pointer/length pairs and UTF-8 without formatting input, and zeroize owned input bytes on every failed conversion.
- [ ] Clone the handle owner before callback dispatch so the registry lock is released; keep its `Arc` alive through callback return and document that callback code must not retain the pointer or unwind/nonlocally jump across the ABI.
- [ ] Run `cargo test -p kmipkit-ffi --test c_api_coverage secret_ --all-features`; verify C ABI cases compile and execute.

### Task 3: Green Java and Python facades

**Files:**
- Modify: `bindings/java/src/main/java/org/kmipkit/credentials/CredentialInterop.java`
- Create: `bindings/java/src/main/java/org/kmipkit/credentials/SecretBytes.java`
- Create: `bindings/java/src/main/java/org/kmipkit/credentials/SecretText.java`
- Modify: `bindings/java/native/kmipkit_jni.cpp`
- Create: `bindings/java/src/test/java/org/kmipkit/credentials/SecretCredentialFacadeTest.java`
- Modify: `bindings/python/src/kmipkit/credentials.py`
- Create: `bindings/python/tests/test_credential_secrets.py`

- [ ] Add `AutoCloseable` Java facades with explicit UTF-8/byte input conversion and callback-scoped temporary arrays; clear mutable arrays in `finally`.
- [ ] Add context-managed Python facades using CFFI handles; encode text as UTF-8 and clear temporary mutable buffers in `finally` where possible.
- [ ] Preserve the documented limits: Java/Python immutable strings, caller copies, and callback-created copies are outside native zeroization guarantees.
- [ ] Run the focused Gradle tests and Python secret tests against the built native library.

### Task 4: Refactor, manifest, documentation, and review

**Files:**
- Modify: `specification/api/public-api.json`
- Regenerate: files listed in `generatedOutputs` by `tools/api_manifest/generate.py`
- Modify: `specs/043-public-api-bindings/tasks.md`
- Modify: `specs/043-public-api-bindings/traceability.md`
- Modify: `specs/043-public-api-bindings/implementation-evidence.md`
- Modify: `docs/architecture/public-api.md`
- Modify: `docs/architecture/ffi-and-bindings.md`
- Modify: `docs/user-guide/en/credential-sessions.md`
- Modify: `docs/user-guide/es/sesiones-credenciales.md`

- [ ] Refactor only after C, Java, and Python behavior tests pass; remove duplicated boundary code without widening secret access.
- [ ] Map each declaration to the reviewed equivalent API, regenerate outputs, and run `python tools/api_manifest/generate.py --check` plus Rust and binding export audits.
- [ ] Record the C callback lifetime and Java/Python copy limitations in both English and Spanish documentation.
- [ ] Run focused tests, `cargo fmt --all --check`, FFI Clippy, and `git diff --check`; preserve the active temporary coverage waiver.
- [ ] Obtain independent security and QA rechecks before updating the draft PR; do not mark T092 complete until all 43 blocked declarations are mapped or specifically excluded.
