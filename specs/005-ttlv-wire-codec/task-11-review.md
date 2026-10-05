# T011 Independent Static Review

## Scope and verdict

- Reviewed Refactor commit: `1e904459b1addda85c7b702400fab58cf038dce8`.
- First parent: `7df762a42040f49d249880cfe2fc342cd087115d`.
- Reviewed provisional report commit: `9d6d35daca7b3d85682e1e5244c5e43708db7828`.
- Verdict: **PASS** for T011 spec compliance and task quality.
- This review was static. I did not run tests, builds, Clippy, rustdoc, or coverage commands. The provisional report records the implementation checks; the root reviewer separately reported rerunning the listed format, Clippy, check, test, rustdoc, and diff checks successfully. T011 remains unchecked in `tasks.md`.

## Review findings

No actionable findings.

- The production decoder's accounting and error behavior are unchanged in this Refactor diff. The comments in `crates/kmipkit-ttlv/src/codec/mod.rs:9` and `crates/kmipkit-ttlv/src/codec/decoder.rs:4` document the existing order: whole-input byte preflight, per-Item count before header parsing, Structure depth after Item Type parsing, checked value and parent extents before value access, and fallible decoder-owned payload reservation before copying. `decode_item` and the existing span/boundary helpers are unchanged, so their validation order, error kinds, and offsets are preserved.
- The decoder's existing test-only `DecodeObserver` records the actual decoder-owned reservation and peer-copy boundaries. In `limits_tests.rs`, configured byte-limit rejection and a truncated `u32::MAX` declaration assert zero reservation attempts and zero copies; bounded synthetic helpers cover exact/one-over default and configured byte, depth, and element-count checks. The observer remains behind test-only compilation.
- The encoder still measures the complete Item tree, checks all three resource limits and each U32 Item Length, makes one complete fallible reservation, then writes. The added comments in `wire_encoder.rs:123`, `:345`, and `:445` make that sequence explicit without changing encoding behavior.
- The new `codec_limits_adapter_enforces_default_and_configured_exact_boundaries` case at `wire_encoder.rs:1346` sends synthetic exact/one-over plans through the production `check_plan` path with the real borrowed `CodecLimits` adapter. It does not allocate giant messages, deep trees, or 100,001-item trees. Existing planner tests retain default and configured limits, synthetic depth 65 and count 100,001 cases, and U32 maximum/one-over checks at `wire_encoder.rs:1423` and `:1430`.
- The `EncodingObserver` and its reservation instrumentation are test-only. `reserve_output_buffer` records immediately before the production `try_reserve_exact` call; preflight rejection asserts zero reservation calls and zero payload-copy calls, while two successful writer calls assert two total reservation and copy calls. Source inspection confirms the production path still makes one call to `reserve_output_buffer` after preflight.
- `writer_uses_the_same_borrowed_codec_limits_instance_for_each_operation` remains present at `wire_encoder.rs:1402`; its observer uses pointer identity and confirms the same `&CodecLimits` reaches both operations. The writer remains private, and this commit changes no client callsite, `Client::execute`, permit, public API, manifest, dependency, or approval/task status.
- The report accurately describes the limited Refactor scope, the synthetic bounded planner test, the observer additions, and the absence of a manufactured Red failure. The changed file list is limited to the two codec modules and private client encoder, plus the report; no production decoder logic or unrelated scope was added.

## Report accuracy and limits

`task-11-report.md` accurately describes the changed behavior and test coverage. Its verification commands and totals were not reproduced by this reviewer; the root reviewer separately reported that its independent checks passed. No deterministic allocator-failure injection was added, so allocation failure handling is supported by static inspection of the `try_reserve_exact` path and existing source-preservation tests rather than an injected failure.

T011 remains unchecked pending task-owner updates.
