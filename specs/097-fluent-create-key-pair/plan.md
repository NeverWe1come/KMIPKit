# KMIPKIT-0097 Implementation Plan

Implement a public fluent Create Key Pair builder over the already validated
`CreateKeyPairRequest`. Keep the change in `kmipkit-client`; do not create a
parallel request encoder or response model. Build each attribute group with the
existing `AttributeSet` and typed `Attribute<D>` APIs, and send through the
existing `Client::execute_one_typed_request_with_async_indicator` boundary.

## Verification

1. Add the failing Rust unit tests first and record Red in a signed-off commit.
2. Implement `CreateKeyPairBuilder` and `RequestBuilder::create_key_pair()`;
   run focused tests and record Green in a separate signed-off commit.
3. Add Rust docs, English and Spanish usage examples, traceability, and generated
   API manifest entries; regenerate checked-in outputs.
4. Run client tests, Clippy, formatting, traceability, manifest/export checks,
   and the PR CI. Keep `[WIP]` until all required checks are green.
