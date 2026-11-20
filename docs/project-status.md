# KMIPKit project status

**Snapshot:** `release/1.0.0` at `ef9a19b770b59d29af1770a1c99e7634e7ff9034` (2026-11-10)

This page records what was present in the dated release-branch snapshot. It
separates implemented capabilities from release evidence and publication; it is
not a conformance statement. See the [roadmap](roadmap.md) for the same
historical implementation sequence.

## Product scope

KMIPKit is a Rust KMIP client with a stable C ABI and Java and Python adapters.
KMIP 2.1 is the default edition. The version-gated modern edition support is
limited to KMIP 2.0, 2.1, and the catalogued KMIP 3.0 draft/standard surface;
it does not claim blanket KMIP 3.0 conformance. KMIP 1.x is excluded from the
client library.

The recorded 1.0 scope includes TTLV, JSON, and XML; raw TLS and HTTPS over
HTTP/1.1 with TLS 1.3 mutual authentication; client-initiated operations and
asynchronous protocol outcomes; the approved server-to-client operations and
role handling; and Rust, C, Java, and Python APIs.

## Capabilities integrated in this snapshot

- Layered Rust workspace for TTLV, KMIP protocol, transport, client, public
  facade, C ABI, test support, and a standalone KMIP test server.
- Typed KMIP request/response models, generic ordered TTLV access, bounded
  codecs, validation, operation execution, batch handling, and vendor
  extension registration.
- Production raw-TLS and HTTPS transports, TLS credential reload and session
  resumption, delivery-state reporting, and secret redaction/zeroization
  controls.
- JSON and XML codecs integrated with the client and transport paths alongside
  default TTLV encoding.
- Rust, C, Java, and Python API parity work integrated through KMIPKIT-0053.
- Server-to-client listeners, dispatch, and role-handling work under
  KMIPKIT-0066.
- Version-gated KMIP 2.0 and KMIP 3.0 support under KMIPKIT-0069–0071. The
  KMIP 3.0 work remains bounded by its draft specification and source revision.
- Shared interoperability tooling and standalone Rust, C, Java, and Python
  validators under KMIPKIT-0058–0062.
- A separate stateless proxy workspace and provider bridges for AWS, Azure,
  Google Cloud, Oracle OCI, Vault/OpenBao, Infisical, IBM, Alibaba, Scaleway,
  and Open Telekom Cloud under KMIPKIT-0072–0082. The proxy is a separate
  deliverable from the core client packages.

These entries describe code integrated into this branch. They do not establish
that every supported server, operation/profile combination, platform, or
distribution package has passed external verification.

## Release readiness observed on 2026-11-10

- The packaging dry-run pull request, KMIPKIT-0068 / PR #138, was open as a
  draft at the time of review.
- The CI run for this snapshot was still in progress. Its Rust, adapter,
  platform coverage, normative-inventory, dependency-policy, and coverage-gate
  jobs reported success; the Windows script-contract job had not completed.
  See the [run summary](https://github.com/NeverWe1come/KMIPKit/actions/runs/42775300982)
  for its final result.
- Interoperability validator and common-foundation workflows had passed on the
  snapshot. The server matrix remains research and integration planning; vendor
  operation results marked pending have not thereby been verified.
- The normative inventory report records 634 requirements and 32 open source
  discrepancies. The report explicitly does not claim profile conformance or
  certification. Baseline profile evidence is incomplete, and the conformance
  workflow requires current protected-branch evidence before a formal claim.
- Independent security review, release package smoke tests, signed artifacts,
  SBOM/provenance, and final release approval remain release gates in the
  testing and release documentation; this snapshot does not establish their
  completion.
- No public 1.0.0 release was listed when the repository state was reviewed.

CI and pull-request states change over time. Treat the dated statements above
as historical evidence and check the linked run, PR, and current release branch
before making a present-day release decision.

## Security properties recorded in this snapshot

The implementation is intended to avoid automatic KMIP request retries,
require peer verification, bound decoding, and keep secrets out of diagnostics.
The [testing strategy](development/testing.md) describes the verification
approach recorded for this snapshot.
