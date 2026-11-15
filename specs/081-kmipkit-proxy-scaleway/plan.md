# Implementation Plan: KMIPKit Proxy Scaleway Key Manager Provider Bridge

## Summary
Implement `ScalewayKmsProvider` in `proxy/kmipkit-proxy/src/providers/scaleway.rs` providing:
1. `X-Auth-Token` authentication for European Sovereign Cloud (Paris, Amsterdam, Warsaw).
2. Key lifecycle mapping to `key-manager/v1alpha1`.
3. Secret lifecycle mapping to `secret-manager/v1beta1`.
4. Bidirectional tag parsing `tags: ["key:value"]`.
5. Strict zeroization of plaintext keys and credentials.
