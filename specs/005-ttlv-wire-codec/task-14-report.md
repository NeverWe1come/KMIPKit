# T014 execution report: decoder fixtures and fuzz target

## Scope and commits

T014 adds reviewable decoder fixture files, a test that consumes and verifies
their attribution, and a bounded libFuzzer target. It changes no production
decoder code and does not modify `tasks.md` or `approval-record.md`.

- Corpus and fixture test: `64e17379c7bba621a44c7c5e649c1a9f2cd510d5`
  (`test(ttlv): add attributed decoder fixture corpus`, DCO-signed).
- Fuzz package and target: `5cf185a418df98493654e4bf3cd9318dc18c6265`
  (`test(ttlv): add bounded decoder fuzz target`, DCO-signed).

The corpus has 11 accepted vectors, one for each Item Type, and 11 rejected
vectors covering malformed framing, values, lengths, padding, Structure
boundaries, and the explicit KMIPKit decoder policies. Each `.hex` file names
the exact OASIS *Key Management Interoperability Protocol Specification
Version 2.1* sections and stable `KMIPKIT-0005-NR-*` identifiers that support
its normative case. Project-only expectations cite their `KMIPKIT-0005-FR-*`
requirement. The fixture test repeats the attribution from each file and
asserts an exact match before decoding it. Reserved-Tag rejection is identified
as the KMIPKit policy in FR-010/ADR-0011; the OASIS citation is labeled as
source context, not as a requirement to reject received bytes. Empty
BigInteger rejection is identified as FR-006, with the source note that
OASIS §10.1.2 does not define a non-empty minimum.

The fuzz target rejects inputs above 4 KiB before decoding and supplies
per-call limits of 4 KiB, Structure depth 64, and 512 Items. Its package pins
`libfuzzer-sys` to `=0.4.12`; the standalone fuzz workspace has a lockfile.
The 22 tracked fixture files remain human-readable `.hex` files. For the fuzz
run, a temporary corpus of raw TTLV bytes was derived by removing each fixture's
comments and hex-decoding its body into `/tmp/kmipkit-ttlv-decode-corpus-t014`;
no binary corpus output is committed.

## TDD evidence

T014 adds test data and verification tooling without changing decoder
behavior. A functional Red commit would require fabricating a failing decoder
behavior that this task does not authorize. The fixture consumer was written
before its corpus files, then the corpus and fuzz target were committed in
separate DCO-signed changes. The final test asserts decoder outcomes for every
fixture, and the fuzz target exercises the existing public bounded decoder.
No Red failure is claimed for this data/tooling-only task.

## Verification

- `cargo fmt --all --check` — passed.
- `cargo fmt --manifest-path fuzz/Cargo.toml --all --check` — passed.
- `cargo test -p kmipkit-ttlv --all-features` — passed: 44 unit tests, 56
  integration tests, and 2 doctests; 102 total, 0 failed.
- `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings` —
  passed.
- `wsl.exe --cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0005-ttlv-wire-codec/KMIPKit --exec bash -lc "cargo check --manifest-path fuzz/Cargo.toml --bin ttlv_decode"`
  — passed with Rust 1.94.
- `wsl.exe --cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0005-ttlv-wire-codec/KMIPKit --exec bash -lc "cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings"`
  — passed.
- `wsl.exe --cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0005-ttlv-wire-codec/KMIPKit --exec bash -lc "cargo +nightly fuzz run ttlv_decode /tmp/kmipkit-ttlv-decode-corpus-t014 -- -runs=1000 -max_len=4096"`
  — completed 1,000 runs from 22 binary seeds; libFuzzer reported 260
  coverage counters, with no crash. The target also enforces the 4 KiB cap
  directly.
- `git diff --check` — passed before the fuzz commit.
- Fixture directory inspection — 22 `.hex` files only; no generated corpus
  artifacts remain in the repository.

## Limitations

The fuzz smoke run is bounded evidence, not a coverage gate or a claim that
fuzzing is exhaustive. `cargo-fuzz` requires nightly on Linux; this run used
Ubuntu WSL with Rust 1.101.0-nightly and `cargo-fuzz` 0.13.2. Stable Rust 1.94
also compiled and linted the fuzz package in WSL.
