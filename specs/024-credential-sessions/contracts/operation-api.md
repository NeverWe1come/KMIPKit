# Operation API Contract: KMIP 2.1 Credential Sessions

This contract fixes the operation-specific behavior required by spec.md. Exact language bindings are rendered from the canonical public API manifest; generated output is never edited by hand.

## Rust protocol surface

Expose the typed models:

- DelegatedLoginRequest and DelegatedLoginResponse.
- LoginRequest and LoginResponse.
- LogoutRequest and LogoutResponse.
- A shared Ticket model with an open raw Ticket Type and secret Ticket Value.

Requests expose only fields present in the corresponding OASIS request table. Optional values remain absent unless supplied. Required local fields are validated before transmission. Nested Rights and Usage Limits use the structurally valid TTLV model unless a separately approved typed model already exists.

## Typed client surface

Add ClientRequest variants for Delegated Login, Login, and Logout and typed response accessors on ClientResponseView. Each operation is explicitly selected and can be included in the existing ClientBatch API. The shared response retains KMIP result status/reason, permitted message, batch association, Pending/correlation, and delivery evidence.

One execution call follows the existing client exchange behavior and never retries or polls automatically. Errors do not include raw body or ticket bytes.

## Ticket lifecycle

The Ticket structure follows OASIS §7.39/Table 391. Ticket Type is an open Enumeration. Ticket Value is secret byte data and uses the existing KMIPKit-owned secret zeroization and redaction rules.

A successful Login or Delegated Login returns a Ticket to the caller. It does not mutate client configuration, install a hidden session, change mTLS identity, or affect subsequent requests. Logout requires the caller to supply its Ticket explicitly.

## Cross-language contract

C, Java, and Python equivalent operation capabilities are added to specification/api/public-api.json and generated using tools/api_manifest/generate.py under the separately approved KMIPKIT-0043 manifest scope. The manifest entry, runtime adapters in crates/kmipkit-ffi/src/lib.rs, bindings/java/native/kmipkit_jni.cpp, bindings/java/src/main/java/org/kmipkit/CredentialSessionOperations.java, and bindings/python/src/kmipkit/credential_sessions.py, parity tests, generated files, and traceability updates ship together. Do not block or alter standalone Rust protocol modules while the manifest work is in progress.
