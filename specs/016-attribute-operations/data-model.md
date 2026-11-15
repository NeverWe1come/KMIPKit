# Data Model: KMIP 2.1 Attribute Operations

All structures below are client-side representations of KMIP 2.1 TTLV payloads. They do not mirror or mutate remote server state.

## Shared attribute values

| Entity | Fields and constraints | Source |
| --- | --- | --- |
| `AttributeReference` | Either a name reference containing required Vendor Identification and required Attribute Name, or a KMIP Attribute Tag carried as an Enumeration. Preserve both exact text values and unknown raw Enumeration values. | §5.5, Table 161 |
| `CurrentAttribute` | A wrapper containing exactly one direct generic TTLV Item representing the existing §4 Object Attribute. Its tag identifies the attribute; a Vendor Attribute Item carries its Vendor Identification in its structure value. | §5.6, Table 162 |
| `NewAttribute` | A wrapper containing exactly one direct generic TTLV Item representing the supplied §4 Object Attribute. Its tag identifies the attribute; a Vendor Attribute Item carries its Vendor Identification in its structure value. | §5.7, Table 163 |
| `AttributeSet` | The accepted KMIPKIT-0014 ordered collection of direct §4 Object Attribute Items, reused for the Get Attributes response's Attributes structure. It is not an Attribute Reference and does not introduce a name/value pair. | §§5.1–5.4; KMIPKIT-0014 |
| `StandardAttributePolicy` | Internal source-backed catalog metadata for each of the 62 named §4 attribute-rule tables. Each catalog record retains verbatim `source_initially_set_by`, `source_modifiable_by_client`, and `source_deletable_by_client` text, exact `source_always_required`, policy-table identifiers, operation restrictions, conditional rules, and source references. The pinned deterministic tool generates the runtime lookup from the 61 records with assigned parent Tags and CI checks it for drift. §4.6 `Certificate Attributes` / Table 40 remains fully represented in the canonical catalog, but §11.56 assigns no Tag to that record; do not invent a key or emit it in the tag-keyed lookup. Apply generated policy using a direct attribute Item tag or an Attribute Reference's tag form. A name-form reference does not identify a standard attribute unless a source-backed mapping defines that exact vendor/name pair. Unknown names/tags receive no inferred policy. Only an unconditional prohibition applicable to the requested operation yields local rejection; qualified entries are not collapsed into booleans, and rules requiring remote state remain server-authoritative. |
| `VendorAttributePolicy` | Separate value-aware source policy on the §4.60 `Vendor Attribute` element, which has no §4 attribute-rule table. Its canonical `source_value_policies` entry records the exact prose and Table 150 reference, the `Vendor Identification == "y"` predicate, and prohibited client actions. Inspect direct Vendor Attribute Items in Current/New Attribute values and name-form Attribute References that include Vendor Identification. A matching `y` is rejected for the source-prohibited client operations; tag-form references do not expose the identifier, so no inference is made. Other identifiers, including `x`, do not match this rule. | §4.60 prose; Table 150 |
| `MutationValidation` | Result of checking supplied attribute identity from direct Item tags or Attribute Reference variants against standard and Vendor Attribute policy. Inspect Vendor Identification only when carried by a name-form reference or a supplied Vendor Attribute value. Outcomes are allowed, locally prohibited (`NotSent`), or source condition dependent on remote state. It never performs a preflight request. | §§4, 5.5–5.7, 6.1.2, 6.1.3, 6.1.13, 6.1.34, 6.1.51; §4.59–4.60 |
| `AdjustmentType` | Increment (1), Decrement (2), Negate (3), or an unknown raw Enumeration value. Unknown values remain representable, but outbound requests accept only the assigned values or §11.1 Table 429 extension range `0x80000000–0x8FFFFFFF`; reserved values are rejected. | §11.1, Tables 428–429; KMIPKIT-0004/ADR-0010 |
| `AdjustmentValue` | Optional generic value sent with Adjustment Type. If omitted, Increment/Decrement use the §11.1 Table 428 parameter default: 1 for numeric types, 1 second for Date Time, and 1 microsecond for Date Time Extended. The client does not calculate the resulting value. If the target has no value, §6.1.3 defines the assumed starting value as 0 for numeric types/intervals and false for Boolean; other types error. | §6.1.3; §11.1, Table 428 |

## Operation request and response entities

| Operation | Request fields | Successful response fields | Source |
| --- | --- | --- | --- |
| Add Attribute | Optional Unique Identifier; required New Attribute | Required Unique Identifier | §6.1.2, Tables 167–168 |
| Adjust Attribute | Optional Unique Identifier; required Attribute Reference and Adjustment Type; optional Adjustment Value | Required Unique Identifier | §6.1.3, Tables 170–171 |
| Delete Attribute | Optional Unique Identifier; optional Current Attribute and Attribute Reference | Required Unique Identifier | §6.1.13, Tables 202–203 |
| Get Attributes | Optional Unique Identifier; zero or more Attribute References with no duplicate reference | Required Unique Identifier; `AttributeSet` of returned direct §4 attribute Items, preserving instances and order | §6.1.20, Tables 223–224 |
| Get Attribute List | Optional Unique Identifier only | Required Unique Identifier; one or more repeated Attribute References | §6.1.21, Tables 226–227 |
| Modify Attribute | Optional Unique Identifier; optional Current Attribute; required New Attribute | Required Unique Identifier | §6.1.34, Tables 265–266 |
| Set Attribute | Optional Unique Identifier; required New Attribute | Required Unique Identifier | §6.1.51, Tables 322–323 |

Every operation result also carries the existing KMIP Result Status and optional Result Reason and Result Message contract. A Pending result is represented through the accepted asynchronous result contract and does not cause hidden polling.

## State and validation rules

- Unique Identifier omission is preserved where allowed by the operation table so the server applies KMIP ID Placeholder behavior.
- Add, Adjust, Delete, Get, Modify, and Set remain different operation variants; request construction never rewrites one operation into another.
- Get Attributes rejects duplicate Attribute References. If no references are provided, the server returns all attributes. If a selected attribute has multiple instances, all instances are returned. Missing selected attributes are omitted; the client fabricates no entries. The response reuses KMIPKIT-0014's `AttributeSet` of direct generic Items.
- Get Attribute List request contains only the optional Unique Identifier; per §6.1.21 prose the server returns all attribute names, and the required response Attribute Reference entries are retained in wire order, including repeats.
- Delete preserves both optional selectors independently. Attribute Reference with no Current Attribute selects all instances per §6.1.13. If both selectors are absent, the client sends the request shape unchanged and surfaces the server result. When a supplied selector identifies a standard attribute that cannot be changed/deleted or is required to retain a value, the client returns a payload-free local error with delivery state `NotSent` before transport.
- Before a mutation exchange, the client checks each target represented by a direct Item tag or Attribute Reference. For Add or Modify with New Attribute identity `Usage Limits`, it always rejects locally: §7.40 Table 392 requires Count, while §4.59 prohibits setting or modifying Count through these operations. It checks §4.60's Vendor Attribute rule when Vendor Identification `y` is present in a supplied direct Item value or name-form Attribute Reference. It cannot infer that identifier from tag-form references. Conditional rules that depend on remote object state are not guessed and do not trigger an extra exchange; other server results are preserved. Unknown names/tags and vendor values without the explicit `y` condition remain lossless and have no inferred mutability. Every local rejection is payload-free and has delivery state `NotSent`.
- Modify with no Current Attribute preserves omission; the server resolves a single instance and reports ambiguity for multiple instances. The client does not choose an instance.
- Set and Adjust preserve server-side single-instance and absent-value semantics. No local read/modify/write occurs.
- Individually assigned tags and tags accepted by the KMIPKIT-0004/ADR-0010 §11.56 allocation gate remain available for generic values. Raw tags failing that gate cannot become public Items or be encoded; unknown Enumeration values remain representable generically, while typed outbound Adjustment Type validates the assigned or `0x80000000–0x8FFFFFFF` extension range.

## Relationships

- Each of the seven request structures selects one operation and becomes one request batch item.
- Each successful response payload is paired with its operation result in the existing response batch model.
- AttributeReference, CurrentAttribute, NewAttribute, and direct attribute Items are shared by the seven operation models but are not interchangeable. KMIPKIT-0014's `AttributeSet` is reused where the OASIS response contains a set of direct §4 Items; it is not used to represent a name/value pair or Attribute Reference.
- Result Status, Result Reason, Result Message, Pending, and delivery-state are shared contracts owned by existing protocol/client foundations.
