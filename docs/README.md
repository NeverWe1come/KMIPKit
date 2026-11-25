# KMIPKit Documentation

> **Version:** 1.0.0 | **Protocol Support:** OASIS KMIP 2.0, 2.1 (default), 3.0 + Legacy KMIP 1.0–1.4 Bridge  
> **Languages:** Rust · C (C11 ABI) · Java (JNI) · Python (CFFI)

## Multilingual Documentation Portal (100% Parity)

Start here for complete user guides, architecture diagrams, multi-language quickstarts, Cloud KMS Proxy setup, KMIP 1.X Legacy Bridge guides, and enterprise deployment playbooks:

| Language | Portal Entrypoint | Scope |
| :--- | :--- | :--- |
| **English** | **[docs/en/00-index.md](en/00-index.md)** | 8 complete guides (Quickstart, Architecture, Protocol, Bindings, Cloud Proxy, KMIP 1.X Bridge, Enterprise Ops) |
| **Español** | **[docs/es/00-index.md](es/00-index.md)** | 8 guías completas (paridad 1:1 con inglés) |
| **Français** | **[docs/fr/00-index.md](fr/00-index.md)** | 8 guides complets (parité 1:1 avec l'anglais) |
| **Deutsch** | **[docs/de/00-index.md](de/00-index.md)** | 8 vollständige Leitfäden (1:1-Parität mit Englisch) |

---

## Core Engineering & Reference Documentation

### Architecture
- [System Overview](architecture/overview.md)
- [Public API Design](architecture/public-api.md)
- [C ABI & Language Bindings](architecture/ffi-and-bindings.md)
- [Transport & Security Architecture](architecture/transport-security.md)
- [Vendor Extensions](architecture/extensions.md)

### Language Bindings
- [Bindings Index](../bindings/README.md) · [C (C11 ABI)](../bindings/c/README.md) · [Java (JNI)](../bindings/java/README.md) · [Python (CFFI)](../bindings/python/README.md)

### Development & Verification
- [Rust Workspace](development/rust-workspace.md) · [Testing Strategy](development/testing.md) · [Coding Standards](development/coding-standards.md)
- [Git & Releases](development/git-and-releases.md) · [API Manifest](development/api-manifest.md) · [KMIP 3.0 API Migration](development/kmip-3.0-api-migration.md)
- [Interoperability Evidence](development/kmip-interoperability.md) · [Transport Performance](development/transport-performance.md)

### Compliance, Interoperability & Security
- [Conformance Ledger](compliance/conformance.md)
- [Cosmian KMS Notes](interop/cosmian-kms.md) · [Thales CipherTrust Notes](interop/thales-ciphertrust.md) · [Utimaco ESKM Notes](interop/utimaco-eskm.md)
- [Threat Model](security/threat-model.md) · [Dependency Policy](security/dependency-policy.md) · [Pre-Release Hardening Evidence](security/pre-release-hardening-evidence.md)

### Project Snapshots
- [Project Status](project-status.md) · [Roadmap](roadmap.md)
