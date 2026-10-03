# Vendor extension architecture

KMIP standardizes extension mechanisms and namespaces. A vendor normally owns
the meaning of its extension. KMIPKit supports the standard mechanism without
building vendor policy into the core.

## Version 1.0 behavior

- Create, send, receive, inspect, and preserve vendor extensions.
- Preserve unknown tags, enum values, bits, fields, and order.
- Query extension lists and maps explicitly.
- Enforce KMIP message extension criticality.
- Reject an unknown critical extension as required by KMIP.
- Continue processing permitted non-critical content.
- Offer a generic TTLV escape hatch.
- Avoid bundled vendor-specific semantic knowledge.

## Extension registry

Registries belong to an individual immutable client configuration. There is no
global mutable registry. Registration identifies:

- Vendor identifier.
- Extension name and version.
- Compatible KMIP and KMIPKit versions.
- Tags, types, enums, attributes, and operations.
- Structural and semantic validation.
- Conversion between extension types and generic TTLV.
- Documentation and conformance fixtures.

Duplicate identifiers and incompatible definitions fail configuration.
Registration order cannot change encoding or validation outcomes.

## Future SDK levels

### Declarative extension

A data-only manifest describes the schema and rules. It is validated and loaded
at runtime by the common core and therefore works through every adapter. It
cannot execute arbitrary code.

### Compiled extension

A Rust crate implements approved extension hooks for behavior that cannot be
expressed declaratively. SDK tooling generates repetitive C, Java, and Python
surfaces. It is compiled into a KMIPKit distribution.

Dynamic executable native plugins are deferred. They introduce process-level
trust, ABI, signing, loading, and platform problems that require a separate
security design.

## Isolation rules

- Extensions cannot access transport internals or TLS secrets.
- Core TTLV framing and resource limits always apply.
- Extension validation cannot disable core security invariants.
- Extension panics are contained at public boundaries.
- Logs follow core redaction rules.
- Vendor packages are versioned independently and declare compatibility.

## Governance

Vendors normally publish independent packages. KMIPKit supplies an SDK,
templates, contract tests, documentation, and a catalog of known packages.
Frequently used extensions may live under the KMIPKit organization while
remaining separate packages and releases.
