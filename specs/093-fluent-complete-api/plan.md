# KMIPKIT-0093 Implementation Plan

**Branch**: `feature/KMIPKIT-0093-fluent-master-contract`
**Source**: [spec.md](spec.md)
**Status**: Contract and dependency plan for the complete fluent API.

## Architecture

The existing Rust `ClientRequest` and operation models are the sole
protocol-owning execution path. A reviewed declarative catalog drives named
values and four-language API generation; thin operation-specific wrappers
compose existing typed requests. An owned response tree backs explicit
TTLV/JSON/XML views. C, Java, and Python use Rust validation and transport
through the existing ABI. Server-initiated operations remain in the listener
and handler path. No dynamic caller-implemented transport or generic message
send is added to the high-level client.

## Implementation order

1. Audit the pinned v2.0, v2.1, and v3.0 catalog against the supplied master
   list. Record assigned, reserved, version, direction, and actual route for
   each operation and value. Reuse `specification/catalog/` metadata where it
   already provides the required fields.
2. Add generation input and deterministic generator for missing enum, tag,
   attribute and operation metadata. Keep aliases for every existing public
   symbol; update the public API manifest via its generator.
3. Extend the owned response view with checked transcoding and an exact
   operation/result adapter. Keep copies short lived and document foreign
   runtime limitations.
4. Add expert attribute placement to existing typed request construction.
   Validate the final request before the private encoder and reject invalid
   or version-incompatible placements.
5. Add Rust request and response wrappers by operation family and the
   listener response bridge. Use Red, Green, Refactor commits for each family.
6. Add C ABI, Java, and Python surfaces in that order, with source and ABI
   compatibility checks and generated export audits in each slice.
7. Add five real fixture journeys per language, then complete all required
   CI, coverage, traceability, and documentation gates.

Each slice has its own Spec Kit implementation detail, exact normative clause
mapping, acceptance tests, and `[WIP]` PR. The next slice branches from the
prior PR head until that parent is merged. A human merges in dependency order.

## Risk controls

- Generate only assigned source values, including the seven 3.0 additions.
- Keep response fields absent when the wire did not supply them.
- Treat rendered bodies as sensitive; never include them in diagnostics.
- Validate after expert insertion and before transport.
- Preserve existing request/response ownership and no-retry semantics.
- Distinguish a source-supported operation, a client route, and server support.
