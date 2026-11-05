# Feature Specification: KMIP 2.1 XML Message Encoding

**Feature Branch**: `feature/KMIPKIT-0056-xml-encoding`
**Status**: Draft — depends on accepted KMIPKIT-0054 common codec contract.
**Owning identifier**: KMIPKIT-0056

Implement the KMIP 2.1 XML message encoding using the shared ordered generic
`kmipkit-ttlv::Item` tree and checked-in catalog naming metadata. The codec is
private and independently testable; it does not change the client dispatcher,
transport selection, public API manifest, or generated declarations.

**Normative sources**: OASIS KMIP Specification v2.1 §§10.2.3; OASIS KMIP
Profiles v2.1 §§5.4.1.1–5.4.1.13, 5.4.2, and 5.4.4.1 (`MSGENC-XML-M-1-21`). HTTPS
interpretation is governed by ADR-0017 and KMIPKIT-0054 FR-009.

**Acceptance criteria**: Implement profile-conformant XML parse/emit, names,
namespace and hexadecimal forms, preserve model-representable unknown tags/
enumerations/bitmasks/extensions, enforce common decode limits before
unbounded allocation, harden XML parsing against external entity resolution,
DTD and network access, redact errors, replay the pinned official XML vector,
and pass derived malformed-input and semantic round-trip tests. Do not claim
profile support until all applicable client clauses and required official
tests pass.

**Dependencies and exclusions**: Requires accepted 0054 and the common item
codec contract. May be developed in parallel with 0055. Does not edit
`specification/api/public-api.json`, generated public API artifacts, or client
and transport dispatch. Public exposure belongs only to 0057.
