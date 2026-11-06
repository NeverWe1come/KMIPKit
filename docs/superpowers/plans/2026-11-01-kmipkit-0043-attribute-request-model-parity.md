# KMIPKIT-0043 Attribute Request Model Parity

## Goal

Close the T097 parity defect by exposing standalone, inspectable, TTLV-serializable typed request models for Add Attribute, Adjust Attribute, Delete Attribute, Modify Attribute, and Set Attribute in Rust, C, Java, and Python. Keep the existing high-level batch builders and operation behavior unchanged; add batch-item factories that accept the typed request models.

## Authority and scope

- Approved owner: KMIPKIT-0043, especially FR-005, FR-007 through FR-010, FR-013, FR-021, and FR-022.
- Canonical operation models and requirements: KMIPKIT-0016 FR-003 through FR-009 and FR-010 through FR-015.
- Normative sources: OASIS KMIP v2.1 §6.1.2 Tables 167–169 (Add Attribute); §6.1.3 Tables 170–172 and §11.1 Tables 428–429 (Adjust Attribute); §6.1.13 Tables 202–204 (Delete Attribute); §6.1.20 Tables 223–225 (Get Attributes); §6.1.21 Tables 226–228 (Get Attribute List); §6.1.34 Tables 265–267 (Modify Attribute); and §6.1.51 Tables 322–324 (Set Attribute). Get Attributes and Get Attribute List mappings are not to be changed by this task.
- Preserve Rust as the sole wire-model implementation. Foreign constructors accept copied typed fields/TTLV item handles and return owned model handles. Getters return independent owned values or copied scalar/text data. Model serialization returns a caller-owned generic TTLV structure. No API submits a raw request or injects a transport.
- Existing convenience batch constructors remain supported. Add typed-model-to-batch-item factories without changing operation semantics.
- Do not modify generated files manually; update the source manifest and regenerate through the pinned generator.

## Execution checklist

- [ ] Add focused failing tests for constructing each standalone model, reading its fields, preserving optionality/open values, serializing to the exact KMIP payload, and converting it into the corresponding batch item. Include a compiled C consumer plus Java and Python parity tests.
- [ ] Run the focused tests and capture expected missing-symbol/API failures; commit Red tests alone.
- [ ] Implement opaque C ABI model handles, constructors, getters, serialization, releases, and additive typed batch factories using the canonical Rust models. Copy borrowed inputs and preserve unknown/open values; keep errors payload-free. Run Rust FFI and real C consumer tests; commit Green implementation.
- [ ] Add Java 17 resource-safe wrappers and Python 3.12 context-managed CFFI wrappers over the C ABI. Keep field ordering, optional values, raw values, error categories, and ownership equivalent; run focused parity tests.
- [ ] Update the API source manifest mappings and evidence, regenerate every derived artifact, and map all exact request declarations to these standalone model APIs. Preserve justified exclusions for accessors that add no behavior only when their equivalent behavior is directly covered.
- [ ] Refactor only after Green. Run focused Rust/C/Java/Python tests, manifest tests and generator check, Rust/binding export audits, formatting, lint, and relevant coverage checks. Record separate Red, Green, and Refactor commits and update T097 evidence without marking it complete until all five models have parity and independent QA finds no gap.

## Acceptance checks

- Each of the five operation request models can exist independently of a batch item in all four languages.
- Each public typed model can be inspected and serialized to generic TTLV with the same canonical field semantics as Rust.
- Borrowed inputs are copied; returned handles have explicit release/context-manager ownership; no raw request bytes or sensitive attribute values appear in diagnostics.
- Typed model batch factories produce the same request payload as existing convenience batch builders for equivalent inputs.
- Manifest, generated output, consumer tests, and traceability agree; T097 remains open if any declaration is still only mapped to a high-level constructor.
