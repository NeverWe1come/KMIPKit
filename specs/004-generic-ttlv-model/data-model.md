# Data Model: KMIP Generic TTLV Value Model

## Entities

### RawTag

- **Representation**: `u32` newtype.
- **Invariant**: value is in `0x000000..=0xFFFFFF`; constructors reject larger values without truncation.
- **Meaning**: preserves 24 bits only. It does not assert that the tag is allocated or legal on the wire.

### Tag

- **Representation**: distinct newtype created only by catalog-backed allocation checking.
- **Invariant**: allocation decision is accepted by the generated table and the policy recorded in accepted ADR-0010.
- **Meaning**: allocation checked only. It does not assert valid TTLV framing, extension semantics, Structure schema, or operation meaning.

### ItemType

- **Variants**: `Structure`, `Integer`, `LongInteger`, `BigInteger`, `Enumeration`, `Boolean`, `TextString`, `ByteString`, `DateTime`, `Interval`, `DateTimeExtended`.
- **Invariant**: each variant maps to exactly one KMIP 2.1 §11.23 item type. No closed semantic Enumeration enum is introduced.

### SecretValue

- **Representation**: private zeroizing wrapper around a separately boxed value payload, with no public payload formatter, serializer, `Clone`, or `Copy` implementation.
- **Payload forms**: signed 32-bit Integer; signed 64-bit Long Integer; exact Big Integer Item Value octets; unsigned 32-bit Enumeration; Boolean; unnormalized Unicode String; Byte String; signed 64-bit Date Time; unsigned 32-bit Interval; signed 64-bit Date Time Extended; ordered child items for Structure.
- **Invariants**: no type coercion; scalar widths/signedness preserved; text and bytes unchanged; Big Integer octets are not mathematically normalized; Structure children preserve caller order and duplicates.
- **Drop**: current KMIPKit-owned secret storage is zeroized while live before release, recursively for Structures. Structure child-vector growth moves only item metadata and secret-box pointers; it does not relocate payloads. Structure nesting is capped at 64 levels. Caller/runtime copies, temporary stack/register copies, and prior allocations no longer owned are outside the guarantee.

### ValueView

- **Representation**: borrowed, non-owning view supplied only to an explicit closure-scoped exposure method.
- **Invariant**: the returned result cannot retain the borrow beyond the callback. Callers can deliberately copy/format observed data; those copies are outside KMIPKit's zeroization guarantee.

### Item

- **Fields**: allocation-checked `Tag`; `SecretValue`.
- **Invariant**: Item Type is derived from the value variant. It cannot be supplied independently and therefore cannot mismatch the payload. The in-memory item does not contain a wire length or padding.

### Structure

- **Fields**: ordered sequence of child `Item`s.
- **Invariant**: preserve exact order and repeated tags; empty structures are representable. At most 64 Structure levels are accepted. An append that would exceed the bound fails without modifying the existing tree. Operation-specific ordering/cardinality and protocol semantics are not checked.

### ModelError

- **Forms**: invalid Raw Tag width, prohibited/unallocated Tag, and any local construction/inspection failure required by the implementation.
- **Invariant**: formatting excludes payload contents and raw KMIP bodies. The type stays in `kmipkit-ttlv` and does not depend on higher protocol/client crates.

## Tag allocation states

The generated lookup consumes exact individual catalog records and aggregate ranges. Under the accepted ADR-0010 policy, an exact individual record wins over a matching aggregate range; assigned entries and the §11.56 Extensions range are accepted, while Reserved and unused values are rejected. This is a KMIPKit project policy, not an OASIS interpretation.

```text
u32 input
  └─ RawTag (reject > 0xFFFFFF)
       └─ allocation check against generated catalog
            ├─ accepted → Tag → Item
            └─ rejected → ModelError (RawTag remains available)
```

## Lifecycle

1. The caller constructs a `RawTag` and requests allocation checking.
2. On acceptance, a `Tag` may be paired with a value variant to construct an `Item`.
3. An application may nest items in a Structure through 64 Structure levels; an insertion beyond that limit fails without changing the existing tree.
4. Explicit exposure borrows a view for the callback duration. Callers may create copies intentionally.
5. Dropping a value clears current KMIPKit-owned payload storage before release, including nested values.

Wire parsing/encoding is a separate lifecycle owned by the future TTLV codec specification.
