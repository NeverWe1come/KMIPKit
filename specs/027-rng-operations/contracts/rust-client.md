# Rust Client Contract: Remote RNG Operations

Additive Rust contract following existing typed operation conventions. Sources are §§6.1.49–6.1.50, Tables 316–321.

## Types and fields

- RngRetrieveRequest requires Integer Data Length; RngRetrieveResponse exposes required opaque Data bytes; RngRetrieveError is sanitized.
- RngSeedRequest requires opaque Data bytes; RngSeedResponse exposes required Integer Data Length; RngSeedError is sanitized.
- Secret bytes are redacted and use existing zeroizing storage when owned. Preserve generic TTLV and future values.

## Client operations

- Admit ClientRequest::RngRetrieve / ClientOperation::RngRetrieve and RngSeed variants.
- Expose Client::rng_retrieve(request, limits) and Client::rng_seed(request, limits), with existing options-bearing conventions.
- Each explicit call uses one exchange, preserves common results/Pending/delivery, and does not retry, Poll, or generate random bytes locally.
- RNG Seed response Data Length = 0 is a successful response, not an error.

Cross-language names and generated/adapted signatures follow KMIPKIT-0043 after its approved mapping is integrated. Generated files are never edited manually.
