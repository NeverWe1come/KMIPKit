# KMIPKit Constitution

## Core Principles

### I. Specification and traceability

Every implementation change MUST have an approved Spec Kit specification with
scope, acceptance criteria, and exact OASIS KMIP 2.1 references. Every applicable
normative client requirement MUST have a stable identifier linking its source,
implementation, and verification. Informative guidance cannot create a
normative requirement. This keeps the implementation auditable against the
standard.

### II. Test first and evidence based conformance

Implementation MUST follow Red, Green, Refactor TDD, with failing tests
recorded before production changes. Tests MUST cover relevant unit behavior,
negative inputs, official OASIS cases, and language boundary contracts. A
profile or interoperability claim MUST be backed by its applicable clauses
and passing tests; certification claims require external evidence. Tests are
part of the specification, not a substitute for normative traceability.

### III. One core, explicit language boundaries

The KMIP implementation MUST live in the Rust workspace. The C ABI MUST use
opaque handles, fixed width types, explicit ownership, stable error codes,
and panic containment. Unsafe code MUST remain within `kmipkit-ffi`; other
crates MUST forbid it. Rust, C, Java, and Python MUST provide equivalent
1.0 capabilities. This keeps protocol behavior consistent across languages.

### IV. Secure defaults and lossless protocol handling

Network input and extensions MUST be treated as untrusted. Decoders MUST
enforce resource limits before allocation. Secrets MUST be redacted from
logs and errors and zeroized when KMIPKit owns their memory. Production TLS
MUST validate the server certificate and hostname; automatic request retries
MUST NOT occur. The model MUST preserve unknown standard and vendor values
so valid data survives round trips and future extensions.

### V. Human governed, reviewable changes

Each approved specification MUST use a dedicated feature branch and worktree.
Agents MAY prepare code, commits, and draft PRs; only a human may approve or
merge them. `master` MUST remain stable, with feature work integrated through
the active release branch. Architectural boundary changes require a reviewed
specification or ADR before implementation. This separates execution from
acceptance.

## Product and Security Constraints

- The 1.0.0 target is a KMIP 2.1 client using TTLV, raw TLS and HTTPS over
  TLS 1.3 with mutual TLS, and synchronous APIs. It covers all client
  initiated operations, including KMIP protocol asynchronous results.
- Server initiated operations begin in 1.1. JSON, XML, and further language
  adapters follow later. A server's policy may reject supported requests.
- The 1.0 public surface has high level operation builders, complete typed
  requests and responses, and generic TTLV access. Cryptographic choices
  MUST remain explicit to callers.
- The initial Rust MSRV is 1.94 with Edition 2024. Public APIs MUST be
  documented; `rustfmt` and Clippy warnings MUST pass.
- OASIS source files under `specification/oasis/` are immutable upstream
  copies. Builds MUST use reviewed checked in inputs and MUST NOT download
  or scrape standard documents.
- Detailed technical and product decisions are recorded in
  `docs/design/project-definition.md`, accepted ADRs, and `AGENTS.md`.

## Development and Review Workflow

1. Approve a bounded specification and identify its normative references.
2. Work on a dedicated feature branch from the active release branch and
   follow Red, Green, Refactor commits with DCO sign-off.
3. Update requirement traceability, generated artifacts, and documentation
   with the behavior they describe.
4. Run relevant tests, formatting, linting, coverage, security, and
   compatibility checks. Record commands and results in the draft PR.
5. Obtain human review and approval before merging. A release branch moves
   to `master` only after its required gates pass.

The foundation migration may branch from `master` before a release branch
exists. No feature may relax a failed check or alter an approved scope
without an explicit reviewed change.

## Governance

This constitution governs project specifications, plans, code, and reviews.
Direct human instructions control the current task; a requested change to
these principles MUST be recorded here and reviewed by the human maintainer.
Amendments MUST describe affected specifications, ADRs, tests, and migration
work. Use semantic versioning: MAJOR for incompatible principle changes,
MINOR for new or materially expanded rules, and PATCH for clarifications.
Every PR review MUST check compliance with this constitution and record any
approved exception. `AGENTS.md` provides operational guidance subordinate
to this document.

**Version**: 1.0.0 | **Ratified**: 2026-10-03 | **Last Amended**: 2026-10-03
