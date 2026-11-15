# Feature Specification: Single-Dependency Rust Facade

**Feature Branch**: `feature/KMIPKIT-0046-rust-single-dependency`
**Created**: 2026-11-05
**Status**: Approved — scope authorized by the maintainer's direct instruction to implement KMIPKIT-0046.
**Input**: User-approved KMIPKIT-0046 plan to make the published `kmipkit` crate sufficient as the sole direct KMIPKit dependency for Rust consumers.

## User Scenarios & Testing

### User Story 1 - Use KMIPKit with one Cargo dependency (Priority: P1)

A Rust application developer wants to configure and execute KMIP operations and use all typed operation models and the generic TTLV model without declaring each KMIPKit layer as a direct dependency.

**Independent Test**: Compile an external fixture whose only direct KMIPKit dependency is `kmipkit`, covering client construction, representative typed requests and responses, asynchronous follow-up types, and generic TTLV access.

**Acceptance Scenarios**:

1. Given only `kmipkit` in the consumer's dependencies, when it imports `kmipkit::protocol`, all 56 client-operation `Request` types are addressable.
2. Given only `kmipkit`, when it imports `kmipkit::ttlv`, generic values and bounded decoding types are addressable.
3. Given valid client and TLS/mTLS configuration values, when the consumer constructs `Client`, it needs no direct `kmipkit-client` or `kmipkit-transport` dependency.
4. Given the existing facade API, when this feature is added, existing root paths continue to compile.

## Requirements

### Functional Requirements

- **KMIPKIT-0046-FR-001**: The `kmipkit` crate MUST expose complete public `protocol` and `ttlv` namespaces for its existing typed protocol and generic TTLV crates.
- **KMIPKIT-0046-FR-002**: The facade MUST expose the client configuration and the transport configuration types needed to construct the supported TLS 1.3 mutual-TLS client.
- **KMIPKIT-0046-FR-003**: The facade MUST expose the public client request, response, asynchronous outcome, and error types required to use the existing client API without a direct dependency on another KMIPKit crate.
- **KMIPKIT-0046-FR-004**: The facade MUST NOT expose `kmipkit-transport::Transport::exchange`, its raw transport response, or concrete low-level transport adapters.
- **KMIPKIT-0046-FR-005**: Every newly reachable public Rust path MUST be represented by an exact public declaration or exact approved exclusion in the reviewed API manifest. Existing cross-language capability semantics and stable requirement identifiers MUST be retained.
- **KMIPKIT-0046-FR-006**: The public API change MUST preserve all existing facade paths and MUST NOT change KMIP wire behavior, retry behavior, secret handling, TLS policy, C ABI, Java API, or Python API.

### Normative Traceability

This feature changes Rust package access paths only; it creates no new KMIP protocol behavior and has no new OASIS normative requirements. The API boundary is traced to ADR-0003 (layered crates) and ADR-0006 (three-level API and four-language capability parity), plus KMIPKIT-0043's manifest-v2 export inventory contract.

| Requirement | Source | Verification |
|---|---|---|
| KMIPKIT-0046-FR-001 | ADR-0003; ADR-0006; KMIPKIT-0043 manifest-v2 | External one-dependency fixture; rustdoc export audit |
| KMIPKIT-0046-FR-002–003 | ADR-0006; current public client and transport contracts | External fixture compiles configuration and typed client usage |
| KMIPKIT-0046-FR-004 | ADR-0003; ADR-0014; KMIPKIT-0043 manifest-v2 | Export audit rejects low-level exchange paths from the facade |
| KMIPKIT-0046-FR-005 | KMIPKIT-0043 manifest-v2, exact declaration inventory requirements | Manifest validation, generator check, full Rust export audit |
| KMIPKIT-0046-FR-006 | KMIPKIT-0003 error contract; ADR-0005; ADR-0006; ADR-0012 | Existing workspace tests and API compatibility checks |

## Scope and Constraints

- Public access paths may be added without adding protocol behavior or changing the supported protocol, transport, TLS, or language boundaries.
- `kmipkit::ttlv` exposes the generic model and bounded decoder. There is no public general-purpose generic TTLV encoder; outbound encoding remains limited to the existing typed client path.
- New aliases are packaging paths for existing capabilities. They do not create new Rust-only capabilities or weaken four-language parity requirements.
- Public declarations without a distinct language-level capability require exact, reviewed exclusions; no wildcard or blanket exclusion is allowed.

## Success Criteria

- **SC-001**: An isolated external Cargo fixture compiles with only `kmipkit` as a direct KMIPKit dependency.
- **SC-002**: The fixture type-checks all 56 client operation request types through `kmipkit::protocol` and uses representative client, response, asynchronous, configuration, and TTLV APIs.
- **SC-003**: The Rust export audit finds no unrecorded route and confirms the facade does not expose low-level transport exchange APIs.
- **SC-004**: Manifest generation/checking is deterministic and all existing facade paths remain valid.
