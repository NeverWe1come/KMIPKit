# ADR-0023: Cloud KMS Stateless Sidecar Proxy Architecture

**Status**: Accepted
**Date**: 2026-11-07
**Decision owner**: KMIPKit maintainer
**Specification**: [KMIPKIT-0072](../../specs/072-kmipkit-proxy-foundation/spec.md)

## Context

Enterprise storage appliances, databases (MongoDB, MySQL, PostgreSQL, Oracle), hypervisors (VMware vSphere, Nutanix, Proxmox), and backup solutions require the OASIS KMIP protocol (TTLV/JSON/XML over mTLS/HTTPS on port 5696) for transparent data encryption (TDE) and key management.

However, major public cloud providers (AWS KMS, Azure Key Vault / Managed HSM, Google Cloud KMS, and Oracle OCI Vault) expose proprietary REST/gRPC APIs and do not implement OASIS KMIP.

To bridge this gap without bloating the core `KMIPKit` client library or introducing external dependencies and database state, a dedicated stateless proxy sidecar is required.

## Decision

1. **Independent Workspace & Zero Library Contamination**:
   The proxy service is developed in `proxy/kmipkit-proxy` as an independent Cargo workspace with its own lockfile and dependency boundary. It consumes `kmipkit-ttlv`, `kmipkit-protocol`, and `kmipkit-transport` directly by path, ensuring zero impact on `kmipkit`, `kmipkit-client`, or `kmipkit-ffi`.

2. **Zero-DB Stateless Architecture (*Cloud Tag-Stuffing*)**:
   The proxy maintains zero persistent local databases. All standard KMIP attributes (`ObjectType`, `State`, `CryptographicUsageMask`, `Name`, `Link`, `ActivationDate`, `ProtectStopDate`, `RevocationReasonCode`, `kmip_mode`) are encoded into native cloud resource tags/labels (`kmip_*`) using a normalized, provider-agnostic encoding (`TagCodec`).

3. **Auto-Envelope Export Mode**:
   For client-side encryption workloads where clients require symmetric key extraction (`Get(UniqueIdentifier)`):
   - On `Create` / `Register`: The proxy wraps symmetric key bytes using the cloud provider's root Key Encryption Key (KEK) and persists only the ciphertext blob in the provider's native secret storage (AWS Secrets Manager, Azure Key Vault Secrets, GCP Secret Manager, OCI Vault Secrets).
   - On `Get`: The proxy retrieves the ciphertext blob, unwraps it via the cloud KMS in protected `Zeroizing<Vec<u8>>` memory, formats the KMIP `KeyBlock`, and zeros memory immediately after transmission.

4. **Multi-Version Protocol Support**:
   The proxy terminates mTLS 1.3 and HTTPS, dynamically handling requests in KMIP 2.0, 2.1, and 3.0 across TTLV, JSON, and XML message encodings.

5. **Provider Modular Trait & Ephemeral Cache**:
   All cloud interactions are mediated through an asynchronous `CloudKmsProvider` trait, backed by a short-lived in-memory read cache for tags/metadata with immediate invalidation upon mutation.

## Consequences

- Applications and hypervisors can use standard KMIP against AWS, Azure, GCP, and OCI.
- No local database maintenance, backups, or replication are required for the proxy.
- Core library packaging and FFI consumers remain completely isolated from cloud SDKs.
- Proxy changes execute dedicated CI workflows without triggering full library matrices.
