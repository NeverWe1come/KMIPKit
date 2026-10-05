# Implementation Plan: KMIP 2.1 Typed Client Execution

**Branch**: `feature/KMIPKIT-0007-client-execution` | **Date**: 2026-10-05 | **Spec**: [`spec.md`](spec.md)
**Status**: Draft. Planning artifacts do not approve the feature or its dependencies.

## Summary

Create the first synchronous typed Rust client path over KMIPKIT-0005's strict TTLV codec and KMIPKIT-0006's message/batch model. The initial operation is explicit Discover Versions. A synchronous `Transport` abstraction and bounded deterministic fake exercise the actual `Client::execute` callsite. The scope proves closed typed input, private permit-gated serialization, response correlation and validation, limits, asynchronous result gating, redacted delivery-aware errors, and no retry. Production TLS/HTTPS implementations and other operation schemas are excluded.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.0
**Primary Dependencies**: `kmipkit-protocol`, `kmipkit-ttlv`, `kmipkit-transport`; exact crate APIs are gated on accepted/merged KMIPKIT-0005 and KMIPKIT-0006.
**Storage**: None.
**Testing**: Unit tests for typed operation and option validation; derived OASIS table tests; property tests for version policy and correlation; fake-transport integration tests through production execute; redaction/zeroization tests.
**Target Platform**: Rust workspace platforms in CI (Linux, Windows, macOS); fake transport is deterministic and has no network dependency.
**Project Type**: Rust workspace library.
**Performance Goals**: No benchmark threshold is established for this foundation slice. The transport and client MUST avoid unbounded response buffering; response reads stop at the configured cap.
**Constraints**: KMIP 2.1 only; TTLV only; TLS 1.3/mTLS policy applies to future production transport; no automatic retry; no raw/generic TTLV execute input; default limits from AGENTS.md §8; never disclose secret/request/response bytes.
**Scale/Scope**: First typed operation only; common client path is designed to admit later typed operation variants without opening generic serialization.

## Constitution Check

| Principle | Design response | Gate |
|---|---|---|
| Normative traceability | Stable requirement IDs map to exact OASIS clauses/catalog records and test IDs. Derived tests are not mislabeled official vectors. | Complete every mapping before implementation completion. |
| TDD | Separate Red, Green, and Refactor commits, with expected-failure output and final command evidence. | Do not implement before the dependency and spec gates pass. |
| Typed public API / generic TTLV | `Client::execute` accepts a closed typed request surface, initially Discover Versions only. The generic TTLV tree cannot be submitted as an operation. | Human accepts exact API and boundary. |
| Secret lifecycle | The private writer owner stays alive through synchronous exchange, then zeroizes on drop. Request and response data are redacted. | The owner-through-transport integration test must pass in CI on the candidate callsite. |
| Transport safety | Fake transport exercises bounded reads and partial writes; production backends are separate. No retry/failover. | Ensure delivery state reflects actual send/receive progress. |
| Compatibility and scope | Rust-only, TTLV, KMIP 2.1, client-initiated operations. | No bindings or other encodings added. |

**Gate outcome**: Design is compatible with stated principles but implementation remains blocked while KMIPKIT-0005/0006 and ADR-0012 approvals are missing.

## Proposed Architecture

1. `kmipkit-protocol` owns typed Discover Versions request/response payloads and conversions to/from the accepted KMIPKIT-0006 messages. The request model has no raw TTLV escape field.
2. `kmipkit-transport` owns the hidden, unstable workspace trait `Transport::exchange(&mut self, request: &[u8], max_response_bytes: usize) -> Result<Vec<u8>, TransportError>`. The trait must be visible across crate boundaries to `kmipkit-client` and the fake, so direct dependents of `kmipkit-transport` can technically access it; it is not re-exported from the supported `kmipkit` facade and has no stable API guarantee. This visibility is unresolved under OD-003 and must pass an independent boundary review before implementation. The transport reads incrementally, stops before allocation/retention beyond the response cap, and reports monotonic `RequestDeliveryState` in failures. The unpublished `kmipkit-test-support` crate owns the scripted fake and fixtures; the fake is not available in production builds. Unit tests inside `kmipkit-client` use that crate as a dev-dependency and call the private test-only client factory from within the client crate. No socket or TLS implementation is part of this feature.
3. `kmipkit-client` owns `Client::execute`, typed batch input, option policy, response correlation, request Time Stamp preservation/omission policy, omission of Server Correlation Value, limit forwarding, the private `OperationEncodingPermit`, and the sole production writer callsite. The exact method signature is `Client::execute(&mut self, batch: ClientBatch, limits: &CodecLimits) -> Result<ClientBatchResponse, ClientError>`. A private child module holds the writer interface with parent-only visibility; it is not `pub(crate)`. Unit tests within this crate can exercise its private test-only client factory; external integration tests cannot. This feature adds no constructor to the supported public facade accepting an arbitrary transport; production constructors belong to the approved TLS/HTTPS transport specification.
4. The facade re-exports only reviewed typed request/response and client surfaces. Public generic TTLV types remain usable independently but cannot implement or bypass execute conversion.
5. Errors preserve delivery state and only safe, sanitized cause/source metadata; original source formatting and payload are never exposed. KMIP result status/reason is a typed result; transport failures and malformed protocol failures remain distinct.

### Transport Contract Proposal

The trait should expose one synchronous exchange per explicit `execute` call. The encoded request is borrowed for the duration of the call. The client passes `limits.max_message_bytes()` as `max_response_bytes`; no separate response cap or global state is introduced. An approved transport adapter MUST stop a streaming read before allocating or retaining bytes beyond that cap. The client independently checks the returned buffer length before decoder entry. A transport failure reports its final delivery state and is not retried. A future TLS/HTTPS spec may implement this contract while preserving TLS 1.3, mTLS, certificate/hostname validation, no redirects/proxies/compression/0-RTT/key logging, and the existing timeout policy.

### API Surface Proposal

- `DiscoverVersionsRequest` is a concrete protocol type; KMIPKit 1.0 populates `ProtocolVersion { major: 2, minor: 1 }` only.
- `ClientRequest` is a non-exhaustive public enum whose known initial variant wraps a concrete Discover Versions request. It is sealed against caller-defined conversion and does not contain generic TTLV.
- `ClientBatch` is a typed ordered list of request variants plus validated common options and optional correlation IDs. Batches are needed here to exercise response association and mixed asynchronous result handling. Repeated Discover Versions items are used only with the deterministic fake; this does not claim that arbitrary servers accept them.
- `Client::execute(&mut self, batch: ClientBatch, limits: &CodecLimits) -> Result<ClientBatchResponse, ClientError>` accepts the typed batch and borrowed caller limits. The exact names for request payload fields must match accepted 0005/0006 APIs, but this method signature and closed operation set are the 0007 proposal. There is no overload accepting `Item`, raw bytes, or a trait implemented by downstream callers.
- `Client` is constructed only by a private test-only factory callable from unit tests inside `kmipkit-client`; the separate unpublished test-support crate supplies only the fake transport. No public arbitrary-transport injection or raw-send method is added to the supported facade. This is an internal foundation, not a standalone user-accessible client. The later approved TLS/HTTPS specification will add a supported `kmipkit-client` constructor from validated transport configuration while keeping arbitrary `Transport` injection private.
- The hidden workspace `Transport` exchange contract is not re-exported by the supported facade and is explicitly unstable/internal, but direct dependents of `kmipkit-transport` can technically access it. Unit tests in `kmipkit-client` use a private test-only factory, which external crates cannot call. No arbitrary-transport constructor or raw-send entry point is added to the supported facade. OD-003 requires an API/visibility audit before implementation; do not claim the internal crate boundary is secure until that review is complete.
- Version discovery remains caller initiated. No connection setup or future operation performs it implicitly.

## Security and Failure Design

- Construct the request through the typed operation and message model; validate header/option values before minting the private permit.
- Mint the private permit only inside `Client::execute`, at the only call to the private writer. Preserve the same borrowed per-call `CodecLimits`; do not clone or reconstruct limits.
- Keep the encoded owner alive until `Transport::exchange` returns, including success, short write, and error paths. Drop then zeroize initialized bytes.
- Preserve a caller-supplied optional request Time Stamp Date-Time exactly and omit it when absent. Do not generate a Time Stamp or expose a countdown-timer source; countdown-derived output is outside this feature's scope under OD-004.
- Never serialize Server Correlation Value in a client-initiated request; Client Correlation Value remains optional metadata and is not used for item matching.
- Bound the response in the transport while reading; independently reject any oversized returned buffer before calling the decoder. Decoder limits remain separate and unchanged.
- Advance delivery state monotonically: before any write `NotSent`; once any request bytes may have reached the peer `PossiblySent`; once response bytes begin `ResponseStarted`. Never infer that a possibly sent operation is safe to replay.
- Reject malformed/mismatched response messages and unrecognized critical extensions without exposing payload content. Preserve unknown non-critical extension TTLV for caller inspection as KMIPKit policy; OASIS allows it to be processed as absent.
- Do not log request, response, credential, private key, key material, or TLS material. No retry, failover, automatic polling, or persistence.

## Verification Strategy

1. **Typed operation**: derived request/response vectors from §6.1.16 Tables 211–213; the request always advertises the fixed singleton `(2,1)`; accept an empty response or the singleton `(2,1)`, reject duplicates/non-offered versions; standard error outcomes.
2. **Options and message policy**: tests from Table 432 and §11.3 for assigned values, extension intervals, and out-of-range negatives; Batch Error Continuation presence/default/cardinality cases; single-item rejection; Pending permitted/prohibited behavior, required correlation presence, and exact opaque-byte preservation through execute; no invented Continue/Undo execution claims. Do not test or implement §9.6/Table 435 extension-range encoding until `KMIPKIT-0007-OD-001` receives an approved source disposition.
3. **Response validation**: matching operations/IDs; response reordering; optional single-item ID; duplicate, missing, unexpected, mismatched operation/version; unknown critical versus non-critical extension.
4. **Bounds**: response exactly at limit and one byte over; transport stops before over-limit allocation; decoder not entered on oversized response; same `CodecLimits` reference reaches encoder and decoder; no peer-visible Maximum Response Size is sent for Discover Versions.
5. **Request metadata**: an explicitly supplied Time Stamp is preserved exactly, an absent Time Stamp stays absent, no Time Stamp is synthesized, Server Correlation Value is absent, and Client Correlation Value is not treated as an item ID.
6. **Execution integration**: invoke real production `Client::execute` using the fake; partial write then success; failures before send, after partial write, and after first response byte; assert final delivery state, request-owner lifetime, zeroization after return/drop, redaction in Debug/Display/exposed error-source chains, and no second exchange/retry.
7. **Static boundary checks**: a Rust-AST-based CI audit confirms one private permit mint and one production writer call, both in `Client::execute`, and rejects alias or macro-token bypasses; a separate facade API audit confirms no arbitrary encoder/raw-send API or caller conversion hook is public.
8. **Workspace**: run formatting, Clippy, workspace tests, documentation tests, coverage gates, generated-artifact checks, and supported-platform CI only after implementation begins and all foundation gates pass.

## Risks and Open Decisions

- **Dependency API drift**: 0005/0006 remain Draft. Freeze the execute and transport signatures only after their exact accepted APIs are merged; otherwise update this draft before approval.
- **Extension recognition**: initial slice recognizes no critical extension; registry semantics must be specified before claiming vendor critical-extension support.
- **Large-response inventory**: Discover Versions is not classified as a likely-large response, so its typed 0007 API omits peer-visible Maximum Response Size. KMIPKIT-0007 owns §9.12 local-limit enforcement and outgoing-field policy; once operation inventory is approved, relevant typed-operation specs identify likely-large responses, expose configured peer Maximum Response Size where appropriate, and add tests.
- **Batch cardinality and repeated operation policy**: §8.3/Table 396 and §9.7/Table 406 define Batch Items without an operation-uniqueness constraint, and §6.1.16/Table 211 adds no Discover Versions batch restriction. Repeated Discover Versions items are used only with the deterministic fake to exercise common correlation and mixed-Pending paths; this is not a claim that every server accepts repeated items. Standard KMIP errors remain valid outcomes.
- **Transport interface ownership**: `Transport::exchange(&mut self, request: &[u8], max_response_bytes: usize) -> Result<Vec<u8>, TransportError>` must be visible across workspace crate boundaries and is therefore technically public from `kmipkit-transport`, though hidden, unstable, and absent from the supported facade. Unit tests inside `kmipkit-client` call its private test-only client factory and import the fake from the unpublished `kmipkit-test-support` dev-dependency. OD-003 requires independent review of direct crate access before implementation.
- **Normative coverage**: OASIS error tables and response-ID conditions require line-by-line traceability review before plan approval.
- **Security boundary**: If reviewers reject the ADR-0012 exception or the private callsite cannot be enforced, stop before implementing any production secret-bearing request path.

## Work Breakdown

Detailed dependency-ordered tasks are in [`tasks.md`](tasks.md). Data shape, contracts, and a deterministic fake-exchange walkthrough are in [`data-model.md`](data-model.md), [`contracts/client-execution.md`](contracts/client-execution.md), and [`quickstart.md`](quickstart.md).
