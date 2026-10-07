# Contract: Explicit Profile Validation

## Entry and scope

Profile validation is opt-in per exchange through `Client::execute_with_profiles(request, selection)` and an explicit `ProfileValidationSelection`. The existing `Client::execute` path remains unchanged and unprofiled; no production constructor or implicit client-wide selection is added. The product-level selected target is separately recorded and is not implicitly copied into a caller's runtime validation settings. A selected profile must have client role and be applicable to the requested product/transport configuration.

## Required behavior

1. Resolve selected profile IDs and all dependency profiles using generated metadata from the reviewed catalog.
2. Reject duplicate, unknown, missing, cyclic, conditional-but-unsatisfied, server-only, out-of-scope, conflicting, incompletely mapped, or incompletely tested profile selections with a stable redacted validation error containing a stable error code, profile ID, requirement ID, exact source clause ID, and delivery state. Do not include free-form causes or arbitrary source text.
3. Evaluate all applicable requirements in the selected profiles' complete normative clause closure, including every subclause and every clause or source document normatively incorporated by reference. A whole-source inclusion contributes all clauses from that pinned source; applicable client requirements are evaluated within the approved product scope and other role/scope dispositions remain visible. Informative references do not extend the closure. Required operations/data are minimum capabilities, not a closed allowlist; additional valid KMIP operations remain usable unless a mapped clause explicitly forbids them.
4. For generation and reporting, load target IDs only from the fixed, bounded, strict repository-owned target manifest. Reject malformed, oversized, duplicate-key, symlinked, or out-of-scope manifest data before producing output. Runtime selection uses caller-selected profile IDs explicitly and does not consult target membership.
5. Apply only exact normative defaults whose conditions hold. Preserve caller values and never choose cryptographic parameters implicitly.
6. Execute only explicit typed Rust rules keyed by stable catalog requirement IDs. Generated profile records and free-form catalog descriptions are metadata; they are never parsed or evaluated as rules. A missing applicable rule or rule test makes the profile unavailable for runtime selection and fails before transport I/O.
7. Reject a request before transport only when an applicable selected-profile message or transport constraint fails. Validate applicable response behavior after receipt and preserve the underlying request delivery state in any failure.
8. Never issue hidden Query or Discover Versions requests, retry a request, contact OASIS, or parse remote documentation.
9. Return source-backed stable requirement IDs and sections for failures. Errors omit secrets and raw protocol bodies.

## Determinism

The same profile selection, client configuration, request/response values, and generated metadata yield the same validation result. Profile ordering does not resolve conflicting constraints.

## Traceability

Every normative rule used by this contract maps through `KMIPKIT-REQ-*` and `KMIPKIT-CLAUSE-*` records in `specification/catalog/kmip-2.1.json`. The profile record links every requirement in its transitive normative-inclusion closure; omission of any applicable source clause, subclause, or requirement blocks selection and readiness. Baseline Client's Profiles §6.1 item 1 includes the whole KMIP Specification v2.1 source, and item 2 includes Profiles §3.1. Tests cite the exact OASIS source and section. This contract adds no independent OASIS requirement.
