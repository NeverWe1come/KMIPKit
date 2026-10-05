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
- Chapter 11 introduction prohibits use of Tags marked Reserved. §11.56 supplies the standard and extension prefix ranges. `KMIPKIT-DISC-037` leaves receipt/preservation behavior open; proposed ADR-0011 recommends rejection before generic model construction.

## Design decisions and alternatives

### Codec input/output boundary

**Selected**: The public `kmipkit-ttlv` surface decodes exactly one generic Item from a bounded byte slice and exposes no byte-producing encoder. After three human approvals, KMIPKIT-0005 may implement/test a private canonical outbound writer in `kmipkit-client`, but adds no `Client::execute`, permit type/constructor, or production callsite. Its unit tests are the only writer callsites in this feature. The first client feature/spec owns the caller API, the `OperationEncodingPermit` type/private constructor in the execute module, execute-only mint and writer callsite, exact-one audit, and integration proof. That execute accepts only a closed set of concrete typed KMIP requests and no public `Item`, raw body, or caller-implementable conversion route. The writer itself does no stream reads, socket I/O, TLS, HTTP, message batching, or operation-level schema validation. The candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send.

**Alternative**: Expose a prefix decoder returning the unconsumed suffix or a streaming reader. Rejected for this feature because transport framing and streaming error states belong to the transport layer; one complete root item has a length in its own header.

### Model coupling

**Selected**: Operate on the approved `kmipkit-ttlv` Item/Structure/Value types from KMIPKIT-0004. Keep all eleven types in one generic model and preserve caller-provided child order.

**Alternative**: Add a second raw wire AST. Rejected for the normal decode path because it duplicates the generic tree and creates conversion/lifetime surfaces. This alternative is needed only if the project later revises proposed ADR-0011 to preserve received Reserved Tags under `KMIPKIT-DISC-037`.

### Allocation strategy

**Proposed, pending KMIPKIT-0005-OD-001**: Validate the complete input byte bound before parsing, validate every header length and checked addition before allocating payload storage, and use fallible reservation for decoder-owned scratch and byte buffers before copying peer payloads. The accepted 004 model still uses `Box::new` and `Vec::push` during value/tree construction, so this codec cannot promise recovery from every allocator failure without a reviewed model-construction change. Before decoder Green work, approve either this bounded-preflight/fallible-buffer guarantee with possible process abort inside existing constructors, or require a separate reviewed and merged model-construction change. KMIPKIT-0005 does not implement model constructors. Enforce per-call item/depth limits under either option.

**Alternative**: Trust declared lengths and allocate a complete child/value buffer up front. Rejected because network-supplied lengths are untrusted and may request excessive memory.

### Encoded output ownership and secret handling

**Proposed, not authorized**: After the three human approvals (ADR-0012 accepted, this feature approved, and the enforceable boundary design approved), KMIPKIT-0005 may implement a private `kmipkit-client` encoder and `EncodedTtlv` owner backed by a direct workspace `zeroize` dependency. It adds no `Client::execute`, permit type/constructor, or production writer callsite; private codec tests alone exercise the writer with configured limits. The first client feature/spec defines the public per-call limit API and adds the request path, execute-owned permit, sole production callsite/mint site, exact-one audit, and integration test. The candidate client feature PR may exercise its callsite and owner in this integration test; CI must pass it before the callsite is merged, enabled, or released on the release branch; dropping the private owner zeroizes the initialized encoded byte range before deallocation/owner drop. Spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless it is explicitly initialized and its cleanup is verified. No cloning, formatting, general-purpose serialization, mutation, or extraction into an ordinary `Vec<u8>` is allowed. Caller-created copies and copies inside external TLS/runtime libraries remain outside KMIPKit's zeroization guarantee. Until all approvals and the first client integration gate pass, the existing `AGENTS.md` §8 prohibition remains in force.

**Policy boundary**: FR-013 is a KMIPKit policy proposal and has no OASIS source clause. ADR-0012 proposes a single narrow exception: temporary outbound TTLV generated solely to carry an explicitly caller-requested typed KMIP operation. Three approvals (ADR-0012 acceptance, approval of KMIPKIT-0005, and approval of the enforceable boundary design) gate codec implementation. KMIPKIT-0005 itself defines no production callsite; the first client feature/spec defines `Client::execute`, its closed typed API, the permit owned/minted there, sole writer callsite, exact-one audit, and transport integration test. The candidate first-client-feature PR necessarily contains that callsite and test so CI can exercise them; until CI passes against that candidate commit, the release branch/runtime MUST NOT enable the callsite or send secret-bearing TTLV, and the PR cannot be merged, enabled, or released. The proposal preserves bans on diagnostic or general-purpose serialization, public encoder APIs, serialization traits, persistence, logging, formatting, error inclusion, and arbitrary inbound raw-byte retention or re-emission.

**Approval gate**: The proposal is the private `kmipkit-client` writer called only by `Client::execute`; it is not a public signature. Its entry point requires a private permit type/constructor owned by the execute module and minted there only; its public execute input is a closed set of concrete typed requests with no generic Item/raw-body/conversion-trait escape hatch. The manifests show an acyclic dependency direction for a direct `kmipkit-client` to `kmipkit-ttlv` dependency (`kmipkit-protocol` already depends on TTLV). Review must explicitly approve and implementation must enforce this boundary. If review rejects it or enforcement cannot be demonstrated, do not approve or implement a secret-bearing encoder. The first client feature/spec defines the exact request/limit API and owns the integration test. Its candidate PR must include the sole callsite and that test; CI must pass the test against the candidate callsite before merge, enablement, or release. Until then, the release branch has no production callsite or secret-bearing send.

**Alternative**: Return `Vec<u8>` and zeroize only partial buffers after errors. Rejected because a successful buffer can contain credentials or key material and ordinary Vec ownership does not clear it on drop.

**Alternative**: Stream bytes to an arbitrary caller-provided writer. Rejected for this feature because it weakens the preflight/failure-atomicity contract, can leave partial secret bytes in caller-controlled sinks, and makes cleanup depend on each sink. The transport can borrow the complete `EncodedTtlv` after successful encoding.

### Limit semantics

Defaults are 16 MiB per message, Structure depth 64, and 100,000 total Items. Count the root Item toward the item limit. Count a root Structure as depth 1; a non-Structure root has Structure depth 0. Per-call message and item limits can be raised or lowered; depth can be configured from 0 through the model's hard ceiling of 64. `CodecLimits` remains immutable and exposes read-only getters for all three fields so the private client writer can inspect the same borrowed instance per operation; it has no setters/builders or global state. A depth above 64 would require changing the KMIPKIT-0004 model contract and is a gate, not an implicit extension.

The U32 Item Length ceiling is unconditional even if a caller raises `max_message_bytes`. A Structure's own Item Value (including complete encoded child spans) must also fit the U32 ceiling. Decoder tests exercise synthetic U32 headers and checked cumulative arithmetic without allocating a multi-gigabyte buffer. Private encoder tests also cover default/configured per-call byte, depth, and count exact/one-over boundaries plus synthetic U32 maximum/one-over size planning; use bounded test-only planner/tree fixtures rather than giant payload buffers or multi-gigabyte allocations. Caller-facing execute limit configuration remains first-client-spec scope.

### Reserved Tags

Proposed ADR-0011 selects rejection of received Reserved Tags before generic model construction. This is the recommended project policy, not an OASIS clarification, and remains an implementation blocker until reviewed/accepted. Opaque preservation would require a separate wire representation incompatible with the current checked-Tag tree.

### Secret-bearing wire-encoding policy

Proposed ADR-0012 is separate from ADR-0011 and remains Proposed. ADR-0011 addresses received Reserved Tags and does not authorize serialization. The current `AGENTS.md` §8 prohibition remains in force unless the three human approvals are satisfied and the first client integration test passes. Public generic values and bounded decoding remain in `kmipkit-ttlv`; KMIPKIT-0005's private writer has no production caller. The first client feature/spec owns the private permit minted only by `Client::execute`, the closed typed request input, sole production writer callsite, and exact-one audit. The exception does not allow public/general-purpose serialization, diagnostics, persistence, or arbitrary inbound raw-byte retention/re-emission. There is no OASIS clause for FR-013. If review rejects the boundary or it cannot be enforced, do not approve or implement a secret-bearing request path.

### Unknown Item Type codes and Enumeration values

Reject Item Type bytes that are not among the eleven types in §11.23 because the current generic model cannot represent them. Preserve all unsigned Enumeration payload bits and Integer mask bits without assigning operation-level meaning; the generic codec does not perform the later catalog/profile validation needed to establish semantic enumeration validity.

### Canonical padding

Accept any pad-byte values when the required padding extent is present, since OASIS does not constrain those values in §§10.1.5. Emit zero padding for Integer, Enumeration, Interval, Text String, and Byte String. A decode/re-encode cycle can normalize nonzero padding and therefore promises canonical semantic re-encoding, not byte identity for arbitrary noncanonical padding.

## Catalog traceability gap

When implementation starts, update only the applicable requirement records in `specification/catalog/kmip-2.1.json` to include this feature and code/test references. Do not manually edit generated outputs. Catalog requirement `KMIPKIT-REQ-SPEC-10.1.2-001` for schema-specific field order is not implemented by this generic codec: catalog coverage remains incomplete until every applicable client 1.0 Structure has approved typed-spec ownership plus implementation and executable order-check references. The inventory workflow must assign those rows before a 100% traceability claim; naming planned specs alone is insufficient.
