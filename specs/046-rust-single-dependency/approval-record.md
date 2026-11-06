# KMIPKIT-0046 maintainer authorization record

**Recorded**: 2026-11-05
**Authorization source**: KMIPKit maintainer instruction in this Codex conversation
**Status**: Authorized for the bounded specification and implementation below

## Authorization

The maintainer directly requested implementation of KMIPKIT-0046 and approved the plan specifying complete `kmipkit::protocol` and `kmipkit::ttlv` namespaces, client configuration access, exact manifest reconciliation, and a draft PR based on the KMIPKIT-0043 feature branch. This record captures that direct authorization for this scope. The repository contract still reserves human approval and merge.

## Approved scope and decisions

1. Expose complete typed protocol and generic TTLV namespaces from `kmipkit`, while preserving current facade root paths.
2. Expose only configuration and result types needed to consume the existing supported client API. Keep raw transport exchange, concrete transport adapters, and transport byte-response types out of the facade.
3. Treat aliases as alternate Rust access paths to existing capabilities. Do not add a general-purpose generic TTLV encoder or any new wire, TLS, retry, secret-handling, C ABI, Java, or Python behavior.
4. Reconcile every newly reachable Rust path with exact manifest entries and exact exclusions, preserving four-language capability ownership and existing requirement identifiers.
5. Create and update a draft feature PR. No agent may approve or merge it.
