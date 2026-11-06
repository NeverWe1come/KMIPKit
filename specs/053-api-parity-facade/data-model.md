# KMIPKIT-0053 Data Model

This feature adds foreign adapters over existing Rust models; it introduces no new wire data.

| Entity | Foreign contract |
|---|---|
| Typed Attribute | Exact tag/value alternative, generic TTLV conversion, source-defined repetition, safe borrowed/owned access. |
| AttributeSet / CurrentAttribute / NewAttribute | Ordered storage with typed access; unknown and repeated Items are retained. |
| ManagedObject | Open dispatch for nine known object tags plus future/generic fallback. |
| KeyBlock / KeyValue / KeyMaterial | Distinguish absent, plaintext Structure, wrapped Byte String, transparent forms and opaque extensions; no crypto/autodetection. |
| KeyWrappingData | Preserve exact fields, nested key information, optionality and order. |
| CryptographicParameters | Eighteen Table 59 fields, future enumerations, extensions and generic bridge. |
| Authentication | One caller-owned value per Request Message; never client-global. |
| ValidationReport / ValidationIssue | Stable code/path/severity/reference metadata only; no values/raw bytes. |
| Client operation result | One of 56 supported operation identities, options, delivery, typed result and async correlation. |
| Response Structure | KMIPKIT-0052 ordered tree; borrowed Rust view or owned foreign copy with explicit close/release. |

Unknown values remain representable without narrowing to closed enums. Views and owners follow documented language lifetimes.
