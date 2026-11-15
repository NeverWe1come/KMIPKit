# Quickstart: Credential Session Operations

## Prerequisites

- Rust toolchain matching the repository's Rust 1.94 MSRV.
- Repository dependencies and test support installed.
- Implementation of this feature is complete before running these checks.
- The pinned Login fixture exists at specification/oasis/kmip-2.1/fixtures/TC-LOGIN-1-21.xml.

## Focused validation

Run from the repository root:

1. cargo test -p kmipkit-protocol ticket_model
   Expected: Ticket Type values, nested Ticket structure, redacted formatting, and secret lifecycle tests pass.
2. cargo test -p kmipkit-protocol delegated_login_operation
   Expected: Table 199–201 payload and error cases pass, including required Rights and optional absence.
3. cargo test -p kmipkit-protocol login_operation
   Expected: Tables 253–255 and the pinned TC-LOGIN-1-21 fixture test pass.
4. cargo test -p kmipkit-protocol logout_operation
   Expected: Tables 256–258 pass; no official direct Logout case is claimed.
5. cargo test -p kmipkit-client delegated_login
   Expected: Delegated Login fake-transport cases pass with one exchange, result/Pending preservation, delivery state, redaction, and no retry.
6. cargo test -p kmipkit-client login
   Expected: Login fake-transport and fixture cases pass.
7. cargo test -p kmipkit-client logout
   Expected: Logout fake-transport cases pass, including NotSent validation.
8. cargo fmt --all --check
9. cargo clippy --workspace --all-targets --all-features -- -D warnings
10. cargo test --workspace --all-features

For manifest parity after the approved API manifest includes these operations, run python tools/api_manifest/generate.py --check and the manifest/adaptor parity tests described in docs/development/api-manifest.md.

## Expected behavior

A valid typed request retains exactly its supplied optional values and uses the common client execution path. A missing required local field fails with NotSent evidence. A server Failure or Pending result remains a server result, not a locally fabricated success/error. Ticket Value is not visible in Debug output or errors, and no subsequent request uses a returned ticket unless the caller supplies it explicitly.
