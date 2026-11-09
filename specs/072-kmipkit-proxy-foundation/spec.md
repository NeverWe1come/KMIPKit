# Specification: KMIPKit Stateless Proxy Foundation (KMIPKIT-0072)

## 1. Overview and Purpose

This specification defines the foundation for `kmipkit-proxy`, a zero-database stateless bridge translating OASIS KMIP client requests (KMIP 2.0, 2.1, and 3.0 over mTLS 1.3 or HTTPS) into cloud KMS and secret store operations.

The foundation provides:
- Protocol listener and request dispatcher based on `kmipkit-transport`, `kmipkit-protocol`, and `kmipkit-ttlv`.
- Cloud Tag-Stuffing codec (`TagCodec`) mapping KMIP normative attributes to provider-compatible tags/labels.
- Auto-Envelope engine for wrapped symmetric key export and import with `Zeroizing` memory protection.
- In-memory metadata and tag cache with immediate write invalidation.
- Pluggable asynchronous `CloudKmsProvider` trait with mock and concrete provider hooks.

## 2. Normative References

- **OASIS KMIP 2.0 Specification**: §8 Messages, §9 Protocol Details.
- **OASIS KMIP 2.1 Specification**: §8 Messages, §9 Protocol Details.
- **OASIS KMIP 3.0 Draft**: §8 Messages, §9 Protocol Details.
- **ADR-0005**: Transport and TLS Policy (TLS 1.3 with mTLS).
- **ADR-0022**: Multi-Version Modern KMIP Architecture.
- **ADR-0023**: Cloud KMS Stateless Sidecar Proxy Architecture.

## 3. Scope and Acceptance Criteria

- **`REQ-072-001` (Zero-DB Stateless Architecture)**:
  The proxy runs without local relational or document databases; metadata is persisted in cloud resource tags/labels.
- **`REQ-072-002` (Universal TagCodec)**:
  `TagCodec` encodes/decodes `ObjectType`, `State`, `CryptographicUsageMask`, `Name` (RFC 4648 base32hex without padding $\le 63$ chars), `Link`, `ActivationDate`, `ProtectStopDate`, `RevocationReasonCode`, and `kmip_mode`.
- **`REQ-072-003` (Auto-Envelope Export Mode)**:
  Symmetric keys requested via `Get` are unwrapped on-demand from encrypted blobs using cloud KEKs in `Zeroizing<Vec<u8>>` memory, and wiped immediately after transmission.
- **`REQ-072-004` (Multi-Version & Multi-Encoding Listener)**:
  Terminates mTLS 1.3 and HTTPS on port 5696, supporting TTLV, JSON, and XML in KMIP 2.0, 2.1, and 3.0.
- **`REQ-072-005` (Metadata Cache)**:
  In-memory read cache with TTL and immediate invalidation on any attribute/state mutation.
- **`REQ-072-006` (Cloud Provider Abstraction)**:
  `CloudKmsProvider` async trait defines contract for key lifecycle, tag retrieval, and cryptographic operations.
- **`REQ-072-007` (CI Selective Routing)**:
  Changes to `proxy/**` run dedicated proxy workflows without triggering the full client library CI matrix.
