# KMIPKIT-0051 CI correction evidence (2026-11-08)

## Deferred Set Endpoint Role pre-send handling

- **Normative scope:** OASIS KMIP v2.1 §6.1.54, Table 331 defines Set Endpoint Role as a client-initiated operation and makes Endpoint Role required. `KMIPKIT-0051-FR-012` records that this operation is deferred to 1.1 and must not be treated as an in-scope 1.0 pre-send validation operation.
- **Red:** commit `665e82163` adds `deferred_endpoint_role_is_not_rejected_by_the_1_0_preflight`. Before the implementation change, `cargo +1.94 test -p kmipkit-client --all-features deferred_endpoint_role_is_not_rejected_by_the_1_0_preflight` failed to compile with E0004 because `ClientRequest::validation_report` omitted `SetEndpointRole`.
- **Green:** commit `072d90a3f` returns an empty validation report for the deferred operation. Its typed constructor requires Endpoint Role, and the open Enumeration preserves future and vendor values; no new 1.0 rejection rule is inferred.
- **Formatting:** `cargo +1.94 fmt --all` reordered the `kem` module and re-export with their alphabetical peers in `crates/kmipkit-protocol/src/lib.rs`. This also corrects the existing `cargo fmt --all --check` failure present in the #142 and #143 stacked heads.
- **Verification:** the focused regression passed (1 test). On the updated #143 worktree, `cargo +1.94 fmt --all --check`, `cargo +1.94 clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo +1.94 test --workspace --all-features` all completed successfully. The workspace run includes 742 `kmipkit-client` unit tests and passing trybuild/doc-test suites. AWS-LC emitted non-fatal stdalign/NASM diagnostics on this Windows host.

These results are local Windows evidence for the updated feature branch. They do not represent a completed hosted matrix or a passing remote CI run.
