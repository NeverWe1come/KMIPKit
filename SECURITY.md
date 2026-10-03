# Security policy

KMIPKit has no production release yet. The policy becomes fully operational
when the repository opens with version 1.0.0.

## Reporting a vulnerability

Use GitHub private vulnerability reporting for this repository when available.
During the private development phase, contact the repository owner through a
private GitHub channel available to project collaborators. Do not disclose the
issue in a public issue, discussion, PR title, test fixture, or log.

Include:

- Affected commit or version.
- Component and platform.
- Reproduction steps or proof of concept.
- Expected and observed impact.
- Whether credentials or real systems were involved.
- Suggested mitigation, if known.

Do not include live credentials, customer data, or reusable private keys.

## Response

The project cannot promise a response SLA during its initial community phase.
Maintainers will acknowledge, validate, coordinate remediation, prepare an
advisory when appropriate, and credit reporters who request credit.

## Supported versions

After 1.0, the latest stable release receives ordinary support. Security
backports are evaluated by severity and feasibility. Older versions are marked
unsupported when maintenance ends.

## Security properties

The intended security properties are documented in `AGENTS.md` and
[`docs/architecture/transport-security.md`](docs/architecture/transport-security.md).
They include strict input limits, TLS verification, secret redaction, explicit
memory ownership, panic containment, no automatic retry, and no unsafe code
outside the FFI crate.

The repository-wide design threat model is
[`docs/security/threat-model.md`](docs/security/threat-model.md). It records
assets, trust boundaries, attack hypotheses, mitigations, and open design
questions that must be updated as implementation evidence becomes available.

## Claims

KMIPKit does not claim certification, FIPS validation, security audit, or
fitness for a regulated use unless a release links to exact applicable
evidence. KMIP interoperability does not itself establish product security.
