# T009 Red Test Report

## Scope and status

This report records only the completed T009 Red phase. The test-only module
`crates/kmipkit-ttlv/src/codec/limits_tests.rs` is wired under `#[cfg(test)]`
from the codec module. `RedLimits`, limited-decode/preflight stubs, a checked-end
stub, and an observer are private to unit tests. No production decoder, API,
client, manifest, dependency, task status, or FR-013 behavior was changed.

T009 is complete. T010 will replace the limited-decode, preflight, and checked-
end candidates with the production limits path.

## Red coverage

Eleven bounded tests cover default and explicit getter-equivalent values;
default and configured message-byte, Structure-depth, and item-count limits at
exact and one-over boundaries; a synthetic big-endian U32 Item Length header;
checked arithmetic overflow; and rejection before size-driven reservation or
copy for truncated lengths and an explicit message limit. The default one-over
preflight cases use synthetic scalar values rather than allocating 16 MiB,
65-level trees, or 100,001 Items. Wire fixtures for configured cases are at
most 24 bytes. Observer counters record attempted reservation size and copy
calls without allocating memory.

Project limit behavior is attributed to KMIPKIT-0005-FR-007, and checked
length, bounds, and pre-allocation behavior to FR-008. The U32 header test also
identifies OASIS KMIP Specification v2.1 §10.1.3 and
KMIPKIT-0005-NR-004. FR-009 payload-redaction coverage remains in the T006
public negative tests; this limits-only Red task does not duplicate it or mix
in FR-013.

## Verification evidence

Commands completed with exit code 0:

- `cargo fmt --all --check`
- `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings`
- `cargo check -p kmipkit-ttlv --all-features`
- `cargo test -p kmipkit-ttlv --no-run`
- `git diff --check`

The compile-only test command built the unit test binary and all integration
test binaries. The intentional Red run,
`cargo test -p kmipkit-ttlv codec::limits_tests::`, compiled and ran all 11
tests: 2 passed (the getter control and synthetic U32-header/truncation
control), and 9 failed at behavioral assertions as expected. Failures show
that limit preflight currently accepts one-over inputs, limited decode ignores
explicit limits, wrapping arithmetic reports `End(4)` instead of overflow,
and the Red observer records size-driven activity before rejection. There
were no missing-symbol, compile, or panic-path failures. The full crate test
suite is expected to remain red until T010 replaces these stubs with the
production limits path.

No production behavior was changed.

## Independent review

The independent static review of `b3bd550a8360a2fbe69a5cbf82072d8c707607e0`
returned **PASS** with no findings. It confirmed the module's test-only scope,
two Red controls and nine behavioral assertion failures, bounded fixtures,
observer behavior, and project/OASIS attribution. The reviewer did not run
tests; the root agent independently ran the directed Red command and observed
2 passed and 9 expected behavioral failures with no compile or panic failures.
The review record is [`task-9-review.md`](task-9-review.md).
