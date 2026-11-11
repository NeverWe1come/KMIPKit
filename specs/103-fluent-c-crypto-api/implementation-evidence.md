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

- **Red (asynchronous indicator validation)**: the same focused command compiled
  and ran the consumer, which failed with probe status 4 because
  `kmipkit_crypto_operation_builder_set_asynchronous_indicator(..., 2)` returned
  success. The Request Header asynchronous indicator is boolean; invalid values
  must be rejected before any transport call.

- **Red (TLS execution acceptance)**: `cargo test -p kmipkit-ffi --features
  coverage-c-consumer --test c_api_coverage
  public_c_fluent_crypto_executes_all_seven_operations_over_mutual_tls --locked`
  failed at link time because the compiled C consumer did not yet provide
  `kmipkit_public_crypto_execute_tls_probe`. The test now defines the seven
  one-operation TLS exchanges and typed Failure metadata it requires.
