# ADR-0007: Vendor extension model

Status: Accepted
Date: 2026-10-03

## Context

KMIP provides extension mechanisms while vendors define their semantics.
KMIPKit must preserve unknown data and allow external extension packages
without coupling the core release to every vendor.

## Decision

Version 1.0 preserves and sends generic extensions and applies criticality
rules. A future SDK first supports runtime declarative manifests. Extensions
requiring code use separately versioned Rust crates compiled into a KMIPKit
distribution with generated adapter layers. Dynamic executable plugins are
deferred.

## Consequences

Most structural extensions can work across languages from one manifest. Custom
logic remains controlled and testable. Rebuilding is required for executable
extensions until a separately secured plugin model exists.

## Alternatives considered

Putting all vendor logic in the core couples releases and maintenance. Native
runtime plugins create ABI, trust, signing, and deployment risk too early.
