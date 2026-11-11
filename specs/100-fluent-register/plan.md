# KMIPKIT-0100 Fluent Register Plan

## Goal

Add a Rust Register builder that is ergonomic for existing typed objects and
remains open to explicitly typed future/vendor objects, while preserving the
current protocol validator and exchange path.

## Design

- Add `RequestBuilder::register() -> RegisterBuilder<'a>`.
- Let callers provide either `ManagedObject` or explicit `ObjectType` and
  `Item`; do not guess object type from unknown values.
- Default Attributes to an empty `AttributeSet`, matching Register Table 291;
  callers can replace or append a prevalidated set.
- Keep Protection Storage Masks and Opaque Object context optional and
  explicit.
- Construct `RegisterRequest` and delegate all shape, cardinality,
  object-type, and Table 293 validation to its existing validator.
- Execute one `ClientRequest::Register` through the existing typed exchange.
- Publish only the Rust surface in this slice; leave C, Java, Python pending.

## Red / Green / Refactor

1. Add unit tests and feature specification first. Run the focused suite and
   record its expected compile failure in the Red commit.
2. Implement the builder and error surface. Run focused, then complete client
   tests and lint in the Green commit.
3. Add bilingual examples, traceability, API manifest records and generated
   artifacts in a separate Refactor commit.
4. Run format, Clippy, client suite, guide examples, manifest/export checks,
   coverage and relevant workflow checks. Open a `[WIP]` PR targeting #181.
   Keep it WIP until all required exact-head checks pass.

## Files

- `crates/kmipkit-client/src/fluent.rs`
- `crates/kmipkit-client/src/lib.rs`
- `crates/kmipkit-client/tests/unit/fluent_register_tests.rs`
- `docs/user-guide/en/lifecycle-and-certificates.md`
- `docs/user-guide/es/operaciones-ciclo-vida-y-certificados.md`
- `specs/100-fluent-register/`
- `specification/api/public-api.json` and generated outputs via the generator

## Risk controls

- Do not change `RegisterRequest` validation or its wire representation.
- Do not add a second transport or arbitrary message-send path.
- Validate via the typed request before batching or sending.
- Retain object and attribute contents only through zeroizing TTLV ownership;
  errors stay payload-free.
- Preserve current version-specific routing, including explicit KMIP 3.0
  validation for newer system objects.
