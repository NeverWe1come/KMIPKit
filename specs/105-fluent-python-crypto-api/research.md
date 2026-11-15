# KMIPKIT-0105 Research

## Decisions

- **Reuse the C/Rust builder ABI.** The KMIPKIT-0103 ABI already provides
  operation constructors, setters, batch conversion, execution, and release
  for all seven KMIP 2.1 operations. Reusing it keeps validation and protocol
  ownership in Rust and avoids a second codec or transport path.
- **Use Python classes with operation-specific methods.** The existing
  `kmipkit.fluent.Request` namespace and `Client.request()` entry point are
  established by KMIPKIT-0091; subclasses/mixins can preserve snake_case
  conventions and restrict each setter surface.
- **Reuse `BatchResult`.** The Python client's existing result wrapper retains
  KMIP status, typed operation response, correlation, and transport delivery
  semantics. This slice must not create a second response model.
- **Clear only KMIPKit-created mutable staging buffers.** CFFI's synchronous
  ABI call copies bytes into Rust-owned storage. Python immutable bytes and
  caller-owned objects cannot be deterministically erased by KMIPKit.
- **Keep exchange options and Request Header fields distinct.** The existing
  `RequestOptions` record controls the exchange; asynchronous indicator is a
  KMIP request-header setting on the one-item batch.

## Alternatives considered

- Reimplement requests through Python's generic batch constructors: rejected
  because it duplicates operation assembly and weakens parity with the
  approved opaque builder ABI.
- Add new C ABI functions for Python: rejected because KMIPKIT-0103 already
  supplies the complete required ABI and no ABI expansion is necessary.
- Return only raw TTLV or custom response objects: rejected because it would
  bypass the existing typed `BatchResult` path.

## Resolved uncertainties

There are no unresolved technology or protocol questions for this slice. The
operation-field matrix is checked against the KMIPKIT-0102 builders and
OASIS v2.1 tables during implementation.
