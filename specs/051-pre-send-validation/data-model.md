# KMIPKIT-0051 Validation Data Model

## Validation Schema

An explicit identity for one KMIP 2.1 Structure or operation Request Payload. Each identity has an ordered list of known fields and the OASIS source rules for TTLV type, requiredness, repeatability, and locally decidable conditions. Schema identity is supplied by a typed request or explicitly by a caller for a generic `Structure`.

## Validation Issue

An immutable, value-free diagnostic with:

- a stable issue code;
- a structure-relative path, composed only from schema field names/tag identifiers and optional repeated-item indices;
- severity (`error`, `warning`, or `info` as justified by the public contract);
- exact OASIS KMIP 2.1 section/table reference when normative, or a stable KMIPKit project-rule reference for API-only checks.

It never contains the invalid value, encoded item bytes, a request body, or a source `Error` whose formatting could disclose them.

## Validation Report

A deterministic collection of issues for one request/schema validation pass. `is_valid()` is false when at least one error-severity issue exists. An explicitly requested but unsupported schema is an error, so callers cannot mistake an unvalidated schema for a valid one. Reports do not mutate the source request or TTLV tree. The client blocks only on error-severity issues that are locally provable.

## Request Validation Contract

Typed operation request models and `ClientBatch` expose borrowed validation. The client uses the same checks during execution before encoding and transport. Validation is distinct from server operation results and from profile-conformance evaluation.

## Delivery Evidence

Existing `RequestDeliveryState` remains unchanged: `NotSent`, `PossiblySent`, or `ResponseStarted`. Validation failures before transport use `NotSent`; once transport may have begun, existing transport evidence remains authoritative.
