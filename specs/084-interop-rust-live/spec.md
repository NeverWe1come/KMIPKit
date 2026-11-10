# Specification: Rust live interoperability consumer

**Feature Branch**: `feature/KMIPKIT-0084-interop-rust-live`
**Status**: Approved for implementation by explicit maintainer plan authorization
**Owning identifier**: KMIPKIT-0084
**Dependency**: KMIPKIT-0083 at `fdafef8a3eb0c7b4b18d6433b78fde4151493ceb`.

## Scope and normative references

The standalone Rust consumer uses only the installed public Rust library built
from the pinned release commit. Targets are KMIP 2.0 and 2.1; vendor extensions,
KMIP 1.x/3.0 and vendor certification are excluded. Shared catalog references
are authoritative for each individual case and remain unchanged.

- OASIS KMIP Specification Version 2.1, OS, 2022-12-15,
  `https://docs.oasis-open.org/kmip/spec/v2.1/os/kmip-spec-v2.1-os.html`:
  §2 (TTLV), §§3–5 (objects, attributes, templates), §6.1 (client operations),
  §6.2.1–§6.2.5 (server operations), §7 (messages), §11 (enumerations),
  §12 (bit masks). Operation-specific exact sections come from each catalog row.
- OASIS KMIP Specification Version 2.0, OS, 2019-05-30,
  `https://docs.oasis-open.org/kmip/spec/v2.0/os/kmip-spec-v2.0-os.html`:
  §§2–7, §11 and §12, subject to the catalog version gates.
- OASIS KMIP Profiles Version 2.1, OS, 2022-12-15,
  `https://docs.oasis-open.org/kmip/profiles/v2.1/os/kmip-profiles-v2.1-os.html`:
  §5.3.1 (HTTPS), §6 (JSON), §7 (XML), applicable transport clauses.
- OASIS KMIP Profiles Version 2.0, OS, 2019-05-30,
  `https://docs.oasis-open.org/kmip/profiles/v2.0/os/kmip-profiles-v2.0-os.html`:
  corresponding selected transport and encoding clauses.
- KMIPKIT-0083, ADR-0017, ADR-0021 and ADR-0022.

## Requirements and acceptance

| ID | Requirement |
|---|---|
| KMIPKIT-0084-FR-001 | `plan` and `preflight` SHALL validate shared configuration/catalog offline, without DNS or sockets. Profile omissions SHALL NOT suppress applicable probes. |
| KMIPKIT-0084-FR-002 | `run` SHALL use the installed public `kmipkit` artifact from the exact pinned commit and record commit and SHA-256. Source-tree dependencies are insufficient. |
| KMIPKIT-0084-FR-003 | The consumer SHALL dispatch each applicable shared case or report an explicit referenced non-applicability; incomplete execution SHALL remain not_run, never success. |
| KMIPKIT-0084-FR-004 | Each run SHALL require a dedicated test environment, and mutations SHALL additionally require explicit enablement. Requests SHALL NOT be retried. |
| KMIPKIT-0084-FR-005 | Results SHALL satisfy schema 2.0 and separate library correctness, server compatibility, policy restrictions, non-run and parity. No credentials, keys, bodies, or untrusted error text may be reported. |
| KMIPKIT-0084-FR-006 | High-level and typed APIs SHALL be compared using deterministic request payloads and normalized response metadata. Generic TTLV construction/navigation SHALL be exercised locally; generic outbound construction unavailability SHALL be recorded without blaming the server. |
| KMIPKIT-0084-FR-007 | Rust 1.94 CI SHALL run formatting, Clippy, unit/integration and deterministic local TLS/controlled-response tests with the installed artifact. No live vendor compatibility claim is authorized. |

## Safety and limits

Preflight success establishes local readiness only. A successful local model
case establishes no server support. Missing object prerequisites or server
initiated channels cannot be manufactured as passing exchanges. Only exact
version exclusions and standard exclusions may be not applicable. Incomplete
adapters remain visible failures of implementation coverage.
