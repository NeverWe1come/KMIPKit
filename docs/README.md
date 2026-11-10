# KMIPKit documentation index

Use this index to move from the approved product scope to implementation,
release evidence, and language guides. An implementation statement requires
source code and tests; a conformance or release statement also requires its
separate evidence gate.

## Current context

- [Project status snapshot](project-status.md)
- [Approved project definition](design/project-definition.md)
- [Roadmap](roadmap.md)
- [OASIS source hierarchy](compliance/document-hierarchy.md)
- [Normative inventory coverage report](../specification/catalog/coverage-report.md)
- [KMIP server interoperability matrix](development/kmip-interoperability.md)

## Required reading for contributors and AI agents

1. [Agent rules](../AGENTS.md)
2. [Spec Kit constitution](../.specify/memory/constitution.md)
3. [Project definition](design/project-definition.md)
4. [OASIS document hierarchy](compliance/document-hierarchy.md)
5. The approved specification and plan for the assigned feature
6. Relevant accepted [architecture decision records](adr/README.md)
7. [Development workflow](development/workflow.md)
8. [Testing strategy](development/testing.md)

## Design and architecture

- [Project definition](design/project-definition.md)
- [System architecture overview](architecture/overview.md)
- [Public API architecture](architecture/public-api.md)
- [C ABI and language bindings](architecture/ffi-and-bindings.md)
- [Transport and security](architecture/transport-security.md)
- [Vendor extension model](architecture/extensions.md)
- [Legacy KMIP version feasibility research](design/kmip-legacy-version-feasibility.md)
- [All accepted and superseded decisions](adr/README.md)

## User guides

### English

- [Client execution](user-guide/en/client-execution.md)
- [Message model](user-guide/en/message-model.md)
- [Typed bindings and response data](user-guide/en/typed-bindings-and-response-data.md)
- [Server-to-client operations and role changes](user-guide/en/server-to-client-operations.md)
- [Attribute operations](user-guide/en/attribute-operations.md)
- [Managed objects and key material](user-guide/en/managed-objects-and-key-material.md)
- [Object retrieval](user-guide/en/object-retrieval.md)
- [Lifecycle operations](user-guide/en/lifecycle-operations.md)
- [Lifecycle and certificates](user-guide/en/lifecycle-and-certificates.md)
- [Cryptographic operations](user-guide/en/cryptographic-operations.md)
- [Encrypt and decrypt](user-guide/en/encrypt-decrypt.md)
- [Key derivation and transfer](user-guide/en/key-derivation-and-transfer.md)
- [Query and Ping](user-guide/en/query-ping.md)
- [Credential sessions](user-guide/en/credential-sessions.md)
- [Policy controls](user-guide/en/policy-controls.md)
- [Vendor extensions](user-guide/en/vendor-extensions.md)
- [Production transports](user-guide/en/production-transports.md)
- [Profile conformance](user-guide/en/profile-conformance.md)
- [Diagnostics and interoperability](user-guide/en/diagnostics-and-interop.md)
- [Language packages](user-guide/en/language-packages.md)

### Español

- [Ejecución del cliente](user-guide/es/ejecucion-cliente.md)
- [Modelo de mensaje](user-guide/es/modelo-mensaje.md)
- [Enlaces tipados y validación de respuestas](user-guide/es/enlaces-tipados-validacion-respuestas.md)
- [Operaciones servidor-cliente y cambios de rol](user-guide/es/operaciones-servidor-cliente.md)
- [Operaciones de atributos](user-guide/es/attribute-operations.md)
- [Objetos gestionados y material de claves](user-guide/es/objetos-gestionados-y-material-de-clave.md)
- [Recuperación de objetos](user-guide/es/object-retrieval.md)
- [Operaciones del ciclo de vida](user-guide/es/operaciones-ciclo-vida.md)
- [Ciclo de vida y certificados](user-guide/es/operaciones-ciclo-vida-y-certificados.md)
- [Operaciones criptográficas](user-guide/es/operaciones-criptograficas.md)
- [Cifrado y descifrado](user-guide/es/cifrado-descifrado.md)
- [Derivación y transferencia de claves](user-guide/es/derivacion-y-transferencia-de-claves.md)
- [Operaciones Query y Ping](user-guide/es/operaciones-query-ping.md)
- [Sesiones de credenciales](user-guide/es/sesiones-credenciales.md)
- [Controles de uso](user-guide/es/controles-de-uso.md)
- [Extensiones de fabricante](user-guide/es/extensiones-de-fabricante.md)
- [Transportes de producción](user-guide/es/transportes-produccion.md)
- [Conformidad de perfiles](user-guide/es/profile-conformance.md)
- [Diagnóstico e interoperabilidad](user-guide/es/diagnostico-e-interop.md)
- [Paquetes de lenguajes](user-guide/es/paquetes-de-idiomas.md)

## Development and operations

- [Agent and specification workflow](development/workflow.md)
- [Rust workspace and local commands](development/rust-workspace.md)
- [Testing and coverage strategy](development/testing.md)
- [Git, versioning, and release gates](development/git-and-releases.md)
- [API manifest](development/api-manifest.md)
- [KMIP interoperability evidence](development/kmip-interoperability.md)
- [Transport performance baseline](development/transport-performance.md)
- [Dependency policy](../SECURITY.md) and [dependency review](security/dependency-policy.md)
- [AI context-loading guide](ai/context-loading.md)
- [AI review checklist](ai/review-checklist.md)

## Compliance and source material

- [Conformance and traceability strategy](compliance/conformance.md)
- [OASIS document hierarchy](compliance/document-hierarchy.md)
- [Pinned KMIP 2.1 sources](../specification/oasis/kmip-2.1/README.md)
- [Normative catalog](../specification/catalog/README.md)
- [Profile targets and readiness](../specification/catalog/profile-targets.json)

## Security

- [Threat model](security/threat-model.md)
- [Vulnerability reporting policy](../SECURITY.md)

## Spec Kit ownership

Spec Kit is installed in this repository. The maintainer owns its framework
files. Agents must not change the constitution or framework without an
explicitly approved task.
