# Feature Specification: KMIP 2.1 XML Message Encoding

**Feature Branch**: `feature/KMIPKIT-0056-xml-encoding`
**Status**: Approved for implementation by direct maintainer instruction on 2026-11-05; depends on the KMIPKIT-0054 common codec contract.
**Owning identifier**: KMIPKIT-0056

Implement the KMIP 2.1 XML message encoding using the shared ordered generic
`kmipkit-ttlv::Item` tree and checked-in catalog naming metadata. The codec is
private and independently testable; it does not change the client dispatcher,
transport selection, public API manifest, or generated declarations.

**Normative sources**: OASIS KMIP Specification v2.1 §10.2.3; OASIS KMIP
Profiles v2.1 §§5.4.1.1–5.4.1.13, explicitly including §5.4.1.6.4 (mask
components), 5.4.2, and 5.4.4.1 (`MSGENC-XML-M-1-21`). HTTPS interpretation
is governed by ADR-0018 and KMIPKIT-0054 FR-009.

**Acceptance criteria**: Implement profile-conformant XML parse/emit, names,
namespace and hexadecimal forms, and encode Integer masks as strings whose
assigned symbolic components and/or 32-bit unsigned big-endian hexadecimal
components are separated by spaces, as specified in Profiles §5.4.1.6.4.
Decode both these component strings and the numeric/hex Integer forms allowed
by §5.4.1.6.3. Preserve model-representable unknown tags, enumeration values,
mask bits, and extensions; enforce common decode limits before unbounded
allocation; harden XML parsing against external entity resolution, DTD and
network access; redact errors; replay the pinned official XML vector; and
pass derived malformed-input and semantic round-trip tests. Do not claim
profile support until all applicable client clauses and required official
tests pass.

**Dependencies and exclusions**: Requires accepted 0054 and the common item
codec contract. May be developed in parallel with 0055. Does not edit
`specification/api/public-api.json`, generated public API artifacts, or client
and transport dispatch. Public exposure belongs only to 0057.
