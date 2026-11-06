# ADR-0018: Shared ownership for raw response-tree views
Status: Accepted
Date: 2026-11-06

## Context

KMIPKit decodes each response message into one ordered, zeroizing TTLV tree and then converts its response batch items into typed values. Typed response models retain different operation-specific subsets of the source tree. A uniform `.raw()` view cannot be reconstructed after the source message is dropped, and copying every payload into every typed model would duplicate potentially sensitive response data.

## Decision

High-level response items privately retain a shared `Arc<ResponseMessage>` plus the response batch-item index. Rust raw accessors borrow a `StructureView` from that owner. The tree remains alive until the final response item or view owner is dropped and is released through the existing zeroizing TTLV owner. C returns a zeroizing owned Structure copy with the existing explicit release API; Java and Python wrap that handle with explicit close semantics.

The shared owner is private and does not promise cross-thread safety beyond the actual auto traits of its contained types. The public raw view is the decoded TTLV tree and does not include discarded wire padding.

## Consequences

- All results from one batch share one response tree rather than cloning each payload.
- Rust views are tied to the response result lifetime.
- Foreign consumers can retain an independent TTLV Structure only through an explicit owned handle.
- Dropping the last response owner zeroizes KMIPKit-owned tree values; copies made by caller code or language runtimes remain outside that guarantee.
- Automatic `Debug`, string conversion, and error output remain redacted.

## Alternatives considered

- Clone each Response Payload into each typed operation model: rejected because it duplicates data across batch items and raises secret-retention cost.
- Keep a callback-only API: rejected because it prevents convenient `StructureView` navigation and uniform cross-language wrappers.
- Reconstruct raw TTLV from typed fields: rejected because it would lose unknown fields, repetitions, and original order.
