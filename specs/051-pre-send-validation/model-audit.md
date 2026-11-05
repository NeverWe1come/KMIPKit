# KMIPKIT-0051 T002/T003 Rust model audit

Static audit of all 189 operation-field rows, current Rust request models/serializers, focused tests, and pinned `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. No implementation changed. Tests were inspected, not executed. A test-file pointer means proximity, not that it already enforces every rule on a row. All OASIS references below mean the pinned KMIP v2.1 specification.

`src/` means `crates/kmipkit-protocol/src/`; `unit/` means `crates/kmipkit-protocol/tests/unit/`. The operation index supplies the exact declaration, serializer and nearest focused test for each field row. No mapping treats absence and present-empty structures/lists as interchangeable.

## Construction status

- **S**: Private scalar/optional storage fixes the tag, outer TTLV type, position and maximum cardinality when serialized. Non-Option required scalars cannot be omitted. This does not establish scalar ranges, enum allocation or conditional relationships.
- **U**: `UniqueIdentifier` allows TextString(String), Enumeration(u32), Integer(i32), as §4.58 Tables 145–146 specify. Serializer fixes tag/order/cardinality and retains all alternatives. Context-specific prohibitions, such as Derive Key's ID Placeholder restriction, are separate. **US** denotes the String-only subset.
- **R**: Raw nested `Structure` fixes only the outer wrapper/tag/order/cardinality. Children have generic TTLV validity, not the shared KMIP schema.
- **A**: `AttributeSet.items: Vec<Item>` insertion validates catalogued outer attribute TTLV types and Vendor Attribute shape/order; preserves duplicates, raw nested structures and unknowns. It is not a fully validated object-attribute set.
- **N**: Current/New Attribute wraps exactly one arbitrary `Item`. Its `new(Item)` does not validate the selected attribute's encoding/schema.
- **AR**: `AttributeReference` privately represents Tag(u32) or Name{vendor_identification:String,attribute_name:String}. Serializer fixes shape; tag allocation and vendor character checks occur during outbound serialization, not construction.
- **D**: `OperationData` fixes ByteString(SecretBytes), Enumeration(u32), Integer(i32). Multipart/context conditions remain separate.
- **L**: Typed collection fixes child tag/type/order; minimum and conditional rules are not generally construction invariants.
- **T**: `Ticket` constructor/parser already validates known required members/order/type/count. Unknown children remain open.
- **B**: Typed nested builder or serializer-only structure; see its shared-schema entry for residual raw paths.
- **O**: Arbitrary managed-object `Item`; raw constructor does not guarantee selected object tag/schema.
- **F**: Fixed serializer-only value; no caller input.

These are limited invariants, not completeness marks or proof that the fixed tag matches OASIS; the final section identifies two current tag mismatches. Generic TTLV construction does not imply a known KMIP schema. Unknown tags/future values on accepted extensible paths remain open.

## Shared schema and local boundary

Borrowed traversal can check tags, Item Types, known-field relative order, requiredness and counts without decoding certificate/key/ciphertext/password/seed bytes. Conditions involving scalar values require reading those scalar values, without formatting, copying or serializing them. Remote state/type, referenced-key parameters, trust and server policy cannot be proven from shape.

| Key | Exact source and model path | Locally decidable boundary |
|---|---|---|
| ATTR / COMMON / PRIVATE / PUBLIC | §5.1 Table157 / §5.2 Table158 / §5.3 Table159 / §5.4 Table160; zero or more direct §4 Object Attribute Items. `AttributeSet.items`, or raw Structure in Certify/Recertify/Rekey. Individual attributes use their exact defining §4 tables; Vendor Attribute uses §4.60 Table150. | Known direct types, Vendor Identification/Name/Value requiredness/type/order/count, context-decidable attribute multiplicity. No universal singleton or attribute sorting. Nested standard Structure attributes require their own selected §4 schema. Vendor Attribute Value is Any; do not infer its schema from content/name. Raw attribute Structures bypass even AttributeSet insertion checks. |
| NEW / CURRENT | §5.7 Table163 / §5.6 Table162. `NewAttribute.item: Item` / `CurrentAttribute.item: Item`. | Wrapper's one child is construction-safe; selected direct attribute type/nested shape is not. Unknown Items stay generic. No remote instance/policy inference. Shared tests: `unit/typed_attribute_tests.rs`, `unit/attribute_tests.rs`. |
| REF | §5.5 Table161 alternatives: Enumeration(Tag), or Structure(Vendor Identification Text String, Attribute Name Text String). | Serializer fixes requiredness/order. `attribute_reference.rs::to_ttlv_item` checks tag allocation/vendor characters. No remote existence or vendor-value schema inference. |
| ADJUST | §6.1.3 Table170 and prose; `adjustment_value: Option<Value>`. | Value is explicitly Any. No universal TTLV type or nested Structure schema. Only an exact known attribute/adjustment-category rule may constrain it. Existing serialization checks Adjustment Type allocation/reference, not an invented generic value schema. |
| MASKS | §7.33 Table385 and prose, including Common/Private/Public variants; raw `Option<Structure>`. | Outer Structure and ordered selections/alternatives are defined. Table385 has no child field rows. No child tag/type/cardinality is inferable from fixture/name. Server decides whether a selection is satisfiable. |
| CP | §4.16 Table59. Raw Structure in seven crypto operations and `DerivationParameters.cryptographic_parameters`; optional typed `CryptographicParameters.structure` input. | Existing 0050 validators check known types/singletons/order/local conditions, preserving unknowns. Raw setters remain available. Reuse existing rules without broadening. Shared tests: `unit/encrypt_parameter_structure_tests.rs`, `unit/cryptographic_parameters_tests.rs`, `tests/cryptographic_parameters_public.rs`. |
| DERIVE | §7.12 Table364/prose. `DerivationParameters { cryptographic_parameters:Option<Structure>, initialization_vector:Option<SecretBytes>, derivation_data:Vec<SecretBytes>, salt:Option<SecretBytes>, iteration_count:Option<i32>, extensions:Structure }`. | Known builder fields have fixed order/types, Derivation Data repeats. `with_extension(Item)` checks only generic limits; known-tag collisions/order/type problems remain possible. PBKDF2 Salt/Iteration Count and HASH algorithm presence are local; a referenced object's Secret Data type is remote, so absent Derivation Data or explicit+referenced forms cannot automatically be rejected without that knowledge. No PRF/secret-byte evaluation. |
| WRAP | §7.18 Table370; Encryption Key Information §3.3 Table16, MAC/Signature Key Information §3.3 Table17, optional CP §4.16 Table59. `GetRequest.key_wrapping_specification`; Export delegates. | Required Wrapping Method Enumeration, at least one of the two key-information groups, repeatable Attribute Name Text String, optional Encoding Option Enumeration. Check nested type/order/count without keys. Tables16/17 explicitly say identifier Text String; reconcile existing broader identifier API before tightening. Current Get/Export request serializers merely wrap raw Structure. Valid wrapping fixtures are not negative schema evidence. |
| USAGE | §7.40 Table392 for Login/DelegatedLogin: required Usage Limits Total Long Integer, Count Long Integer, Unit Enumeration. | Known requiredness/type/order/count are local. Raw setters currently wrap only. Do not conflate with the §4.59 Usage Limits attribute definition/rules. Quota/policy decisions are remote. |
| RIGHTS | §7.35 Table387 (zero or more Right Structures); §7.34 Table386 (optional Usage Limits, Operations, Objects, Object Groups). Recursion: §7.40 Table392, §7.25 Table377, §7.21 Table373, §7.23 Table375. `DelegatedLoginRequest.rights`. | Listed shape/order/type/count are local. Table373 explicitly allows repeated Text String/Enumeration/Integer IDs. Omitted Right aspect is unrestricted; Table387 does not require nonempty Rights. Whether a Right authorizes an actual action is server-side. |
| OBJECT | §2.1 Table2 Certificate; §2.2 Table3 Certificate Request; §2.3 Table4 Opaque Object; §2.4 Table5 PGP Key; §2.5 Table6 Private Key; §2.6 Table7 Public Key; §2.7 Table8 Secret Data; §2.8 Table9 Split Key; §2.9 Table10 Symmetric Key. `ImportRequest.object` / `RegisterRequest.object: Item`; optional ManagedObject constructors derive matching Object Type. | A known selected Object Type allows checking matching tag/Structure/known children; unknown object types remain open. Key objects recurse to §3.1 Table12 Key Block, §3.2 Table13 plaintext Key Value, §3.3 Tables14–17 wrapping metadata; transparent format-selected Key Material §3.4–§3.12 Tables18–26. Wrapped Byte String plaintext cannot be inspected. Existing object validators are selected checks, not universal recursive validity. |
| CERT / CERTREQ | §2.1 Table2 / §2.2 Table3. ReProvision raw Option<Structure>; Validate Vec<Structure>. | Required Type Enumeration then Value Byte String; check known count/order/type while preserving unknowns. No DER/CSR/signature/trust evaluation. Validate currently checks Table2; ReProvision just wraps. |
| REASON | §4.47 Table123. Serializer-only Structure from `RevokeRequest.revocation_reason:RevocationReason(u32)` plus `revocation_message:Option<String>`. | Required Revocation Reason Code Enumeration then optional Message Text String are construction-safe. Reason enum plus date presence decides Table295 conditional. No caller raw reason Structure. |
| TICKET | §7.39 Table391. `Ticket` owns validated Structure and typed/accessor state; `try_from_ttlv(Structure)` is checked raw acceptance. | Required Type Enumeration then Value Byte String known shape safe through constructor/parser. Unknowns retained. No ticket acceptance/expiry/verification inference. Shared file: `unit/ticket_model_tests.rs`. |
| PKCS_IF / PKCS_FN / PKCS_IN | §7.28 Table380 / §7.26 Table378 / §7.27 Table379 in §6.1.37 Table273 context. Raw interface/function/input Structures. | Optional same-tag Text String Interface child; required same-tag Enumeration Function child, plus vendor Integer allowed by Table273 context; optional same-tag Byte String Input child. Existing serializer checks known types/counts and preserves unknown/profile members. Function-specific input bytes remain opaque. |
| GROUPS | §7.23 Table375 Object Groups: zero or more Object Group attributes, whose §4.35 Table99 definition supplies Text String encoding. `QueryRequest.object_groups:Option<Vec<String>>`. | Serializer constructs repeated Object Group Text Strings; empty/order/repeats retained. Server group existence unknown. |
| ASYNC / OPS | §7.1 Table352 defines plural correlation Structure; §6.1.41 Table285 explicitly permits zero or more singular values; singular §9.1 Table400 Byte String. Operations §7.25 Table377 with zero or more Operation Enumeration. | Option<Vec<SecretBytes>> / Option<Vec<u32>> fix child types/order; empty legal. No equal-length/positional matching rule is stated. Source prose describes matches against correlation values and against one of the Operations, without defining positional pairing. §11.56 Table487 explicitly assigns plural Asynchronous Correlation Values 0x420176, matching current code. |
| CONSTRAINTS | §7.7 Table358: zero or more Constraint; §7.6 Table357: optional Object Types, Object Groups, Attributes. Recursion §7.24 Table376, §7.23 Table375, §5.1 Table157. `SetConstraintsRequest.constraints`. | Known structure/order/type/count local; empty allowed. Raw setter presently accepts arbitrary children. Satisfiability/authorization/policy remain remote. |
| DEFAULTS | §7.11 Table363: zero or more Object Defaults; §7.22 Table374: required Object Type Enumeration OR Object Types Structure, required Attributes, optional Object Groups. Recursion §7.24 Table376, §5.1 Table157, §7.23 Table375. `SetDefaultsRequest.defaults_information`. | Exact alternatives/order/type/count and nested shape local. Absent removes defaults by Table328; present empty legal. No server factory-method/policy evaluation. |
| VERSION | §9.16 Table421 Protocol Version Major Integer then Minor Integer; §6.1.16 Table211 repeats versions. `DiscoverVersionsRequest.protocol_versions()` returns fixed [ProtocolVersion(2,1);1]. | Typed request is safe by construction and has no raw caller version list. Generic explicit-schema version input still needs validation. |


## Operation model/serializer/test index

| Operation | Model declaration | Serializer | Closest focused test |
|---|---|---|---|
| Activate | `src/activate.rs:21` `ActivateRequest` | `to_ttlv_payload` | `unit/activate_operation_tests.rs` |
| AddAttribute | `src/add_attribute.rs:24` `AddAttributeRequest` | `to_ttlv_payload` | `unit/add_attribute_tests.rs` |
| AdjustAttribute | `src/adjust_attribute.rs:60` `AdjustAttributeRequest` | `to_ttlv_payload` | `unit/adjust_attribute_tests.rs` |
| Archive | `src/archive.rs:22` `ArchiveRequest` | `to_ttlv_payload` | `unit/archive_operation_tests.rs` |
| Cancel | `src/cancel.rs:64` `CancelRequest` | `to_ttlv_payload` | `unit/cancel_tests.rs` |
| Certify | `src/certify.rs:28` `CertifyRequest` | `into_ttlv_payload` | `unit/certify_operation_tests.rs` |
| Check | `src/check.rs:28` `CheckRequest` | `into_ttlv_payload` | `unit/check_operation_tests.rs` |
| Create | `src/create.rs:67` `CreateRequest` | `into_ttlv_payload` | `unit/create_tests.rs` |
| CreateKeyPair | `src/create_key_pair.rs:43` `CreateKeyPairRequest` | `into_ttlv_payload` | `unit/create_key_pair_tests.rs` |
| CreateSplitKey | `src/create_split_key.rs:62` `CreateSplitKeyRequest` | `into_ttlv_payload` | `unit/create_split_key_tests.rs` |
| Decrypt | `src/decrypt.rs:32` `DecryptRequest` | `to_ttlv_payload` | `unit/decrypt_tests.rs` |
| DelegatedLogin | `src/delegated_login.rs:34` `DelegatedLoginRequest` | `into_ttlv_payload` | `unit/delegated_login_operation_tests.rs` |
| DeleteAttribute | `src/delete_attribute.rs:25` `DeleteAttributeRequest` | `to_ttlv_payload` | `unit/delete_attribute_tests.rs` |
| DeriveKey | `src/derive_key.rs:195` `DeriveKeyRequest` | `into_ttlv_payload` | `unit/derive_key_operation_tests.rs` |
| Destroy | `src/destroy.rs:26` `DestroyRequest` | `to_ttlv_payload` | `unit/destroy_operation_tests.rs` |
| DiscoverVersions | `src/discover_versions.rs:30` `DiscoverVersionsRequest` | `to_ttlv_payload` | `unit/discover_versions_tests.rs` |
| Encrypt | `src/encrypt.rs:32` `EncryptRequest` | `to_ttlv_payload` | `unit/encrypt_tests.rs` |
| Export | `src/export.rs:33` `ExportRequest` | `into_ttlv_payload` | `unit/export_operation_tests.rs` |
| Get | `src/get.rs:87` `GetRequest` | `to_ttlv_payload` | `unit/get_tests.rs` |
| GetAttributes | `src/get_attributes.rs:22` `GetAttributesRequest` | `to_ttlv_payload` | `unit/get_attributes_tests.rs` |
| GetAttributeList | `src/get_attribute_list.rs:25` `GetAttributeListRequest` | `to_ttlv_payload` | `unit/get_attribute_list_tests.rs` |
| GetUsageAllocation | `src/get_usage_allocation.rs:22` `GetUsageAllocationRequest` | `to_ttlv_payload` | `unit/get_usage_allocation_tests.rs` |
| Hash | `src/hash.rs:17` `HashRequest` | `to_ttlv_payload` | `unit/hash_operation_tests.rs` |
| Import | `src/import.rs:44` `ImportRequest` | `into_ttlv_payload` | `unit/import_operation_tests.rs` |
| Interop | `src/interop.rs:26` `InteropRequest` | `into_ttlv_payload` | `unit/interop_operation_tests.rs` |
| JoinSplitKey | `src/join_split_key.rs:75` `JoinSplitKeyRequest` | `into_ttlv_payload` | `unit/join_split_key_operation_tests.rs` |
| Locate | `src/locate.rs:69` `LocateRequest` | `to_ttlv_payload` | `unit/locate_tests.rs` |
| Log | `src/log.rs:25` `LogRequest` | `into_ttlv_payload` | `unit/log_operation_tests.rs` |
| Login | `src/login.rs:32` `LoginRequest` | `into_ttlv_payload` | `unit/login_operation_tests.rs` |
| Logout | `src/logout.rs:24` `LogoutRequest` | `into_ttlv_payload` | `unit/logout_operation_tests.rs` |
| Mac | `src/mac.rs:18` `MacRequest` | `to_ttlv_payload` | `unit/mac_operation_tests.rs` |
| MacVerify | `src/mac_verify.rs:17` `MacVerifyRequest` | `to_ttlv_payload` | `unit/mac_verify_operation_tests.rs` |
| ModifyAttribute | `src/modify_attribute.rs:26` `ModifyAttributeRequest` | `to_ttlv_payload` | `unit/modify_attribute_tests.rs` |
| ObtainLease | `src/obtain_lease.rs:27` `ObtainLeaseRequest` | `into_ttlv_payload` | `unit/obtain_lease_operation_tests.rs` |
| Pkcs11 | `src/pkcs11.rs:22` `Pkcs11Request` | `into_ttlv_payload` | `unit/pkcs11_operation_tests.rs` |
| Poll | `src/poll.rs:32` `PollRequest` | `to_ttlv_payload` | `unit/poll_tests.rs` |
| Process | `src/process.rs:17` `ProcessRequest` | `to_ttlv_payload` | `unit/process_tests.rs` |
| Query | `src/query.rs:106` `QueryRequest` | `to_ttlv_payload` | `unit/query_operation_tests.rs` |
| QueryAsyncRequests | `src/query_async_requests.rs:22` `QueryAsyncRequestsRequest` | `to_ttlv_payload` | `unit/query_async_requests_tests.rs` |
| Recover | `src/recover.rs:22` `RecoverRequest` | `to_ttlv_payload` | `unit/recover_operation_tests.rs` |
| Register | `src/register.rs:57` `RegisterRequest` | `into_ttlv_payload` | `unit/register_operation_tests.rs` |
| Revoke | `src/revoke.rs:64` `RevokeRequest` | `into_ttlv_payload` | `unit/revoke_operation_tests.rs` |
| Recertify | `src/recertify.rs:31` `RecertifyRequest` | `into_ttlv_payload` | `unit/recertify_operation_tests.rs` |
| Rekey | `src/rekey.rs:29` `RekeyRequest` | `into_ttlv_payload` | `unit/rekey_operation_tests.rs` |
| RekeyKeyPair | `src/rekey_key_pair.rs:35` `RekeyKeyPairRequest` | `into_ttlv_payload` | `unit/rekey_key_pair_operation_tests.rs` |
| ReProvision | `src/reprovision.rs:26` `ReProvisionRequest` | `into_ttlv_payload` | `unit/reprovision_operation_tests.rs` |
| RngRetrieve | `src/rng_retrieve.rs:18` `RngRetrieveRequest` | `to_ttlv_payload` | `unit/rng_retrieve_operation_tests.rs` |
| RngSeed | `src/rng_seed.rs:19` `RngSeedRequest` | `to_ttlv_payload` | `unit/rng_seed_operation_tests.rs` |
| SetAttribute | `src/set_attribute.rs:25` `SetAttributeRequest` | `to_ttlv_payload` | `unit/set_attribute_tests.rs` |
| SetConstraints | `src/set_constraints.rs:20` `SetConstraintsRequest` | `to_ttlv_payload` | `unit/set_constraints_tests.rs` |
| SetDefaults | `src/set_defaults.rs:20` `SetDefaultsRequest` | `to_ttlv_payload` | `unit/set_defaults_tests.rs` |
| Sign | `src/sign.rs:17` `SignRequest` | `to_ttlv_payload` | `unit/sign_operation_tests.rs` |
| SignatureVerify | `src/signature_verify.rs:17` `SignatureVerifyRequest` | `to_ttlv_payload` | `unit/signature_verify_operation_tests.rs` |
| Validate | `src/validate.rs:33` `ValidateRequest` | `into_ttlv_payload` | `unit/validate_operation_tests.rs` |

## R001-R189 field-use mapping

Types are exact stored Rust field types except explicit nested projections/serializer-only values. Option<T> accepts T in setters or an optional constructor input. Input exceptions: Cancel/Poll/Process use &[u8]; Query functions use IntoIterator<Item=QueryFunction>, Object Groups use IntoIterator<Item=S:Into<String>>; QueryAsyncRequests uses IntoIterator<Item=Vec<u8>> / IntoIterator<Item=u32>; GetAttributes uses IntoIterator<Item=AttributeReference>. Raw Structure/Item/Value are owned generic TTLV. CP setters also accept typed CryptographicParameters converted to the same raw storage. Private LeaseTime is Interval(u32)|DateTime(i64), exposed via separate setters.

| Rule | Operation / payload field | Exact Rust field/projection | Stored/input type | Construction; shared schema |
|---|---|---|---|---|
| R001 | Activate / Unique Identifier | `ActivateRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R002 | AddAttribute / Unique Identifier | `AddAttributeRequest.unique_identifier` | `Option<String>` | US |
| R003 | AddAttribute / New Attribute | `AddAttributeRequest.new_attribute -> NewAttribute.item: Item` | `NewAttribute` | N; NEW |
| R004 | AdjustAttribute / Unique Identifier | `AdjustAttributeRequest.unique_identifier` | `Option<String>` | US |
| R005 | AdjustAttribute / Attribute Reference | `AdjustAttributeRequest.attribute_reference` | `AttributeReference` | AR; REF |
| R006 | AdjustAttribute / Adjustment Type | `AdjustAttributeRequest.adjustment_type` | `AdjustmentType` | S |
| R007 | AdjustAttribute / Adjustment Value | `AdjustAttributeRequest.adjustment_value` | `Option<Value>` | R; ADJUST |
| R008 | Archive / Unique Identifier | `ArchiveRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R009 | Cancel / Asynchronous Correlation Value | `CancelRequest.asynchronous_correlation_value` | `SecretBytes` | S |
| R010 | Certify / Unique Identifier | `CertifyRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R011 | Certify / Certificate Request Type | `CertifyRequest.certificate_request_type` | `Option<u32>` | S |
| R012 | Certify / Certificate Request Value | `CertifyRequest.certificate_request_value` | `Option<SecretBytes>` | S |
| R013 | Certify / Attributes | `CertifyRequest.attributes` | `Option<Structure>` | R; ATTR |
| R014 | Certify / Protection Storage Masks | `CertifyRequest.protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R015 | Check / Unique Identifier | `CheckRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R016 | Check / Usage Limits Count | `CheckRequest.usage_limits_count` | `Option<i64>` | S |
| R017 | Check / Cryptographic Usage Mask | `CheckRequest.cryptographic_usage_mask` | `Option<i32>` | S |
| R018 | Check / Lease Time | `CheckRequest.lease_time` | `Option<u32>` | S |
| R019 | Create / Object Type | `CreateRequest.object_type` | `ObjectType` | S |
| R020 | Create / Attributes | `CreateRequest.attributes` | `AttributeSet` | A; ATTR |
| R021 | Create / Protection Storage Masks | `CreateRequest.protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R022 | CreateKeyPair / Common Attributes | `CreateKeyPairRequest.common_attributes` | `Option<AttributeSet>` | A; COMMON |
| R023 | CreateKeyPair / Private Key Attributes | `CreateKeyPairRequest.private_key_attributes` | `Option<AttributeSet>` | A; PRIVATE |
| R024 | CreateKeyPair / Public Key Attributes | `CreateKeyPairRequest.public_key_attributes` | `Option<AttributeSet>` | A; PUBLIC |
| R025 | CreateKeyPair / Common Protection Storage Masks | `CreateKeyPairRequest.common_protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R026 | CreateKeyPair / Private Protection Storage Masks | `CreateKeyPairRequest.private_protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R027 | CreateKeyPair / Public Protection Storage Masks | `CreateKeyPairRequest.public_protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R028 | CreateSplitKey / Object Type | `CreateSplitKeyRequest.object_type` | `ObjectType` | S |
| R029 | CreateSplitKey / Unique Identifier | `CreateSplitKeyRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R030 | CreateSplitKey / Split Key Parts | `CreateSplitKeyRequest.split_key_parts` | `i32` | S |
| R031 | CreateSplitKey / Split Key Threshold | `CreateSplitKeyRequest.split_key_threshold` | `i32` | S |
| R032 | CreateSplitKey / Split Key Method | `CreateSplitKeyRequest.split_key_method` | `SplitKeyMethod` | S |
| R033 | CreateSplitKey / Prime Field Size | `CreateSplitKeyRequest.prime_field_size` | `Option<Vec<u8>>` | S |
| R034 | CreateSplitKey / Attributes | `CreateSplitKeyRequest.attributes` | `AttributeSet` | A; ATTR |
| R035 | CreateSplitKey / Protection Storage Masks | `CreateSplitKeyRequest.protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R036 | Decrypt / Unique Identifier | `DecryptRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R037 | Decrypt / Cryptographic Parameters | `DecryptRequest.cryptographic_parameters` | `Option<Structure>` | R; CP |
| R038 | Decrypt / Data | `DecryptRequest.data` | `Option<OperationData>` | D |
| R039 | Decrypt / IV/Counter/Nonce | `DecryptRequest.iv_counter_nonce` | `Option<SecretBytes>` | S |
| R040 | Decrypt / Correlation Value | `DecryptRequest.correlation_value` | `Option<SecretBytes>` | S |
| R041 | Decrypt / Init Indicator | `DecryptRequest.init_indicator` | `Option<bool>` | S |
| R042 | Decrypt / Final Indicator | `DecryptRequest.final_indicator` | `Option<bool>` | S |
| R043 | Decrypt / Authenticated Encryption Additional Data | `DecryptRequest.authenticated_encryption_additional_data` | `Option<SecretBytes>` | S |
| R044 | Decrypt / Authenticated Encryption Tag | `DecryptRequest.authenticated_encryption_tag` | `Option<SecretBytes>` | S |
| R045 | DelegatedLogin / Lease Time | `DelegatedLoginRequest.lease_time` | `Option<LeaseTime>` | S |
| R046 | DelegatedLogin / Request Count | `DelegatedLoginRequest.request_count` | `Option<i32>` | S |
| R047 | DelegatedLogin / Usage Limits | `DelegatedLoginRequest.usage_limits` | `Option<Structure>` | R; USAGE |
| R048 | DelegatedLogin / Rights | `DelegatedLoginRequest.rights` | `Structure` | R; RIGHTS |
| R049 | DeleteAttribute / Unique Identifier | `DeleteAttributeRequest.unique_identifier` | `Option<String>` | US |
| R050 | DeleteAttribute / Current Attribute | `DeleteAttributeRequest.current_attribute -> CurrentAttribute.item: Item` | `Option<CurrentAttribute>` | N; CURRENT |
| R051 | DeleteAttribute / Attribute Reference | `DeleteAttributeRequest.attribute_reference` | `Option<AttributeReference>` | AR; REF |
| R052 | DeriveKey / Object Type | `DeriveKeyRequest.object_type` | `ObjectType` | S |
| R053 | DeriveKey / Unique Identifier | `DeriveKeyRequest.unique_identifiers` | `Vec<UniqueIdentifier>` | U |
| R054 | DeriveKey / Derivation Method | `DeriveKeyRequest.derivation_method` | `DerivationMethod` | S |
| R055 | DeriveKey / Derivation Parameters | `DeriveKeyRequest.derivation_parameters` | `DerivationParameters` | B; DERIVE |
| R056 | DeriveKey / Attributes | `DeriveKeyRequest.attributes` | `AttributeSet` | A; ATTR |
| R057 | Destroy / Unique Identifier | `DestroyRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R058 | DiscoverVersions / Protocol Version | `DiscoverVersionsRequest.protocol_versions() -> serializer-only Protocol Version` | `[ProtocolVersion; 1] fixed (2,1); no input` | F; VERSION |
| R059 | Encrypt / Unique Identifier | `EncryptRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R060 | Encrypt / Cryptographic Parameters | `EncryptRequest.cryptographic_parameters` | `Option<Structure>` | R; CP |
| R061 | Encrypt / Data | `EncryptRequest.data` | `Option<OperationData>` | D |
| R062 | Encrypt / IV/Counter/Nonce | `EncryptRequest.iv_counter_nonce` | `Option<SecretBytes>` | S |
| R063 | Encrypt / Correlation Value | `EncryptRequest.correlation_value` | `Option<SecretBytes>` | S |
| R064 | Encrypt / Init Indicator | `EncryptRequest.init_indicator` | `Option<bool>` | S |
| R065 | Encrypt / Final Indicator | `EncryptRequest.final_indicator` | `Option<bool>` | S |
| R066 | Encrypt / Authenticated Encryption Additional Data | `EncryptRequest.authenticated_encryption_additional_data` | `Option<SecretBytes>` | S |
| R067 | Export / Unique Identifier | `ExportRequest.selectors: GetRequest -> GetRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R068 | Export / Key Format Type | `ExportRequest.selectors: GetRequest -> GetRequest.key_format_type` | `Option<KeyFormatType>` | S |
| R069 | Export / Key Wrap Type | `ExportRequest.selectors: GetRequest -> GetRequest.key_wrap_type` | `Option<KeyWrapType>` | S |
| R070 | Export / Key Compression Type | `ExportRequest.selectors: GetRequest -> GetRequest.key_compression_type` | `Option<KeyCompressionType>` | S |
| R071 | Export / Key Wrapping Specification | `ExportRequest.selectors: GetRequest -> GetRequest.key_wrapping_specification` | `Option<Structure>` | R; WRAP |
| R072 | Get / Unique Identifier | `GetRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R073 | Get / Key Format Type | `GetRequest.key_format_type` | `Option<KeyFormatType>` | S |
| R074 | Get / Key Wrap Type | `GetRequest.key_wrap_type` | `Option<KeyWrapType>` | S |
| R075 | Get / Key Compression Type | `GetRequest.key_compression_type` | `Option<KeyCompressionType>` | S |
| R076 | Get / Key Wrapping Specification | `GetRequest.key_wrapping_specification` | `Option<Structure>` | R; WRAP |
| R077 | GetAttributes / Unique Identifier | `GetAttributesRequest.unique_identifier` | `Option<String>` | US |
| R078 | GetAttributes / Attribute Reference | `GetAttributesRequest.attribute_references` | `Vec<AttributeReference>` | L/AR; REF |
| R079 | GetAttributeList / Unique Identifier | `GetAttributeListRequest.unique_identifier` | `Option<String>` | US |
| R080 | GetUsageAllocation / Unique Identifier | `GetUsageAllocationRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R081 | GetUsageAllocation / Usage Limits Count | `GetUsageAllocationRequest.usage_limits_count` | `i64` | S |
| R082 | Hash / Cryptographic Parameters | `HashRequest.parameters` | `Structure` | R; CP |
| R083 | Hash / Data | `HashRequest.data` | `Option<OperationData>` | D |
| R084 | Hash / Correlation Value | `HashRequest.correlation` | `Option<SecretBytes>` | S |
| R085 | Hash / Init Indicator | `HashRequest.init` | `Option<bool>` | S |
| R086 | Hash / Final Indicator | `HashRequest.final_part` | `Option<bool>` | S |
| R087 | Import / Unique Identifier | `ImportRequest.unique_identifier` | `UniqueIdentifier` | U |
| R088 | Import / Object Type | `ImportRequest.object_type` | `ObjectType` | S |
| R089 | Import / Replace Existing | `ImportRequest.replace_existing` | `Option<bool>` | S |
| R090 | Import / Key Wrap Type | `ImportRequest.key_wrap_type` | `Option<KeyWrapType>` | S |
| R091 | Import / Attributes | `ImportRequest.attributes` | `AttributeSet` | A; ATTR |
| R092 | Import / Any Object (Section 2) | `ImportRequest.object` | `Item` | O; OBJECT |
| R093 | Interop / Interop Function | `InteropRequest.interop_function` | `u32` | S |
| R094 | Interop / Interop Identifier | `InteropRequest.interop_identifier` | `SecretText` | S |
| R095 | JoinSplitKey / Object Type | `JoinSplitKeyRequest.object_type` | `ObjectType` | S |
| R096 | JoinSplitKey / Unique Identifier | `JoinSplitKeyRequest.references -> SplitKeyReference.unique_identifier: UniqueIdentifier (threshold metadata not serialized)` | `Vec<SplitKeyReference>` | L/U |
| R097 | JoinSplitKey / Secret Data Type | `JoinSplitKeyRequest.secret_data_type` | `Option<u32>` | S |
| R098 | JoinSplitKey / Attributes | `JoinSplitKeyRequest.attributes` | `Option<AttributeSet>` | A; ATTR |
| R099 | JoinSplitKey / Protection Storage Masks | `JoinSplitKeyRequest.protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R100 | Locate / Maximum Items | `LocateRequest.maximum_items` | `Option<i32>` | S |
| R101 | Locate / Offset Items | `LocateRequest.offset_items` | `Option<i32>` | S |
| R102 | Locate / Storage Status Mask | `LocateRequest.storage_status_mask` | `Option<StorageStatusMask>` | S |
| R103 | Locate / Object Group Member | `LocateRequest.object_group_member` | `Option<ObjectGroupMember>` | S |
| R104 | Locate / Attributes | `LocateRequest.attributes` | `AttributeSet` | A; ATTR |
| R105 | Log / Log Message | `LogRequest.log_message` | `SecretText` | S |
| R106 | Login / Lease Time | `LoginRequest.lease_time` | `Option<LeaseTime>` | S |
| R107 | Login / Request Count | `LoginRequest.request_count` | `Option<i32>` | S |
| R108 | Login / Usage Limits | `LoginRequest.usage_limits` | `Option<Structure>` | R; USAGE |
| R109 | Logout / Ticket | `LogoutRequest.ticket` | `Ticket` | T; TICKET |
| R110 | Mac / Unique Identifier | `MacRequest.identifier` | `Option<UniqueIdentifier>` | U |
| R111 | Mac / Cryptographic Parameters | `MacRequest.parameters` | `Option<Structure>` | R; CP |
| R112 | Mac / Data | `MacRequest.data` | `Option<OperationData>` | D |
| R113 | Mac / Correlation Value | `MacRequest.correlation` | `Option<SecretBytes>` | S |
| R114 | Mac / Init Indicator | `MacRequest.init` | `Option<bool>` | S |
| R115 | Mac / Final Indicator | `MacRequest.final_part` | `Option<bool>` | S |
| R116 | MacVerify / Unique Identifier | `MacVerifyRequest.identifier` | `Option<UniqueIdentifier>` | U |
| R117 | MacVerify / Cryptographic Parameters | `MacVerifyRequest.parameters` | `Option<Structure>` | R; CP |
| R118 | MacVerify / Data | `MacVerifyRequest.data` | `Option<OperationData>` | D |
| R119 | MacVerify / MAC Data | `MacVerifyRequest.mac_data` | `Option<SecretBytes>` | S; current tag mismatch, see final section |
| R120 | MacVerify / Correlation Value | `MacVerifyRequest.correlation` | `Option<SecretBytes>` | S |
| R121 | MacVerify / Init Indicator | `MacVerifyRequest.init` | `Option<bool>` | S |
| R122 | MacVerify / Final Indicator | `MacVerifyRequest.final_part` | `Option<bool>` | S |
| R123 | ModifyAttribute / Unique Identifier | `ModifyAttributeRequest.unique_identifier` | `Option<String>` | US |
| R124 | ModifyAttribute / Current Attribute | `ModifyAttributeRequest.current_attribute -> CurrentAttribute.item: Item` | `Option<CurrentAttribute>` | N; CURRENT |
| R125 | ModifyAttribute / New Attribute | `ModifyAttributeRequest.new_attribute -> NewAttribute.item: Item` | `NewAttribute` | N; NEW |
| R126 | ObtainLease / Unique Identifier | `ObtainLeaseRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R127 | Pkcs11 / PKCS#11 Interface | `Pkcs11Request.interface` | `Option<Structure>` | R; PKCS_IF |
| R128 | Pkcs11 / PKCS#11 Function | `Pkcs11Request.function` | `Structure` | R; PKCS_FN |
| R129 | Pkcs11 / Correlation Value | `Pkcs11Request.correlation` | `Option<SecretBytes>` | S |
| R130 | Pkcs11 / PKCS#11 Input Parameters | `Pkcs11Request.input` | `Option<Structure>` | R; PKCS_IN |
| R131 | Poll / Asynchronous Correlation Value | `PollRequest.asynchronous_correlation_value` | `SecretBytes` | S |
| R132 | Process / Asynchronous Correlation Value | `ProcessRequest.asynchronous_correlation_value` | `SecretBytes` | S |
| R133 | Query / Query Function | `QueryRequest.functions` | `Vec<QueryFunction>` | L |
| R134 | Query / Object Groups | `QueryRequest.object_groups -> serializer-only Structure/Object Group children` | `Option<Vec<String>>` | L; GROUPS |
| R135 | QueryAsyncRequests / Asynchronous Correlation Values | `QueryAsyncRequestsRequest.correlation_values -> serializer-only Structure/singular value children` | `Option<Vec<SecretBytes>>` | L; ASYNC |
| R136 | QueryAsyncRequests / Operations | `QueryAsyncRequestsRequest.operations -> serializer-only Structure/Operation children` | `Option<Vec<u32>>` | L; OPS |
| R137 | Recover / Unique Identifier | `RecoverRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R138 | Register / Object Type | `RegisterRequest.object_type` | `ObjectType` | S |
| R139 | Register / Attributes | `RegisterRequest.attributes` | `AttributeSet` | A; ATTR |
| R140 | Register / Any Object (Section 2) | `RegisterRequest.object` | `Item` | O; OBJECT |
| R141 | Register / Protection Storage Masks | `RegisterRequest.protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R142 | Revoke / Unique Identifier | `RevokeRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R143 | Revoke / Revocation Reason | `RevokeRequest.revocation_reason + revocation_message: Option<String> -> serializer-only Structure` | `RevocationReason` | B; REASON |
| R144 | Revoke / Compromise Occurrence Date | `RevokeRequest.compromise_occurrence_date` | `Option<i64>` | S |
| R145 | Recertify / Unique Identifier | `RecertifyRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R146 | Recertify / Certificate Request Unique Identifier | `RecertifyRequest.certificate_request_unique_identifier` | `Option<UniqueIdentifier>` | U |
| R147 | Recertify / Certificate Request Type | `RecertifyRequest.certificate_request_type` | `Option<u32>` | S |
| R148 | Recertify / Certificate Request Value | `RecertifyRequest.certificate_request_value` | `Option<SecretBytes>` | S |
| R149 | Recertify / Offset | `RecertifyRequest.offset` | `Option<u32>` | S |
| R150 | Recertify / Attributes | `RecertifyRequest.attributes` | `Option<Structure>` | R; ATTR |
| R151 | Recertify / Protection Storage Masks | `RecertifyRequest.protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R152 | Rekey / Unique Identifier | `RekeyRequest.unique_identifier` | `Option<UniqueIdentifier>` | U |
| R153 | Rekey / Offset | `RekeyRequest.offset` | `Option<u32>` | S |
| R154 | Rekey / Attributes | `RekeyRequest.attributes` | `Option<Structure>` | R; ATTR |
| R155 | Rekey / Protection Storage Masks | `RekeyRequest.protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R156 | RekeyKeyPair / Private Key Unique Identifier | `RekeyKeyPairRequest.private_key_unique_identifier` | `Option<UniqueIdentifier>` | U |
| R157 | RekeyKeyPair / Offset | `RekeyKeyPairRequest.offset` | `Option<u32>` | S |
| R158 | RekeyKeyPair / Common Attributes | `RekeyKeyPairRequest.common_attributes` | `Option<AttributeSet>` | A; COMMON |
| R159 | RekeyKeyPair / Private Key Attributes | `RekeyKeyPairRequest.private_key_attributes` | `Option<AttributeSet>` | A; PRIVATE |
| R160 | RekeyKeyPair / Public Key Attributes | `RekeyKeyPairRequest.public_key_attributes` | `Option<AttributeSet>` | A; PUBLIC |
| R161 | RekeyKeyPair / Common Protection Storage Masks | `RekeyKeyPairRequest.common_protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R162 | RekeyKeyPair / Private Protection Storage Masks | `RekeyKeyPairRequest.private_protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R163 | RekeyKeyPair / Public Protection Storage Masks | `RekeyKeyPairRequest.public_protection_storage_masks` | `Option<Structure>` | R; MASKS |
| R164 | ReProvision / Certificate Request | `ReProvisionRequest.certificate_request` | `Option<Structure>` | R; CERTREQ |
| R165 | ReProvision / Certificate | `ReProvisionRequest.certificate` | `Option<Structure>` | R; CERT |
| R166 | RngRetrieve / Data Length | `RngRetrieveRequest.data_length` | `i32` | S; current tag mismatch, see final section |
| R167 | RngSeed / Data | `RngSeedRequest.data` | `SecretBytes` | S |
| R168 | SetAttribute / Unique Identifier | `SetAttributeRequest.unique_identifier` | `Option<String>` | US |
| R169 | SetAttribute / New Attribute | `SetAttributeRequest.new_attribute -> NewAttribute.item: Item` | `NewAttribute` | N; NEW |
| R170 | SetConstraints / Constraints | `SetConstraintsRequest.constraints` | `Structure` | R; CONSTRAINTS |
| R171 | SetDefaults / Defaults Information | `SetDefaultsRequest.defaults_information` | `Option<Structure>` | R; DEFAULTS |
| R172 | Sign / Unique Identifier | `SignRequest.identifier` | `Option<UniqueIdentifier>` | U |
| R173 | Sign / Cryptographic Parameters | `SignRequest.parameters` | `Option<Structure>` | R; CP |
| R174 | Sign / Data | `SignRequest.data` | `Option<OperationData>` | D |
| R175 | Sign / Digested Data | `SignRequest.digested_data` | `Option<SecretBytes>` | S |
| R176 | Sign / Correlation Value | `SignRequest.correlation` | `Option<SecretBytes>` | S |
| R177 | Sign / Init Indicator | `SignRequest.init` | `Option<bool>` | S |
| R178 | Sign / Final Indicator | `SignRequest.final_part` | `Option<bool>` | S |
| R179 | SignatureVerify / Unique Identifier | `SignatureVerifyRequest.identifier` | `Option<UniqueIdentifier>` | U |
| R180 | SignatureVerify / Cryptographic Parameters | `SignatureVerifyRequest.parameters` | `Option<Structure>` | R; CP |
| R181 | SignatureVerify / Data | `SignatureVerifyRequest.data` | `Option<OperationData>` | D |
| R182 | SignatureVerify / Digested Data | `SignatureVerifyRequest.digested_data` | `Option<SecretBytes>` | S |
| R183 | SignatureVerify / Signature Data | `SignatureVerifyRequest.signature_data` | `Option<SecretBytes>` | S |
| R184 | SignatureVerify / Correlation Value | `SignatureVerifyRequest.correlation` | `Option<SecretBytes>` | S |
| R185 | SignatureVerify / Init Indicator | `SignatureVerifyRequest.init` | `Option<bool>` | S |
| R186 | SignatureVerify / Final Indicator | `SignatureVerifyRequest.final_part` | `Option<bool>` | S |
| R187 | Validate / Certificate | `ValidateRequest.certificates` | `Vec<Structure>` | R; CERT |
| R188 | Validate / Unique Identifier | `ValidateRequest.identifiers` | `Vec<UniqueIdentifier>` | L/U |
| R189 | Validate / Validity Date | `ValidateRequest.validity_date` | `Option<i64>` | S |

## Existing checks and limits of current evidence

The index gives each operation's nearest test file. Named tests below prove only the specified existing behavior; other nearby response tests do not validate request rules.

- Activate/Archive/Destroy/Recover/ObtainLease, Check, GetUsageAllocation: fixed scalar serializers; no broad value validator. Tests cover omission/order/wire-value preservation, not ranges or remote policy. Lifecycle `request_preserves_each_supported_unique_identifier_wire_form` tests intentionally exercise Integer. Check preserves optional values rather than imposing policy.
- Add/Modify/Set/Delete/Adjust Attribute: Current/New wrappers serialize one direct Item; Attribute Reference serialization checks allocation/name. Adjust rejects reserved Adjustment Type values (`adjust_attribute_tests::reserved_adjustment_type_boundaries_are_preserved_but_rejected_for_outbound_requests`). Delete model permits both/neither optional selectors; no XOR inferred. Focused tests prove preservation/order, not full nested attribute validation. Policy helpers in `attribute.rs` are not proof every request serializer calls them.
- GetAttributes: `try_new` rejects identical duplicate references; serializer checks each `to_ttlv_item`. `get_attributes_tests::request_rejects_a_repeated_identical_attribute_reference` proves constructor rejection; `request_rejects_a_reserved_tag_form_reference_before_encoding` proves serializer rejection. Zero references is allowed.
- Certify: consuming serializer checks Attribute single-instance cardinality, Request Value => Request Type, explicit non-placeholder public-key identifier without request bytes, and Certificate Type/request-type fallback. `certify_operation_tests::request_requires_certificate_request_type_when_request_value_is_present` and `request_without_request_value_requires_public_key_identifier_and_certificate_type_attribute` are existing evidence. Raw Attributes do not acquire AttributeSet's insertion checks merely because selected cardinality checks execute.
- Recertify: consuming serializer checks selected attribute cardinality, Request Value => Request Type and explicit non-placeholder Unique Identifier without request bytes. Certificate Request Unique Identifier is a separate field. Tests include `request_requires_type_when_certificate_request_value_is_present`, `request_requires_unique_identifier_when_certificate_request_value_is_omitted`, `request_can_omit_request_fields_to_reuse_existing_certificate_type`.
- Create/CreateKeyPair/CreateSplitKey/DeriveKey/JoinSplitKey/Rekey/RekeyKeyPair/Import/Register: source-derived attribute cardinality is checked during serialization or Register's borrowed shape check, using operation-specific object/certificate context. AttributeSet itself preserves duplicates (`typed_attribute_tests::typed_attribute_set_preserves_singleton_duplicates_for_operation_validation`). Locate intentionally preserves repeated search predicates.
- CreateKeyPair additionally checks Table191 effective pair values via `validate_table_191`. Tests cover common fallback, overrides, one-sided/conflicting values and nested comparisons. This is not Protection Storage Masks or universal structured-attribute validation.
- CreateSplitKey `validate_client_policy` checks only known polynomial-prime method => supplied Prime Field Size. It does not check positivity or threshold <= parts. `polynomial_prime_field_request_requires_the_explicit_client_input` and `polynomial_prime_field_size_is_preserved_as_big_integer` prove presence/preservation, not primality. Prime Field Size accepts arbitrary Vec<u8>; generic Big Integer model/wire constraints are separate.
- DeriveKey serializer checks nonempty identifiers, ID Placeholder Enumeration(1) prohibition, required output algorithm/length, PBKDF2 Salt/Iteration Count presence and HASH algorithm presence via `valid_for_method`. It does not recursively apply full CP validation to every raw nested parameter: `has_hashing_algorithm` merely finds Enumeration. `derive_key_operation_tests::both_derivation_input_forms_are_sent_and_server_error_is_not_rejected_locally` deliberately preserves both forms when remote Secret Data type is unknown. `DerivationParameters.with_extension` only generic-try_pushes; known-tag shadowing is possible.
- JoinSplitKey `validate_request_shape` and serializer share checks for nonempty references, caller-supplied threshold metadata on each entry, count >= maximum supplied threshold, and source-derived Attribute cardinality. `SplitKeyReference.split_key_threshold:Option<u32>` is local metadata absent from wire, not a verified server-object fact. `typed_request_validation_tests.rs` covers the public report; focused `join_split_key_operation_tests.rs` proves serializer behavior, not remote threshold verification.
- Encrypt/Decrypt have borrowed `validate_multipart_shape`; consuming serializers run Table59 CP validation and multipart checks. `multipart_validation_tests.rs` covers init/final/correlation/data matrix; `encrypt_parameter_structure_tests.rs` covers each known Table59 wrong type/duplicate. Optional IV/AAD/tag setters ensure Byte String, but these checks do not establish every algorithm/first-part-only condition in operation prose. Unknown nested members remain retained.
- Hash/Mac/MacVerify/Sign/SignatureVerify have borrowed multipart methods; common consuming encoding applies CP rules. Borrowed Hash validation now checks Data framing, required Hashing Algorithm, and recursively validates the supplied §4.16 Cryptographic Parameters shape. Focused tests cover separate required inputs: Data for Hash/Mac, MAC Data for MacVerify, Data-or-Digested Data for Sign, Signature Data for SignatureVerify. Original Data is optional in verify operations. Do not infer every prose condition from common framing. `cryptographic_operation_contract_tests.rs` tests malformed parameters and borrowed shape methods, not cryptographic byte semantics.
- Import consuming serializer checks object Structure, attribute/key-value cardinality and Key Wrap Type iff detected wrapped-key state. `import_operation_tests::key_wrap_type_is_present_if_and_only_if_key_block_has_wrapping_data` proves the conditional. It does not decode wrapped bytes; ManagedObject constructor is a stronger alternate path, not removal of raw Item input.
- Register borrowed `validate_local_shape` checks object Structure, attribute/key-value cardinality, selected Table293 metadata and OpaqueObjectContext applicability. Opaque validation checks Table4 fields. Selected key metadata checks are not universal object validity. `register_operation_tests.rs` covers shape/metadata pairs/opaque context/preservation. `opaque_object_context:Option<OpaqueObjectContext>` is local metadata, not a Table291 field.
- Get and Export retain caller-supplied raw Key Wrapping Specification. Borrowed `ValidateKmipRequest::validate()` now recursively checks the shared Table 370 schema for both operations; the lower-level payload serializers remain encoders and do not themselves perform the preflight. `typed_request_validation_tests::get_validation_checks_the_supplied_key_wrapping_specification` and `typed_request_validation_tests::export_validation_checks_the_supplied_key_wrapping_specification` provide rejection evidence, while the operation tests retain valid wire vectors.
- Login/DelegatedLogin separate private LeaseTime enums encode Interval or Date-Time; Usage Limits/Rights are wrapped without shared-schema validation. Focused tests cover order/both lease forms/redaction/nesting limits/returned Ticket. They do not enforce request Tables386/387/392.
- Logout required Ticket shape comes from its checked constructor/parser, plus outer nesting checks at serialization. `ticket_model_tests::ttlv_parse_rejects_missing_duplicate_wrong_type_and_out_of_order_fields` is shared construction evidence. No ticket acceptance inference.
- Pkcs11 `validate_request_shape` and the serializer share checks for the required Function structure, optional Interface/Input nested types and counts, and extension collisions. Unknown/profile members remain opaque; function-specific Input Parameters bytes remain uninterpreted. `pkcs11_operation_tests.rs` covers the public report and serializer cases; `execute_lifecycle_tests::malformed_pkcs11_function_is_not_sent` verifies preflight delivery state and zero transport writes.
- Query serializer rejects empty function list/reserved Query Function and constructs Object Groups from typed Strings. `query_operation_tests::query_rejects_an_empty_function_list_before_it_can_be_encoded` proves the former. Server function support/group existence are excluded.
- QueryAsyncRequests serializes typed lists, distinguishing absent/present-empty and retaining duplicates/raw operation values. Focused `query_async_requests_tests.rs` does not prove positional correspondence or equal-length rules, which source does not state. Source filter prose does not require local outstanding-request state; do not invent positional pairing or an additional combined-filter restriction.
- Interop/Log raw `extensions:Vec<Item>` can be appended by builder; borrowed request validation now rejects a known field collision before client encoding, while `diagnostics::append_extensions` remains the serializer's defensive check. Unknown extension tags retain relative order. `typed_request_validation_tests::diagnostic_requests_preflight_reject_extensions_that_override_defined_fields` and `execute_lifecycle_tests::diagnostic_extension_cannot_override_interop_fields_before_transport` cover validation, redaction, and zero transport exchanges/writes.
- Revoke consuming serializer rejects Compromise Occurrence Date except for reason2 or3. `revoke_operation_tests::request_rejects_compromise_date_for_every_other_reason` proves this. Source SHOULD for supplying a compromise date is not a MUST and needs separate warning/deviation treatment.
- ReProvision serializer fixes wrappers/order but does not check Tables2/3 children. Tests prove request/certificate/both/none preservation and nesting limits, not malformed child rejection.
- RngRetrieve/RngSeed fix i32 Integer / SecretBytes Byte String. `rng_retrieve_operation_tests::request_preserves_all_caller_supplied_integer_values` intentionally preserves signed boundaries, not positive-length validation. Seed tests preserve empty/non-UTF8 bytes; they prove no entropy-quality rule.
- SetConstraints/SetDefaults preserve raw Structures and now run borrowed recursive checks for the identified §7.6/Table357, §7.7/Table358, §7.11/Table363, §7.22/Table374, §7.23/Table375, §7.24/Table376, and §5.1/Table157 shapes. `nested-schemas.md` lists executable evidence. Other nested schema families remain open under T009a and their operation-family tasks; policy acceptance remains server-side.
- Validate consuming serializer checks at least one Certificate or Unique Identifier, and Table2 known child shape. Exact tests: `req_001_rejects_empty_chain_even_when_validity_date_is_present`, `table_2_certificate_structure_rejects_missing_wrong_and_duplicate_fields`, `table_2_certificate_value_before_type_is_rejected_structurally`. Its raw `extensions:Vec<Item>` collisions with Certificate/Unique Identifier/Validity Date are rejected by append_extensions during serialization. No DER/trust evaluation.
- DiscoverVersions has only serializer-fixed (2,1); `discover_versions_tests::request_advertises_only_kmip_2_1` is exact evidence. Empty GetConstraints/Ping have no R IDs: fieldless models produce empty structures, proven by `get_constraints_tests::table_229_request_is_exactly_empty` and `ping_operation_tests::ping_request_has_an_empty_operation_payload`.

## T002/T003 project rulings

1. **IV/Counter/Nonce request encoding:** Tables 196 and 214 omit the type. Use Byte String for the explicit request schema as the recommended encoding, consistent with the KMIPKit `SecretBytes` typed request and §3.3 Table 15, while documenting it as project decision `KMIPKIT-0051-P004`, not a direct statement in those request tables.
2. **Request Count and standalone Usage Limits Count:** use Integer for Request Count (the operation descriptions say “integer count” and the typed fields are `i32`) and Long Integer for Usage Limits Count (the shared field is Long Integer in §7.40 Table 392 and the typed Check field is `i64`). Record both as project encoding decisions where the standalone Request Payload table is silent; do not impose a positive-only range because the cited rows define none.
3. **Adjustment Value:** use the generic TTLV Value model. The target attribute and adjustment type determine any possible value rule; no universal scalar type is appropriate.
4. **Unique Identifier:** preserve Text String, Enumeration, and Integer forms from §4.58 Tables 145–146; Table 489 supplies ID Placeholder enum values. Context-specific restrictions remain separate. Some attribute-operation models expose only String; record that representational limitation for KMIPKIT-0053 rather than rejecting the broader schema type.
5. **Nested wrapping identifiers:** Tables 16–17 specify Text String in the key-information structures. The explicit Key Wrapping Specification schema enforces that local type in context; the general UniqueIdentifier model remains a union elsewhere.
6. **Ambiguous combinations:** optional fields remain optional unless the operation text supplies a one-way or explicit conditional. Do not invent reverse dependencies, XOR, list pairing, numeric ranges, or state-dependent rules; preserve the caller's values for server evaluation.

## Genuine reconciliation points

1. **Protection Storage Masks:** Table385/prose supplies outer Structure/ordered selections but no normative child schema. Do not derive tag/type/count from fixtures.
2. **Plural correlation container resolved:** §7.1 Table 352 defines its shared Structure and §11.56 Table 487 explicitly assigns Asynchronous Correlation Values 0x420176. Current serializer matches; the matrix records that tag, not singular 0x420006.
3. **Nested wrapping identifiers:** Tables 16–17 state Text String. Their context-specific schema validation uses Text String even though the broader builder accepts the UniqueIdentifier union.
4. **Unique Identifier encoding:** Tables 145–146 list all three forms and current tests preserve Integer. The matrix uses the union; only explicit operation-specific restrictions, such as Derive Key’s ID Placeholder prohibition, narrow it.

## Completeness

All R001-R189 occur once; all 54 field-bearing operation models have serializer/test pointers. GetConstraints/Ping account for the other two operations. Extra raw surfaces outside those field rows are explicitly covered: DerivationParameters/Interop/Log/Pkcs11/Validate extensions, Current/New Attribute direct Items, AttributeSet nested values, typed ManagedObject/CP/Ticket raw acceptance. Header Authentication/Credential belongs to the cross-cutting header audit, not an invented field in Login/DelegatedLogin Tables199/253.

## Current serializer/source mismatches (not proposed matrix replacements)

These are current implementation defects. Keep source-backed row tags and correct the serializers/readers with test-first commits T020–T022; tests must use pinned-source tag values independently of production constants.

| Field use | Current code | Pinned source | Implication |
|---|---|---|---|
| R119 MacVerify / MAC Data | `mac_verify.rs` uses shared `cryptographic_operation::MAC_DATA = 0x4200C4`; MAC response parsing in `mac.rs` also uses it. | §11.56 Table 487 MAC Data = 0x4200C6; §7.20 Table 372 defines Byte String. | Correct the shared field tag to 0x4200C6; this also restores parsing of the same MAC Data field in the existing MAC response reader without adding response validation rules. |
| R166 RngRetrieve / Data Length | `rng_retrieve.rs::DATA_LENGTH = 0x4200D5`; `rng_seed.rs` uses the same wrong value for the RNG Seed response Data Length reader. | §11.56 Table 487 Data Length = 0x4200C4; §7.10 Table 362 defines Integer; §6.1.50 Table 320 contains the RNG Seed response field. | Correct the shared Data Length tag to 0x4200C4 for the RNG Retrieve request and RNG Seed response reader. |

The scan compared identifiable per-operation/shared Rust constants against exact matching Table487 field names. It is not exhaustive proof of polymorphic objects or arbitrary extension tags. Reported mismatches were manually rechecked in the named source modules.
