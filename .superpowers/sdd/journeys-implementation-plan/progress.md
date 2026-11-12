# SDD ledger — KMIPKIT-0107 fluent journeys

- Parent PR: #193, exact head dc9819f610d9f34322f34e3b0b4fbfdbe8f5f776.
- Requirement: approved KMIPKIT-0093 FR-013 / F001.
- Branch: feature/KMIPKIT-0107-fluent-journeys.
- Pre-flight audit: five end-to-end journeys are incomplete across the four languages; existing tests are per-operation or legacy-facade tests. See the read-only FR-013 audit in the parent task.
- Tasks 1–6 are dependency ordered. Task 1 establishes the fixture contract and end-to-end lifecycle semantics consumed by tasks 2–5.
- Constraints: do not modify generated output manually; no production behavior change except if an approved fluent contract gap is found, in which case report it instead of expanding this task.

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
- Status: Red complete; Green in progress.
