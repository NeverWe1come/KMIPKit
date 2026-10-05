# Deterministic Client Execution Walkthrough (Draft)

This developer walkthrough records the intended test path; it is not a user-facing code example. The feature depends on unapproved KMIPKIT-0005/0006 APIs and does not yet include a public transport constructor. Do not copy this text as executable API documentation. Any code example added to the public user guide must compile as a doctest after the approved TLS/HTTPS constructor exists.

## Verification scenario

1. A `#[cfg(test)]` unit test inside `kmipkit-client` explicitly constructs a typed Discover Versions request and calls `Client::execute(&mut self, batch: ClientBatch, limits: &CodecLimits) -> Result<ClientBatchResponse, ClientError>` through a private test-only client factory. It imports the deterministic fake from the unpublished `kmipkit-test-support` dev-dependency; external integration tests cannot access the private factory.
2. Execute validates the `(2, 1)` request header, supported-version list, IDs, options, timestamp source, and configured limits.
3. The private writer returns a zeroizing request owner under the execute-owned permit.
4. The unpublished deterministic fake transport observes the borrowed bytes through partial-write completion and returns a bounded response.
5. Execute checks response length before decode, validates the 2.1 header and response association, then returns the typed result.
6. The request owner is dropped and zeroizes its initialized bytes after the exchange returns.

The fake also scripts failures before writing, after a short write, and after response reception begins. Assertions inspect the final delivery state and typed result, never request or response bytes in formatted diagnostics.
