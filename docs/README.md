# KMIPKit documentation

This directory contains technical documentation, user guides, and engineering references for the code in this repository. The review branch keeps documentation that helps readers use, build, inspect, and verify KMIPKit.

Internal development planning notes and operational scratch records are excluded from this release distribution.

## Architecture

- [System overview](architecture/overview.md)
- [Public API](architecture/public-api.md)
- [C ABI and language bindings](architecture/ffi-and-bindings.md)
- [Transport security](architecture/transport-security.md)
- [Vendor extensions](architecture/extensions.md)

## User guides

The [English guides](user-guide/en/) and [Spanish guides](user-guide/es/) explain client execution, protocol messages, operations, transports, bindings, and diagnostics.

## Language bindings

The [bindings index](../bindings/README.md) links the common documentation layout
for [C](../bindings/c/README.md), [Java](../bindings/java/README.md), and
[Python](../bindings/python/README.md). Each includes installation, native library
handling, testing, and examples adapted to that language.

## Development

- [Rust workspace](development/rust-workspace.md)
- [Testing](development/testing.md)
- [Coding standards](development/coding-standards.md)
- [Git and releases](development/git-and-releases.md)
- [API manifest](development/api-manifest.md)
- [KMIP 3.0 API migration](development/kmip-3.0-api-migration.md)
- [Interoperability evidence](development/kmip-interoperability.md)
- [Transport performance](development/transport-performance.md)
- [Dependency review](development/transport-dependency-review.md)

## Verification and security

- [Conformance](compliance/conformance.md)
- [Cosmian KMS notes](interop/cosmian-kms.md)
- [Thales CipherTrust notes](interop/thales-ciphertrust.md)
- [Utimaco ESKM notes](interop/utimaco-eskm.md)
- [Threat model](security/threat-model.md)
- [Dependency policy](security/dependency-policy.md)
- [Hardening evidence](security/pre-release-hardening-evidence.md)

## Project snapshots

- [Project status](project-status.md)
- [Roadmap](roadmap.md)
