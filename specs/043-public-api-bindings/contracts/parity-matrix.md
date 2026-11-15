# Four-Language API Parity Contract

## Parity dimensions

For each released capability, the parity record covers all applicable dimensions below. A `four-language` row is complete only when the same behavior is reachable from Rust, C, Java, and Python. Every package export is inventoried separately in the manifest, even when several symbols map to one logical capability.

| Dimension | Required observable equivalence |
|---|---|
| Availability | Same capability and API level; no language silently omits an approved feature. |
| Inputs | Equivalent representable values, ranges, requiredness, defaults, and validation failures. |
| Requests | Same canonical typed KMIP request and explicit caller choices; no adapter-only protocol semantics. |
| Results | Same typed result, raw-preserving values, batch order, and per-item outcome. |
| Errors | Same stable error category and delivery state where applicable; language-specific exception/status form is declared. |
| Unknown values | Preserve all unknown values retained by the Rust model; no closed-enum loss. |
| Ordering | Preserve caller ordering where KMIP or an approved feature contract makes it observable. |
| Lifetime | C release functions, Java `AutoCloseable`, Python context managers, and Rust ownership expose equivalent resource boundaries. |
| Security | Same redaction, secret ownership/zeroization limits, and no request retries or runtime code downloads. |
| Documentation | All four surfaces describe the same supported scope, caveats, and examples. |

## API levels

1. **High level**: idiomatic, explicitly configured operation builders and client methods. Each operation's builder shape comes from its approved operation specification.
2. **Typed protocol**: complete typed request/response models and client outcomes. Bindings project the canonical Rust model and do not maintain an independent wire model.
3. **Generic TTLV**: structurally valid in-memory TTLV access, including the existing checked tags, supported value kinds, limits, order, unknown raw enum values, and exact octets the Rust model preserves. This level does not create a raw request submission bypass or change the existing codec/message validation boundary.

The generic TTLV capability preserves the accepted defaults and limits from specifications 0004/0005: 16 MiB maximum message bytes, 64 nested Structure levels, and 100,000 elements. Message-size and element-count limits remain configurable within their owner contract; depth cannot exceed 64. The same effective values and configuration failures are observable across four-language bindings.

## Package and declaration coverage

The manifest inventories the five published Rust crates and the official C, Java, and Python distribution families. It records every publicly reachable package declaration and ties it to a logical capability, stable requirement IDs, and an approved owner. The declaration audit covers Rust public exports, C exported symbols and public headers, Java public/protected API members, and Python package exports. Any deliberate omission needs an exact reviewed exclusion record with the approving source and reason; there are no wildcard exclusions.

### Accepted Rust-only transport boundary

The bounded `kmipkit-transport::Transport::exchange` contract and `TransportResponse` are the sole Rust-only capability in this scope, under ADR-0003 and ADR-0014. The manifest records the full declaration and its ADR acceptance evidence. Direct Rust callers provide and own request bytes and choose a response limit; the transport returns the redacting, zeroizing response wrapper under ADR-0014's constraints. `kmipkit` does not re-export this trait, `kmipkit-client` accepts no arbitrary caller-implemented transport, and `Client::execute` remains closed over typed requests. This exception does not authorize raw request submission or transport injection in C, Java, or Python.

## Shared test vectors

Each parity fixture has a stable ID and declares the capability requirement IDs, input setup, expected observable output, error category (if any), and expected resource cleanup. The manifest's `parityEvidenceIds` reference the fixture IDs and their consuming test names. Fixtures cover, at minimum:

- construction and inspection of each public API level;
- one successful request and representative KMIP failure for each operation after its approved contract is linked;
- heterogeneous ordered batch outcomes, including `Operation Pending` when permitted;
- exact asynchronous correlation-value preservation through explicit follow-up operations;
- unknown enumeration values, bitmask bits, extension values, and generic TTLV child order;
- invalid handle, invalid range, closed resource, configured limit, and native ABI mismatch;
- diagnostic redaction and output-handle release behavior.

## Operation gates

`contracts/operation-dependencies.md` is the operation-level part of this matrix. An operation may be listed in the target inventory while its binding status remains blocked. A language must never advertise the operation as available until its approved owner contract exists and its tests pass. At 1.0 completion, every operation row is verified and all four language mappings resolve.
