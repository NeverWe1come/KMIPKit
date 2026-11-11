# KMIPKIT-0103 Implementation Evidence

This delivery is approved for implementation under the maintainer's direct
instruction to complete the fluent API plan. The implementation and CI evidence
will be added here as each Red, Green, and Refactor commit is produced.

## Baseline

- Branch: `feature/KMIPKIT-0103-fluent-c-crypto-api`.
- Parent: `feature/KMIPKIT-0102-fluent-crypto-operations` at
  `e578c1fe027b0232e789d336bad2c72128f71d1a`.
- Normative operation shapes are owned by the approved KMIPKIT-0102 spec.

## Verification

- **Red**: `cargo test -p kmipkit-ffi --features coverage-c-consumer --test
  c_api_coverage public_c_fluent_crypto_builders_expose_semantic_opaque_constructors
  --locked` failed as expected while compiling the public C consumer: the public
  header does not yet declare `kmipkit_crypto_operation_builder_t` or the seven
  operation-specific constructors and common builder functions. No production
  ABI code existed at this point.

- **Red (asynchronous indicator validation)**: the first probe mistakenly used
  raw value `2` as invalid and exposed an implementation guard that also
  rejected the assigned `Optional` value. The corrected test uses raw value `0`
  (outside the assigned and extension ranges); the focused command then failed
  with probe status 4 because the setter accepted it. KMIP Asynchronous
  Indicator is an Enumeration, not a boolean; the Green implementation must
  accept the assigned Mandatory/Optional/Prohibited values and extension range.

- **Red (TLS execution acceptance)**: `cargo test -p kmipkit-ffi --features
  coverage-c-consumer --test c_api_coverage
  public_c_fluent_crypto_executes_all_seven_operations_over_mutual_tls --locked`
  failed at link time because the compiled C consumer did not yet provide
  `kmipkit_public_crypto_execute_tls_probe`. The test now defines the seven
  one-operation TLS exchanges and typed Failure metadata it requires.

- **Red (configured protocol-version preflight)**: extended the TLS consumer to
  execute an Encrypt builder using client configuration 99.99 before the seven
  KMIP 2.1 exchanges, and require `KMIPKIT_ERROR_INVALID_INPUT` with no
  additional connection. Before the fix, the peer received this extra Encrypt
  exchange and then observed the next Encrypt where it expected Decrypt. The
  failure traced to `kmipkit_client_create_mtls_v1`, which copied the wire
  encoding but dropped the configured protocol version.

## Green

- **Asynchronous indicator validation**: the focused C-consumer command passes.
  It rejects 0, accepts KMIP Mandatory/Optional/Prohibited values, and retains
  an Enumeration extension value. The C setter therefore follows the existing
  client validation contract instead of treating the field as a boolean.
- **Seven operation TLS execution**: `cargo test -p kmipkit-ffi --features
  coverage-c-consumer --test c_api_coverage
  public_c_fluent_crypto_executes_all_seven_operations_over_mutual_tls --locked`
  passes. The compiled C consumer executes one Encrypt, Decrypt, Sign,
  Signature Verify, MAC, MAC Verify, and Hash request over separate controlled
  TLS 1.3 mutual-TLS exchanges. The peer verifies one Batch Item per exchange
  and the operation identity. The returned outcomes cover Pending (Encrypt),
  Success (Hash), and Failure (the remaining operations); the C assertions
  inspect result metadata and the Pending correlation bytes. Encrypt's optional
  asynchronous request-header value and exchange-local options are set through
  separate inputs. The Pending fixture includes the required correlation value
  and response payload.
- **Configured protocol-version preflight**: the same focused TLS command
  passes after `kmipkit_client_create_mtls_v1` preserves the configured
  protocol version. Version 99.99 returns `KMIPKIT_ERROR_INVALID_INPUT` before
  opening a connection, and the fixture receives exactly the seven subsequent
  KMIP 2.1 requests.
- **Byte ownership and builder validation**: the TLS consumer overwrites its
  source bytes after calling the C setters; the peer confirms every transmitted
  Byte String retained the original input. A repeated Data setter sends only
  the final value. The compiled C probe also rejects a Hash request without
  Cryptographic Parameters, wrong-operation setters, null pointers, invalid
  multipart flags, and invalid asynchronous indicators while preserving output
  handles.

## Refactor

- The first strict Clippy run found `build_request` exceeded the repository's
  100-line limit. The request-part preparation, seven operation-specific typed
  constructors, and common request validation are now separate helpers. Clippy
  then identified two unused receivers and duplicate fixture match arms; those
  were converted to associated functions and merged. The final command
  `cargo clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings`
  passes. The first full FFI suite also found the static panic-boundary audit
  did not scan the new submodule. The audit now includes it, and exported C
  wrappers enter `ffi_entrypoint` directly before calling internal helpers.
  `cargo test -p kmipkit-ffi --features coverage-c-consumer --locked` passes:
  471 unit tests, 55 C API coverage tests, and the 56-operation parity test.
