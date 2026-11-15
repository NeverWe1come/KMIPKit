# Public Rust Contract: KMIP 2.1 Object Lifecycle and Certificates

This additive contract follows existing `kmipkit-protocol` request/response and `kmipkit-client` typed execution conventions. Exact payload fields, order, and conditional rules are those of the OASIS tables named below and [spec.md](../spec.md).

## Protocol types

Each operation exposes a dedicated request, success response, and sanitized model error from `kmipkit-protocol`:

| Operation | Types | Source |
|---|---|---|
| Certify | `CertifyRequest`, `CertifyResponse`, `CertifyError` | §6.1.6, Tables 179–181 |
| Check | `CheckRequest`, `CheckResponse`, `CheckError` | §6.1.7, Tables 183–185 |
| Obtain Lease | `ObtainLeaseRequest`, `ObtainLeaseResponse`, `ObtainLeaseError` | §6.1.35, Tables 268–270 |
| Re-certify | `RecertifyRequest`, `RecertifyResponse`, `RecertifyError` | §6.1.45, Tables 298–302 |
| Re-key | `RekeyRequest`, `RekeyResponse`, `RekeyError` | §6.1.46, Tables 303–307 |
| Re-key Key Pair | `RekeyKeyPairRequest`, `RekeyKeyPairResponse`, `RekeyKeyPairError` | §6.1.47, Tables 308–312 |
| Re-Provision | `ReProvisionRequest`, `ReProvisionResponse`, `ReProvisionError` | §6.1.48, Tables 313–315 |
| Register | `RegisterRequest`, `RegisterResponse`, `RegisterError` | §6.1.43, Tables 291–294 |
| Revoke | `RevokeRequest`, `RevokeResponse`, `RevokeError` | §6.1.44, Tables 295–297 |

Constructors/consuming setters expose every table-defined field, preserve optional absence, and do not invent defaults. Response conversion validates operation identity, common result, success-payload cardinality, and field types; it retains the source response item for generic inspection. Server failure reasons remain open common result values.

## Typed client calls

Add matching `ClientRequest` variants/conversion helpers and synchronous `Client` convenience methods following the existing method/options pattern:

```rust
client.certify(request, limits)
client.check(request, limits)
client.obtain_lease(request, limits)
client.recertify(request, limits)
client.rekey(request, limits)
client.rekey_key_pair(request, limits)
client.reprovision(request, limits)
client.register(request, limits)
client.revoke(request, limits)
```

Each returns the existing per-item response/result type; options-bearing variants follow current client conventions. `Client::execute` and `ClientBatch` remain the route for caller-selected batch headers, multiple operations, and explicit asynchronous indicators. Calls make no automatic follow-up or retry.

## Operation behavior

- **Certify**: preserve certificate request type and value separately; validate that the public key's Unique Identifier is explicitly supplied when Certificate Request is omitted (KMIPKIT-DEC-013), validate conditional type requirements and one-certificate scope. Never generate or parse a certificate locally.
- **Re-certify**: preserve certificate request type and value separately; validate conditional type requirements and one-certificate scope. Never generate or parse a certificate locally.
- **Check**: preserve policy request/response values and document caller-selected Batch Order and Stop/Undo options; do not alter caller batch options.
- **Obtain Lease**: expose Lease Time and Last Change Date without timekeeping or lifecycle state.
- **Re-key / Re-key Key Pair**: retain Offset and separate attribute/mask groups; do not query objects or synthesize links.
- **Re-Provision**: accept Certificate Request, Certificate, both optional fields, or no parameters for server-created credentials. The standard does not prohibit the two Table 313 fields from co-occurring. Table 315's caption follows the approved discrepancy disposition.
- **Register**: retain Object Type, Attributes, object representation, and masks without implicit import.
- **Revoke**: preserve open reason values; enforce the source-backed Compromise Occurrence Date prohibition and retain its SHOULD recommendation without turning it into an unapproved hard error.

## Security and language boundary

- Shared contracts preserve result status/reason/message, Pending, correlation, extensions, and delivery evidence.
- CSR, certificate, key/object, credential, and raw body values are redacted. Existing owned-secret storage zeroizes KMIPKit-owned buffers.
- Unknown enum values, tags, bitmask bits, and extension children remain lossless.
- Rust is the canonical typed contract. C, Java, and Python signatures and generated declarations follow KMIPKIT-0043 after its manifest mapping is available; this file does not create a parallel binding contract.
