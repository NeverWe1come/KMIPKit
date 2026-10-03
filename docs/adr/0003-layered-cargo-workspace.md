# ADR-0003: Layered Cargo workspace

Status: Accepted
Date: 2026-10-03

## Context

Codec, protocol, I/O, orchestration, facade, and FFI have different invariants
and test needs. Agents also need stable ownership boundaries.

## Decision

Use public synchronized crates for `kmipkit-ttlv`, `kmipkit-protocol`,
`kmipkit-transport`, `kmipkit-client`, and `kmipkit`, plus unpublished FFI
and test-support crates. Dependencies flow down the layers and never introduce
protocol logic into bindings or I/O into protocol models.

## Consequences

Advanced Rust users can reuse layers directly and each unit is independently
testable. More public crates enlarge the SemVer surface and require coordinated
publication.

## Alternatives considered

A monolithic crate reduces package count but weakens ownership, reuse, and
parallel development. Excessively small crates would increase coordination
without meaningful isolation.
