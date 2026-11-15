# Feature Specification: KMIP 2.0 Session Tag Version Eligibility (KMIPKIT-0089)

- **Feature Branch**: `feature/KMIPKIT-0089-kmip20-session-tags`
- **Base**: `release/1.0.0` at `fdafef8a3`
- **Status**: Draft for maintainer review; implementation authorized by the task owner
- **Scope owner**: `kmipkit-protocol::VersionGate`

## 1. Purpose

Correct the version spans used by `VersionGate::tag_span` so the client accepts
the KMIP 2.0 tags used by standard Login, Logout, and Delegated Login requests
and responses. The gate currently marks `0x42014A..=0x420150` and `0x420152`
as KMIP 2.1-only. Those assignments are already present in the pinned KMIP
2.0 Tag Enumeration. The range also misidentifies KMIP 2.1 Constraints and
Rotate tags: their assignments begin at `0x420168`.

This feature changes only tag-version eligibility. It does not change request
schemas, encodings, operation availability, ticket policy, authentication
behavior, or the default protocol version.

## 2. Normative References and Pinned Evidence

The authoritative KMIP 2.0 source is
`specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html`, SHA-256
`41346355fc966e555dbcbae68b4d73421b0da0f32fb1a86fc8e3cb5fd35bb4a2` as
recorded in `specification/oasis/kmip-2.0/CHECKSUMS.sha256`.

- OASIS KMIP Specification v2.0 §6.1.12, Tables 184–186: Delegated Login
  request/response/error. The request permits Request Count (`0x42014C`) and
  requires Rights (`0x42014D`); its returned Ticket contains Ticket Type
  (`0x42014A`) and Ticket Value (`0x42014B`).
- OASIS KMIP Specification v2.0 §§6.1.29–6.1.30, Tables 235–240: Login and
  Logout request/response/error. Login uses Request Count and returns Ticket;
  Logout sends Ticket with Ticket Type and Ticket Value.
- OASIS KMIP Specification v2.0 §§7.28–7.29, Tables 348–349: Right and Rights
  structures. The Right structure's members include Objects (`0x42014E`),
  Operations (`0x42014F`), and Right (`0x420150`).
- OASIS KMIP Specification v2.0 §7.33, Table 353: Ticket structure.
- OASIS KMIP Specification v2.0 §11.54, Table 446: Tag Enumeration. It assigns
  Ticket (`0x420149`), Ticket Type (`0x42014A`), Ticket Value (`0x42014B`),
  Request Count (`0x42014C`), Rights (`0x42014D`), Objects (`0x42014E`),
  Operations (`0x42014F`), Right (`0x420150`), Endpoint Role (`0x420151`), and
  Defaults Information (`0x420152`) in KMIP 2.0.
- OASIS KMIP Specification v2.1 §11.56, Tag Enumeration: Constraints and
  Rotate assignments `0x420168..=0x420172` are introduced in KMIP 2.1. The
  relevant values include Constraints (`0x420168`), Constraint (`0x420169`),
  Rotate Interval (`0x42016A`), Rotate Automatic (`0x42016B`), and Rotate
  Latest (`0x420172`).

The pinned KMIP 2.1 source is
`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`; the catalog
`specification/catalog/kmip-2.1.json` confirms names and wire values above.
Tests derived from the normative clauses are project tests, not official OASIS
test cases.

## 3. Requirements

### FR-001: Accept KMIP 2.0 session tags

`VersionGate` MUST NOT classify any KMIP 2.0 tag in the assigned session and
Rights range `0x420149..=0x420152` as KMIP 2.1-only. These tags MUST pass
`validate_tag` under KMIP 2.0 and KMIP 2.1, subject to the existing 3.0 span
rules.

### FR-002: Preserve the 2.1-only Constraints and Rotate gate

The assigned KMIP 2.1 Constraints and Rotate tags `0x420168..=0x420172` MUST
continue to be rejected under KMIP 2.0 with `introduced: V2_1`, and accepted
under KMIP 2.1. KMIP 3.0-only tag spans MUST remain unchanged.

### FR-003: Exercise valid session requests through the client gate

KMIP 2.0 Login, Logout, and Delegated Login requests built with the existing
typed public request models MUST pass the pre-send `VersionGate` checks and
reach the configured transport. Their response headers and Ticket fields MUST
also validate as KMIP 2.0 when present.

## 4. Acceptance Criteria

- Unit tests establish the Red failure for the misclassified KMIP 2.0 tags and
  preserve rejection of actual KMIP 2.1-only tags before production behavior
  changes.
- After the fix, focused protocol and client tests pass for KMIP 2.0 Login,
  Logout, and Delegated Login, and KMIP 2.1-only constraints/rotation tags stay
  blocked under KMIP 2.0.
- Rust workspace formatting, lint, unit/integration tests, generator checks,
  and applicable CI checks pass. The branch is pushed for review; no PR is
  opened before the task owner independently reviews the branch.

## 5. Out of Scope

KMIP 3.0 scope changes, operation-span changes, schema repairs, test-server or
interop validator changes, manufacturer extensions, changes to public APIs,
and merging or approving this branch are excluded.
