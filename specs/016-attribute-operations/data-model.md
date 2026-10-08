# Data Model: KMIP 2.1 Attribute Operations

All structures below are client-side representations of KMIP 2.1 TTLV payloads. They do not mirror or mutate remote server state.

## Shared attribute values

| Entity | Fields and constraints | Source |
| --- | --- | --- |
| `AttributeEntry` | Exact Attribute Name text and one complete generic TTLV Item; preserve the Item tag/type/value, unknown names, vendor values, and wire order. Redact its value from Debug and errors. | §§4, 5.1, 5.7; KMIPKIT-0014 FR-014 |
| `AttributeReference` | Exact Attribute Name used to select an attribute; it is not an Attribute value. | §5.5, Table 161 |
| `CurrentAttribute` | One complete existing Attribute name/value pair selecting the value to modify or delete. | §5.6, Table 162 |
| `NewAttribute` | One complete Attribute name/value pair supplied for add, modify, or set. | §5.7, Table 163 |
| `StandardAttributePolicy` | Internal source-backed metadata for each of the 62 named §4 attribute-rule tables. Each catalog record stores the exact source cell text for `source_initially_set_by`, `source_modifiable_by_client`, and `source_deletable_by_client` verbatim (including qualifications), `source_always_required` as exact `Yes`/`No`, the exact second-table identifier as `source_policy_table`, and explicit `source_operation_restrictions` and `source_conditional_rules` lists (empty when none apply), each with source references. The canonical catalog is authoritative; a pinned deterministic tool generates the internal lookup and CI checks it for drift. Only an exact unconditional prohibition applicable to the requested operation yields a local rejection; qualified entries are never collapsed into booleans, and rules requiring remote state remain server-authoritative. This does not change the public `AttributeEntry` shape. Unknown/vendor names have no inferred policy.
| `VendorAttributePolicy` | Separate value-aware source policy on the 63rd catalog `attribute` element, the §4.60 `Vendor Attribute` wrapper, which has no §4 attribute-rule table. Its canonical `source_value_policies` entry has `source_refs`, `structure_table`, verbatim `source_text`, a `value_predicate` (`member_element_id`, `equals`, and `source_indicates_origin`), and `prohibited_client_operations`. It records §4.60's exact prose, Table 150 reference, the `Vendor Identification == "y"` predicate that identifies the server-created case, and the source-prohibited client actions without inventing a normative requirement ID for that prose. A supplied value with `y` is rejected for Add, Modify, Set, and Delete when the relevant value is present in the request. Modify checks both supplied Current Attribute and New Attribute values; Delete can inspect Current Attribute. Adjust and reference-only Delete are prohibited by the source rule but expose only an Attribute Reference, so this client cannot determine the vendor identifier and preserves the server result. Other values, including `x`, do not match this rule and receive no inferred local prohibition. | §4.60 prose; Table 150 |
| `MutationValidation` | Result of checking the operation's supplied Attribute Name(s) against `StandardAttributePolicy`: New Attribute for Add/Set; both Current Attribute and New Attribute for Modify; Attribute Reference for Adjust; every supplied Current Attribute and Attribute Reference for Delete; no inferred target when Delete selectors are absent. For the §4.60 exception, inspect Vendor Identification only when the corresponding Vendor Attribute value is present. Outcomes are allowed, locally prohibited (`NotSent`), or source condition dependent on remote state. It never performs a preflight request. | §§4, 6.1.2, 6.1.3, 6.1.13, 6.1.34, 6.1.51; §4.59–4.60 |
| `AdjustmentType` | Increment (1), Decrement (2), Negate (3), or an unknown raw Enumeration value. Unknown values remain representable, but outbound requests accept only the assigned values or §11.1 Table 429 extension range `0x80000000–0x8FFFFFFF`; reserved values are rejected. | §11.1, Tables 428–429; KMIPKIT-0004/ADR-0010 |
| `AdjustmentValue` | Optional generic value sent with Adjustment Type. If omitted, Increment/Decrement use the §11.1 Table 428 parameter default: 1 for numeric types, 1 second for Date Time, and 1 microsecond for Date Time Extended. The client does not calculate the resulting value. If the target has no value, §6.1.3 defines the assumed starting value as 0 for numeric types/intervals and false for Boolean; other types error. | §6.1.3; §11.1, Table 428 |

## Operation request and response entities

| Operation | Request fields | Successful response fields | Source |
| --- | --- | --- | --- |
| Add Attribute | Optional Unique Identifier; required New Attribute | Required Unique Identifier | §6.1.2, Tables 167–168 |
| Adjust Attribute | Optional Unique Identifier; required Attribute Reference and Adjustment Type; optional Adjustment Value | Required Unique Identifier | §6.1.3, Tables 170–171 |
| Delete Attribute | Optional Unique Identifier; optional Current Attribute and Attribute Reference | Required Unique Identifier | §6.1.13, Tables 202–203 |
| Get Attributes | Optional Unique Identifier; zero or more Attribute References with no duplicate reference | Required Unique Identifier; returned Attributes, with instances preserved | §6.1.20, Tables 223–224 |
| Get Attribute List | Optional Unique Identifier only | Required Unique Identifier; one or more repeated Attribute References | §6.1.21, Tables 226–227 |
| Modify Attribute | Optional Unique Identifier; optional Current Attribute; required New Attribute | Required Unique Identifier | §6.1.34, Tables 265–266 |
| Set Attribute | Optional Unique Identifier; required New Attribute | Required Unique Identifier | §6.1.51, Tables 322–323 |

Every operation result also carries the existing KMIP Result Status and optional Result Reason and Result Message contract. A Pending result is represented through the accepted asynchronous result contract and does not cause hidden polling.

## State and validation rules

- Unique Identifier omission is preserved where allowed by the operation table so the server applies KMIP ID Placeholder behavior.
- Add, Adjust, Delete, Get, Modify, and Set remain different operation variants; request construction never rewrites one operation into another.
- Get Attributes rejects duplicate Attribute References. If no references are provided, the server returns all attributes. If a selected attribute has multiple instances, all instances are returned. Missing selected attributes are omitted; the client fabricates no entries.
- Get Attribute List request contains only the optional Unique Identifier; per §6.1.21 prose the server returns all attribute names, and the required response Attribute Reference entries are retained in wire order, including repeats.
- Delete preserves both optional selectors independently. Attribute Reference with no Current Attribute selects all instances per §6.1.13. If both selectors are absent, the client sends the request shape unchanged and surfaces the server result. When a supplied selector identifies a standard attribute that cannot be changed/deleted or is required to retain a value, the client returns a payload-free local error with delivery state `NotSent` before transport.
- Before a mutation exchange, the client checks every supplied target name using `StandardAttributePolicy`. For Add or Modify with New Attribute name `Usage Limits`, it always rejects locally: §7.40 Table 392 requires Count, while §4.59 prohibits setting or modifying Count through these operations. It checks the canonical Vendor Attribute `source_value_policies` rule against each supplied value and rejects Vendor Identification `y` under §4.60; it does not infer the identifier for Adjust or reference-only Delete. Conditional rules that depend on remote object state are not guessed and do not trigger an extra exchange; other server results are preserved. Unknown names and vendor values without the explicit `y` condition remain lossless and have no inferred mutability. Every local rejection is payload-free and has delivery state `NotSent`.
- Modify with no Current Attribute preserves omission; the server resolves a single instance and reports ambiguity for multiple instances. The client does not choose an instance.
- Set and Adjust preserve server-side single-instance and absent-value semantics. No local read/modify/write occurs.
- Individually assigned tags and tags accepted by the KMIPKIT-0004/ADR-0010 §11.56 allocation gate remain available for generic values. Raw tags failing that gate cannot become public Items or be encoded; unknown Enumeration values remain representable generically, while typed outbound Adjustment Type validates the assigned or `0x80000000–0x8FFFFFFF` extension range.

## Relationships

- Each of the seven request structures selects one operation and becomes one request batch item.
- Each successful response payload is paired with its operation result in the existing response batch model.
- AttributeEntry, AttributeReference, CurrentAttribute, and NewAttribute are shared by operation models but are not interchangeable.
- Result Status, Result Reason, Result Message, Pending, and delivery-state are shared contracts owned by existing protocol/client foundations.
