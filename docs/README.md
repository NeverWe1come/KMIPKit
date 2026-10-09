# KMIPKit documentation index

This directory contains the validated design and operating rules for the
project. It describes intended behavior; implementation claims require code,
tests, and the conformance matrix.

## Required reading for an AI agent

1. [`../AGENTS.md`](../AGENTS.md)
2. [The Spec Kit constitution](../.specify/memory/constitution.md)
3. [`design/project-definition.md`](design/project-definition.md)
4. [`compliance/document-hierarchy.md`](compliance/document-hierarchy.md)
5. The approved Spec Kit specification for the assigned task
6. The relevant architecture documents and accepted ADRs
7. [`development/workflow.md`](development/workflow.md)
8. [`development/testing.md`](development/testing.md)

## Design

- [Project definition](design/project-definition.md)
- [Legacy KMIP version compatibility research](design/kmip-legacy-version-feasibility.md)
- [Roadmap](roadmap.md)

## User guides

- [Typed client execution (English)](user-guide/en/client-execution.md)
- [Typed client execution (Español)](user-guide/es/ejecucion-cliente.md)
- [Managed-object lifecycle operations (English)](user-guide/en/lifecycle-operations.md)
- [Operaciones de ciclo de vida de objetos (Español)](user-guide/es/operaciones-ciclo-vida.md)
- [Inspecting KMIP messages (English)](user-guide/en/message-model.md)
- [Inspeccionar mensajes KMIP (Español)](user-guide/es/modelo-mensaje.md)

## Architecture

- [System overview](architecture/overview.md)
- [Public API](architecture/public-api.md)
- [C ABI and language bindings](architecture/ffi-and-bindings.md)
- [Transport and security](architecture/transport-security.md)
- [Vendor extensions](architecture/extensions.md)

## Development

- [Agent and specification workflow](development/workflow.md)
- [Rust workspace and PowerShell commands](development/rust-workspace.md)
- [Testing strategy](development/testing.md)
- [KMIP server interoperability matrix](development/kmip-interoperability.md)
- [Coding standards](development/coding-standards.md)
- [Git, versioning, and releases](development/git-and-releases.md)

## Compliance

- [OASIS document hierarchy](compliance/document-hierarchy.md)
- [Conformance and traceability](compliance/conformance.md)
- [Pinned OASIS sources](../specification/oasis/kmip-2.1/README.md)

## Security

- [Repository threat model](security/threat-model.md)
- [Vulnerability reporting policy](../SECURITY.md)

## Architecture decisions

See the [ADR index](adr/README.md). Accepted ADRs record why a decision was
made. New ADRs supersede old decisions rather than rewriting history.

## Spec Kit ownership

Spec Kit is installed in this repository. The maintainer owns its framework
files. Agents must not change the constitution or framework without an
explicitly approved task.
