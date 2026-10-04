# Research: KMIP Generic TTLV Value Model

**Status**: Accepted design research supporting KMIPKIT-0004. ADR-0010 and the feature specification were accepted through merged PR #10 (`3638c6c7929992e8ced59a3903a0a6640f847069`).

## Repository findings

- `crates/kmipkit-ttlv` is the intended model crate and its root already has `#![forbid(unsafe_code)]`.
- The crate currently has no zeroization dependency or model implementation. The KMIPKIT-0003 implementation merge was a prerequisite for the shared foundation and error contract; it is now satisfied by PR #13 at `b52df30648312f8c7703f511afe80a412cda66cd`.
- `specification/catalog/kmip-2.1.json` stores individual tags as `elements` with `kind: "tag"` (374 records: 354 assigned, 20 Reserved), plus five `tag_ranges` entries. Existing `tools/normative_catalog/` scripts validate/report the catalog but do not generate the Rust tag allocation table.
- ADR-0004 calls for deterministic committed generated output from checked-in normative inputs and prohibits scraping OASIS pages.
- `docs/architecture/public-api.md` currently describes generic TTLV wire validation. This feature supplies only an in-memory model; its implementation must clarify the staged boundary, and a later codec feature must establish actual wire validity and decoder limits.

## Decisions and alternatives

### Tag allocation source and precedence

**Decision**: Generate a Rust allocation table from the reviewed catalog's exact tag records and range records. In the lookup, exact individual entries take precedence over aggregate ranges; the `0x540000–0x54FFFF` §11.56 Extensions range is accepted by the allocation gate; every other unused or Reserved value is rejected. This accepted precedence is project policy in ADR-0010, not an OASIS clarification.

**Rationale**: The source/catalog contains individually assigned `0x420174`–`0x420176` values overlapping the aggregate `420XXX–42FFFF` Reserved notation. Applying exact records first makes individually assigned values usable while preserving the residual Reserved range.

**Alternatives considered**:

- Make the aggregate Reserved range dominant: rejects standard values also individually assigned.
- Accept every `0x42xxxx` or `0x54xxxx` value: admits unused and Reserved values.
- Hand-maintain a Rust table: duplicates the checked-in catalog and risks drift.
- Scrape OASIS during builds: violates ADR-0004 and is not reproducible.

Decoder behavior for received Reserved tags remains KMIPKIT-DISC-037 and is outside this feature.

### Deterministic generation

**Decision**: Add `tools/normative_catalog/generate_ttlv_tags.py` as a reviewed repository tool. It reads the local JSON catalog only and reuses `load_validated_catalog` from `validate.py`, then deterministically emits private Rust allocation metadata to `crates/kmipkit-ttlv/src/generated/tag_allocations.rs`. It does not generate public Tag declarations, typed KMIP APIs, language bindings, or a public API manifest. Sort exact values numerically, preserve allocation classifications, and emit each range as numeric endpoints and allocation kind. Keep lookup precedence hand-written in `tag.rs` so the project policy is directly reviewable. Provide safe `--write` and read-only `--check` modes; add `--check` to `.github/workflows/ci.yml` next to catalog validation/report. Test all records, residual range boundaries, output determinism under reordered inputs, malformed/duplicate values, and clean/stale/missing-file check behavior.

**Rationale**: It follows ADR-0004 while limiting generation to the model's needed allocation data. The future public API manifest and cross-language generation are not needed for this Rust-only model feature.

**Alternatives considered**: Manually coded tag records cannot guarantee synchronization with the catalog; a second ad hoc JSON parser would duplicate existing validation; full public API manifest/schema generation or public tag declarations are broader than this feature.

### Payload ownership and zeroization

**Decision**: Propose `zeroize` 1.9.0 as a workspace dependency with `default-features = false` and `features = ["alloc"]`, used directly by `kmipkit-ttlv`. Keep the resolved version in `Cargo.lock`; do not enable derive or serialization features. The crate provides `Zeroizing<T>` but does not provide a generic `Zeroize for Box<T>`, so do not wrap `Box<T>` directly in `Zeroizing<T>`. Instead define a private `Secret<T>` that owns `Box<T>`, implements the local `Zeroize` trait by calling `self.boxed.as_mut().zeroize()`, and calls that method from its safe `Drop` implementation. Implement `Zeroize` recursively for the payload enum and items. The stable box means growing the parent Structure's `Vec<Item>` moves only tag/box-pointer metadata, not scalar payload bytes. Keep owned String/Vec payload buffers immutable after ownership enters KMIPKit so those buffers cannot reallocate while containing secret data; their current backing capacity is cleared before release. Confirm the exact version and dependency review in the implementation PR.

**Rationale**: This meets the project requirement without introducing unsafe code into a crate that forbids it. A private wrapper prevents payload formatting/serialization from being added accidentally and makes exposure an explicit API operation. The published `zeroize` 1.9.0 documentation lists MSRV 1.85, Apache-2.0/MIT licensing, and no required third-party dependencies; that is compatible with the workspace's Rust 1.94 MSRV. The dependency rationale must still cover capability, alternatives, maintenance/security history, MSRV, license, platforms, and transitive cost as required by `docs/development/coding-standards.md`.

**Guarantee boundary**: Clear only storage KMIPKit currently owns. Document that caller-side buffers/reallocations from before ownership, explicit caller copies, temporary stack/register copies, and managed-language runtime copies cannot be guaranteed cleared by this model. Internal Structure growth moves only metadata and stable secret-box pointers; payload buffers are immutable after ownership. Do not claim to securely erase external or historical copies.

**Verification**: Use `trybuild` UI fixtures with expected diagnostic snapshots and a test-only `serde` dev-dependency. Each fixture imports `serde::Serialize`, bounds a helper by `T: Serialize`, then instantiates it for `Value`, `ValueView`, `StructureView`, `Item`, and `Structure` so the intended trait-bound error is tested. Also snapshot missing `Clone`/`Copy` bounds and borrowed-reference escape. Use a safe test-only zeroize spy to verify the Secret wrapper Drop path. Add live-object unit tests that call internal `Zeroize` on every payload variant and nested Structure and inspect the still-live object to confirm its fields clear. A safe address-stability test verifies that adding children does not relocate existing boxed payloads. The spy does not inspect released bytes. Separately review the exact locked `zeroize` source to confirm String/Vec capacity semantics and ensure payload storage cannot reallocate after KMIPKit takes ownership. Never inspect memory after deallocation or add unsafe allocator instrumentation.

**Alternatives considered**: Bare `Vec<u8>`/`String` do not clear on drop; fixed boxed byte slices avoid reallocation but complicate String ownership without improving this immutable model's current-buffer guarantee; adding `serde::Serialize` would create an easy payload disclosure path; unsafe memory inspection is prohibited and would not prove post-free erasure safely.

### Closure-scoped exposure

**Decision**: Expose payloads through a callback that lends references to a view for the callback's duration. Use a higher-ranked lifetime contract so the callback result cannot retain those borrows. Nested Structures expose child views while child payload storage remains wrapped. Explicit caller cloning/copying is allowed and documented as the caller's responsibility.

**Rationale**: It makes payload access explicit and prevents an accidental borrowed reference from escaping the wrapper's guarded scope.

**Verification**: A compile-fail API test rejects returning a borrowed view/reference from the callback; a runtime test demonstrates an explicit copy is possible. Documentation states that closure scope prevents borrow escape but cannot prevent disclosure or copies.

### Diagnostics and serialization

**Decision**: Payload-bearing values, borrowed `ValueView`, items, and trees implement redacted `Debug`; do not provide `Display` unless also redacted. Do not implement `serde::Serialize`; this model supports no other automatic payload serializer, and any future serializer requires a separate reviewed specification. Model-local errors include only safe type/tag/context and do not carry payload bytes. This in-memory model adds no logging call sites.

**Rationale**: The model handles values that may contain key material or message data, and default diagnostics are a common accidental disclosure route.

**Boundary**: Transport/client raw-body logging and error redaction belong to those layers' specifications.

### Compile-fail API harness

**Decision**: Use test-only `trybuild` 1.0.121 UI fixtures with checked-in `.stderr` snapshots, plus a direct dev-only `serde` dependency for fixtures that assert no `serde::Serialize` implementation exists. Cover `Value`, `ValueView`, `StructureView`, `Item`, and `Structure`; include `Clone`, `Copy`, and closure-borrow escape failures. Snapshot matching makes the test fail if compilation fails for an unrelated reason such as a missing import or type.

**Rationale**: Rustdoc `compile_fail` tests pass for any compiler error and do not ensure that the intended trait-bound/lifetime error caused the failure. Trybuild compares diagnostics against snapshots. The reviewed trybuild release is MIT OR Apache-2.0, declares MSRV 1.88 (below workspace 1.94), and is test-only; its transitive dependencies are recorded in the test dependency review and lockfile.

**Alternatives considered**: Rustdoc `compile_fail` has no snapshot check for the intended error; custom `rustc` subprocess plumbing would duplicate a maintained UI-test harness and be less portable.

Reference: [trybuild 1.0.121 package metadata](https://docs.rs/crate/trybuild/1.0.121/source/Cargo.toml), [trybuild compiler-diagnostic testing API](https://docs.rs/trybuild/1.0.121/trybuild/).

### Error ownership

**Decision**: Keep `kmipkit-ttlv` errors local to that crate. Higher protocol/client layers can map or wrap them through the shared KMIPKIT-0003 error contract without making this lower layer depend upward.

**Rationale**: Preserves workspace dependency direction and avoids cycles.

### Model versus codec

**Decision**: Represent semantic values and Big Integer Item Value octets exactly in memory. Do not encode/decode, validate encoded lengths/padding, enforce decoder resource limits, or claim a generic tree is wire-valid. Treat empty Big Integer octets as an allowed in-memory model value only; wire acceptability belongs to the codec.

**Rationale**: Separates the semantic model from the codec's exact framing and hostile-input obligations and avoids a false completeness claim.

## Normative references

Use the immutable local OASIS KMIP Specification v2.1 copy (`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`): §§10.1.1–10.1.5 for TTLV item representation, §10.1.2 for value forms, §11.23 for the eleven Item Types, Chapter 11 introduction and §11.56 for tag allocation, and §§12.1–12.3 for bit masks. The spec's Normative Traceability table maps each stable KMIPKIT-NR identifier to these clauses.

## Dependency review

The selected zeroization dependency and exact version must be confirmed against the project's Rust 1.94 MSRV, license policy, feature set, and supply-chain checks in the implementation PR. Record the exact resolved version in `Cargo.lock`; do not infer compatibility solely from this planning artifact.

Reference reviewed for this proposal: [zeroize 1.9.0 documentation](https://docs.rs/zeroize/1.9.0/zeroize/), [zeroize 1.9.0 source](https://docs.rs/zeroize/1.9.0/src/zeroize/lib.rs.html), and [zeroize 1.9.0 changelog](https://github.com/RustCrypto/utils/blob/master/zeroize/CHANGELOG.md). The source confirms `Vec` clears its entire current capacity but cannot erase prior reallocations, `Box<[T]>` cannot reallocate, and `Zeroizing<T>` calls `zeroize()` on Drop. It does not implement `Zeroize` for generic `Box<T>`, so the private Secret Drop must explicitly zeroize its boxed payload before deallocation. Reconfirm these properties against the exact locked source when implementing.
