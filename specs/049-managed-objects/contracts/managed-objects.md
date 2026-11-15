# KMIPKIT-0049 Public Contract

This document fixes the observable Rust API behavior before implementation.
Final names may follow local Rust naming conventions, but these contracts may
not change without an updated approved specification.

## Managed objects

- Provide typed owners and borrowed views for Certificate,
  CertificateRequest, OpaqueObject, PgpKey, PrivateKey, PublicKey, SecretData,
  SplitKey, and SymmetricKey.
- Provide an open `ManagedObject` conversion that associates each known
  object-specific Structure Item/tag with its Object Type and returns an
  unknown/generic representation for future Object Type values. Structure
  tags come from the pinned KMIP tag catalog and remain distinct from the
  Object Type Enumeration values.
- Every typed owner converts to the existing ordered generic `Item`; borrowed
  inspection provides access to the same raw structure without an implicit
  payload copy.
- Register and Import typed helpers pass this object-specific tagged Item
  directly in the existing wire position; they add no wrapper and keep
  existing public generic request constructors and response accessors.

## Key structures

- `KeyBlock` preserves Key Format Type and models optional Key Value, Key
  Compression Type, Cryptographic Algorithm, Cryptographic Length, and Key
  Wrapping Data.
- Key Value distinguishes absent, plaintext structured data, and wrapped
  Byte String. Plaintext Key Value contains required Key Material and optional
  Attributes.
- `KeyMaterial` supports the nine transparent structures in §§3.4–3.12 plus
  exact byte/Structure fallback for other and extension formats.
- `KeyWrappingData` models the source fields and nested key-information
  structures but performs no cryptographic operation.
- Builders emit source-defined order regardless of method call order.

## Errors, formatting, and memory

- Conversion errors identify the category (unexpected tag/type, absent
  required field, invalid conditional combination, or local TTLV limit) and
  never retain raw values, key material, or KMIP bodies.
- `Debug` output is structural metadata only.
- Owned secret-bearing payloads use the recursively zeroizing TTLV `Value`
  owner; borrowed views cannot outlive their owner.
- Unknown tags, future Enumeration values, and vendor extension children
  remain accessible through generic Items and generic fallback values.

## Non-contracts

No crypto operation, format autodetection, hidden copy, server policy check,
profile conformance claim, or C/Java/Python API is introduced here. Typed
response readers and nested response `.raw()` accessors belong to 0052.
