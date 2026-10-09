# Quickstart: Encrypt and Decrypt

## Preconditions

- Use a KMIP v2.1 server and an explicit Managed Cryptographic Object identifier or a valid in-batch ID Placeholder.
- Choose algorithms, sizes, modes, padding, IVs, and object policies explicitly. KMIPKit only submits requests; it is not a cryptographic provider.
- Supply Cryptographic Parameters when required; include IV Length for known variable-IV modes and Tag Length for GCM.
- For an archived object, issue Recover before Encrypt or Decrypt.
- Keep plaintext, ciphertext, AAD, authentication tags, and returned material out of application logs.

## Single-part calls

Build an EncryptRequest or DecryptRequest with Data and an explicit key identifier. Data accepts the KMIP §7.9 Byte String, Enumeration, or Integer representation; callers normally use Byte String for cryptographic input. A successful response requires Unique Identifier. Response Data is optional and always a Byte String.

Do not expect local encryption/decryption, implicit parameters, implicit retries, or automatic polling. Inspect the returned KMIP result and transport-delivery classification.

## Multipart calls

1. Send the first caller-built request with Init Indicator true, no Correlation Value, and any supplied AAD (plus Decrypt's supplied authentication tag).
2. Read the server Correlation Value from the operation response.
3. Send each subsequent request explicitly with that Correlation Value. A middle part with neither Init nor Final true contains Data.
4. Mark the last request with Final Indicator true and the same Correlation Value.
5. Handle each result, including Pending, yourself. Pending's Asynchronous Correlation Value is distinct from multipart Correlation Value.

The typed client returns a local validation error before transmission for a one-request form setting both Init and Final true while KMIPKIT-DISC-045 remains open. This does not select Data requiredness. If the caller knows an object is archived, issue an explicit Recover call first, then issue Encrypt or Decrypt; KMIPKit does not inspect object state or combine this into an implicit recovery.

## Verification commands

Run focused Rust tests first, then the workspace gates from the repository root: cargo fmt --all --check; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo llvm-cov --workspace --all-features.

The implementation PR must record actual failing Red runs, passing Green runs, Refactor reruns, coverage evidence, and separate results for all 28 in-scope Encrypt/Decrypt request-response pairs extracted from the three pinned XML fixtures. Label these fixture-derived operation-item tests, not complete official-case results. Use deterministic test-only substitutions for `$NOW`, `$UNIQUE_IDENTIFIER_0`, and `$CORRELATION_VALUE`; the fixture adapter must fail on any unrecognized symbol. Derived source-table vectors must also be labeled separately.
