# ADR-0009: Conformance and pinned OASIS sources

Status: Accepted
Date: 2026-10-03

## Context

AI agents and reviewers need deterministic standards context. Protocol coverage
and profile conformance must be demonstrated rather than asserted.

## Decision

Pin exact OASIS Specification, Profiles, Usage Guide, and Test Cases copies
with notices, URLs, stages, dates, and SHA-256 hashes. Keep them immutable.
Maintain 100 percent applicable normative requirement traceability and claim a
profile only after all clauses and tests pass. Apply official errata only
through reviewed changes.

## Consequences

Build and review do not depend on mutable web pages. The repository grows by a
few megabytes and must preserve third-party notices. Traceability requires
continuous work in every protocol PR.

## Alternatives considered

External links alone can change or be unavailable. Copying prose into prompts
loses status and citations. A coverage percentage without clause mapping cannot
prove standard conformance.
