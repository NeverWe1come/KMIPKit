# Research: KMIP 2.1 Typed Client Execution

**Date**: 2026-10-05
**Status**: Draft research for KMIPKIT-0007.

## Local source evidence

- OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213 defines Discover Versions. The request may repeat Protocol Version items and the client-supported versions are ranked in decreasing preference order. The response contains the server-supported versions in decreasing preference order. When the request supplies its supported versions, the server returns only versions supported by both parties. Table 213 includes `Operation Not Supported` among Discover Versions errors.
- `specification/catalog/kmip-2.1.json` maps the operation to `KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS` and `KMIPKIT-ELEM-OP-S2C-DISCOVER-VERSIONS`; request requirements include `KMIPKIT-REQ-SPEC-6.1.16-001-001/-002` and `KMIPKIT-REQ-SPEC-6.1.16-004`.
- ADR-0002 fixes the KMIPKit 1.0 boundary to KMIP 2.1. `KMIPKIT-DISC-022` records the scope conflict with §9.16; the feature must not claim same-major backward-compatible acceptance.
- The approved API architecture says Discover Versions is an explicit operation and not a hidden prerequisite. See `docs/architecture/public-api.md`.
- `docs/roadmap.md` assigns paired response checks, mixed Pending handling, outbound async/batch-option validation, response size, and 2.1 enforcement to KMIPKIT-0007. The set of likely-large operations is explicitly left to the later operation inventory.
- Existing reusable delivery state and transport errors are in `crates/kmipkit-transport`; client errors in `crates/kmipkit-client`; generic TTLV ordered/zeroizing value ownership in `crates/kmipkit-ttlv`; result redaction in `crates/kmipkit-protocol`. At draft time, no transport trait, message model, codec, limits type, or production client/writer exists in the workspace.
- The 0006 specification assigns response pairing, mixed async results, outbound option checks, response-size policy, extension criticality, and runtime 2.1 enforcement to 0007. It also states that `KMIPKIT-DISC-001` leaves Continue/Undo effects unresolved.
- At release baseline `ff159a8b3fdb92b6ff6174c1c7e5ad2588dc5138` (checked 2026-10-05), the 0005/0006 specs remain Draft and ADR-0011/0012 remain Proposed. Their approval gates are dependencies, not resolved facts. Verify exact release branch status again before implementation.

## Initial operation choice

Discover Versions is the smallest typed request/response candidate because it does not require a managed object identifier, cryptographic payload, or secret result. It is a public explicit operation, not a hidden handshake. Its use does not establish support for other operations, and the fake transport is sufficient to test request-to-response mechanics before a live server exists. The first request advertises only KMIP 2.1 in accordance with the 1.0 product boundary.

Query has a broader required payload (one or more Query Functions); Get requires a Unique Identifier and may return managed-object/key material. Those are poor first requests for isolating the execute boundary.

## Reviewer decisions, dispositions, and open questions

1. Source review for the fake-only repeated Discover Versions test vehicle: §8.3/Table 396 and §9.7/Table 406 define the required Batch Item structure without an operation-uniqueness rule; §6.1.16/Table 211 states no operation-specific batch restriction. This is only test data for the common client path and is not an interoperability or server-support claim. T001 re-verifies these references before implementation.
2. Audit the proposed hidden workspace transport contract against the supported `kmipkit` facade and direct `kmipkit-transport` crate access. Cross-crate use requires a technically public trait; confirm this does not create a supported generic raw-send route or revise the ownership boundary (`KMIPKIT-0007-OD-003`).
3. Confirm whether initial 0007 should recognize no critical extensions (rejecting all unknown critical extensions) or include a minimal immutable extension registry API; no registry is implemented at draft time.
4. Resolved for this draft: Discover Versions is not classified likely-large, so its typed API omits peer-visible Maximum Response Size; local caps always apply. Later operation specs assess likely-large responses and define/test peer-size behavior where appropriate.
5. Reconcile 0006's “monotonic-timer request Date” assignment and timestamp model with the optional Time Stamp and countdown-timer allowance in pinned OASIS §9.20; do not infer wall-clock semantics (`KMIPKIT-0007-OD-004`).
6. Define and independently review a repeatable Rust-AST CI audit for the single execute-owned permit mint and writer callsite. Rust privacy alone is module-scoped, not function-scoped; do not describe the one-callsite rule as compile-time enforced without the audit (`KMIPKIT-0007-OD-005`).

## Normative/source conflicts retained

- `KMIPKIT-DISC-001`, OASIS §11.5: do not infer Continue/Undo execution and rollback effects.
- §9.6 says Batch Error Continuation SHALL have one of three values, while Table 435 lists an extension range. The current catalog has no separate disposition for this value-range conflict. Track it as `KMIPKIT-0007-OD-001`; do not silently select an interpretation.
- `KMIPKIT-DISC-022`, OASIS §9.16: ADR-0002 product scope requires exact 2.1 even though §9.16 states same-major backward compatibility.
- Unknown extension-range Asynchronous Indicator values have no known async permission semantics absent a registered definition. Preserve/validate their wire range but do not treat them as permission for Pending.
- OASIS §9.8 defines Batch Order Option as a server execution-order constraint. Client response association remains ID-based regardless of response ordering; no response order guarantee is inferred from this option.
- Since KMIPKit 1.0 advertises only `(2,1)` in Discover Versions, a successful response's version list is limited to the empty list or the intersection `{(2,1)}`. A non-offered version violates §6.1.16 server response semantics and is rejected.
- Local inventory records no linked OASIS Test Case IDs for the listed client execution requirements; derived tests cite source clauses/tables and are not represented as official OASIS vectors.
