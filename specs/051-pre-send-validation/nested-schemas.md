# Reusable Nested Structure Validation

This inventory records the nested KMIP 2.1 structures that the 0051 validator
currently recognizes. Validation is borrowed and schema-directed. Unknown tags
remain in the caller's TTLV tree and are not rejected solely for being unknown.

| Schema identity | Exact source | Locally checked shape | Stable rules | Executable evidence |
|---|---|---|---|---|
| `Constraints` | §7.7, Table 358 | Zero or more `Constraint` Structure children. | R191 | `typed_request_validation_tests::constraints_validation_checks_known_nested_shapes_and_preserves_empty_values` |
| `Constraint` | §7.6, Table 357 | Optional singleton `Object Types`, `Object Groups`, and `Attributes` Structures, in table order. | R192–R194 | `typed_request_validation_tests::constraints_validation_checks_known_nested_shapes_and_preserves_empty_values` |
| `Object Types` | §7.24, Table 376 | Zero or more `Object Type` Enumeration children. | R195 | `validation_schema_tests::nested_object_types_schema_checks_known_members_and_preserves_vendor_fields` |
| `Object Groups` | §7.23, Table 375 | Zero or more `Object Group` Text String children. | R196 | `validation_schema_tests::constraints_schema_recurses_through_known_children_and_keeps_extensions` |
| `Attributes` | §5.1, Table 157 | Retain direct attribute items without imposing an operation-independent value schema; catalogued value types are handled by typed attribute construction where applicable. | — | `validation_schema_tests::constraints_schema_recurses_through_known_children_and_keeps_extensions`; existing `attribute_tests` |
| `Defaults Information` | §7.11, Table 363 | Zero or more `Object Defaults` Structure children. | R197 | `typed_request_validation_tests::defaults_validation_checks_required_attribute_container_and_preserves_empty_defaults`; `validation_schema_tests::request_payload_schema_recurses_into_known_nested_payloads` |
| `Object Defaults` | §7.22, Table 374 | Exactly one of `Object Type` Enumeration or `Object Types` Structure; one `Attributes` Structure is required; `Object Groups` is optional. | R198–R200 | `validation_schema_tests::nested_object_defaults_enforces_alternative_and_required_attributes`; `typed_request_validation_tests::defaults_validation_checks_required_attribute_container_and_preserves_empty_defaults` |
| `Cryptographic Parameters` | §4.16, Table 59 | Eighteen known optional singleton fields in table order, with their specified TTLV types; require `IV Length` for locally recognized CTR, CCM, and GCM modes, and `Tag Length` for GCM. | R201–R220 | `validation_schema_tests::encrypt_payload_recursively_validates_known_cryptographic_parameter_fields`; `validation_schema_tests::cryptographic_parameters_require_iv_and_gcm_tag_lengths_when_mode_requires_them`; `validation_schema_tests::cryptographic_parameters_preserve_unknown_members_and_future_enumerations` |
| `Derivation Parameters` | §7.12, Table 364 | Optional Cryptographic Parameters, Initialization Vector, Salt, and Iteration Count; repeatable Derivation Data; all known children use the table's TTLV type and order. The enclosing Derive Key request requires Salt and Iteration Count for PBKDF2 and Hashing Algorithm for HASH. | R221–R228 | `validation_schema_tests::derive_key_payload_recursively_validates_derivation_parameters`; `validation_schema_tests::derive_key_schema_applies_pbkdf2_parameter_requirements_from_outer_method`; `validation_schema_tests::derive_key_schema_requires_hashing_algorithm_for_hash_method` |
| `Key Wrapping Specification` | §7.18, Table 370; §3.3, Tables 16–17 | Required Wrapping Method; at least one key-information Structure, with both allowed; repeated Attribute Name; optional Encoding Option. Each key-information Structure requires one Text String Unique Identifier and may contain Cryptographic Parameters. | R229–R236 | `validation_schema_tests::get_payload_recursively_validates_key_wrapping_specification`; `validation_schema_tests::key_wrapping_specification_requires_key_information_but_allows_both_forms`; `validation_schema_tests::key_information_requires_text_string_unique_identifier` |
| `Rights` | §7.35, Table 387 | Zero or more `Right` Structure children; the enclosing Delegated Login request still requires one Rights Structure. | R237 | `validation_schema_tests::rights_schema_preserves_empty_and_repeated_rights_and_unknown_extensions` |
| `Right` | §7.34, Table 386 | Optional singleton Usage Limits, Operations, Objects, and Object Groups Structures, in table order. | R238–R241 | `validation_schema_tests::rights_schema_recursively_validates_usage_limits_and_preserves_empty_rights`; `validation_schema_tests::rights_schema_preserves_empty_and_repeated_rights_and_unknown_extensions` |
| `Usage Limits` | §7.40, Table 392 | Required singleton Usage Limits Total and Count Long Integers followed by Usage Limits Unit Enumeration; unknown fields remain available. No numeric range is inferred. | R242–R244 | `validation_schema_tests::rights_schema_recursively_validates_usage_limits_and_preserves_empty_rights` |
| `Objects` | §7.21, Table 373 | Zero or more Unique Identifier values, each encoded as Integer, Enumeration, or Text String. | R245 | `validation_schema_tests::rights_schema_preserves_empty_and_repeated_rights_and_unknown_extensions` |
| `Operations` | §7.25, Table 377 | Zero or more Operation Enumeration values; future and vendor values are retained. | R246 | `validation_schema_tests::rights_schema_preserves_empty_and_repeated_rights_and_unknown_extensions` |
| `Current Attribute` | §5.6, Table 162 | Exactly one attribute Item with any attribute tag and value type; do not infer attribute-specific multiplicity or reject unknown/vendor tags. | R248 | `validation_schema_tests::current_attribute_wrapper_rejects_multiple_attribute_items` |
| `New Attribute` | §5.7, Table 163 | Exactly one attribute Item with any attribute tag and value type; do not infer attribute-specific multiplicity or reject unknown/vendor tags. | R247 | `validation_schema_tests::attribute_wrappers_require_exactly_one_attribute_item` |

## Boundaries

This file records implemented reusable schemas, not closure of all 0051 nested
validation work. T009a remains open until the remaining audited nested schema
families have been reconciled and covered by the operation-family tasks.

- `Set Constraints` applies constraints to server operations. This API validates
  only the source-defined structure and does not claim that a server supports
  or accepts any supplied constraint.
- An empty `Constraints`, `Object Types`, `Object Groups`, `Defaults
  Information`, `Rights`, `Objects`, or `Operations` list is preserved. Where the cited table marks a child as
  repeatable and not required, the validator does not invent a minimum count.
- `Attributes` contains tag-selected Object Attributes. The generic nested
  schema does not guess attribute-specific multiplicity or server policy.
- Object Type values remain open raw Enumeration values; this schema validates
  their TTLV encoding, not whether a server implements that object type.
- Cryptographic parameter Enumeration values and vendor fields remain open.
  The IV/Tag Length presence rules reuse the existing KMIPKit interpretation of
  §4.16 for locally recognized CTR, CCM, and GCM modes; unknown modes are not
  rejected and object-derived parameters are not inferred.
- A standalone Derivation Parameters schema does not guess the outer Derivation
  Method or whether a Unique Identifier denotes a Secret Data object. PBKDF2,
  HASH, and input-object-dependent rules require the enclosing Derive Key
  Request Payload context; a Secret Data object's type remains server state.
- Rights validation checks only the §7 structure. It does not establish ticket
  authorization or enforce server-specific operation, object, group, or usage
  policy, and it does not impose numeric ranges on Usage Limits values.
- Attribute wrappers validate only their one-item shape. The child tag selects
  an attribute, so this generic schema does not infer that attribute's value
  type or cardinality and preserves future/vendor attributes.
