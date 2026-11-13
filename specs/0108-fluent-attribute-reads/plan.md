# KMIPKIT-0108 Implementation Plan

1. Add failing focused client tests for builder constructors, ordered
   references, omitted identifiers, duplicate validation, typed batch items,
   and controlled exchanges.
2. Implement `GetAttributesBuilder` and `GetAttributeListBuilder` using the
   existing protocol request types, `execute_fluent_request`, and typed
   response accessors. Do not modify protocol parsing or transport.
3. Add English and Spanish guide examples, stable requirement traceability,
   public API manifest declarations, and generated parity outputs.
4. Run focused tests, all `kmipkit-client` tests, formatting, Clippy,
   manifest/export checks, and applicable repository gates. Record Red,
   Green, and Refactor in separate commits.
5. Open a `[WIP]` draft PR from this branch, fix any red CI on its exact head,
   and mark it ready only when every required check is green.
