# KMIPKIT-0095 Implementation Plan

**Source**: [spec.md](spec.md)
**Branch**: `feature/KMIPKIT-0095-fluent-response-views`

## Design

Reuse the validated, ordered TTLV tree owned by `ResponseMessage`. Clone it
through the bounded model API, put the clone under the assigned
`ResponseMessage` root tag, then call the existing private TTLV, JSON, or XML
encoder with the caller's `CodecLimits`. Return only a `Zeroizing<String>`.

The inspection writer is a separate private path with no transport access or
request permit. The source boundary audit explicitly recognizes only its exact
TTLV and text codec call sites; all other request-writing calls retain their
existing single permit and transport path.

The public Rust error is payload-free and covers invalid retained trees,
configured size limits, allocation failure, and encoding failure. Binding
methods will map this model without exposing rendered bodies in diagnostics.

## Verification

1. Commit a compile-failing test that asks the response owner for all three
   representations.
2. Implement the inspection-only renderer and payload-free typed error.
3. Extend the boundary audit narrowly for the response inspection path; verify
   its codec calls remain unreachable from arbitrary public request input.
4. Check manifest outputs, Rust exports, diff, client tests and CI on the exact
   head. Binding ownership tests belong to the later C/Java/Python deliveries.
