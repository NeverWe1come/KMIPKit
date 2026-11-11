# Implementation Plan: KMIPKIT-0103

## Design

Add a stable opaque C builder handle for each semantic operation. Internally,
the FFI stores only the corresponding typed request state and creates the
existing `ClientRequest` variant at build time. Common semantic setters cover
identifier, byte data, typed Cryptographic Parameters, expert ordered
parameters, multipart correlation and indicators, Request Header async
indicator, and exchange options. Operation-specific setters cover IV/nonce,
AEAD additional data/tag, digest, signature, and MAC data. Setters copy bytes
into Rust-owned `SecretBytes`; no request TTLV is accepted by the fluent send
path.

`build_batch_item` remains a local operation. `execute_v1` and its options
variant lock the existing C Client handle and make one existing typed Client
exchange. Existing batch response accessors expose KMIP Success, Pending, and
Failure. Builders are independent of clients and can be reused to create
multiple local batch items; execute never mutates or consumes a builder.

The generated C header is updated only by `tools/api_manifest/generate.py`.
All hand-authored C declarations are represented in the manifest and generated
outputs are audited in the same PR.

## Red / Green / Refactor

1. Add public-header consumer tests for constructors, semantic setters,
   operation identity, validation, ownership, and wrong-kind/null handling.
   Run the focused C ABI consumer command and record the expected missing-symbol
   failure as Red evidence.
2. Implement opaque request state and ABI functions, first supporting local
   build and then the existing one-exchange execution route. Add operation
   tests against a controlled TLS peer.
3. Add manifest records and generated outputs; update C documentation and
   traceability. Preserve output-handle and secret-buffer ownership rules.
4. Refactor repeated validation and typed conversion only after Green, keeping
   every exported signature explicit and auditable.
5. Run C ABI tests and all repository-mandated applicable validation, then
   open a stacked `[WIP]` PR against KMIPKIT-0102 and keep it WIP until every
   exact-head CI check is green.

## Risks and mitigations

- The seven operations differ in required fields and multipart semantics;
  model each with its own request variant and validate through the existing
  typed path.
- C callers may assume a copied input buffer is modified or owned by KMIPKit;
  document synchronous copy semantics and test caller-buffer independence.
- Cryptographic parameters can contain extension values; accept an explicit
  ordered Structure but never use it as a generic message submission surface.
- Large byte inputs can exhaust memory; enforce the existing FFI input bound
  before copying and preserve resource-limit status codes.
- Header generation is manifest-backed; regenerate rather than editing the
  checked-in header.
