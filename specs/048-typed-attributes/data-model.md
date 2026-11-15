# KMIPKIT-0048 Data Model

## Attribute definition

| Field | Source | Invariant |
|---|---|---|
| Stable trace ID | `gap-matrix.md` | Derived from the source tag; never reused |
| Marker type | Catalog `element_id` | Unique Rust type name; does not imply server support |
| TTLV tag | Catalog tag parent, or §4.60 special Vendor Attribute tag | Immutable and exactly source-assigned |
| Value category | Catalog `source_encoding` and OASIS table | Scalar or Structure category; validated on construction/read |
| Multiplicity | Source policy table in catalog | `AttributeSet` retains all permitted instances in input order |

## Owned and borrowed values

`Attribute<D>` owns exactly one `Item` whose tag is `D::TAG` and whose value
matches `D`'s source-defined TTLV category. `AttributeView<'a, D>` borrows that
Item and returns the category-specific borrowed view. Structure views borrow
the original child tree; scalar byte/text views borrow their original buffers.
Enumeration values remain raw `u32`. Unique Identifier is an open union of
Text String, Enumeration, and Integer.

## Collection and wrapper semantics

`AttributeSet` remains an ordered vector of direct Items. Typed insertions use
the existing `try_push` validation and retain all duplicates just like generic
insertion. Typed queries return every matching occurrence in original order;
they do not collapse multiple values into a map. Existing operation-level
cardinality validation rejects source-prohibited duplicates when an operation
requires that check. `CurrentAttribute` and `NewAttribute` each contain exactly
one direct Item, as specified by KMIP v2.1 §§5.6–5.7, Tables 162–163.

## Exclusion

Certificate Attributes is not modeled as a direct Item because its §4.6/Table
39 source record has no assigned Attribute tag or TTLV encoding. This is a
source-derived scope exclusion, not an arbitrary catalog filter.
