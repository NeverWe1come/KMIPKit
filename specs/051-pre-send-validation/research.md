# KMIPKIT-0051 Research

## Source of truth

The immutable local source is `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, SHA-256 `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF`. Do not edit it or scrape an external page during builds. The project catalog in `specification/catalog/kmip-2.1.json` supplies stable requirement/element references, but does not contain a complete operation-payload schema for all 56 operations; schema descriptors therefore require explicit OASIS-backed inventory and source links.

## Normative findings

| Source | Normative meaning for this feature |
|---|---|
| KMIP v2.1 §8.1, Table 394 | Request Message has a Request Header followed by Request Batch Items. |
| KMIP v2.1 §8.2, Table 395; §9.4; §9.11, Table 403 | Request Header field order and Authentication/Credential structure; Authentication is optional at request-header scope, but if present contains one or more Credentials. Credential repetition is governed by Table 403. |
| KMIP v2.1 §8.3, Table 396; §9.5 | Batch Item field order and per-item requirements; Batch Count equals the number of items and is one for a single-item message. Operation-specific payload contents depend on Operation. |
| KMIP v2.1 §6.1 operation subsections and corresponding Request Payload tables | Each operation has its own fields, data types, requiredness, cardinality, and local conditional rules. There is no universal request-payload schema. §6.1 contains 57 client-initiated operation subsections. This feature inventories the 56 values supported by KMIPKIT-0050; Set Endpoint Role (§6.1.54, Table 331) is itself client-initiated but deferred to 1.1 under the approved product scope. §6.2 contains the server-initiated operations, also outside this feature. |
| KMIP v2.1 §9.13, Table 418 | Message Extension is the defined extensibility mechanism; this does not imply arbitrary unknown children are valid in every defined Structure. |
| KMIP v2.1 §10.1.2 | Item types/encodings are defined; fields of a specification-defined Structure are encoded in the order given by that Structure's description. Order is not inferred from numeric tag values. |
| KMIP v2.1 §10.1.3 | TTLV Item Length constraints used by generic wire-shape validation. |
| KMIP v2.1 §4.16, Table 59 | Cryptographic Parameters' known fields/types/order and applicable conditional rules; do not broaden KMIPKIT-0050's established local conditions. |

OASIS defines what makes a structure valid and the order in which it is encoded. It does not prescribe a method named `validate()`, require client-side validation before network I/O, or define `RequestDeliveryState::NotSent`; those are project API and transport decisions.

## Existing implementation findings at base `b2c455a99a54c1a93876aa2e1dbbaa78f509b3d6`

- `ClientRequest` is the typed operation enum in `crates/kmipkit-client/src/execute.rs`; it has private local-shape checks but no public request-wide validation API.
- Local shape dispatch currently covers multipart rules for Encrypt, Decrypt, Hash, MAC, MAC Verify, Sign, Signature Verify, plus Register.
- `RequestMessage::try_from_ttlv` and message validation check outer message, header, and Batch Item shape/order. The Request Payload remains a generic Structure at that boundary.
- Individual operation serializers construct fields in their specified order and enforce some operation-specific rules. There is no single profile-free validator that checks every Request Payload schema.
- `Client::execute` validates the batch, selected request shapes, ownership/policy constraints, query limits, encodes, and validates the Request Message before the exchange. Failures before `transport.exchange` map to `NotSent`.
- `ValidateRequest` and Register have existing local checks. Several operation-specific non-send tests already assert zero exchanges; the shared fake transport can also observe writes and partial-write/read delivery states.
- Public validation must borrow typed values and must not implement validation by cloning secret-bearing request trees or consuming serializers just to inspect them.

## Decisions

1. Use exact OASIS structure descriptions for field order, type, cardinality, requiredness, and conditionality; treat the project catalog as a traceability index, not a substitute schema.
2. Maintain an explicit schema identity for supported operation payloads and reusable known structures. Do not guess schema from tags, and do not claim unsupported structures were validated.
3. Reuse existing typed construction and model checks where they already implement a normative rule. Add a shared structured report and request/batch validation entry points without duplicating the existing wire encoder.
4. Run client validation before encoding or transport. Reject only error-severity issues that are provable from the supplied request; server capabilities, authorization, policies, and object state remain server decisions.
5. Keep unknown values and extension nodes lossless. Unknown nodes are not rejected solely because the local schema table does not know them; diagnostics identify only a safe path and never include the value.
6. Validation is read-only and uses borrowed fields/views. No serialization, deep clone, or secret duplication is introduced solely to create diagnostics.
7. Prefer deterministic diagnostics ordered by request structure order and then schema rule order. Return all independent local issues discoverable in one pass, without altering the request.

## Operation direction and release-scope ruling

The OASIS table of contents distinguishes client-initiated operations in §6.1 from server-initiated operations in §6.2. Set Endpoint Role has both a client-initiated definition in §6.1.54 (Table 331) and a server-initiated definition in §6.2.5 (Table 349). KMIPKit's approved 1.0 scope implements 56 of the 57 §6.1 operations and defers the client-initiated Set Endpoint Role path to 1.1. Do not describe that operation as server-initiated or count its deferral as an OASIS direction rule.

## Alternatives considered

- **Infer schemas from matching tag numbers**: rejected because the same tag can appear in different contexts and OASIS field order is structure-specific.
- **Treat every unknown tag as invalid**: rejected because it would break KMIPKit's lossless generic TTLV and explicit extension paths and could reject forward-compatible/vendor data.
- **Validate by serializing or cloning each request**: rejected because request serializers consume some secret-bearing values and cloning would create unnecessary secret copies.
- **Rely only on server rejection**: rejected because known local normative errors should be caught before network I/O; server-dependent policy remains outside local validation.
