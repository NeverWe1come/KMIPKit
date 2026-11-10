# Specification: Independent Python KMIP Interoperability Consumer

**Branch**: `feature/KMIPKIT-0087-interop-python-live`  
**Status**: Approved for implementation by the maintainer's explicit request to complete the interoperability validators  
**Identifier**: KMIPKIT-0087  
**Base**: `release/1.0.0` at `52a2be180340e5da3166401a52f652dfd368ad37`

## Scope and normative references

The Python validator is an independent Python 3.12 consumer. It installs the
wheel built from one exact `release/1.0.0` commit in an isolated virtual
environment and imports only that installed package. It validates standard
KMIP 2.0 and 2.1 behavior using the shared catalog, connection contract and
result schema. KMIP 1.x, KMIP 3.0, vendor extensions, vendor certification and
server-specific behavior are excluded.

Normative references are the OASIS KMIP 2.1 Specification, OS
(2020-12-14), §§2–12; KMIP 2.0 Specification, OS (2019-10-31), §§2–12; and
the corresponding OASIS KMIP Profiles, OS, for TTLV, JSON, XML, raw TLS and
HTTPS. Each executable catalog case remains the source of its exact operation
and section citation. Transport behavior follows ADR-0017, ADR-0021 and the
approved KMIPKIT-0083 and KMIPKIT-0088 shared contract.

## Requirements and acceptance

| ID | Requirement |
|---|---|
| KMIPKIT-0087-FR-001 | `plan` SHALL enumerate every applicable case in the shared catalog for KMIP 2.0 or 2.1. Capability profiles SHALL be treated as hypotheses and SHALL NOT suppress probes. `preflight` SHALL validate files and artifact provenance offline, without DNS or sockets. |
| KMIPKIT-0087-FR-002 | `run` SHALL import the Python wheel installed in an isolated Python 3.12 environment. The wheel SHALL be built from the exact pinned release commit; its full commit and SHA-256 SHALL appear in every report. Source-tree imports SHALL fail CI. |
| KMIPKIT-0087-FR-003 | Every applicable case SHALL dispatch a verifiable recipe or report a truthful `not_run`/API limitation. No mock or profile-based result may be reported as an observed exchange. CI SHALL fail for missing dispatch classification or invented success. |
| KMIPKIT-0087-FR-004 | Live runs SHALL require a dedicated test environment, valid TLS inputs, and a matching server profile. State-changing cases SHALL require configuration and CLI mutation opt-ins. Requests SHALL NOT be retried; cleanup SHALL be attempted after setup. |
| KMIPKIT-0087-FR-005 | Reports SHALL validate against shared schema 2.0 and separately expose KMIPKit correctness, observed server compatibility, policy restrictions, parity and pending coverage. They SHALL omit credentials, key material, TTLV bodies and remote error text. |
| KMIPKIT-0087-FR-006 | Local tests SHALL exercise installed high-level and typed APIs, generic TTLV construction/decoding, normalized request/response comparisons, and deterministic byte equality only where inputs and encoding are identical. |
| KMIPKIT-0087-FR-007 | CI SHALL test external wheel installation, Python 3.12, KMIP 2.0 and 2.1 configuration, and deterministic controlled TLS exchanges. Controlled fixtures SHALL NOT be presented as vendor interoperability evidence. |

## Security and API limits

Preflight performs no network access. Missing API routes, state prerequisites,
or unsupported local recipes remain pending; they are never converted to server
incompatibility. Mutation permission is checked before creating a client or
opening a socket. Reports contain only allowlisted numeric result codes and
sanitized local diagnostics. The application uses the public Python package;
it does not invoke the library's C ABI directly.
