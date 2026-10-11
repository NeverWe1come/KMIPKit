# Public Rust Contract: KMIP 2.1 Key Derivation and Transfer

This additive contract follows the existing typed protocol and synchronous client conventions. Request/response fields and conditions follow [spec.md](../spec.md) and its cited OASIS tables.

## Protocol types

| Operation | Protocol types | OASIS source |
|---|---|---|
| Derive Key | `DeriveKeyRequest`, `DeriveKeyResponse`, `DeriveKeyError` | §6.1.14 Tables 205–207; §7.12 Table 364 |
| Export | `ExportRequest`, `ExportResponse`, `ExportError` | §6.1.18 Tables 217–219 |
| Import | `ImportRequest`, `ImportResponse`, `ImportError` | §6.1.25 Tables 238–240 |
| Join Split Key | `JoinSplitKeyRequest`, `JoinSplitKeyResponse`, `JoinSplitKeyError` | §6.1.27 Tables 244–246 |

Each request has an explicit typed builder over existing shared protocol/object types. Each response validates operation identity, common result, success field cardinality/type, and retains the source response item for generic TTLV access. Each local model error is sanitized. Server error alternatives use the shared open KMIP result.

## Typed client calls

Add typed `ClientRequest` variants and explicit synchronous client conveniences following the existing method/options pattern:

```rust
client.derive_key(request, limits)
client.export(request, limits)
client.import(request, limits)
client.join_split_key(request, limits)
```

Options-bearing forms follow neighboring methods. Use `Client::execute` and `ClientBatch` when the caller needs multiple operations, batch options, or an explicit asynchronous indicator. Each method sends one explicit request and returns the common per-item result; it does not retry, poll, or perform another KMIP operation.

## Operation-specific requirements

- **Derive Key**: require source identifier(s), explicit new-object Attributes, and all §6.1.14/§7.12 conditions. Preserve caller data/method/parameters and open algorithm/derivation values. Never substitute ID Placeholder or derive material locally.
- **Export**: preserve optional format/wrapping/compression choices exactly. Keep returned object and attributes sensitive. Never perform wrap, unwrap, compression, or format conversion locally.
- **Import**: require Object Type, caller-supplied Attributes, and Any Object. Replacement is set only when caller intent is true; preserve the conditional Key Wrap Type and wrapped object form. Do not import or unwrap locally.
- **Join Split Key**: model a `SplitKeyReference` as a Unique Identifier plus caller-known Split Key Threshold. Require one reference context for each identifier occurrence. Serialize only identifiers, exactly in order with repetitions intact. Before sending, require identifier-entry count (including repeated entries and values) >= the maximum supplied threshold. Different thresholds are permitted; maximum satisfies each minimum. Missing/misaligned context or insufficient count returns a local NotSent error. The threshold is never a wire field and is never fetched implicitly.

## Shared security and parity behavior

- Reuse KMIPKit-owned zeroizing bytes and redact all sensitive values from formatting/errors.
- Preserve unknown tags, enumeration values, bitmask bits, child order, extensions, status/reason/message, Pending, correlation, and delivery state.
- Rust remains canonical. C, Java, and Python declarations and parity fixtures follow KMIPKIT-0043's manifest mappings T046, T047, T050, and T052 once available; this contract does not invent four independent signatures.
