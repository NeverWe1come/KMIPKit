# Implementation Plan: Preserve the FFI Client Protocol Version

1. Add a controlled mTLS regression test that selects KMIP 2.0 through the C
   ABI, constructs a client, sends Discover Versions, and verifies the emitted
   Message Header.
2. Run the regression test against the baseline and record the observed KMIP
   2.1 minor version as the Red evidence.
3. Preserve `configuration.protocol_version()` when rebuilding the internal
   `ClientConfiguration` in `kmipkit_client_create`.
4. Run focused and complete `kmipkit-ffi` tests, formatting, Clippy, and
   repository CI; update traceability and changelog evidence.

## Constraints

- Work only on `feature/KMIPKIT-0090-ffi-client-protocol-version`, based on
  `release/1.0.0` commit `9692057c9ade474fb0c4e0b7130539da368eb794`.
- Keep the protocol default at KMIP 2.1 and preserve existing encoding and
  extension-registry behavior.
- Do not change the C ABI, approve, or merge the resulting PR.
