# Research: KMIPKit Fluent Object Creation

## Decisions

### Reuse the approved typed operation paths

- **Decision**: Fluent builders adapt the existing `CreateKeyPairRequest`,
  `CreateSplitKeyRequest`, and `RegisterRequest` models and existing client
  execution/batch routes. They do not introduce operation encoders or
  transports.
- **Rationale**: KMIPKIT-0014 already implements the 2.1 server-generated
  creation payloads, including Create Key Pair Table 191 matching and the
  Split Key Prime Field Size client policy. KMIPKIT-0022 already implements
  Register and its table-specific request checks.
- **Alternatives considered**: Add a second protocol/request layer; rejected
  because it would duplicate validation and risk inconsistent wire behavior.

### Keep all nine approved managed-object types and their formats

- **Decision**: Use KMIPKIT-0049's typed object models and existing secret
  owners. Expose semantic constructors for the approved Certificate,
  Symmetric Key, Public Key, Private Key, Split Key, Secret Data, Opaque
  Object, PGP Key, and Certificate Request forms. Keep unknown/vendor objects
  on the explicit generic expert route.
- **Rationale**: KMIPKIT-0049 already covers KMIP 2.1 §§2.1–2.9 and
  §3.1–§3.12 Key Block, Key Value, and Key Material structures. The typed
  models preserve source order and unknown children, and owned secret payloads
  use the existing redaction and zeroization policy.
- **Alternatives considered**: Restrict Register to symmetric keys; rejected
  because the approved plan requires the formats already supported by
  KMIPKIT-0049. Add new formats; rejected because this delivery must not widen
  protocol support.

### Propose a bounded public/private key-pair preset matrix

- **Decision**: Propose only RSA 3072, ECDSA with P-256, and ECDH with P-256.
  Each preset supplies the algorithm and required size/curve parameters; the
  caller supplies usage masks and all other policy. These are proposed
  KMIPKit convenience choices in this specification, not a claim that the
  existing catalog alone approves a cryptographic policy.
- **Rationale**: The values have existing KMIP 2.1 Enumeration and
  Create-Key-Pair representation paths. NIST SP 800-57 Part 1 Rev. 5 Table 2
  maps a 3072-bit RSA modulus and 256-bit ECC curve to 128-bit security
  strength. The caller remains responsible for choosing whether the object
  serves signing, verification, or key agreement; KMIP server policy remains
  authoritative. No preset includes algorithm mode, padding, wrapping, or
  usage decisions.
- **Alternatives considered**: Offer arbitrary algorithm/parameter presets;
  rejected because that would blur the explicit expert path and the project's
  approved scope. Offer no presets; rejected because the user-approved plan
  calls for simple public/private key-pair presets.
- **Reference**: [NIST SP 800-57 Part 1 Rev. 5](https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final),
  Table 2. This is design rationale only, not a FIPS or profile claim.

### Preserve the current execution and memory contracts

- **Decision**: All three operations return the existing typed outcomes and
  delivery state. There is no automatic Poll/retry. C uses existing fixed
  width statuses and ownership/release rules. Java and Python document
  runtime-managed copies.
- **Rationale**: KMIPKIT-0007, -0052, and -0053 already define the behavior and
  ownership boundaries. Reusing those contracts maintains cross-language
  parity and redaction.
- **Alternatives considered**: Add a new error-handle or outcome model;
  rejected because it would create parallel API and ownership behavior.

## Resolved Constraints

- Existing typed Create Key Pair and Create Split Key models are under
  `crates/kmipkit-protocol/src/create_key_pair.rs` and
  `crates/kmipkit-protocol/src/create_split_key.rs`.
- Existing Register construction and validation are under
  `crates/kmipkit-protocol/src/register.rs`.
- The existing `ManagedObject` model and `KeyMaterial` model are under
  `crates/kmipkit-protocol/src/managed_object.rs` and
  `crates/kmipkit-protocol/src/key_material.rs`.
- `bindings/c/include/kmipkit.h` currently provides lower-level typed
  operation/batch entry points and managed-object TTLV constructors; the new
  fluent C surface must add semantic task-level and opaque builder inputs
  without replacing those functions.
- Existing TLS integration and consumer suites provide the fixture and
  cross-language verification points. A fake transport alone is not the
  end-to-end acceptance evidence.
