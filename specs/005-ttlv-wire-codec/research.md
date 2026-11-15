# Research: KMIP TTLV Wire Codec

## Source validation

The normative source is the repository's immutable copy at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, identified by `specification/oasis/kmip-2.1/SOURCES.md`. The relevant sections are §§10.1.1–10.1.5, §11.23, Chapter 11 introduction, and §11.56. The checked-in catalog contains five requirement IDs directly associated with §§10.1.2 and 10.1.5. It does not define clause/requirement IDs for §§10.1.1, 10.1.3, or 10.1.4; this specification uses stable `KMIPKIT-0005-NR-*` IDs for those rules and records the gap for catalog reconciliation.

An independent source audit verified the pinned copy against its SHA-256 digest `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF` and identified the exact source section and catalog clause locations. No upstream source file was changed.

## Protocol findings

- §10.1.1: Tag is three bytes, unsigned, big-endian.
- §10.1.2 and §11.23: Type is one byte. The eleven assigned types have distinct representations and fixed or variable widths. Specification-defined Structure fields have a required schema order.
- §10.1.3: Item Length is a 32-bit big-endian count of Item Value bytes, subject to type-specific allowed lengths.
- §10.1.4: Item Value is interpreted according to Item Type.
- §10.1.5: Structure Item Length includes child encodings and their padding. Integer, Enumeration, Text String, Byte String, and Interval Item Length exclude their following padding. Text/Byte padding is the minimum trailing amount to an eight-byte boundary; Integer/Enumeration/Interval have four following bytes.
- §10.1.2 Big Integer rules: minimal leading sign-extension bytes make the value length a multiple of eight; these bytes are part of Item Value and Item Length.
- §10.1.2 does not explicitly give Big Integer a minimum wire length. KMIPKit adopts the project validity rule that an empty value is rejected because an empty octet sequence represents no two's-complement integer. This policy is distinct from an OASIS MUST.
- §10.1.3 encodes each Item Length in an unsigned 32-bit field. Raising the configured total-message limit cannot make any individual Item Value larger than `u32::MAX`; this representability check is independent of platform-sized allocation limits.
- §10.1.5 gives no required value for the following padding octets. The decoder therefore checks padding extent, not content. The encoder emits zero for deterministic canonical output; this is project policy.
- Chapter 11 introduction prohibits use of Tags marked Reserved. OASIS KMIP Specification v2.1 §11.56 supplies the standard and extension prefix ranges. T001 records accepted decision `KMIPKIT-DEC-001` against `KMIPKIT-DISC-037` and validates/regenerates the catalog report before T006; ADR-0011 selects rejection before generic model construction.

## Design decisions and alternatives

### Codec input/output boundary

**Selected**: The public `kmipkit-ttlv` surface decodes exactly one generic Item from a bounded byte slice and exposes no byte-producing encoder. Under the delegated approvals recorded in `approval-record.md`, KMIPKIT-0005 may implement/test a private canonical outbound writer in `kmipkit-client`, but adds no `Client::execute`, permit type/constructor, or production callsite. Its unit tests are the only writer callsites in this feature. The first client feature/spec owns the caller API, the `OperationEncodingPermit` type/private constructor in the execute module, execute-only mint and writer callsite, exact-one audit, and integration proof. That execute accepts only a closed set of concrete typed KMIP requests and no public `Item`, raw body, or caller-implementable conversion route. The writer itself does no stream reads, socket I/O, TLS, HTTP, message batching, or operation-level schema validation. The candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send.

**Alternative**: Expose a prefix decoder returning the unconsumed suffix or a streaming reader. Rejected for this feature because transport framing and streaming error states belong to the transport layer; one complete root item has a length in its own header.

### Model coupling

**Selected**: Operate on the approved `kmipkit-ttlv` Item/Structure/Value types from KMIPKIT-0004. Keep all eleven types in one generic model and preserve caller-provided child order.

**Alternative**: Add a second raw wire AST. Rejected for the normal decode path because it duplicates the generic tree and creates conversion/lifetime surfaces. This alternative is out of scope under the accepted ADR-0011 decision; a future change would require a new approved specification and ADR.

### Allocation strategy

**Selected, resolved under KMIPKIT-0005-OD-001**: Validate the complete input byte bound before parsing, validate every header length and checked addition before allocating payload storage, and use fallible reservation for decoder-owned scratch and byte buffers before copying peer payloads. The accepted 004 model still uses `Box::new` and `Vec::push` during value/tree construction, so this codec cannot promise recovery from every allocator failure; allocation failure inside those constructors may abort. KMIPKIT-0005 does not modify model constructors. Enforce per-call item/depth limits.

**Alternative**: Trust declared lengths and allocate a complete child/value buffer up front. Rejected because network-supplied lengths are untrusted and may request excessive memory.

### Encoded output ownership and secret handling

**Approved conditional policy**: The maintainer authorization in `approval-record.md` accepts ADR-0012, KMIPKIT-0005, and the enforceable request-only boundary. KMIPKIT-0005 may implement/test a private `kmipkit-client` encoder and owner but adds no `Client::execute`, permit type/constructor, or production writer callsite. The first client feature/spec owns the request path and owner-through-transport integration test; its candidate PR must include the sole production callsite and that test, and CI must pass before merge, enablement, or release. The release branch has no production callsite or secret-bearing send until then. The initialized encoded byte range is zeroized before owner deallocation/drop; spare/uninitialized capacity is outside the guarantee unless initialized and verified. No cloning, formatting, general-purpose serialization, mutation, or extraction into ordinary `Vec<u8>` is allowed.

**Policy boundary**: FR-013 is a KMIPKit policy requirement and has no OASIS source clause. ADR-0012 and the KMIPKIT-0005 specification/boundary have delegated maintainer authorization recorded in `approval-record.md`. That authorization permits only the private, uncalled encoder/owner implemented by this feature. KMIPKIT-0005 defines no production callsite; the first client feature/spec defines `Client::execute`, its closed typed API, the permit owned/minted there, sole writer callsite, exact-one audit, and transport integration test. The candidate first-client-feature PR necessarily contains that callsite and test so CI can exercise them; until CI passes against that candidate commit, the release branch/runtime MUST NOT enable the callsite or send secret-bearing TTLV, and the PR cannot be merged, enabled, or released. The policy preserves bans on diagnostic or general-purpose serialization, public encoder APIs, serialization traits, persistence, logging, formatting, error inclusion, and arbitrary inbound raw-byte retention or re-emission.

**Production-path gate**: The eventual private `kmipkit-client` writer may be called only by `Client::execute`; it is not a public signature. Its entry point requires a private permit type/constructor owned and minted only by the execute module; execute accepts a closed set of concrete typed requests with no generic Item/raw-body/conversion-trait escape hatch. The manifests show an acyclic dependency direction for a direct `kmipkit-client` to `kmipkit-ttlv` dependency (`kmipkit-protocol` already depends on TTLV). The delegated design approval is recorded, but implementation must enforce the boundary. If enforcement cannot be demonstrated, do not implement a production secret-bearing encoder. The first client feature/spec defines the exact request/limit API and owns the integration test. Its candidate PR must include the sole callsite and that test; CI must pass the test against the candidate callsite before merge, enablement, or release. Until then, the release branch has no production callsite or secret-bearing send.

### Client dependency review gate

Before any change to `crates/kmipkit-client/Cargo.toml`, complete and independently review `specs/005-ttlv-wire-codec/dependency-review.md` for every newly direct dependency of the client, including internal crates and test-only dependencies. The record covers capability, alternatives, maintenance, security history, MSRV, license, platform support, transitive cost, and exact versions and enabled features. The existing 004 `zeroize` review is reusable by reference only where its version, features, and scope match; the client-specific rationale for using `zeroize` to exercise and later own the private encoded output is also recorded. Before T003 promotes a dependency to normal scope, confirm that the reviewed record covers that scope and exact version/features; update and independently review the record before promotion if any differ. The independent dependency review and delegated maintainer disposition are recorded in this feature's review artifacts.

**Alternative**: Return `Vec<u8>` and zeroize only partial buffers after errors. Rejected because a successful buffer can contain credentials or key material and ordinary Vec ownership does not clear it on drop.

**Alternative**: Stream bytes to an arbitrary caller-provided writer. Rejected for this feature because it weakens the preflight/failure-atomicity contract, can leave partial secret bytes in caller-controlled sinks, and makes cleanup depend on each sink. The transport can borrow the complete `EncodedTtlv` after successful encoding.

### Limit semantics

Defaults are 16 MiB per message, Structure depth 64, and 100,000 total Items. Count the root Item toward the item limit. Count a root Structure as depth 1; a non-Structure root has Structure depth 0. Per-call message and item limits can be raised or lowered; depth can be configured from 0 through the model's hard ceiling of 64. `CodecLimits` remains immutable and exposes read-only getters for all three fields so the private client writer can inspect the same borrowed instance per operation; it has no setters/builders or global state. A depth above 64 would require changing the KMIPKIT-0004 model contract and is a gate, not an implicit extension.

The U32 Item Length ceiling is unconditional even if a caller raises `max_message_bytes`. A Structure's own Item Value (including complete encoded child spans) must also fit the U32 ceiling. Decoder tests exercise synthetic U32 headers and checked cumulative arithmetic without allocating a multi-gigabyte buffer. Private encoder tests also cover default/configured per-call byte, depth, and count exact/one-over boundaries plus synthetic U32 maximum/one-over size planning; use bounded test-only planner/tree fixtures rather than giant payload buffers or multi-gigabyte allocations. Caller-facing execute limit configuration remains first-client-spec scope.

### Reserved Tags

Accepted ADR-0011 selects rejection of received Reserved Tags before generic model construction. This is the selected project policy, not an OASIS clarification. T001 records the decision against the catalog discrepancy before decoder implementation; opaque preservation remains out of scope because it requires a separate wire representation incompatible with the checked-Tag tree.

### Secret-bearing wire-encoding policy

Accepted ADR-0012 is separate from ADR-0011. ADR-0011 addresses received Reserved Tags and does not authorize serialization. The delegated approvals recorded in `approval-record.md` authorize only the private, uncalled KMIPKIT-0005 writer/owner. The first client candidate production path remains prohibited until its owner-through-transport integration test passes CI. Public generic values and bounded decoding remain in `kmipkit-ttlv`; KMIPKIT-0005's private writer has no production caller. The first client feature/spec owns the private permit minted only by `Client::execute`, the closed typed request input, sole production writer callsite, and exact-one audit. The exception does not allow public/general-purpose serialization, diagnostics, persistence, or arbitrary inbound raw-byte retention/re-emission. There is no OASIS clause for FR-013. If review rejects the boundary or it cannot be enforced, do not approve or implement a secret-bearing request path.

### Unknown Item Type codes and Enumeration values

Reject Item Type bytes that are not among the eleven types in §11.23 because the current generic model cannot represent them. Preserve all unsigned Enumeration payload bits and Integer mask bits without assigning operation-level meaning; the generic codec does not perform the later catalog/profile validation needed to establish semantic enumeration validity.

### Canonical padding

Accept any pad-byte values when the required padding extent is present, since OASIS does not constrain those values in §§10.1.5. Emit zero padding for Integer, Enumeration, Interval, Text String, and Byte String. A decode/re-encode cycle can normalize nonzero padding and therefore promises canonical semantic re-encoding, not byte identity for arbitrary noncanonical padding.

## Catalog traceability gap

T001 closes `KMIPKIT-DISC-037` as a project policy decision. T012 updates only applicable requirement records in `specification/catalog/kmip-2.1.json` with this feature's implementation and test references; it does not add or repeat the ADR/discrepancy disposition. Do not manually edit generated outputs. Catalog requirement `KMIPKIT-REQ-SPEC-10.1.2-001` for schema-specific field order is not implemented by this generic codec: catalog coverage remains incomplete until every applicable client 1.0 Structure has approved typed-spec ownership plus implementation and executable order-check references. The inventory workflow must assign those rows before a 100% traceability claim; naming planned specs alone is insufficient.
