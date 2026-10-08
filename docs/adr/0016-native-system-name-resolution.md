# ADR-0016: Preserve native system name-resolution policy

**Status**: Accepted
**Date**: 2026-10-08
**Decision owner**: KMIPKit maintainer under delegated authorization
**Related ADRs**: [ADR-0015](0015-asynchronous-transport-worker.md)
**Specification**: [KMIPKIT-0013](../../specs/013-production-transport/spec.md)

## Context

KMIPKIT-0013 initially selected Hickory 0.26.3 with its system-configuration
adapter. The implementation review and T012 evidence found that Hickory
flattens configured name servers and cannot preserve per-interface routing
used by split DNS and VPN policies. A list of system name servers is not
equivalent to the operating system's resolver policy. KMIPKit must use the
host resolver so configured NSS, hosts-file, search-domain, split-DNS, and
VPN behavior remain under the platform's control on Linux, Windows, and macOS.

The synchronous Rust standard-library resolver can block. Tokio's
`spawn_blocking` cannot interrupt a call once it has started, and dropping its
join handle detaches the task. These limits must be part of the transport
contract rather than hidden behind a caller timeout.

## Decision

KMIPKit resolves an endpoint host with `std::net::ToSocketAddrs` inside the
existing per-client Tokio worker using `spawn_blocking`. The blocking closure
only resolves and returns socket addresses; it MUST NOT open TCP/TLS
connections, construct KMIP requests, or dispatch request bytes.

Before submitting a resolver closure, KMIPKit MUST acquire a permit from one
shared 32-permit governor for the loaded KMIPKit library instance. Admission is
fail-fast when no permit is available. The permit remains inside the blocking
closure until the native resolver call returns, including after its caller
has timed out or canceled. Each worker runtime also sets
`max_blocking_threads(1)`; this is per-worker containment and does not replace
the shared governor. The shared governor is a narrow, documented exception to
the general preference against global mutable state; per-client limits alone
would not bound aggregate resolver work across clients.

One `ToSocketAddrs` call is made for the configured host and port during a
connection-establishment attempt. KMIPKit retains and tries no more than the
first 16 returned addresses, in the operating system's order. Those addresses
are alternatives for the same configured endpoint and MUST be tried
sequentially in that order. They may be tried only within the connect and
total deadlines and before request-dispatch commit.
KMIPKit does not sort addresses, select another hostname, or cache results.

The operating system owns address-family query behavior, packet retries,
upstream concurrency, resolver caching, cache invalidation, hosts-file lookup,
search rules, and split-DNS/VPN routing. KMIPKit makes no numeric guarantees
about those OS-controlled details. A timed-out or canceled caller returns
without waiting for a native lookup that has already started. An unstarted
blocking job is aborted when possible; an already-started call may continue in
the OS/Tokio blocking thread and retain its governor permit until it exits.
Its late result MUST be discarded. Cancellation and deadline checks MUST
prevent that result from starting a TCP/TLS candidate connection or dispatching
KMIP request data. Shutdown MUST NOT wait indefinitely for an already-started
native resolver call. Native resolver traffic already in progress cannot be
promised to stop when the KMIPKit caller returns.

The 16-address limit applies to addresses KMIPKit retains and attempts. The
platform resolver may allocate or materialize a larger intermediate result
before returning it; KMIPKit cannot bound allocations internal to the OS API.

The 32-permit scope is one loaded KMIPKit library instance. Multiple separately
loaded static or dynamic copies in one process do not share the governor and
are outside this bound.

## Consequences

- Hickory is removed from production dependencies and the lockfile.
- Resolver behavior follows the host resolver's active configuration instead
  of KMIPKit maintaining its own upstream list or DNS response cache.
- The connect deadline bounds KMIPKit's wait, not the lifetime of an
  already-started native lookup.
- The 16-address candidate cap bounds KMIPKit-retained candidates, not any
  intermediate allocation made inside the operating-system resolver.
- Resolver tests inject a blocking address-lookup function to deterministically
  verify admission, cancellation, late-result isolation, address ordering, and
  shutdown. Platform smoke tests use the system resolver without asserting
  packet-level retry or cache behavior.
- The per-library governor is shared state and must be reviewed as part of the
  security and lifecycle audit.

## Alternatives considered

| Alternative | Result | Reason |
|---|---|---|
| Hickory 0.26.3 with system configuration | Rejected | Its flattened upstream list cannot preserve per-interface split-DNS/VPN routing; its packet and cache controls also cannot be reconciled with delegating routing to the OS. |
| `tokio-system-resolver` 0.5.0 | Rejected | It does not support Windows, and its per-instance blocking limits do not bound aggregate work across KMIPKit clients. |
| Separate native resolver APIs per platform | Rejected for 1.0 | They require three implementations and platform-specific callback/cancellation lifecycles; an already-running native lookup is not reliably interruptible on all targets. |
| `ToSocketAddrs` without a shared governor | Rejected | Per-client runtime limits would allow unbounded aggregate queued and active resolver jobs as clients are created. |

## Supersession

This ADR supersedes only the Hickory resolver choice and packet-level/cache
limits recorded in [ADR-0015](0015-asynchronous-transport-worker.md) and the
corresponding KMIPKIT-0013 artifacts. ADR-0015 remains accepted for the worker,
deadline, TLS, and connection-lifecycle decisions.
