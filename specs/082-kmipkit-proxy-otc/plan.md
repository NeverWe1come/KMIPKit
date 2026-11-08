# Implementation Plan: KMIPKit Proxy Open Telekom Cloud KMS Provider Bridge

## Summary
Implement `OtcKmsProvider` in `proxy/kmipkit-proxy/src/providers/otc.rs` providing:
1. `X-Auth-Token` authentication for European Sovereign Cloud (Germany/EU).
2. Key lifecycle mapping to `/v1.0/{project_id}/kms/*`.
3. Secret lifecycle mapping to CSMS `/v1/{project_id}/secrets/*`.
4. Bidirectional tag parsing with TMS format `{"key": "...", "value": "..."}`.
5. Strict zeroization of plaintext keys and credentials, avoiding sensitive identifier names in URL formatting.
