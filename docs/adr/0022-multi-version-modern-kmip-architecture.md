# ADR-0022: Multi-Version Modern KMIP Architecture

**Status**: Accepted by maintainer instruction on 2026-11-07
**Date**: 2026-11-07
**Decision owner**: KMIPKit maintainer
**Updates**: Version scope of [ADR-0002](0002-kmip-21-release-scope.md) and [ADR-0018](0018-multi-encoding-1-0-scope.md)
**Specification**: [KMIPKIT-0069](../../specs/069-multi-version-foundation/spec.md)

## Context

The initial release scope in ADR-0002 focused strictly on OASIS KMIP 2.1. However, modern cryptographic key management infrastructure across enterprise environments requires seamless interoperability across the modern KMIP standard family:
- **KMIP 2.0**: The first modern version introducing the direct `Attributes` model and structured message encodings.
- **KMIP 2.1**: The stable OASIS standard baseline of `KMIPKit`.
- **KMIP 3.0**: The upcoming standard introducing System Objects (`User`, `Group`, `Credentials`), typed links, granular counters, and Post-Quantum Cryptography (PQC) KEM operations (`Encapsulate`, `Decapsulate`).

Simultaneously, legacy **KMIP 1.x (1.0–1.4)** relies on obsolete `Template-Attribute` dynamic string dictionaries abandoned by OASIS in 2019. Modern applications deployed in 2026+ do not implement KMIP 1.x, and supporting 1.x in the client library would introduce disproportionate complexity and security surface.

## Decision

1. **Modern KMIP Client Scope**:
   `KMIPKit` officially targets modern KMIP versions: **KMIP 2.0**, **KMIP 2.1**, **KMIP 3.0 Draft/Standard**, and future catalogued versions. Legacy KMIP 1.x (1.0–1.4) is explicitly excluded from the client library.

2. **Test Server Backward Compatibility Preserved**:
   `crates/kmipkit-test-server` retains its existing `v1_compat.rs` translation layer to allow testing against legacy artifacts without compromising the clean architecture of the production client.

3. **Zero-Breakage Principle**:
   - The default protocol version in `ClientConfiguration` is unconditionally `ProtocolVersion::V2_1`.
   - Existing code calling `Client::execute` or operation builders without specifying a version continues to execute KMIP 2.1 without any behavioral or ABI changes.

4. **Two-Tiered Version Selection & Override**:
   - **Client Creation (`ClientConfiguration`)**: Configures the client's default version via `.with_protocol_version(version)` or `.with_kmip_version(version)`.
   - **Per-Request / Per-Batch Override (`RequestOptions` and `ClientBatch`)**: Callers may override/force the effective protocol version for a specific execution using `.with_protocol_version(version)` or its explicit alias `.force_kmip_version(version)`. The override applies solely to that exchange and does not mutate the client's configured default.

5. **Strict Catalog-Enforced Pre-Send Validation (`VersionGate`)**:
   - Version compatibility is strictly enforced against compiled normative catalog spans (`VersionSpan { introduced, deprecated, removed }`).
   - If an operation, attribute tag, operation structure, or enumeration variant is unsupported in the effective protocol version, `Client::execute` rejects the request locally before transport I/O, returning a typed `ClientError` with `RequestDeliveryState::NotSent`.
   - Automatic version downgrades or re-transmissions are strictly prohibited by `AGENTS.md` §8.

6. **Four-Language Parity**:
   Equivalent version configuration, per-request override, and typed error inspection are provided across Rust, C, Java, and Python.

## Consequences

- The KMIPKit Constitution is amended to Version 1.3.0.
- `ProtocolVersion` is expanded with canonical constants `V2_0`, `V2_1`, `V3_0`, ordering, and catalog validation.
- `DiscoverVersionsRequest` supports multi-version preference lists.
- Public API manifests and bindings are updated in an additive, non-breaking manner.
