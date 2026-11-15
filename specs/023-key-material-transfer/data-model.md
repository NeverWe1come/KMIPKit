# Data Model: KMIP 2.1 Key Derivation and Transfer

This design composes with existing TTLV, message/batch, object/attribute, Cryptographic Parameters, operation-data, common result, and zeroizing secret-value models.

## Shared entities

| Entity | Model and constraints |
|---|---|
| Operation request | One typed ordered request per operation table; absent, empty, and zero-valued fields remain distinct. |
| Operation response | Common result plus operation success payload; retain the complete source response item for generic access and unknown fields. |
| KMIP result | Existing status, reason, allowed message, Pending/correlation, extensions, and delivery evidence. Result reasons remain open values. |
| Sensitive material | Key/secret/object/derivation bytes and wrapping data use the existing secret owner; never expose values via Debug, Display, or errors. |
| Cryptographic Parameters | Reuse the existing ordered generic structure. Preserve unknown children/raw enum values and caller-selected parameter presence. |
| Operation Data | Reuse existing operation-data representation for the KMIP §7.9 values accepted by Derive Key. Do not transform data locally. |
| Split Key reference context | An ordered local pair of one transmitted Unique Identifier occurrence and its known Split Key Threshold. One context is required per transmitted occurrence. The threshold is validation metadata and is not serialized. |

## Operation shapes

| Operation | Request and success shape | Source and local validation |
|---|---|---|
| Derive Key | Required source Unique Identifier(s), requested result attributes, and operation-specific derivation fields; response contains the resulting object's identifier/payload fields in Table 206. | §6.1.14 Tables 205–207 and §7.12 Table 364. Do not use ID Placeholder; validate caller-provided source usage/length/algorithm/attribute/parameter requirements; do not inspect remote key metadata or derive locally. |
| Export | Optional identifier, format, wrapping, compression, and wrapping specification; success returns Object Type, identifier, Attributes, and object form. | §6.1.18 Tables 217–219. Preserve the exact caller format/wrap options and response object as sensitive data; no local transformation. |
| Import | Required identifier, Object Type, Attributes, and Any Object; optional Replace Existing and conditional Key Wrap Type; success returns the new identifier. | §6.1.25 Tables 238–240. Always carry supplied attributes; set replacement only by explicit caller intent; preserve wrapped representation; no local unwrapping/import. |
| Join Split Key | Required Object Type and an ordered list of Split Key references, optional Secret Data Type/Attributes/masks; success returns the result identifier. | §6.1.27 Tables 244–246. Preserve repeated identifier entries. Require one caller-known threshold context per entry, then validate entry count >= maximum context threshold before sending. Never query remote attributes. |

## Join Split Key validation context

The request stores each `SplitKeyReference` as the exact Unique Identifier to transmit plus the caller-supplied threshold known for that referenced key. Serialization emits only each reference's identifier, in original order, including repeated identifier values. Validation counts identifier entries as transmitted (it does not deduplicate values), computes the maximum supplied threshold, and requires the entry count to be at least that maximum. This guarantees the count meets every supplied per-key threshold even when thresholds differ. It does not impose threshold equality. Missing reference context or an insufficient count fails locally with NotSent delivery evidence; no network lookup is performed.

The context is derived from KMIP §6.1.27's client SHALL and Table 244's repeated Unique Identifier field, as approved by the maintainer on 2026-10-11. No new normative requirement ID is created; existing IDs KMIPKIT-REQ-SPEC-6.1.27-001 and -005 remain authoritative.

## Invariants

1. Serialization preserves the exact pinned table field order, explicit option choices, repeated fields, and caller data.
2. Local errors are limited to payload structure and approved client obligations. No server-side policy/state is inferred.
3. Success responses preserve unknown structurally valid fields; KMIP failure/pending results are not converted into fabricated success payloads.
4. Derive Key, Export, Import, and Join Split Key submit one explicit request each; no retry, poll, remote preflight, local crypto, or implicit material transformation.
5. Any sensitive copies owned by KMIPKit remain redacted and zeroized using current shared types.
6. Every stable requirement ID and mandatory/conditional payload table row maps to implementation and verification in [traceability.md](traceability.md).
