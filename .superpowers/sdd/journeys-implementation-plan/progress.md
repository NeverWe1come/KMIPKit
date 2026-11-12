# SDD ledger — KMIPKIT-0107 fluent journeys

- Parent PR: #193, exact head dc9819f610d9f34322f34e3b0b4fbfdbe8f5f776.
- Requirement: approved KMIPKIT-0093 FR-013 / F001.
- Branch: feature/KMIPKIT-0107-fluent-journeys.
- Pre-flight audit: five end-to-end journeys are incomplete across the four languages; existing tests are per-operation or legacy-facade tests. See the read-only FR-013 audit in the parent task.
- Tasks 1–6 are dependency ordered. Task 1 establishes the fixture contract and end-to-end lifecycle semantics consumed by tasks 2–5.
- Constraints: do not modify generated output manually. The maintainer's direct authorization to implement KMIPKIT-0093 covers a missing binding facade when it prevents an approved journey; record the gap and close it through its typed API and manifest before the fixture uses it.

## Task 2: AES-GCM journey

- Red command: `sh tests/consumers/0093-journeys/verify.sh`.
- Red result: failed on request 5 (`index=4`): fixture expected Encrypt (`0x1f`) but received Get (`0x0a`); the current C lifecycle consumer exits with status 9. The four consumers currently omit the AES-GCM create/encrypt/decrypt/destroy sequence.
- Green implementation: all four consumers now create an AES-256 key, send Encrypt and Decrypt with explicit AES-GCM parameters (96-bit IV, 128-bit tag, explicit AAD), compare the encoded fixture response with the same known-answer vector, and destroy the key.
- Fixture contract: seven ordered TLS 1.3/mTLS connections per language; it decodes and validates operation, identifier, key algorithm/usage, request data, IV, AAD, mode, IV length, and tag length. Encrypt and Decrypt return actual encoded TTLV responses; consumers assert ciphertext, tag, IV, and recovered plaintext.
- C failure found during Green: the new journey had not initialized the versioned Unique Identifier output records before calling the typed response accessor. Initialized both records according to the public ABI contract; the C TLS journey then passed.
- Runner hardening: `verify.sh` and `verify.ps1` now wait for the responder and require a successful exit, so a consumer cannot pass if the fixture rejected an exchange or failed to receive the full expected sequence.
- Verification: `sh tests/consumers/0093-journeys/verify.sh` completed successfully on Linux. It built and ran all four consumers against the controlled listener, verified all 28 requests and the listener's successful exit; the C consumer was compiled with `-Wall -Wextra -Werror`. `git diff --check` passed.
- Refactor: centralized initialization of ABI-versioned Unique Identifier output records in the C consumer to prevent another uninitialized-record call. The full four-language mTLS journey passed again after this cleanup.
- Status: Task 2 Red, Green, and Refactor complete in separate commits.

## Task 3: Key-pair signing and verification journey

- Red command: `sh tests/consumers/0093-journeys/verify.sh`.
- Red result: the Rust and C lifecycle/AES-GCM journeys passed, then the controlled listener rejected connection 7 because it expected Create Key Pair (`0x02`) but received the C lifecycle Create (`0x01`). The fixture now requires Create Key Pair, Sign (`0x21`), and Signature Verify (`0x22`) after the first two journeys in each language.
- Contract: use ECDSA P-256 with explicit SIGN and VERIFY usage masks; set SHA-256 and ECDSA-with-SHA-256 in Cryptographic Parameters; Sign must use the private ID and Verify the public ID returned by Create Key Pair. The fixture returns the signature as encoded TTLV and a positive validity result.
- Scope audit: the Rust fluent Create Key Pair builder exists, but the C, Java, and Python fluent request facades currently do not expose a semantic Create Key Pair constructor. The Green implementation must close that gap before using the operation in the four-language journey.
- Green implementation: Rust uses the existing fluent Create Key Pair builder; C now has semantic one-exchange Create Key Pair functions over a versioned input record; Java and Python expose `createKeyPair()` / `create_key_pair()` builders with typed algorithm, length and separate private/public usage masks. All four consumers then Sign and Signature Verify using explicit SHA-256 and ECDSA-with-SHA-256 parameters.
- Fixture contract: ten ordered TLS 1.3/mTLS requests per language, 40 total. It checks ECDSA/256, SIGN on the private key, VERIFY on the public key, the identifiers returned by Create Key Pair, the signed bytes, signature and signature-verification parameters. It returns encoded TTLV responses containing the key identifiers, fixture signature and valid indicator.
- Verification: `sh tests/consumers/0093-journeys/verify.sh` completed on Linux. Rust, C, Java and Python each completed Create Key Pair → Sign → Signature Verify after their lifecycle and AES-GCM journeys; the responder validated all 40 requests and exited successfully. C compiled with `-Wall -Wextra -Werror`.
- Manifest: the API source now declares the C v1 input record and direct functions, the Java/Python operation builders, and the completed Rust/C/Java/Python mappings. `python3 tools/api_manifest/generate.py --check` and the full C/Java/Python export audit passed.
- Contract regression checks: preserve the exact 12 journey-supported Rust declarations and the remaining 34 blocked paths; increase the approved declaration snapshot from 16,359 to 16,396 for the 37 added binding declarations. Local build mappings use the existing C batch-item constructor rather than an execution function.
- Fresh verification: the 40-exchange journey passed again; Python fluent tests passed (27), Java facade tests passed (10), focused Python 3.12 manifest/fixture contracts passed (29), and FFI ABI validation tests passed (2). FFI Clippy with all features, including test targets, passed.
- Refactor: Python attribute groups now use scoped ownership, with two injected-allocation regression cases that failed before the cleanup and pass afterward. The full Python fluent suite passes 29 tests; the 40-exchange journey passes again. The manifest describes the actual staged-options execution signatures; 30 focused Python 3.12 contracts pass.
- Audit snapshot: regenerated the dated key-pair journey binding report through the audit tool; all four current documentation/report contracts pass. A separately reproduced 0053 candidate-exclusion contract fails identically on the original Red manifest and the current tree; its unrelated test/policy discrepancy is recorded in the report.
- Status: Task 3 Red, Green and Refactor are complete in separate commits. Commands, logs, scope limits and self-review are recorded in `task-3-report.md`; exact-head CI remains outside this local task.

## Task 3 review fix round 1

- Confirmed missing explicit curve and unclosed Java crypto builders.
- Red: the tightened fixture rejects the current Rust key-pair request for
  missing Cryptographic Domain Parameters / Recommended Curve P-256.
- Evidence and final verification are recorded in `task-3-report.md`.
- Green: all four consumers now send typed Cryptographic Domain Parameters
  containing the catalogued Recommended Curve P-256. The C v1 input remains
  28 bytes; a new direct adapter accepts an existing Attribute Set. Java/Python
  expose typed common-attribute setters. Java Sign/Signature Verify builders
  close through try-with-resources.
- Fresh results: the 40-exchange journey, Python 30, Java 15, FFI 3, manifest 13,
  fixture 2 and documentation 4 tests pass. Generation verifies 12 outputs;
  the refreshed export audit and all-feature FFI/test Clippy pass.
- Red: `f0bd6c7431a328a9a5ad99a6ff2ea7eebd1f0a9d`; Green:
  `bd338ac06b57bdef2164dc9e158f2d218a9b47a1`.
- Refactor centralizes duplicate Java signature parameter construction and
  closes its parameter builder. The 40-exchange journey and 15 Java tests pass
  again. Both review findings are addressed; the separate Refactor commit and
  exact evidence are available in the report for independent review.
