# KMIPKIT-0050 Existing-Capability Gap Matrix

Reviewed against the KMIPKIT-0049 parent commit `815f3cc351ac234004a8c0cd532a1a8ecee5cde0`.

| Capability | Existing state | Gap and approved change | Acceptance evidence |
|---|---|---|---|
| Cryptographic Parameters | `crates/kmipkit-protocol/src/cryptographic_parameters.rs` validates a generic `Structure`; request builders and wrapping key-information builders accept generic `Structure`. There is no public typed `CryptographicParameters` model. | Add a public lossless Table 59 value and canonical builder; keep all current generic `Structure` entry points source-compatible and add additive typed setters for Encrypt, Decrypt, Derive Key, `DerivationParameters`, Sign, Signature Verify, MAC, MAC Verify, Encryption Key Information, and MAC Signature Key Information. | Tests cover all 18 members, ordering, types, cardinality, future enumeration values, unknown fields, typed setter integration, and §4.16 conditions. |
| Request Authentication | The protocol layer already exposes validated `Authentication` and a Request Header read view. `ClientBatch` has no Authentication field and `build_request_header` writes none. `Client::execute` documentation explicitly says it does not submit credentials. | Add an optional one-shot Authentication value to `ClientBatch` and the Request Header writer. Apply it once per Request Message, so it covers one or many Batch Items; do not place it in `ClientConfiguration` or `Client`. | Fake-transport assertions for omission/presence, one-item and multi-item scope, header position, Credential order, payload-free construction failure for invalid generic Authentication, redaction, and secret-buffer lifecycle. |
| Direct operation methods | `ClientOperation` has 56 variants. `ClientRequest` has 52 variants, with the four asynchronous request paths handled by `execute_poll`, `execute_cancel`, `execute_process`, and `execute_query_async_requests`. Fourteen `ClientRequest` variants lack a same-named direct method on `Client`. | Add: `discover_versions`, `delegated_login`, `login`, `logout`, `get`, `locate`, `add_attribute`, `adjust_attribute`, `delete_attribute`, `modify_attribute`, `set_attribute`, `get_attributes`, `encrypt`, and `decrypt`, each with `_with_options`. | Compile-time consumer names all 56 methods and option variants; fake transport compares new wrappers to one-item `ClientBatch` behavior. |
| Direct option parity | Ten existing direct methods have no `_with_options` counterpart: `get_constraints`, `get_attribute_list`, `set_constraints`, `set_defaults`, `get_usage_allocation`, `validate`, `rng_retrieve`, `log`, `interop`, and `rng_seed`. | Add `_with_options` counterparts using the same `RequestOptions` type and forwarding behavior already used by peer methods. | Compile-time signature inventory and option pass-through tests. |
| Secret lifetime / diagnostics | Credential values use secret wrapper types and the protocol layer redacts typed credential formatting. The request execution path owns transient TTLV and encoded request buffers. | Keep all secret ownership on the passed batch; inspect and test every clone/encoding boundary; ensure no new error or debug path formats the body or credentials and owned transient copies are zeroized. | Tests assert absence of canary credential text/bytes in formatting and verify owned buffer/model drop paths under established zeroization hooks. |

The direct-method counts come from `crates/kmipkit-client/src/execute.rs`:
`ClientRequest` at line 223, `ClientOperation` at line 1538, and Client methods
beginning at line 3825. They describe API mapping only; operation behavior and
normative payload schemas remain owned by their existing operation specs.

## Client operation to direct method inventory

This inventory is derived from the 56 `client_1_0` `KMIPKIT-ELEM-OP-C2S-*`
catalog entries. “Operation owner” is the catalog's existing `feature_spec`;
“unassigned” is retained where the catalog has no owner rather than inventing
one. Set Endpoint Role is intentionally excluded because its catalog scope is
`client_1_1`. Each row's exact stable operation element ID is
`KMIPKIT-ELEM-OP-C2S-` plus the uppercase, hyphen-separated operation key
(for example, `PKCS#11` uses `PKCS-11`); the catalog was queried directly to
verify every ID and owner recorded below.

| Operation | Current/proposed direct method | Current state | Operation owner |
|---|---|---|---|
| Activate (`KMIPKIT-ELEM-OP-C2S-ACTIVATE`) | `activate` | present; retain both forms | KMIPKIT-0018 |
| Add Attribute (`KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE`) | `add_attribute` | missing; add both forms | KMIPKIT-0016 |
| Adjust Attribute (`KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE`) | `adjust_attribute` | missing; add both forms | KMIPKIT-0016 |
| Archive (`KMIPKIT-ELEM-OP-C2S-ARCHIVE`) | `archive` | present; retain both forms | KMIPKIT-0018 |
| Cancel (`KMIPKIT-ELEM-OP-C2S-CANCEL`) | `execute_cancel` | present; retain both forms | unassigned; shared/core API |
| Certify (`KMIPKIT-ELEM-OP-C2S-CERTIFY`) | `certify` | present; retain both forms | KMIPKIT-0036 |
| Check (`KMIPKIT-ELEM-OP-C2S-CHECK`) | `check` | present; retain both forms | KMIPKIT-0036 |
| Create (`KMIPKIT-ELEM-OP-C2S-CREATE`) | `create` | present; retain both forms | KMIPKIT-0014 |
| Create Key Pair (`KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR`) | `create_key_pair` | present; retain both forms | KMIPKIT-0014 |
| Create Split Key (`KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY`) | `create_split_key` | present; retain both forms | KMIPKIT-0014 |
| Decrypt (`KMIPKIT-ELEM-OP-C2S-DECRYPT`) | `decrypt` | missing; add both forms | KMIPKIT-0019 |
| Delegated Login (`KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN`) | `delegated_login` | missing; add both forms | KMIPKIT-0038 |
| Delete Attribute (`KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE`) | `delete_attribute` | missing; add both forms | KMIPKIT-0016 |
| Derive Key (`KMIPKIT-ELEM-OP-C2S-DERIVE-KEY`) | `derive_key` | present; retain both forms | KMIPKIT-0037 |
| Destroy (`KMIPKIT-ELEM-OP-C2S-DESTROY`) | `destroy` | present; retain both forms | KMIPKIT-0018 |
| Discover Versions (`KMIPKIT-ELEM-OP-C2S-DISCOVER-VERSIONS`) | `discover_versions` | missing; add both forms | unassigned; shared/core API |
| Encrypt (`KMIPKIT-ELEM-OP-C2S-ENCRYPT`) | `encrypt` | missing; add both forms | KMIPKIT-0019 |
| Export (`KMIPKIT-ELEM-OP-C2S-EXPORT`) | `export` | present; retain both forms | KMIPKIT-0037 |
| Get (`KMIPKIT-ELEM-OP-C2S-GET`) | `get` | missing; add both forms | KMIPKIT-0017 |
| Get Attribute List (`KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST`) | `get_attribute_list` | present; add `_with_options` | KMIPKIT-0016 |
| Get Attributes (`KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES`) | `get_attributes` | missing; add both forms | KMIPKIT-0016 |
| Get Constraints (`KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS`) | `get_constraints` | present; add `_with_options` | KMIPKIT-0039 |
| Get Usage Allocation (`KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION`) | `get_usage_allocation` | present; add `_with_options` | KMIPKIT-0039 |
| Hash (`KMIPKIT-ELEM-OP-C2S-HASH`) | `hash` | present; retain both forms | KMIPKIT-0021 |
| Import (`KMIPKIT-ELEM-OP-C2S-IMPORT`) | `import` | present; retain both forms | KMIPKIT-0037 |
| Interop (`KMIPKIT-ELEM-OP-C2S-INTEROP`) | `interop` | present; add `_with_options` | KMIPKIT-0042 |
| Join Split Key (`KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY`) | `join_split_key` | present; retain both forms | KMIPKIT-0037 |
| Locate (`KMIPKIT-ELEM-OP-C2S-LOCATE`) | `locate` | missing; add both forms | KMIPKIT-0017 |
| Log (`KMIPKIT-ELEM-OP-C2S-LOG`) | `log` | present; add `_with_options` | KMIPKIT-0042 |
| Login (`KMIPKIT-ELEM-OP-C2S-LOGIN`) | `login` | missing; add both forms | KMIPKIT-0038 |
| Logout (`KMIPKIT-ELEM-OP-C2S-LOGOUT`) | `logout` | missing; add both forms | KMIPKIT-0038 |
| MAC (`KMIPKIT-ELEM-OP-C2S-MAC`) | `mac` | present; retain both forms | KMIPKIT-0021 |
| MAC Verify (`KMIPKIT-ELEM-OP-C2S-MAC-VERIFY`) | `mac_verify` | present; retain both forms | KMIPKIT-0021 |
| Modify Attribute (`KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE`) | `modify_attribute` | missing; add both forms | KMIPKIT-0016 |
| Obtain Lease (`KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE`) | `obtain_lease` | present; retain both forms | KMIPKIT-0036 |
| PKCS#11 (`KMIPKIT-ELEM-OP-C2S-PKCS-11`) | `pkcs11` | present; retain both forms | KMIPKIT-0040 |
| Ping (`KMIPKIT-ELEM-OP-C2S-PING`) | `ping` | present; retain both forms | KMIPKIT-0020 |
| Poll (`KMIPKIT-ELEM-OP-C2S-POLL`) | `execute_poll` | present; retain both forms | unassigned; shared/core API |
| Process (`KMIPKIT-ELEM-OP-C2S-PROCESS`) | `execute_process` | present; retain both forms | KMIPKIT-0009 |
| Query (`KMIPKIT-ELEM-OP-C2S-QUERY`) | `query` | present; retain both forms | KMIPKIT-0020 |
| Query Asynchronous Requests (`KMIPKIT-ELEM-OP-C2S-QUERY-ASYNCHRONOUS-REQUESTS`) | `execute_query_async_requests` | present; retain both forms | KMIPKIT-0009 |
| RNG Retrieve (`KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE`) | `rng_retrieve` | present; add `_with_options` | KMIPKIT-0041 |
| RNG Seed (`KMIPKIT-ELEM-OP-C2S-RNG-SEED`) | `rng_seed` | present; add `_with_options` | KMIPKIT-0041 |
| Re-Provision (`KMIPKIT-ELEM-OP-C2S-RE-PROVISION`) | `reprovision` | present; retain both forms | KMIPKIT-0036 |
| Re-certify (`KMIPKIT-ELEM-OP-C2S-RE-CERTIFY`) | `recertify` | present; retain both forms | KMIPKIT-0036 |
| Re-key (`KMIPKIT-ELEM-OP-C2S-RE-KEY`) | `rekey` | present; retain both forms | KMIPKIT-0036 |
| Re-key Key Pair (`KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR`) | `rekey_key_pair` | present; retain both forms | KMIPKIT-0036 |
| Recover (`KMIPKIT-ELEM-OP-C2S-RECOVER`) | `recover` | present; retain both forms | KMIPKIT-0018 |
| Register (`KMIPKIT-ELEM-OP-C2S-REGISTER`) | `register` | present; retain both forms | KMIPKIT-0036 |
| Revoke (`KMIPKIT-ELEM-OP-C2S-REVOKE`) | `revoke` | present; retain both forms | KMIPKIT-0036 |
| Set Attribute (`KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE`) | `set_attribute` | missing; add both forms | KMIPKIT-0016 |
| Set Constraints (`KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS`) | `set_constraints` | present; add `_with_options` | KMIPKIT-0039 |
| Set Defaults (`KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS`) | `set_defaults` | present; add `_with_options` | KMIPKIT-0039 |
| Sign (`KMIPKIT-ELEM-OP-C2S-SIGN`) | `sign` | present; retain both forms | KMIPKIT-0021 |
| Signature Verify (`KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY`) | `signature_verify` | present; retain both forms | KMIPKIT-0021 |
| Validate (`KMIPKIT-ELEM-OP-C2S-VALIDATE`) | `validate` | present; add `_with_options` | KMIPKIT-0042 |
