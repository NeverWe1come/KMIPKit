# Public API Manifest Contract: Registry Slice

## Canonical input

The first public API manifest lives at specification/api/public-api.json. It is a reviewed generation input, not generated output. Its format schema and generator are checked in. This feature adds only the registry surface; later approved API specifications extend this manifest and own the complete 1.0 API parity gate.

Each registry entry declares:

- Manifest format version and feature requirement IDs.
- Public type/function identifiers and language mapping.
- Rust and cross-language constructors to validate extension payloads and create a request-use wrapper that requires an explicit Criticality Indicator Boolean.
- Equivalent ExtensionRegistryLimits fields, defaults, hard maxima, and stable limit-error mapping in each adapter, including identity/Extension Information text bytes, discriminator scalar/aggregate bytes, per-rule/aggregate constraint members, payload-index records, and lookup comparisons. Adapters may raise or lower defaults up to the same hard maxima.
- Repeatable attachment of typed wrappers to a ClientBatchItem, preserving caller-selected batch and extension order as permitted by §8.3/Table 396.
- C symbol name, fixed-width parameter/return types, opaque-handle kind, ownership, and release function.
- All variable-length C byte/string inputs use typed pointers with uint64_t byte lengths; constructors reject over-limit lengths before dereferencing, reading, copying, or scanning input, and never use unbounded NUL scans.
- Rust protocol/client type ownership and typed request boundary.
- Java package/class/method, JNI mapping, and error mapping.
- Python module/function, CFFI mapping, and error mapping.
- Redaction, zeroization, and runtime-copy notes.
- Generated source/test/document destinations.

The manifest contains declarations and metadata only. It contains no executable schema predicate or vendor payload.

## Deterministic generation

The repository-pinned generator must:

- Validate known format and type names and reject unknown required fields.
- Emit Rust/C/Java/Python adapter declarations and parity fixture scaffolding deterministically.
- Preserve manifest-specified stable names and order.
- Write only declared generated outputs; reject path traversal or symlink destinations.
- Support check mode that compares expected bytes without changing files.
- Never scrape OASIS pages or modify pinned upstream copies.
- Be invoked in CI; a generated diff fails the job.

Generated files are review artifacts and MUST NOT be hand-edited. Handwritten validation logic and idiomatic facades remain source files.

## Required consumer checks

- Compile and call the C ABI from a real C consumer.
- Run Java 17 JNI tests and Python 3.12 CFFI tests on the supported native library path.
- Use identical definitions, repeated outbound extension fixtures, and inbound values; compare results, error categories, typed inspection, and preserved generic subtrees.
- Verify outbound request bytes contain the expected registered Vendor Identification, explicit Criticality Indicator, and schema-validated Vendor Extension Structure.
- Confirm every symbol begins with kmipkit_, opaque handles have explicit lifecycle, and default diagnostics redact payloads.
