# Specification: Independent Java KMIP Interoperability Consumer

**Branch**: `feature/KMIPKIT-0086-interop-java-live`  
**Status**: Approved for implementation by explicit authorization of the interoperability plan  
**Identifier**: KMIPKIT-0086  
**Base**: release commit `e437b95e48c43493dfca3adccfb1ceb67addda7b`

## Scope

The Java validator is an independent consumer application. It installs a JNI
JAR built from a clean tree at an exact commit of `release/1.0.0` and uses the
public classes in that installed package. The application must not compile or
link `bindings/java` sources from its checkout. Its runtime is Java 17, the
library's minimum supported Java version.

The validator covers KMIP 2.0 and 2.1 standard content only. KMIP 1.x, KMIP
3.0, and vendor extensions are out of scope. The shared catalog and schemas
govern case selection, classification, and results. Each normative test case
must retain its exact OASIS document and section from the catalog, which cites
the OASIS KMIP 2.0 Specification and OASIS KMIP 2.1 Specification.

The Java API exposes typed client operations and generic TTLV construction and
decoding. It does not expose a separate high-level API or direct transmission
of arbitrary TTLV requests. The validator reports these unavailable routes as
API limitations, not server incompatibilities.

## Requirements and acceptance criteria

| ID | Requirement |
|---|---|
| KMIPKIT-0086-FR-001 | `plan` and `preflight` validate configuration, profile, catalog, and installed artifact without DNS lookup or sockets. Capability profiles are hypotheses only. |
| KMIPKIT-0086-FR-002 | `run` links a JNI JAR installed in an isolated Maven repository, verifies its exact release commit and digests, and records the commit and artifact SHA-256 in every report. |
| KMIPKIT-0086-FR-003 | Results account for every catalog case applicable to the selected protocol version. Each case runs a verifiable typed or generic recipe, or has an explicit, sanitized and auditable reason that the route is unavailable or not applicable. CI fails for unclassified cases or invented positive results. |
| KMIPKIT-0086-FR-004 | Typed routes send requests over TTLV, JSON, and XML and raw TLS or HTTPS where the public API supports them. They cover responses, batch order, KMIP status codes, persisted attributes, retrieval, and cleanup when catalog preconditions allow. |
| KMIPKIT-0086-FR-005 | Local tests exercise available typed builders and models, generic TTLV construction and decoding, semantic equivalence, and byte equality only for deterministic inputs with identical encoding. |
| KMIPKIT-0086-FR-006 | Every live run requires a dedicated test environment. State-changing operations require opt-in in both configuration and CLI. Credentials come from environment variables and are excluded from reports and diagnostics. |
| KMIPKIT-0086-FR-007 | JSON, Markdown, and HTML reports conform to the shared schema and separately show KMIPKit correctness, observed server compatibility, parity differences, and pending coverage. They contain no secrets, KMIP bodies, or remote text. |
| KMIPKIT-0086-FR-008 | CI installs the JAR from the pinned commit, builds and runs the external consumer, and tests controlled TLS exchanges. Fixtures do not claim interoperability with any vendor. |

## Security and API limits

Preflight must not open sockets. State-changing requests are never retried;
fixture objects use unique names and cleanup runs on all exit paths. Remote
errors are reduced to numeric codes and sanitized reasons. If the public API
does not expose a route, the report states the limitation and keeps the
coverage pending.

The artifact at the base commit does not currently preserve the configured
protocol version when creating a Java client. Live KMIP 2.0 is therefore
blocked until the corresponding FFI fix is included in `release/1.0.0`. After
that human-approved integration, update the exact artifact pin and run both
protocol versions.
