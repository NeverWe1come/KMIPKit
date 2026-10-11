# Research: KMIP 2.1 Policy and Usage Controls

## Source review

The normative source is the pinned OASIS KMIP Specification v2.1 in `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. The approved feature spec identifies these exact payload/error tables: Get Constraints §6.1.22, Tables 229–231; Get Usage Allocation §6.1.23, Tables 232–234; Set Constraints §6.1.52, Tables 325–327; Set Defaults §6.1.53, Tables 328–330; Set Endpoint Role §6.1.54, Tables 331–333. Nested structures are Constraints §7.7, Defaults Information §7.11, and Usage Limits §4.59/Table 148. The Usage Limits Unit enumeration is §11.60/Table 491. Pinned source files remain immutable.

The normative catalog has four client requirements for §6.1.23:

- `KMIPKIT-REQ-SPEC-6.1.23-001-001`: do not use a Usage-Limited object for protection before an allocation is obtained.
- `KMIPKIT-REQ-SPEC-6.1.23-001-002`: request allocation only after Activation Date and before Protect Stop Date.
- `KMIPKIT-REQ-SPEC-6.1.23-002-001`: specify the number of protection units needed.
- `KMIPKIT-REQ-SPEC-6.1.23-002-002`: stop protection when the entire allocation is consumed until a new allocation is obtained.

The pinned operation text additionally says the server assumes the entire requested amount will be consumed. The response table returns a required Unique Identifier but does not echo the requested count or Usage Limits Unit. The payload carries Usage Limits Count as a required Integer. The Usage Limits Unit comes from the Usage Limits attribute; standard values are Byte and Object and extensions remain open. The server decides whether the object is eligible, whether the window is open, whether capacity is available, and whether to accept the allocation. KMIPKit cannot establish those remote facts from the allocation request alone.

The pinned source has no operation-specific official Test Cases catalog entry for any of the five operations. Tests are derived from the OASIS payload tables and labeled as project tests. Do not report an unavailable official case or fixture as passing.

## Design decisions

### Typed operation payloads

Create one typed request/response model per operation and reuse the common message, result, and delivery contracts. Constraints and Defaults Information stay as ordered generic TTLV Structures: typed wrappers provide field boundaries while preserving unknown nested members and vendor extensions. Endpoint Role and Usage Limits Unit use open raw Enumeration values so unknown future/vendor values round-trip. Requiredness follows the approved spec tables exactly, including optional Unique Identifier and Defaults Information.

Alternative considered: make all fields opaque generic TTLV. This loses typed requiredness and discoverable operation APIs. Alternative considered: reject unknown nested fields or enumeration values. That would break forward compatibility and conflict with the constitution.

### Explicit local allocation context

The maintainer-approved interpretation is to expose §6.1.23 enforcement through an explicit caller-supplied local guard. The caller provides (1) successful allocation result, (2) resolved object identifier returned by that result, (3) exact requested count, and (4) unit from known object metadata. Since the wire response omits count and unit, KMIPKit must not invent them. The context is non-cloneable and local to one caller/process instance.

For a typed Encrypt, MAC, or Sign protection request, the caller supplies the amount to reserve in that unit. KMIPKit checks the exact object identifier, positive cost, and known local unreserved allowance before transport. It does not infer cost from operation type or byte length. Decrypt, MAC Verify, Signature Verify, and Hash are outside the guard. The source says decryption/verification processing is not usage-limited; Hash does not apply protection to a managed cryptographic object.

Alternative considered: query remote attributes or track server-side usage automatically. The 1.0 client has no reliable cross-client/process consumption feed, and the allocation response provides neither the unit nor the count. Inferring or polling remote state would exceed this feature's contract and still could not account for external consumers. Alternative considered: automatically count payload bytes. This is wrong for Object units, vendor-defined units, multipart operations, and server-specific costs. Alternative considered: globally block every Encrypt/MAC/Sign call without a guard. The client cannot know whether an object has a Usage Limits attribute or whether its protection window is open without introducing remote metadata/policy inference. The approved behavior therefore requires callers who know they are using a Usage-Limited object to route that use through the explicit guard and documents that unguarded/external calls are outside the local count.

### Reservation and delivery behavior

Reserve caller cost before exchange. If client validation proves `NotSent`, release it. Once transmission may have begun (`PossiblySent`) or response bytes began (`ResponseStarted`), retain the reservation. Also retain it for any complete server Success, Failure, or Pending result because the client cannot prove whether protection occurred or how much the server consumed. Retained units are described as locally reserved/unavailable, never as remotely consumed. This conservative approach prevents local over-allocation without making a false claim about server usage.

Alternative considered: restore cost after server Failure. A Failure status may not prove that no protection was applied for every server or operation-specific failure; the client has no normative basis to infer consumption from generic status. Alternative considered: release after timeout. A timeout after transmission is ambiguous and must not make the allowance reusable.

### Execution, state, and security

Use the shared typed dispatcher and one-item execution path. Keep operations explicit and make one exchange per call. Preserve Pending and correlation bytes; do not add automatic Poll/retry. All malformed-response and transport errors retain existing sanitized cause and delivery evidence. Do not log request bodies, sensitive operation data, or raw TTLV. The usage context is ordinary policy metadata rather than a cryptographic secret; nevertheless, errors should avoid echoing caller identifiers or payload values.

### Cross-language implementation sequence

The canonical public API manifest in `specification/api/public-api.json` and generator remain authoritative. KMIPKIT-0043 is needed before adding C/Java/Python operation signatures and guard equivalents. That dependency gates only manifest-based parity work; it does not block Rust protocol types, typed client execution, or local guard logic. Do not hand-edit generated files.

### No new runtime dependency

Existing TTLV, typed client execution, operation result, and request-delivery contracts are sufficient. No state store, timer, remote polling facility, cryptographic dependency, or new crate is justified.
