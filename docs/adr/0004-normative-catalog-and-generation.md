# ADR-0004: Normative catalog and generated code

Status: Accepted
Date: 2026-10-03

## Context

KMIP 2.1 defines many tags, enumerations, structures, operations, and repeated
cross-language surfaces. Manual duplication risks omissions and drift.

## Decision

Create a reviewed machine-readable catalog and a separate public API manifest.
Generate repetitive Rust, C, Java, Python, tests, and documentation. Commit
generated output, prohibit manual edits, pin generators, and verify a clean
regeneration in CI. Builds never parse remote OASIS HTML.

## Consequences

Reviewers can trace one source change through every language. Generator quality
and schema stability become critical. Hand-written high level APIs remain
necessary for idiomatic design.

## Alternatives considered

Fully manual code is error-prone. Build-time scraping is non-reproducible and
would confuse normative and derived inputs. Fully generated user APIs would be
less idiomatic.
