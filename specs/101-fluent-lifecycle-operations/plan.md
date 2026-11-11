# Implementation Plan: KMIPKIT-0101

## Design

Add one public builder per selected operation in `kmipkit-client/src/fluent.rs`.
Each builder owns the existing typed request model, applies semantic values
through that model's current constructors and setters, and delegates
validation, batch conversion, and execution to existing typed client code.
Share only small internal helpers for common exchange options and execution;
keep operation-specific fields and response outcomes typed.

Use the normative operation inventory and existing request models as the source
of constructor names, wire operation values, field types, and version gates.
Do not infer protocol fields from a numeric range or generate unassigned
operation names.

## Red/Green/Refactor

1. Add focused tests for all eight missing `RequestBuilder` constructors and
   request variants. Run them and record the expected unresolved-API failure.
2. Add the eight builders and the semantic setters already represented by
   their typed protocol request models. Keep execution on `Client`.
3. Add local-validation, batch-conversion, pending-result, version, and
   redaction tests; update the API manifest and traceability.
4. Run the focused and full client suites, format, Clippy, coverage, generator,
   export-audit, and documentation checks. Commit Red, Green, and Refactor
   evidence separately with DCO sign-off.

## Risks and mitigations

- Similar lifecycle requests have different required fields: retain each
  request type and validator rather than flattening them into one generic
  builder.
- Locate and Re-key include nested attributes: reuse their existing types and
  preserve item order.
- Asynchronous operations can return Pending: expose the existing pending
  outcome and correlation without adding polling or retry behavior.
- The current base is stacked on an in-review WIP: open this PR as WIP and
  keep it that way until its own exact-head CI is green.
