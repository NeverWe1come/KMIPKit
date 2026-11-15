# Implementation Plan: KMIPKit Stateless Proxy Foundation (KMIPKIT-0072)

## Goal Description

Establish the common foundation crate `proxy/kmipkit-proxy` including configuration, tag codec, auto-envelope encryption engine, in-memory cache, dispatcher, and provider trait.

## Proposed Changes

- `proxy/kmipkit-proxy/Cargo.toml`
- `proxy/kmipkit-proxy/src/main.rs`
- `proxy/kmipkit-proxy/src/lib.rs`
- `proxy/kmipkit-proxy/src/config.rs`
- `proxy/kmipkit-proxy/src/tag_codec.rs`
- `proxy/kmipkit-proxy/src/envelope.rs`
- `proxy/kmipkit-proxy/src/cache.rs`
- `proxy/kmipkit-proxy/src/dispatcher.rs`
- `proxy/kmipkit-proxy/src/server.rs`
- `proxy/kmipkit-proxy/src/providers/mod.rs`
- `proxy/kmipkit-proxy/src/providers/aws.rs`
- `proxy/kmipkit-proxy/src/providers/azure.rs`
- `proxy/kmipkit-proxy/src/providers/gcp.rs`
- `proxy/kmipkit-proxy/src/providers/oci.rs`
- `proxy/kmipkit-proxy/tests/foundation_tests.rs`
- `.github/workflows/proxy-common.yml`
