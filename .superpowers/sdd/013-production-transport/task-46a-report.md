# T046a Red — request-extension registry provenance

## Scope and traceability

T046a covers KMIPKIT-0013 FR-018 and the US4 configuration-ownership
acceptance scenarios. The public request Message Extension shape is based on
OASIS KMIP Specification v2.1 §9.13, Table 418. The typed operation fixture
uses Discover Versions under §6.1.16, Tables 211–212. The test does not claim
server profile conformance.

## Red evidence

Development commits:

- `f89f1b0 test(client): add production extension provenance red contracts`
- `c18e542 test(client): align provenance acceptance test names`

The focused `execute.rs` AST regression verifies this call order:

1. `validate_request_extension_ownership` runs before `build_request_message`.
2. `build_request_message` runs before `exchange_operation`.
3. Inside `exchange_operation`, the bounded encoder runs before the transport
   adapter exchange.

`cargo test -p kmipkit-client --lib provenance_order_tests::registry_provenance_precedes_request_build_codec_and_adapter_handoff --offline`
passed 1/1. The existing execution path already performed the provenance
check at the required point, so no artificial product failure was introduced.

The two public integration cases construct extension values from separate
immutable registries. The foreign-registry case expects `InvalidInput` and
`NotSent`, checks that no loopback connection was accepted, and checks that
registry identity and extension payload sentinels are absent from Display and
Debug output. The same-registry case executes through the production raw-TLS
adapter, expects a typed response, and checks that the exact expected
Message Extension TTLV sequence appears in the captured request.

`cargo test -p kmipkit-client --test production_client --offline` fails at the
still-absent public production constructor and shared `RequestOptions` export.
This is the expected Red state before T047; neither public integration case
can run until that API is implemented.

## Checks

- Focused provenance ordering regression — passed, 1/1.
- Public production-client integration target — expected Red compile failure
  at the missing constructor and options export.
- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.
