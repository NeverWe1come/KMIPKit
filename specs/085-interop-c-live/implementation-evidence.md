# KMIPKIT-0092 C route evidence implementation (WIP)

The approved KMIPKIT-0092 contract is consumed directly by
`interop/c-kmip-validator/tools/route_matrix.py`. It invokes the installed public
C ABI for exact local catalog values and the compiled C consumer for controlled
network recipes. Library sources are unchanged.

| Requirement | C evidence and remaining work |
|---|---|
| KMIPKIT-0092-FR-001, FR-006 | Shared matrix generator, installed header/symbol/constant probe, verified exact release commit and library digest, compiled-consumer provenance comparison. |
| KMIPKIT-0092-FR-002, FR-007, FR-009 | Per-cell evidence and explicit pending sidecar; duplicate protection; common schema and validator in CI. Incomplete recipes keep the gate failing. |
| KMIPKIT-0092-FR-003 | Exact assigned scalar values and reserved-tag rejection are checked locally. Broader semantic branch/model recipes remain pending. No absent encoder/send/high-level API is fabricated. |
| KMIPKIT-0092-FR-004 | Ordered, redacted tree digests are retained per cell. Cross-language digest normalization/parity is pending. |
| KMIPKIT-0092-FR-005 | Controlled peers parse all six encoding/transport pairs, check recipe fields, numeric ordered results, lifecycle transitions and deletion. Cryptographic, asynchronous, and remaining positive operation recipes are pending. |
| KMIPKIT-0092-FR-008 | Reports contain public identities, axes, digests, sanitized failure codes and numeric outcomes. Request bodies and fixture keys/tickets never enter evidence. |

The exact OASIS document and section remain on each case in the generated shared
catalog. Scalar generic-TTLV checks follow KMIP 2.0/2.1 Specification §10.1; local
numeric tag/value fixtures follow the source references attached to the exact
normative target. Controlled recipe tests follow each operation's catalog
§6.1 reference and verify ordered batch structure from Specification §8.

This work does not satisfy the complete 0092 C matrix, the 1.0 coverage gates,
or vendor interoperability acceptance. PR #170 remains a draft marked WIP.

## CI repair Red evidence (2026-11-12)

PR #170 head `02b4aafe3964825829f046bbdfbb3b53e3e62d54` failed
existing executable tests before this repair:

- CI run `43297892137`, Ubuntu stable job `131461810582`: workspace
  tests `operation_parity_0053` and `c_api_coverage` fail to link
  `-lkmipkit_ffi`.
- The same run, Windows Rust 1.94 job `131461810515`: `c_api_coverage`
  links both the Rust rlib and the native library, producing LNK2005/LNK1169.
- Coverage Ubuntu job `131461810884`: the instrumented
  `operation_parity_0053` test fails to find the native library under the
  newer Cargo build-output layout. The selected coverage gate consequently
  rejects the incomplete producers.

Reproduce the affected executable tests with
`cargo test --locked -p kmipkit-ffi --all-features --test operation_parity_0053 --test c_api_coverage`.
The regression is explicit native-library linkage in tests that must resolve
C ABI symbols from their instrumented Rust crate, in the same process.
The complete C matrix remains required and pending recipes must keep failing.
