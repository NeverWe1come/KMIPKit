# Rust TTLV Codec Contract

**Status**: Proposed contract for review. Public names and signatures must be reconciled with the merged KMIPKIT-0004 API before implementation.

Secret-bearing wire encoding is not authorized. `AGENTS.md` §8 remains in force unless a human accepts ADR-0012, approves the KMIPKIT-0005 feature specification, approves the enforceable boundary design, and the first client feature's integration test passes. The proposal exposes no public `encode(&Item)` function and no general-purpose byte-producing encoder. KMIPKIT-0005 does not implement a client execute path, permit, or production writer callsite. If review rejects the request-only boundary below, or it cannot be enforced, do not approve or implement any secret-bearing request path.

## Public `kmipkit-ttlv` surface

The public TTLV crate retains the generic `Item` tree and bounded decoder. It does not export an encoder or an encoded-byte owner.

```rust
pub struct CodecLimits { /* private fields; immutable after checked construction */ }

impl CodecLimits {
    pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
    pub const DEFAULT_MAX_STRUCTURE_DEPTH: usize = 64;
    pub const DEFAULT_MAX_ELEMENTS: usize = 100_000;

    pub fn new(max_message_bytes: usize, max_structure_depth: usize, max_elements: usize)
        -> Result<Self, LimitsError>;
    pub fn defaults() -> Self;

    pub fn max_message_bytes(&self) -> usize;
    pub fn max_structure_depth(&self) -> usize;
    pub fn max_elements(&self) -> usize;
}

pub fn decode(bytes: &[u8]) -> Result<Item, DecodeError>;
pub fn decode_with_limits(bytes: &[u8], limits: &CodecLimits)
    -> Result<Item, DecodeError>;
```

## Red-to-Green test sequencing

T005 and T009 Red tests compile only through private test-local candidate seams and fixtures wired under `#[cfg(test)]`. T005 must not call the public `decode`/`codec` API before T006 creates it; T009 must not call public `CodecLimits` or `decode_with_limits` before T010 creates them. Their stubs are deliberately incomplete and the tests fail behavioral assertions, not compilation due to absent production symbols. Private client encoder limit Red tests use a private fixture/borrowed limits-view seam, not the not-yet-available public `CodecLimits`. T006/T010 promote the tested behavior into production APIs and add external-crate tests that prove public decoder/limits API accessibility. T010 then adapts the same borrowed `&CodecLimits` instance to the private writer and tests that it is neither cloned nor reconstructed.

The proposed `CodecLimits` API is immutable after construction: `new` validates values, `defaults` returns the defaults, and the three public getters expose read-only values. It has no public setters/builders and no global mutable state. Getter tests verify default and explicitly configured values. This lets the separate `kmipkit-client` crate inspect all limits without exposing fields. The proposed API is layered on the merged 004 model, which exposes `RawTag::new`, `RawTag::try_checked`, `Item::new`, `Structure::new`, `Structure::try_push`, `Value` constructors for all eleven types, and closure-scoped `Item::with_value`. Decoder option names remain proposed. The eventual public decoder module may be `kmipkit_ttlv::codec`; this does not imply a public encoder in that module.

## Proposed future client outbound boundary

The proposed future client path is gated on all three FR-013 approvals: human acceptance of ADR-0012, approval of KMIPKIT-0005, and explicit approval of the enforceable boundary design. KMIPKIT-0005 implements/tests the private writer and zeroizing owner, but does not create `Client::execute`, the permit type/constructor, or any production writer callsite. In KMIPKIT-0005, unit tests are the only writer callsites. The first client feature/spec owns the caller-facing API and the production integration: it defines `OperationEncodingPermit`, its private constructor in the module containing `Client::execute`, makes execute its sole mint site and the writer's sole production callsite, and places/refactors the writer as a private child with parent-restricted access (`pub(super)` or equivalent, never `pub(crate)`). Export neither the module, permit, owner, nor writer. The first client feature/spec audits exactly one production writer callsite and one permit mint site. Its execute path converts only a closed set of concrete typed requests into the generic TTLV model and accepts no caller-supplied `Item`, raw KMIP body, or public caller-implementable conversion trait.

The public input to the future `Client::execute` must be a closed set of concrete, protocol-typed KMIP request variants. It must not accept `Item`, raw body bytes, or a public caller-implementable conversion trait that could bypass typed request selection. The first client feature/spec must define its exact request variants/signature and per-call limit configuration, then prove the typed-input/permit boundary with an integration (and external API compile-fail or equivalent) test. Its transport test must drive partial writes and both success and error returns, prove the owner stays alive for every write and until the transport result returns, and then verify owner drop/zeroization. On write failure, it must verify the failed-request delivery state required by `AGENTS.md` §8 (not sent, possibly sent, or response reception had begun) and that no automatic retry occurs. The first client feature PR must include the sole production callsite and its owner-through-transport integration test together. CI must pass that test against the candidate callsite before merge, enablement, or release; the release branch must have neither the callsite nor a secret-bearing send until then. The codec's private unit tests still exercise encoder limits and owner policy; deferring the caller-facing API does not waive those codec tests.

The client crate currently depends on protocol and transport, and protocol depends on `kmipkit-ttlv`; a direct client-to-TTLV dependency is acyclic according to the current manifests. KMIPKIT-0005 plans direct workspace dependencies on `kmipkit-ttlv` and `zeroize` for its private writer/owner. The writer receives the same borrowed `&CodecLimits` instance in its private unit-test seam and reads values through the public getters, without cloning/reconstructing limits or consulting global state. The first client feature/spec defines exactly how its caller passes that instance into `Client::execute` and connects it to the writer. This is a proposed architecture only: this task changes no Cargo manifest or source code.

KMIPKIT-0005 does not define the caller-facing client API that passes configurable `CodecLimits` into `Client::execute`. After T010, its private encoder accepts the same per-operation `&CodecLimits` instance through its internal test seam and reads it through the getters. T009's pre-T010 Red cases use private limit fixtures; T010 adds coverage with the public `CodecLimits` instance. These codec tests do not depend on the future caller-facing `Client::execute` API. The first client feature/spec must define that per-call limit path without exposing the encoder, and its request-path integration test must verify the approved boundary and owner lifetime with the candidate client feature PR containing the sole callsite and test together; CI must pass it against the candidate callsite before merge, enablement, or release. Until then, the release branch has no production callsite or secret-bearing send.

Conceptual follow-on boundary for the first client feature/spec (not implemented by KMIPKIT-0005 and not a public signature):

```text
Client::execute(closed_typed_request, &limits)
  -> create private OperationEncodingPermit (sole mint site)
  -> private request-to-Item conversion
  -> parent-only wire_encoder::encode(item, the_same_limits, &permit)
  -> private zeroizing encoded owner borrowed through every partial write until transport returns success/error
```

Only the first client feature/spec that sends secret-bearing TTLV may own the integration test proving the closed typed input, permit boundary, sole production writer callsite/mint site, owner lifetime through partial writes and until transport success/error return, post-return zeroization, failed-request delivery-state reporting under `AGENTS.md` §8, and no automatic retry. The candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send. KMIPKIT-0005 owns codec vectors and private encoder owner/policy/limit unit-test requirements; it does not claim the client integration test exists. If review rejects this architecture or the boundary cannot be enforced, do not approve or implement a secret-bearing encoder.

## Behavioral contract

- After all three FR-013 approvals and the first client integration gate pass, the private outbound path may emit exactly one canonical Item for the caller-requested KMIP operation into a private zeroizing owner. Its writer requires the permit minted only by `Client::execute`; its input comes only from the closed typed-request conversion. It validates the complete tree, checks per-call byte/depth/count limits and the U32 Item Length ceiling, then reserves the complete output capacity before copying payload bytes. It preserves Structure child order and has no fallible exit after payload copying begins; if that invariant cannot be maintained, partial output is zeroized on every error path. Successful output is zeroized when its owner is dropped. KMIPKIT-0005 has no production callsite, so this proposed path cannot send secret-bearing data.
- The public decoder accepts exactly one complete Item and rejects empty input or trailing bytes. It validates lengths and available bytes before payload allocation.
- Both default decoder entry points use 16 MiB, 64 Structure levels, and 100,000 Items.
- `decode_with_limits` may use lower or higher message/count limits. Maximum Structure depth remains 64 unless the 004 model contract is deliberately changed.
- The private encoder receives the same borrowed `&CodecLimits` instance for one operation, reads its values through the public read-only getters, and applies those per-call semantics without cloning limits or using global state. Getter tests verify defaults and configured constructor values. Private encoder tests cover default/configured byte/depth/count limits, exact-boundary and one-over behavior, and synthetic U32 maximum/one-over size planning without multi-gigabyte allocations. The caller-facing way to supply the instance belongs to the first client feature/spec.
- Each Item Value length must fit `u32::MAX`, even when the configured message limit is larger. The private encoder checks this before writing a header; the decoder obtains the value from the U32 header and checks cumulative arithmetic and bounds before allocation.
- The decoder rejects unsupported Item Type bytes, type-specific invalid lengths, invalid UTF-8, noncanonical Boolean encodings, truncated values/padding, arithmetic overflow, parent-boundary violations, and configured limit excess.
- The codec rejects an empty Big Integer as KMIPKit project policy. OASIS §10.1.2 requires a two's-complement byte sequence with a length multiple of eight but does not explicitly set a minimum length.
- Padding octets whose values are not constrained by OASIS are accepted at the required extent; the private encoder writes zero for those padding octets. Big Integer leading sign-extension bytes are part of the represented Item Value and are not discarded.
- A Reserved Tag received from the wire is rejected before construction under proposed ADR-0011; implementation remains gated on review/acceptance of that ADR. Decoding must not retain or expose arbitrary original inbound raw bytes for later re-emission.
- The public TTLV crate performs no I/O, retries, KMIP operation-schema validation, or payload logging. In KMIPKIT-0005 the private outbound writer has no production callsite; no client can send secret-bearing bytes. The first client feature/spec must make the writer reachable only from the `Client::execute` module path through a private permit and closed typed input; no public caller may supply an `Item`, raw body, or conversion trait to reach it.
- Proposed FR-013 is a KMIPKit policy requirement, not an OASIS requirement. Only after ADR-0012 is human-accepted, the feature specification approved, the private boundary design approved, and the first client integration test passes may temporary outbound TTLV be generated solely for an explicitly caller-requested KMIP operation. The proposal preserves the prohibition on diagnostics, general-purpose serialization, persistence, logging, formatting, error inclusion, and arbitrary inbound raw-byte retention/re-emission.
- If the exception is approved, the private encoded owner remains alive during every partial write and until the transport returns its success/error result; it then drops after zeroizing the initialized encoded byte range; spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and its cleanup is verified. The first client feature/spec must test this lifecycle for completed and failed writes and verify `AGENTS.md` §8 delivery-state reporting (not sent, possibly sent, or response reception had begun) with no automatic retry. The codec itself does no I/O and does not control copies made by a transport implementation.

## Errors

Public `DecodeError` exposes stable error kinds and safe structural context only. The private encoder error does likewise within its owning client path. Neither may contain input/output bytes, Text String values, Byte String contents, Big Integer octets, or child payloads. Error formatting is redacted; non-payload causes such as allocation failure may be retained when available.

## Safety and allocation invariants

- All offset/length arithmetic uses checked operations.
- Input size is checked before decoder traversal.
- Declared Item Length is checked against allowed type lengths, parent end, total input, and caller limits before allocating/copying value bytes.
- Structure parsing stops at its declared parent boundary and increments element/depth counters before accepting each child.
- Private encoding uses one complete fallible output reservation before any payload copy; allocation failure returns an error, never panic. Decoder length/limit preflight must occur before any size-driven allocation. The approved 004 `Value`/`Structure` constructors use infallible `Box::new`/`Vec::push`; until `KMIPKIT-0005-OD-001` is resolved, do not claim that every allocation during full decoded-model construction is recoverable. The approved plan must either scope the guarantee to bounded preflight and fallible decoder-owned buffers, documenting possible process abort inside those constructors, or require a separate reviewed and merged model-construction change before decoder Green work. KMIPKIT-0005 does not implement model constructors.
- The private encoded owner zeroizes the initialized encoded byte range before deallocation/owner drop; spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless it is explicitly initialized and its cleanup is verified. It exposes no mutable borrow, cloning, plain-`Vec<u8>` extraction, or formatting/serialization trait that would silently create an uncontrolled copy. Any copy made by a caller or external TLS/runtime library is outside KMIPKit's zeroization guarantee.
- The TTLV model/decoder and proposed client writer continue to forbid unsafe code.
