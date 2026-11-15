# Specification: Standalone Multi-Version KMIP Test Server with File-Based Storage (KMIPKIT-0065)

## 1. Overview and Purpose

This specification governs the architecture and implementation of `crates/kmipkit-test-server`, a lightweight, standalone in-memory/file-persisted KMIP test server supporting OASIS KMIP versions **1.0, 1.1, 1.2, 1.3, 1.4, 2.0, and 2.1** across **TTLV, JSON, and XML** encodings over **Raw TLS 1.3** and **HTTPS/1.1**.

The server enables hermetic local and CI integration testing for polyglot client bindings (Rust, C, Java, Python) without external HSM or Docker dependencies.

Key architectural requirements:
1. **Multi-version interoperability:** Responds to KMIP 1.0 through 2.1 requests dynamically using `ProtocolVersion` extracted from the message header.
2. **Auto-PKI generation:** Automatically provisions an in-memory CA root, server TLS certificate, and client mTLS keypair upon startup, exporting PEM assets to `--export-pki`.
3. **File-based JSON storage:** Persists managed cryptographic objects as human-readable JSON files in `--data-dir ./kmip-data` (`kmip-data/objects/<uuid>.json`).
4. **Realistic cryptographic simulation:** Utilizes `aws-lc-rs` to execute real cryptographically secure key generation, symmetric encryption/decryption, digital signatures, and HMAC validation.
5. **Zero client crate modifications:** Crate `kmipkit-test-server` is an internal test package (`publish = false`) that does not modify public API surface (`0%` drift in `specification/api/public-api.json`).

## 2. Normative References

- **OASIS KMIP 1.0, 1.1, 1.2, 1.3, 1.4, 2.0, 2.1**: Message formats, operations, attribute models, and error responses.
- **OASIS KMIP Profile §5.1 / §5.3**: Raw TTLV over TLS and HTTPS/HTTP 1.1 transport.
- **RFC 8446**: The Transport Layer Security (TLS) Protocol Version 1.3.
- **RFC 9112**: HTTP/1.1.
- **NIST SP 800-57 Part 1**: Key management lifecycle states (PreActive, Active, Deactivated, Destroyed).
- **KMIPKit `AGENTS.md`**: Architectural boundaries and security invariants.

## 3. Scope and Public API Impact

- **Public API change**: Exactly 0%. `specification/api/public-api.json` is completely unchanged.
- **New crate**: `crates/kmipkit-test-server` (`publish = false`).
- **Target workspace crates**: Reuses `kmipkit-ttlv` and `kmipkit-protocol` as read-only internal dependencies.

## 4. Acceptance Criteria

1. **`REQ-065-001` (Auto-PKI Startup)**:
   The server generates an autonomous root CA, server certificate, and client certificate/key upon start and exports valid PEM files to `--export-pki`.

2. **`REQ-065-002` (mTLS and HTTPS Transport)**:
   The server listens on the configured socket address (`--listen`), enforcing TLS 1.3 mutual authentication for Raw TLS (port 5696) and accepting `POST /kmip` for HTTPS (port 5697) across TTLV, JSON, and XML encodings.

3. **`REQ-065-003` (Multi-Version Protocol Support)**:
   The server inspects the `RequestHeader.ProtocolVersion` and handles KMIP 1.0, 1.1, 1.2, 1.3, 1.4, 2.0, and 2.1 requests, translating legacy 1.x `Attribute` / `TemplateAttribute` structures to and from canonical 2.1 representations.

4. **`REQ-065-004` (File-Based JSON Storage)**:
   Objects are persisted as structured, readable `.json` files in `<data-dir>/objects/<id>.json`. Survives server restarts unless `--clean` is supplied.

5. **`REQ-065-005` (Realistic Cryptographic Execution)**:
   Cryptographic operations (`Create`, `CreateKeyPair`, `Encrypt`, `Decrypt`, `Sign`, `SignatureVerify`, `MAC`, `MACVerify`, `Hash`, `RNGRetrieve`) execute actual cryptographic algorithms using `aws-lc-rs`.

6. **`REQ-065-006` (Standalone CLI)**:
   Binary `kmipkit-test-server` supports `--listen`, `--data-dir`, `--export-pki`, `--clean`, and `--print-env` CLI flags.
