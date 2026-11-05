# KMIPKIT-0049 Research Decisions

## Source and gap review

- Decision: Use the pinned OASIS KMIP Specification v2.1 and
  `specification/catalog/kmip-2.1.json` as the source for tags, encodings,
  requiredness, conditional requirements, and table order.
- Rationale: The source is the project authority and already supplies stable
  IDs in `KMIPKIT-REQ-SPEC-*` for operative client requirements.
- Alternatives considered: Usage Guide examples and tag-number ordering were
  rejected because they are informative and do not define structure order.
- The exact order is taken from each table. Table 9 places Key Block last in
  Split Key; Table 12 places Key Compression Type before Key Value. Object
  structure tags come from the pinned §11.56 tag catalog and are distinct from
  Object Type Enumeration values.

- Decision: Represent all nine structures in §§2.1–2.9, including PGP Key.
- Rationale: The approved feature explicitly requires all nine 1.0 object
  types. The pinned catalog lists Certificate, Certificate Request, Opaque
  Object, PGP Key, Private Key, Public Key, Secret Data, Split Key, and
  Symmetric Key.
- Alternatives considered: Omitting PGP Key or substituting Template was
  rejected; neither matches the source's nine Object Type structures.

## Model and compatibility choices

- Decision: Add typed models over the existing ordered TTLV owner rather than
  creating a second wire tree.
- Rationale: `Item`/`Structure` already preserve order, repetitions, unknown
  tags, and nested data; `Value` redacts Debug output and recursively
  zeroizes its owned payload. Borrowed `StructureView` provides inspection
  without copying.
- Alternatives considered: A new independent secret buffer/tree was rejected
  because it would create a second ownership and cleanup contract.

- Decision: Keep known typed forms plus a raw-preserving fallback for unknown
  Object Type, Key Format Type, and extension fields.
- Rationale: KMIP values are open to future/vendor additions and the project
  constitution requires lossless unknown-value preservation.
- Alternatives considered: Closed Rust enums that reject unknown values were
  rejected.

- Decision: Represent Key Block's Key Value as absent, plaintext structured
  Key Value, or wrapped bytes. Keep the inner Key Material's format-selected
  byte/Structure representation separate from the wrapped outer Byte String.
- Rationale: §§3.1–3.2 distinguish these wire forms and extension formats may
  use either source-defined material representation.
- Alternatives considered: One untyped `Vec<u8>` for both forms was rejected
  because it would erase the plaintext/wrapped distinction and Key Value
  Attributes.

- Decision: Additive constructors for Register and Import derive the
  Object Type from the typed object when appropriate; keep existing generic
  signatures and response readers unchanged.
- Rationale: This avoids a caller pairing an object payload with the wrong
  outer Object Type while preserving source compatibility. Response-level
  methods are deliberately scheduled for 0052.
- Alternatives considered: Replacing `RegisterRequest::new` or
  `ImportRequest::new` was rejected as a breaking change.

## Security and boundary choices

- Decision: Reuse `Value` ownership and expose borrowed views; do not add
  implicit clone paths for key material.
- Rationale: Owned TTLV Values clear KMIPKit-owned payload storage on drop;
  copying secret-bearing trees would create extra sensitive allocations.
- Alternatives considered: Promising zeroization for source buffers,
  application copies, JNI/Python copies, or TLS internals was rejected as
  outside KMIPKit's ownership.

- Decision: The 0049 API describes wrapping metadata and bytes but performs no
  encryption, decryption, signing, MAC, certificate validation, PGP parsing,
  algorithm selection, or server-capability inference.
- Rationale: KMIPKit transports/manages material and is not a local crypto
  provider; callers select cryptographic choices explicitly.
- Alternatives considered: Auto-detecting format, choosing algorithm/size, or
  executing wrapping operations was rejected as a product-boundary change.

- Decision: Treat PGP Key Block contents as opaque. KMIPKit is not a
  PGP-aware client, so the conditional PGP decomposition recommendation in
  `KMIPKIT-REQ-SPEC-2.4-002-002` does not apply. Preserve the stable ID and
  reason in `traceability.md`.
- Rationale: §2.4 recommends PGP-aware implementations decompose PGP data;
  this client intentionally provides a generic opaque key-material transport
  model and does not parse cryptographic formats.
- Alternatives considered: Decomposing PGP packets into separate objects was
  rejected because it expands the product into a PGP parser/provider.

## Verification decisions

- Decision: Use test vectors linked by the catalog when present, plus
  deterministic local vectors generated from the pinned schema. Do not
  describe a hand-written vector as an official OASIS test case.
- Decision: Test exact schema order from each table, unknown/extension
  preservation, source-requiredness, negative conditions, redaction, drop
  cleanup, Register/Import message composition, and real compiling Rust
  examples.
- Decision: Keep C, Java, and Python typed-object parity blocked until 0053;
  do not expose a Rust-only route as four-language ready in the manifest.
