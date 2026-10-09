# KMIP legacy-version compatibility feasibility

Status: Exploratory research; no implementation is authorized by this document.
Date: 2026-10-09

## Purpose and release boundary

This document records the feasibility and a proposed direction for supporting
KMIP versions older than 2.1. It does not change the approved KMIPKit 1.0.0
scope. Version 1.0.0 remains a KMIP 2.1-only client, as recorded in the
[project definition](project-definition.md) and [ADR-0002](../adr/0002-kmip-21-release-scope.md).
Legacy-version implementation starts only after KMIPKit 1.0.0 is complete and
published, and requires its own reviewed ADR and approved Spec Kit specification.

This is exploratory research, not a conformance claim, feature specification,
or approval to change the release boundary.

## Findings

### Current OASIS version status

As of this research date, KMIP 2.1 is an OASIS Standard. KMIP 3.0 is at
Committee Specification Draft 02, dated 7 May 2026, and is not cited here as an
approved OASIS Standard. The 1.x family has OASIS Standards through version
1.4; the 1.4 specification incorporates Approved Errata 01.

Sources:

- [KMIP Specification 2.1, OASIS Standard, 14 December 2020](https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html)
- [KMIP Specification 3.0, Committee Specification Draft 02, 7 May 2026](https://docs.oasis-open.org/kmip/kmip-spec/v3.0/csd02/kmip-spec-v3.0-csd02.html)
- [KMIP Specification 1.4 with Approved Errata 01, 18 July 2019](https://docs.oasis-open.org/kmip/spec/v1.4/errata01/os/kmip-spec-v1.4-errata01-os-redlined.html)

### Compatibility requirements

KMIP 2.1 §9.16 says clients and servers **SHALL** support backward
compatibility with versions having the same major version; backward
compatibility across different major versions is **OPTIONAL**. KMIP 1.0 §6.1
contains the same rule. These clauses concern protocol compatibility; they do
not mean a client can safely produce a 2.1 message and edit it into a 1.x
message. The current 1.0.0 boundary and its treatment of §9.16 are recorded in
`KMIPKIT-DISC-022` in the checked-in normative catalog and in the message-model
specification. That scope exception remains visible until a later reviewed
decision resolves it.

Sources:

- [KMIP Specification 2.1 §9.16](https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html)
- [KMIP Specification 2.0, OASIS Standard](https://docs.oasis-open.org/kmip/kmip-spec/v2.0/os/kmip-spec-v2.0-os.html)
- [KMIP Specification 1.0 §6.1](https://docs.oasis-open.org/kmip/spec/v1.0/os/kmip-spec-1.0-os.html)
- [KMIPKIT-0006 message-model specification](../../specs/006-message-batch-model/spec.md), which records `KMIPKIT-DISC-022`

### The message schemas differ

`Create` illustrates why translation needs version-specific protocol rules.
KMIP 2.1 §6.1.8, Table 186, defines `Object Type`, `Attributes`, and optional
`Protection Storage Masks` in the request payload. KMIP 1.0 §4.1, Table 107,
defines `Object Type` and `Template-Attribute`. The enclosing attribute models
also differ: KMIP 2.1 §§5.1–5.4 define direct Attributes structures, while KMIP
1.0 §2.1.8 defines Template-Attribute structures that can contain names and
individual attributes.

Defaults can change observable behavior too. KMIP 2.1 §9.8 makes an omitted
Batch Order Option mean `True`; KMIP 1.0 §6.12 makes it mean `False`. A
version-specific encoder must apply the selected specification's rules rather
than copy a 2.1 message tree and alter only the version header.

Sources:

- [KMIP Specification 2.1 §§5.1–5.4, 6.1.8, and 9.8](https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html)
- [KMIP Specification 1.0 §§2.1.8, 4.1, and 6.12](https://docs.oasis-open.org/kmip/spec/v1.0/os/kmip-spec-1.0-os.html)

### Version discovery has limits

KMIP 2.1 §6.1.16 defines `Discover Versions`. The operation is also defined in
KMIP 1.1 §4.26; it is absent from the KMIP 1.0 client-operation list in §4.
Therefore, automatic discovery cannot be a universal prerequisite for
connecting to a 1.0 server. Discovery should remain explicit, with behavior
selected according to the client's and server's supported versions.

KMIPKit also prohibits automatic request retries. A failed operation must not
be silently replayed under an older version: the server may already have
processed a request whose response was lost. See `AGENTS.md` §8 and
[transport security](../architecture/transport-security.md).

Sources:

- [KMIP Specification 2.1 §6.1.16](https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html)
- [KMIP Specification 1.1 §4.26](https://docs.oasis-open.org/kmip/spec/v1.1/os/kmip-spec-v1.1-os.html)
- [KMIP Specification 1.0 §4](https://docs.oasis-open.org/kmip/spec/v1.0/os/kmip-spec-1.0-os.html)

## Architecture options

### Edit a completed KMIP 2.1 TTLV message

This does not provide general version compatibility. Payload structures,
effective defaults, and operation inventories vary across versions; for
example, `Discover Versions` is defined in KMIP 1.1 §4.26 and is absent from
the KMIP 1.0 client-operation list in §4. A post-encoding rewrite would need to
reconstruct protocol intent from an already version-specific representation
and could lose or misinterpret information.

### Use version-specific codecs over a shared operation model — recommended

Represent a caller's operation intent once where semantics are shared, then
validate, encode, and decode it using the selected KMIP version's rules. Keep
version-specific typed request/response models where a single shared type
would erase meaningful differences. Reuse the generic TTLV machinery where
the wire structures are genuinely shared. Reject unsupported operations or
fields before transport I/O, and never silently drop information or make
cryptographic choices on the caller's behalf.

This preserves a convenient high-level API without claiming every KMIP version
has the same operations, fields, or semantics.

### Maintain independent implementations per version

Separate protocol implementations could model each specification directly,
but duplicate shared behavior and increase the chance that Rust, C, Java, and
Python diverge. This is not recommended unless future analysis shows that
version differences prevent a well-bounded shared operation layer.

## Proposed developer-facing behavior

The following is a design proposal for a future specification, not an API
available in KMIPKit 1.0.0:

- Configure a default `KmipVersion` on the client; default to 2.1 to preserve
  the current API behavior.
- Allow a per-request override. It applies to that execution only and does not
  mutate the client's default. The effective version controls request
  validation, encoding, and response decoding.
- Resolve one effective version for a whole message. A batch cannot contain
  items encoded under different protocol versions because the version is in
  the shared message header (KMIP 2.1 §§8.1–8.3).
- Return a local unsupported-version/operation/field error before sending if
  the request cannot be represented faithfully in the selected version.
- Preserve explicit cryptographic choices and unknown-value policy already
  required by KMIPKit's public API rules.
- Expose equivalent capabilities in Rust, C, Java, and Python, following the
  project's language-parity requirement.

`Discover Versions` does not change this selection rule automatically. Any
future negotiation policy, including the initial version used to send a
discovery request, must be defined and tested in its own approved
specification.

## Proposed sequencing after 1.0.0

1. Complete and publish KMIPKit 1.0.0 under the approved KMIP 2.1-only scope.
2. Prepare a reviewed ADR and approved feature specification for older-version
   support. Revisit `KMIPKIT-DISC-022` explicitly rather than silently
   changing the 1.0.0 behavior.
3. Evaluate KMIP 2.0 first as the same-major compatibility step, then evaluate
   the 1.x versions progressively from 1.0 through 1.4. The final supported
   set and order remain decisions for that future specification.
4. Pin each applicable OASIS specification, profile, errata, and test-case
   source; add version-specific requirement traceability, generated catalog
   data, conformance tests, and interoperability results in the same reviewed
   change.

Complete support for a protocol version means implementing and testing all
applicable client requirements and representing the applicable optional
capabilities for that version. It does not assert that every server supports
every operation, profile, algorithm, or policy. Profile claims remain
separate and require their own clause-level evidence.

The existing KMIPKit transport boundary requires TLS 1.3 with mutual TLS.
Message compatibility alone therefore cannot promise connectivity to a legacy
server whose transport configuration does not meet that boundary. Any future
transport-policy change requires separate review; see
[ADR-0005](../adr/0005-transport-and-tls.md).

## Open questions for the future specification

- Which exact versions and profiles will be included in the first legacy
  compatibility release?
- Will the implementation proceed 2.0 first and then 1.0–1.4, or will
  interoperability demand a different order?
- Which versions can use `Discover Versions`, and what explicit fallback
  policy should callers configure?
- Which independent server implementations and TLS configurations will form
  the compatibility matrix?
- Which operation semantics require separate typed models rather than shared
  high-level builders?
