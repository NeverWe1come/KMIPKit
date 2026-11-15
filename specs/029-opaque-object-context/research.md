# Research: Opaque Object cryptographic context

## Decision 1: Make the usage context explicit and default to generic opaque handling

**Decision**: The caller explicitly selects Managed Cryptographic Object context for an Opaque Object Register request. If the caller makes no selection, the request keeps generic opaque handling.

**Rationale**: OASIS KMIP 2.1 §2.3 says an Opaque Object MAY be a Managed Cryptographic Object depending on client usage context. The client must not infer this from the object type, Opaque Data Type, attributes, or bytes.

**Alternatives considered**:
- Treat every Opaque Object as cryptographic: rejected because it changes behavior for opaque non-cryptographic data and contradicts the conditional context.
- Infer context from Opaque Data Type, attributes, or payload bytes: rejected because §2.3 assigns the context to client usage, and Table 4 does not define Opaque Data Value encoding as a Key Block.
- Add a KMIP wire marker: rejected because §2.3 defines client-side context, while Register §6.1.43/Table 291 has no such request field.

## Decision 2: Apply Register Table 293 only after explicit context selection

**Decision**: When explicit context is Managed Cryptographic Object, apply the relevant §6.1.43/Table 293 rules. Require Cryptographic Usage Mask, Cryptographic Algorithm, and Cryptographic Length as direct attributes for Opaque Object. Require Algorithm and Length together. Do not allow a Key Block omission because Table 4's Opaque Object has no Key Block field; do not infer one inside Opaque Data Value.

**Rationale**: Section 2.3 says that in this context the Opaque Object is handled in the same manner as a Managed Cryptographic Object for attributes. Table 293 marks Usage Mask required and permits Algorithm/Length omission only when encapsulated in a Key Block. Table 4 defines Opaque Data Value as Byte String, not a Key Block. Certificate-only requirements do not apply. The generic Opaque Object exemptions in §§4.13, 4.15, and 4.17 remain applicable to generic opaque handling and later operations; they do not override the explicit §2.3 Register context. This implementation does not enforce Cryptographic Usage Mask authorization for later uses.

**Alternatives considered**:
- Allow omission based on a caller assertion that arbitrary Opaque Data Value bytes contain a Key Block: rejected because there is no structured Key Block field or a normative interpretation of the bytes.
- Enforce all Table 293 fields, including certificate-only fields: rejected because Table 293 limits those fields to Certificates.

## Decision 3: Validate Table 4 structure without interpreting opaque data

**Decision**: For Object Type Opaque Object, require the Object item as a Structure with Opaque Data Type encoded as Enumeration and Opaque Data Value encoded as Byte String. Preserve unknown enumeration values, unknown fields, duplicates allowed by the generic model, and item order.

**Rationale**: Register §6.1.43/Table 291 requires the Any Object payload item. Section 2.3/Table 4 marks the Opaque Data Type and Opaque Data Value children REQUIRED=Yes. The Object Type value is 8 and the Object item tag is Opaque Object per §11.34/Table 468. Their values remain opaque to this feature.

**Catalog finding and representation**: The pinned keyword audit identifies the Table 4 `REQUIRED` header as source-clause candidate ordinal 3; that audited locator must remain unchanged so source-audit CI stays deterministic. The enclosing Opaque Object structure member has blank requiredness, while its Opaque Data Type and Opaque Data Value child elements each record `source_requiredness: Yes` and their respective encodings. Link KMIPKIT-REQ-SPEC-2.3-003 to those existing child elements, correct its summary to name the two required fields, and add implementation/verification references to the elements. Do not invent a new catalog locator schema or modify the pinned source.

## Decision 4: Preserve the existing no-send behavior and use the API manifest for bindings

**Decision**: Run validation before transport serialization/write. Invalid choices or missing/wrongly typed required fields return the existing sanitized local validation error with NotSent delivery evidence and zero transport writes. Add binding selectors through the public API manifest and pinned generator.

**Rationale**: This is the established client delivery contract and preserves Rust/C/Java/Python parity. The manifest continuation, KMIPKIT-0043, is open as PR #100 but is not yet integrated; adapter generation depends on the accepted manifest being available on the release branch.

**Alternatives considered**:
- Modify generated source files directly: rejected by repository policy.
- Ship Rust-only behavior in 1.0.0: rejected by the constitution's parity rule.
- Add a dependency to or wait for PR #100's generated changes without writing the independent Rust spec: unnecessary; the behavior contract is independently specifiable now.
