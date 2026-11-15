# T048 Refactor evidence

## Refactor

- Refactor commit: `75f0767` (`refactor(client): isolate production adapter ownership`).
- Extracted adapter selection and validated construction into private
  `ClientTransport::from_configuration`; public `Client::new` owns that private wrapper and keeps
  the client extension configuration separate.
- The typed client exposes no caller-provided `Transport`; the concrete adapters retain their
  private worker ownership. The worker contract allows one in-flight and one queued exchange.

## Verification

- `cargo test -p kmipkit-client --test production_client --offline`: passed 9/9, including raw TLS,
  HTTPS, options variants, extension provenance mismatch/acceptance, and compile-fail transport
  injection.
- `cargo test -p kmipkit-client --lib provenance_order_tests::registry_provenance_precedes_request_build_codec_and_adapter_handoff --offline`:
  passed 1/1.
- `cargo test -p kmipkit-transport --lib worker_serializes_one_in_flight_exchange_and_one_queued_exchange --offline`:
  passed 1/1.
- `cargo clippy -p kmipkit-client -p kmipkit-transport -p kmipkit --all-targets --all-features --offline -- -D warnings`:
  passed.
- `cargo fmt --all --check` and `git diff --check`: passed.
