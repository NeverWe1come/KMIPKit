# Research: KMIP 2.1 Client Asynchronous Operations

**Feature**: [spec.md](spec.md)
**Normative source**: pinned OASIS KMIP Specification v2.1, `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.

## Decisions

### Preserve correlation values as opaque bytes

- **Decision**: Retain the exact server-provided Asynchronous Correlation Value and expose Pending values only through KMIPKIT-0007's explicit borrowed accessor. Keep KMIPKit-owned Pending storage zeroizing and do not create ordinary unzeroized duplicates. Place the same bytes into caller-selected Poll, Cancel, or Process request payloads. Preserve arbitrary bytes, including NUL and non-UTF-8 values; do not parse or normalize them. Query filter input stays caller-owned; any copy KMIPKit creates is redacted and zeroized.
- **Evidence**: §9.1/Table 400 and §9.19/Table 424 identify the server-generated value as the value used in subsequent Poll/Cancel requests. §6.1.5/Table 176, §6.1.38/Table 276, and §6.1.39/Table 278 require it in their respective payloads. Existing KMIPKIT-0006/0007 models preserve generic TTLV and own Pending's value in zeroizing memory.
- **Alternatives rejected**: Treating the value as a string or deriving it from Client Correlation Value/Unique Batch Item ID would change protocol data and could associate a result with a different operation. Exposing an ordinary owned clone would weaken the accepted 0007 secret-lifetime contract.

### Poll Pending is the original operation's state, not a polling instruction

- **Decision**: A Poll Pending response is surfaced to the caller as a Pending outcome with no original-operation payload and the response's required correlation value. It does not automatically issue another Poll. On successful terminal completion, expose the original operation's response payload generically until a typed model exists; terminal Failure exposes status/Result Reason without a payload.
- **Evidence**: §6.1.38 says an incomplete operation yields no payload and Result Status Pending; successful completion carries the payload the original operation would have returned synchronously. §6.1.38 also says the response to Poll is not itself asynchronous. §8.6/Table 399 requires an Asynchronous Correlation Value when Result Status is Pending and defines the general failure response shape.
- A terminal Failure uses the general Response Batch Item failure shape: expose its status and Result Reason with no response payload. It is not a successful completion payload and is not another Pending outcome.
- **Alternatives rejected**: Decoding every successful completion as the currently supported Discover Versions response would misrepresent payloads for other original operations. Treating every completed Poll as requiring a payload would mis-handle a terminal Failure. An automatic polling loop is outside the product contract and would hide request count, timing, and delivery-state decisions from callers.

### Keep Process distinct and one-shot

- **Decision**: Expose Process as its own operation and outcome. Request correlation comes from the explicit pending outcome. A caller makes one Process request; any later Poll or handling of a Pending Process response requires another explicit caller action.
- **Evidence**: §6.1.39/Table 278 defines a dedicated Process request and Table 279 has no Process response-payload members. §8.6/Table 399 requires a Response Payload for every non-Failure result, so a Pending Process response must carry that empty structure while Failure has no payload. The Process prose says the server changes processing so the next Poll does not return Pending; this may affect other items when Batch Order Option is true (default). The clause does not state that Process itself cannot be returned asynchronously.
- **Alternatives rejected**: Treating Process as Poll or as a local wait helper would erase its server-side operation semantics. Assuming Process always completes synchronously is unsupported by the cited clause.

### Reject asynchronous Cancel responses

- **Decision**: Reject a Cancel response with Result Status Pending, even if the original operation allowed asynchronous execution.
- **Evidence**: §6.1.5 explicitly states that the response to Cancel cannot be asynchronous; Table 177 defines the synchronous response payload.
- **Alternatives rejected**: Reusing the generic client's Pending allowance for Cancel would contradict the operation-specific prohibition.

### Protect Query filter correlation values

- **Decision**: Apply the existing asynchronous correlation-value redaction and zeroization policy to values supplied in Query Asynchronous Requests filters, as well as server-provided Pending values. Caller-owned filter input storage remains the caller's responsibility; any copy KMIPKit retains is zeroizing and has no ordinary unzeroized duplicate.
- **Evidence**: The filter values are the same protocol Asynchronous Correlation Value data described in §7.1/Table 352. KMIPKIT-0007 and ADR-0014 treat correlation values as sensitive diagnostic data.
- **Alternatives rejected**: Protecting only values stored in a Pending outcome would leave caller-supplied values in request Debug/error/log paths and KMIPKit-owned buffers. Claiming to zeroize caller-owned input storage would exceed KMIPKit's ownership contract.

### Preserve Query response generically while DISC-039 is open

- **Decision**: Implement a typed Query Asynchronous Requests request and expose its response through generic TTLV without a typed Table 286 mapping.
- **Evidence**: In §6.1.41, Table 285 is explicitly the Query request payload. Prose describes reporting outstanding requests. The next response payload table includes a repeatable `Asynchronous Request` row, but the printed caption is “Table 286: PKCS#11 Response Payload”; the following Table 287 is explicitly Query Asynchronous Requests errors. The catalog records this as open `KMIPKIT-DISC-039`/`source_defect` with competing interpretations. Section 7.2/Table 353 gives the Asynchronous Request structure but cannot independently settle Table 286's caption/placement.
- **Alternatives rejected**: Silently rewriting the caption as Query Asynchronous Requests would treat an inference as an authoritative erratum. Dropping the operation entirely would unnecessarily remove the generic TTLV escape hatch and optional request capability.
- **Gate**: A typed interpretation, conformance claim, or catalog discrepancy closure requires a reviewed source decision. Upstream source and generated catalog are not edited in this feature PR.

### Record the Process field's missing catalog requirement

- **Decision**: Trace Table 278's required client request field to `KMIPKIT-CLAUSE-SPEC-6.1.39-002` and a stable project requirement ID in this spec; keep the missing catalog requirement ID open and track a separate catalog workflow correction.
- **Evidence**: The pinned source table marks Asynchronous Correlation Value required. Current catalog clauses classify the prose and rows, but does not assign a client requirement ID to that field.
- **Alternatives rejected**: Treating absence of a catalog ID as permission to omit the required field would violate Table 278. Hand-editing generated catalog JSON here would violate repository generation rules and mix an inventory correction into an operation feature.

## Existing architecture evidence

- **KMIPKIT-0006**: Generic request/response envelopes preserve ordered TTLV; Pending exposes the response correlation value only through a callback-scoped borrowed view; Poll/Cancel/Process semantics are deferred to KMIPKIT-0009.
- **KMIPKIT-0007**: `Client::execute` performs one synchronous exchange for a closed typed request set, associates response items, validates whether Pending was permitted, and returns delivery-aware errors. Initial Pending execution is currently typed only for Discover Versions.
- **ADR-0014**: `kmipkit-transport::Transport::exchange` is synchronous, bounded, and no-retry. The typed client does not accept arbitrary public transport injection. Request/response ownership and zeroization limitations are explicit.
- **Scope consequence**: Generic original-operation payloads are required for useful Poll completion before all operation-specific response types are implemented. The production transport constructor is owned by its separate transport/configuration feature; this feature exercises deterministic semantics through the existing private fake-transport seam.

## Normative source map

| Section/table | Client-visible behavior in this feature | Server-only behavior not claimed as client conformance |
|---|---|---|
| §6.1.5, Tables 176–178 | Send required correlation; parse echo and Cancellation Result; preserve unknown result value | Whether the server cancels and which result it returns |
| §6.1.38, Tables 276–277; §8.6, Table 399 | Send required correlation; accept Pending/no payload or terminal original-operation status/reason; successful completion carries its payload and Failure has no payload; do not recursively poll | When work completes and which successful payload is returned |
| §6.1.39, Tables 278–280 | Send required correlation; represent empty response and operation result | Change in server processing mode and effects on other ordered batch items |
| §6.1.41, Tables 285–287 | Encode optional filters and preserve generic response | Which outstanding requests the server reports and the unresolved typed response mapping |
| §7.1, Table 352; §7.2, Table 353 | Preserve query filter and generic asynchronous-request subtrees | No typed Query response interpretation while DISC-039 remains unresolved |
| §8.6, Table 399 | Validate/retain a correlation value on Pending response items | Server's production of required response members |
| §9.1, Table 400; §9.19, Table 424 | Reuse exact server-provided correlation bytes for explicit continuation | Server choice of synchronous/asynchronous execution |

## Test evidence status

The current catalog does not link requirement-specific official Test Case IDs for these rows. The feature therefore defines derived tests from exact cited clauses and TTLV tables; it must not call them official conformance vectors. The official fixture references tracked elsewhere as `KMIPKIT-DISC-036` are unavailable locally. The implementation phase must record the exact derived fixture source and test path for every FR and must retain those limits in documentation.
