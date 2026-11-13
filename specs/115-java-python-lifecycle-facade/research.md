# KMIPKIT-0115 Research

## Decisions

- Reuse Java `ClientBatchItem` constructors and Python `extensions.client_batch_item_*`
  constructors for Activate, Archive, Locate, Recover, and Revoke. Both bindings
  already expose those typed batch-item models and typed response accessors.
- Implement lifecycle builders in each existing fluent request namespace and
  delegate `execute` to the existing client. This preserves one exchange,
  Request Options, asynchronous header handling, delivery state, and redaction.
- Implement direct convenience methods as thin wrappers over those builders.
  This prevents a second request serializer or transport path.
- Keep Locate filters explicit and optional. Require the caller's existing
  typed Attribute collection; preserve an empty collection as a valid explicit
  selection.
- Snapshot Revoke reason values in the builder and create fresh owned native
  reason/request handles per build. This keeps builders reusable while
  respecting the consuming native constructors.
- Retain raw-preserving unsigned reason codes and identifiers; do not collapse
  unknown protocol values into a closed enum.

## Evidence

- Java exposes typed constructors `ClientBatchItem.activate`, `.archive`,
  `.locate`, `.recover`, and `.revoke`, plus typed response accessors.
- Python exposes matching `extensions.client_batch_item_*` constructors and
  typed response accessors.
- Existing `RequestOptions` and batch asynchronous indicator APIs already
  separate exchange-local options from Request Header fields.
- Existing Rust fluent builders in `crates/kmipkit-client/src/fluent.rs` own
  validation and request semantics for these operations.

## Alternatives Considered

- Adding new native C ABI operation-builder types was rejected because the
  existing bindings already expose the typed request item constructors; a new
  ABI route would duplicate semantics and expand ownership obligations.
- Requiring callers to manually construct `ClientBatch` was rejected because
  it fails the task-oriented facade goal.
- Implicit reason, identifier, or Locate-filter values were rejected because
  the approved API contract requires explicit inputs and exact omission.
