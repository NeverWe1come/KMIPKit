# KMIPKIT-0048 Research and Decisions

## Source-of-truth review

The checked-in normative catalog contains 63 records with `kind=attribute` in
client 1.0 scope. Comparing each to the immutable KMIP Specification v2.1
shows that 62 records have a direct or special TTLV tag and a value encoding.
The exception is `Certificate Attributes` (§4.6, Table 39): the source describes
items included inside a certificate, and the catalog has neither a direct tag
nor a `source_encoding`. It is therefore excluded from typed direct-Item
construction. The separate §§5.1–5.7 Attribute Structures are catalogued as
`attribute_structure`, not among the 63 §4 attributes, and remain the wrappers
already represented by `AttributeSet`, `AttributeReference`,
`CurrentAttribute`, and `NewAttribute`.

## API shape

**Decision**: Use one generated marker type per direct attribute and a shared
generic owned `Attribute<D>` plus borrowed `AttributeView<D>`. Definitions
provide the immutable tag and typed value conversion. `AttributeSet` exposes
generic insertion and ordered repeated lookup parameterized by the marker;
Current/New wrappers provide generic typed construction and inspection.

**Rationale**: This models all 62 definitions without creating hundreds of
nearly identical methods, keeps the tag/type relationship in the Rust type,
and makes repeated-value behavior explicit. It leaves existing generic
`Item` entry points intact.

## Value mapping

- TTLV scalar encodings map to their existing owned `Value` type and borrowed
  `ValueView` counterpart.
- Structure encodings map to the existing lossless `Structure` and
  `StructureView`; this feature does not create new nested schemas.
- RNG Parameters is encoded as its source-defined Structure category.
- Unique Identifier uses an open value representation for its permitted Text
  String, Enumeration, or Integer alternatives.
- Enumeration is kept as raw `u32`, so future and vendor values remain valid.
- `Date Time` and `Date-Time` catalog spellings normalize to the same TTLV
  DateTime category; generated output records the source spelling for audit.

## Compatibility and safety

Generic constructors, `Item` accessors, encoded output, and policy checks stay
unchanged. Typed errors are fixed categories and never format an attribute's
value. Typed reads borrow the existing Item to avoid copies and preserve
Structure child order, repetitions, and unknown children.

## Dependencies and exclusions

The 0047 named-value generation provides raw-preserving protocol constants but
does not constrain attribute Enumeration values. Detailed managed-object and
key-material child builders belong to 0049. C/Java/Python APIs are implemented
in 0053 after the Rust contract is stable. No new dependency or ADR is needed.
