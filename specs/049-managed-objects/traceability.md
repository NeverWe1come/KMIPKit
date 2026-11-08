# KMIPKIT-0049 Normative Traceability

**Source**: `KMIPKIT-SRC-spec`, the pinned OASIS KMIP Specification v2.1 in
`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, plus the
maintainer-approved KMIP 3.0 source in
`specification/oasis/kmip-3.0/upstream/kmip-spec-v3.0.html`; KMIP 2.1 catalog
records are in `specification/catalog/kmip-2.1.json`.

The table below lists the operative stable catalog requirement IDs in KMIP
2.1 §§2.1–3.12 and their stable feature requirements. The approved KMIP 3.0
addendum adds stable source-derived requirement IDs for §§2.1.3.1–2.1.3.4,
§11.35, and §11.58. The resolved mapping below names the exact source tables,
implementation modules, and executable test functions.

| Requirement ID | Section | KMIPKIT requirement | Implementation and verification |
|---|---:|---|---|
| KMIPKIT-REQ-SPEC-2.3-001 | 2.3 | FR-001, FR-005 | `managed_object.rs`; opaque context compatibility tests |
| KMIPKIT-REQ-SPEC-2.3-002 | 2.3 | FR-001, FR-008 | Opaque Object tests; preserve local context separation |
| KMIPKIT-REQ-SPEC-2.3-003 | 2.3 | FR-001, FR-004 | Opaque Object required-field and negative tests |
| KMIPKIT-REQ-SPEC-2.4-001-001 | 2.4 | FR-001, FR-005 | PGP Key public-only/public-private preservation tests |
| KMIPKIT-REQ-SPEC-2.4-001-002 | 2.4 | FR-001 | PGP Key Version round-trip test |
| KMIPKIT-REQ-SPEC-2.4-002-001 | 2.4 | FR-001, FR-008 | Opaque PGP Key Block preservation tests |
| KMIPKIT-REQ-SPEC-2.4-002-002 | 2.4 | FR-008 | Not applicable: the standard condition is a PGP-aware client; KMIPKit does not parse/decompose PGP. Verified by preserving opaque Key Block and recording the boundary. |
| KMIPKIT-REQ-SPEC-2.7-001 | 2.7 | FR-002, FR-004 | Secret Data wrapped/plain Key Value tests |
| KMIPKIT-REQ-SPEC-2.8-001-001 | 2.8 | FR-001 | Split Key round-trip and part-distribution metadata tests |
| KMIPKIT-REQ-SPEC-2.8-001-002 | 2.8 | FR-004 | Split Key Identifier range tests |
| KMIPKIT-REQ-SPEC-2.8-003 | 2.8 | FR-004 | Prime Field Size conditional requiredness tests |
| KMIPKIT-REQ-SPEC-3.1-001 | 3.1 | FR-002, FR-003 | Key Compression Type known/future value tests |
| KMIPKIT-REQ-SPEC-3.1-002 | 3.1 | FR-004 | Wrapped Key Block requires Key Wrapping Data tests |
| KMIPKIT-REQ-SPEC-3.1-004-001 | 3.1 | FR-004 | Cryptographic Algorithm omission only with source information tests |
| KMIPKIT-REQ-SPEC-3.1-004-002 | 3.1 | FR-004 | Algorithm presence requires Cryptographic Length tests |
| KMIPKIT-REQ-SPEC-3.1-005-001 | 3.1 | FR-004 | Cryptographic Length omission rule tests |
| KMIPKIT-REQ-SPEC-3.1-005-002 | 3.1 | FR-004 | Length presence requires Cryptographic Algorithm tests |
| KMIPKIT-REQ-SPEC-3.1-006 | 3.1 | FR-004 | Unwrapped Key Block forbids Key Wrapping Data tests |
| KMIPKIT-REQ-SPEC-3.2-001 | 3.2 | FR-002, FR-003 | Key Material and optional Attributes tests |
| KMIPKIT-REQ-SPEC-3.3-001-001 | 3.3 | FR-002, FR-005 | Wrapping metadata construction tests |
| KMIPKIT-REQ-SPEC-3.3-001-002 | 3.3 | FR-002, FR-004 | Optional wrapping metadata omission tests |
| KMIPKIT-REQ-SPEC-3.3-002 | 3.3 | FR-004 | Required IV/counter/nonce cases from each wrapping method tested; no cryptographic inference |
| KMIPKIT-REQ-SPEC-3.3-003 | 3.3 | FR-004 | `No Encoding` forbids Key Value Attributes negative test |
| KMIPKIT-REQ-SPEC-3.3-004 | 3.3 | FR-004 | At least one encryption or MAC/signature key-information structure test |
| KMIPKIT-REQ-SPEC-3.3-006 | 3.3 | FR-004 | Absent Encoding Option preserves TTLV encoding test |
| KMIPKIT-REQ-SPEC-3.3-009 | 3.3 | FR-002, FR-005 | Required MAC/signature key identifier tests |
| KMIPKIT-REQ-SPEC-3.1-007 | 3.1 | FR-002 | Key Format Type required field test |
| KMIPKIT-REQ-SPEC-3.2-003 | 3.2 | FR-003 | Key Material required field test |
| KMIPKIT-REQ-SPEC-3.3-010 | 3.3 | FR-002 | Wrapping Method required field test |
| KMIPKIT-REQ-SPEC-3.3-011 | 3.3 | FR-002 | Key information Unique Identifier required field test |
| KMIPKIT-REQ-SPEC-3.4-002 | 3.4 | FR-003 | Transparent Symmetric Key Key field test |
| KMIPKIT-REQ-SPEC-3.5-002 | 3.5 | FR-003 | DSA private P field test |
| KMIPKIT-REQ-SPEC-3.5-003 | 3.5 | FR-003 | DSA private Q field test |
| KMIPKIT-REQ-SPEC-3.5-004 | 3.5 | FR-003 | DSA private G field test |
| KMIPKIT-REQ-SPEC-3.5-005 | 3.5 | FR-003 | DSA private X field test |
| KMIPKIT-REQ-SPEC-3.6-002 | 3.6 | FR-003 | DSA public P field test |
| KMIPKIT-REQ-SPEC-3.6-004 | 3.6 | FR-003 | DSA public G field test |
| KMIPKIT-REQ-SPEC-3.6-005 | 3.6 | FR-003 | DSA public Y field test |
| KMIPKIT-REQ-SPEC-3.7-002 | 3.7 | FR-003, FR-004 | RSA private alternative field-set tests |
| KMIPKIT-REQ-SPEC-3.7-003 | 3.7 | FR-003 | RSA private Modulus field test |
| KMIPKIT-REQ-SPEC-3.8-002 | 3.8 | FR-003 | RSA public Modulus field test |
| KMIPKIT-REQ-SPEC-3.8-003 | 3.8 | FR-003 | RSA public exponent field test |
| KMIPKIT-REQ-SPEC-3.9-002 | 3.9 | FR-003 | DH private P field test |
| KMIPKIT-REQ-SPEC-3.9-003 | 3.9 | FR-003 | DH private G field test |
| KMIPKIT-REQ-SPEC-3.9-004 | 3.9 | FR-003 | DH private X field test |
| KMIPKIT-REQ-SPEC-3.10-002 | 3.10 | FR-003 | DH public P field test |
| KMIPKIT-REQ-SPEC-3.10-003 | 3.10 | FR-003 | DH public G field test |
| KMIPKIT-REQ-SPEC-3.10-004 | 3.10 | FR-003 | DH public Y field test |
| KMIPKIT-REQ-SPEC-3.11-002 | 3.11 | FR-003 | EC private Recommended Curve field test |
| KMIPKIT-REQ-SPEC-3.11-003 | 3.11 | FR-003 | EC private D field test |
| KMIPKIT-REQ-SPEC-3.12-002 | 3.12 | FR-003 | EC public Recommended Curve field test |
| KMIPKIT-REQ-SPEC-3.12-003 | 3.12 | FR-003 | EC public Q String field test |
| KMIPKIT-REQ-SPEC-6.1.43-005 | 6.1.43 | FR-006 | Register fake-transport vector with typed wrapped Key Value and Attributes |

## Local typed-API contract traceability

`KMIPKIT-0049-FR-011` is an approved KMIPKit API behavior, not a protocol
requirement attributed to OASIS. It applies to the structures whose wire
shapes are defined in KMIP 2.1 §3.1/Table 12 and §3.3/Tables 14–17. The
implementation is `key_block.rs`, `key_wrapping.rs`, and
`managed_object.rs::rebuild_ordered_structure`; executable evidence is
`key_structures.rs::key_block_setters_update_decoded_tree_and_preserve_extensions`
and `key_wrapping.rs::key_wrapping_setters_update_decoded_tree_and_preserve_extensions`.

## Catalog records not treated as operative requirements

The catalog contains table-heading records whose summary explicitly says the
`REQUIRED` heading is metadata, not an operative protocol requirement. They
are accounted for in the gap matrix's requiredness columns and schema tests,
but are not reported as independent normative behavior. Requirements whose
only applicable behavior belongs to NIST key-type attributes or Re-Provision
are outside this feature and remain assigned to their existing scope.

## Resolved implementation-to-test mapping

All source references below are in the pinned `KMIPKIT-SRC-spec`. Table numbers
refer to KMIP Specification v2.1. The requirement IDs listed in each row are
the complete set covered by its implementation and executable evidence.

| Requirement IDs | Exact OASIS source | Implementation | Executable evidence |
|---|---|---|---|
| `KMIPKIT-REQ-SPEC-2.3-001`, `-002`, `-003` | §2.3, Table 4; §6.1.43, Tables 291–293 | `crates/kmipkit-protocol/src/managed_object.rs`; additive Register/Import helpers in `register.rs` and `import.rs` | `managed_objects.rs::every_known_managed_object_uses_its_oasis_structure_tag_and_field_order`; `managed_objects.rs::unknown_object_types_and_vendor_children_remain_lossless`; `register_execution_tests.rs::typed_certificate_register_uses_direct_object_tag_on_fake_transport`; `import_execution_tests.rs::typed_wrapped_symmetric_key_import_uses_direct_object_tag_on_fake_transport` |
| `KMIPKIT-REQ-SPEC-2.4-001-001`, `-001-002`, `-002-001`, `-002-002` | §2.4, Table 5; §6.1.25, Tables 205–207 | `managed_object.rs` preserves PGP version and opaque Key Block; PGP decomposition is explicitly out of scope | `managed_objects.rs::every_known_managed_object_uses_its_oasis_structure_tag_and_field_order`; `managed_objects.rs::every_known_object_type_dispatches_only_its_matching_structure_tag`; `managed_object_examples.rs::plaintext_typed_object_uses_direct_object_structure`; `spec.md` Edge Cases and Scope and Exclusions record the non-PGP-aware boundary |
| `KMIPKIT-REQ-SPEC-2.7-001` | §2.7, Table 8; §§3.1–3.3, Tables 12–17 | `managed_object.rs`, `key_block.rs`, `key_wrapping.rs` | `managed_objects.rs::every_known_managed_object_uses_its_oasis_structure_tag_and_field_order`; `key_structures.rs::plaintext_and_wrapped_key_values_have_distinct_ttlv_types`; `managed_object_examples.rs::wrapped_object_keeps_wrapping_metadata_and_uses_byte_string` |
| `KMIPKIT-REQ-SPEC-2.8-001-001`, `-001-002`, `-003` | §2.8, Table 9; §10.1.2 | `managed_object.rs::SplitKey` schema and conditional validation | `managed_objects.rs::split_key_rejects_identifier_outside_one_through_parts`; `managed_objects.rs::polynomial_prime_field_requires_prime_field_size`; `managed_objects.rs::split_key_reader_rejects_invalid_numeric_and_conditional_fields`; `managed_objects.rs::split_key_prime_field_size_precedes_key_block_in_table_nine_order` |
| `KMIPKIT-REQ-SPEC-3.1-001`, `-002`, `-004-001`, `-004-002`, `-005-001`, `-005-002`, `-006`, `-007` | §3.1, Table 12; §10.1.2 | `key_block.rs` | `key_structures.rs::key_block_builder_emits_canonical_order_independent_of_setter_order`; `key_structures.rs::key_block_rejects_unpaired_algorithm_and_length`; `key_structures.rs::wrapped_key_value_requires_key_wrapping_data`; `key_structures.rs::plaintext_key_value_forbids_key_wrapping_data`; `key_structures.rs::key_block_reader_preserves_the_tree_and_exposes_typed_header_values`; parse-mutate-convert behavior is separately tracked as local FR-011 |
| `KMIPKIT-REQ-SPEC-3.2-001`, `-003` | §3.2, Table 13; §10.1.2 | `key_block.rs::KeyValue` and `key_material.rs::KeyMaterial` | `key_material.rs::key_value_keeps_optional_attributes_after_required_material`; `key_material.rs::key_material_reader_preserves_format_and_ttlv_type`; `key_material.rs::known_key_formats_reject_the_wrong_key_material_ttlv_form` |
| `KMIPKIT-REQ-SPEC-3.3-001-001`, `-001-002`, `-004`, `-009`, `-010`, `-011` | §3.3, Tables 14–17; §10.1.2 | `key_wrapping.rs`; `key_block.rs` verifies nested wrapping data | `key_wrapping.rs::key_wrapping_data_emits_method_before_optional_metadata`; `key_wrapping.rs::wrapping_reader_checks_required_identifiers_and_keeps_unknown_fields`; `key_wrapping.rs::wrapping_data_requires_encryption_or_mac_signature_key_information`; `key_structures.rs::wrapped_key_value_requires_key_wrapping_data`; parse-mutate-convert behavior is separately tracked as local FR-011 |
| `KMIPKIT-REQ-SPEC-3.3-002`, `-006` | §3.3, Tables 14–15 | Caller-owned conditions are represented in profile evidence; no unsupported universal inference is made when wrapping-method details or the encoded plaintext are unavailable | `crates/kmipkit-protocol/src/profile.rs` documents the limitation; `crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_caller_owned_wrapping_iv_and_ttlv_encoding_obligations` proves the requirements remain explicitly caller-attested |
| `KMIPKIT-REQ-SPEC-3.3-003` | §3.3, Table 14; §11.18 | Existing profile request validation rejects Key Value Attributes with No Encoding | `crates/kmipkit-protocol/src/profile.rs`; `crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_key_value_attributes_when_wrapping_uses_no_encoding` |
| `KMIPKIT-REQ-SPEC-3.4-002` | §3.4, Table 18 | `key_material.rs::TransparentSymmetricKey` | `transparent_key_material.rs::transparent_symmetric_key_contains_required_key_bytes` |
| `KMIPKIT-REQ-SPEC-3.5-002`, `-003`, `-004`, `-005` | §3.5, Table 19 | `key_material.rs::TransparentDsaPrivateKey` | `transparent_key_material.rs::transparent_dsa_private_and_public_fields_follow_tables_19_and_20` |
| `KMIPKIT-REQ-SPEC-3.6-002`, `-004`, `-005` | §3.6, Table 20 | `key_material.rs::TransparentDsaPublicKey` | `transparent_key_material.rs::transparent_dsa_private_and_public_fields_follow_tables_19_and_20` |
| `KMIPKIT-REQ-SPEC-3.7-002`, `-003` | §3.7, Table 21 | `key_material.rs::TransparentRsaPrivateKey` | `transparent_key_material.rs::transparent_rsa_private_supports_only_source_defined_alternatives` |
| `KMIPKIT-REQ-SPEC-3.8-002`, `-003` | §3.8, Table 22 | `key_material.rs::TransparentRsaPublicKey` | `transparent_key_material.rs::transparent_rsa_public_requires_modulus_and_public_exponent` |
| `KMIPKIT-REQ-SPEC-3.9-002`, `-003`, `-004` | §3.9, Table 23 | `key_material.rs::TransparentDhPrivateKey` | `transparent_key_material.rs::transparent_dh_private_and_public_preserve_optional_q_and_j` |
| `KMIPKIT-REQ-SPEC-3.10-002`, `-003`, `-004` | §3.10, Table 24 | `key_material.rs::TransparentDhPublicKey` | `transparent_key_material.rs::transparent_dh_private_and_public_preserve_optional_q_and_j` |
| `KMIPKIT-REQ-SPEC-3.11-002`, `-003` | §3.11, Table 25 | `key_material.rs::TransparentEcPrivateKey` | `transparent_key_material.rs::transparent_ec_private_and_public_preserve_curve_and_material` |
| `KMIPKIT-REQ-SPEC-3.12-002`, `-003` | §3.12, Table 26 | `key_material.rs::TransparentEcPublicKey` | `transparent_key_material.rs::transparent_ec_private_and_public_preserve_curve_and_material` |
| `KMIPKIT-REQ-SPEC-6.1.43-005` | §§3.2–3.3, Tables 13–17; §6.1.43, Tables 291–293; §11.18 | `register.rs`, `import.rs`, `key_block.rs`, `key_wrapping.rs` | `register_execution_tests.rs::typed_certificate_register_uses_direct_object_tag_on_fake_transport`; `import_execution_tests.rs::typed_wrapped_symmetric_key_import_uses_direct_object_tag_on_fake_transport`; `profile_unit.rs::profile_rejects_key_value_attributes_when_wrapping_uses_no_encoding` |
| `KMIPKIT-REQ-SPEC-3.0-2.1.3.1-001` | KMIP 3.0 §2.1.3.1, Tables 5–6 | `system_credential_objects.rs::password_credential_builders_and_readers_follow_source_rules`; `system_credential_objects.rs::system_credential_objects_reject_invalid_source_schemas` | Password Credential structure rules: field types/order and salted form MUST NOT include Password; Password Link is an attribute and is checked with attributes in 0051 |
| `KMIPKIT-REQ-SPEC-3.0-2.1.3.2-001` | KMIP 3.0 §2.1.3.2, Tables 7–8 | `system_credential_objects.rs::device_credential_builders_and_readers_follow_table_order`; `system_credential_objects.rs::system_credential_objects_reject_invalid_source_schemas` | Device Credential requires at least one defined identifier field; uniqueness remains server policy |
| `KMIPKIT-REQ-SPEC-3.0-2.1.3.3-001` | KMIP 3.0 §2.1.3.3, Tables 9–10 | `system_credential_objects.rs::one_time_password_builders_and_readers_follow_table_order`; `system_credential_objects.rs::system_credential_objects_reject_invalid_source_schemas` | OTP Algorithm is required; other source fields retain their optionality and order |
| `KMIPKIT-REQ-SPEC-3.0-2.1.3.4-001` | KMIP 3.0 §2.1.3.4, Tables 11–12 | `system_credential_objects.rs::hashed_password_builders_and_readers_preserve_defaults_and_order`; `system_credential_objects.rs::system_credential_objects_reject_invalid_source_schemas` | Both byte-string hash fields are required; optional Hashing Algorithm reports the SHA-256 default without inserting a field, and unknown raw values are preserved |
| `KMIPKIT-REQ-SPEC-3.0-11.35-001` | KMIP 3.0 §11.35, Table 582; §11.58, Table 605 | `managed_object.rs`; `system_credential_objects.rs::managed_object_dispatches_each_system_credential_structure`; `system_credential_objects.rs::system_credential_object_type_requires_its_exact_structure_tag` | Object Types 0x0D–0x10 dispatch to exact structure tags; generic TTLV and unknown descendants remain lossless |
| `KMIPKIT-0049-FR-013` | KMIPKit security policy, AGENTS.md §8; TTLV zeroization contract | `system_credential_objects.rs`; `system_credential_objects.rs::system_credential_objects_retain_extensions_and_redact_owned_secrets`; existing TTLV owned-payload zeroization tests | Constructors consume `SecretText`/`SecretBytes`, Debug/errors reveal no values, and model-owned TTLV storage uses the existing recursive zeroization path |

The original KMIP 2.1 feature scope has 53 operative catalog requirement IDs.
The approved KMIP 3.0 addendum adds five stable source-derived IDs and the
local security requirement FR-013. Catalog records that are table-heading
metadata are not counted as independent requirements. Caller-attested
obligations are traceable but are not presented as automatically verified by
local structural parsing.
