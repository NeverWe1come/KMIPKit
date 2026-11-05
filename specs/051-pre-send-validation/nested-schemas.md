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

## Boundaries

This file records implemented reusable schemas, not closure of all 0051 nested
validation work. T009a remains open until the remaining audited nested schema
families have been reconciled and covered by the operation-family tasks.

- `Set Constraints` applies constraints to server operations. This API validates
  only the source-defined structure and does not claim that a server supports
  or accepts any supplied constraint.
- An empty `Constraints`, `Object Types`, `Object Groups`, or `Defaults
  Information` list is preserved. Where the cited table marks a child as
  repeatable and not required, the validator does not invent a minimum count.
- `Attributes` contains tag-selected Object Attributes. The generic nested
  schema does not guess attribute-specific multiplicity or server policy.
- Object Type values remain open raw Enumeration values; this schema validates
  their TTLV encoding, not whether a server implements that object type.
- Cryptographic parameter Enumeration values and vendor fields remain open.
  The IV/Tag Length presence rules reuse the existing KMIPKit interpretation of
  §4.16 for locally recognized CTR, CCM, and GCM modes; unknown modes are not
  rejected and object-derived parameters are not inferred.
