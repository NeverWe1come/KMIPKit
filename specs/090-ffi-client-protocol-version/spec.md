# Feature Specification: Preserve FFI Client Protocol Version (KMIPKIT-0090)

- **Feature Branch**: `feature/KMIPKIT-0090-ffi-client-protocol-version`
- **Base**: `release/1.0.0` at `9692057c9ade474fb0c4e0b7130539da368eb794`
- **Status**: Draft for maintainer review; implementation authorized by the task owner
- **Scope owner**: `kmipkit-ffi` client construction

## 1. Purpose

Preserve the protocol version selected on a C ABI client configuration when
`kmipkit_client_create` constructs the Rust client. The configuration setter
already records the selected version, but client construction currently resets
it to the default KMIP 2.1 version. As a result, a caller selecting KMIP 2.0
emits a 2.1 request header.

This corrects configuration propagation only. It does not change the default
version, operation availability, request-level version overrides, wire
encoding, or the Rust high-level client API.

## 2. Normative References and Pinned Evidence

- OASIS KMIP Specification v2.0 §9.16, Table 383: the Protocol Version
  structure carried in the Message Header contains major and minor integers.
  Pinned source: `specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html`,
  SHA-256 `41346355fc966e555dbcbae68b4d73421b0da0f32fb1a86fc8e3cb5fd35bb4a2`.
- OASIS KMIP Specification v2.1 §9.16, Table 421: the same Message Header
  protocol-version representation is used by KMIP 2.1. Pinned source:
  `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, SHA-256
  `8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf`.

The implementation requirement is API configuration propagation. The OASIS
references establish the observable wire field; they do not prescribe the
KMIPKit C ABI.

## 3. Requirements

### FR-090-001: Preserve the selected version

When a C caller creates a client from a configuration with protocol version
`(major, minor)`, the client MUST retain that version as its immutable default
and emit it in Message Headers unless a per-request or per-batch override is
supplied.

### FR-090-002: Preserve existing settings

Client construction MUST preserve the configuration's extension registry and
wire encoding along with its protocol version. The default remains KMIP 2.1;
explicit KMIP 2.0 and KMIP 2.1 configuration MUST remain distinguishable.

## 4. Acceptance Criteria

- A controlled mutual-TLS peer observes KMIP 2.0 in the Message Header after a
  C caller selects `(2, 0)` and creates a client.
- The regression test fails on the baseline because the client emits minor
  version 1, and passes after the propagation fix.
- Existing FFI tests, formatting, Clippy, and applicable CI checks pass.
- The change is published on its own feature branch in a draft PR; no merge or
  approval is performed by the implementer.

## 5. Out of Scope

KMIP 3.0, new C ABI functions, operation or tag gates, request-level override
behavior, server interoperability claims, and changes to other language
bindings are excluded.
