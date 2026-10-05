# T013 Executable Decoder Documentation Report

## Scope and status

This report records the provisional T013 documentation change in commit
`0131fc042f643fe93172e649f7538113d062ff95`. T013 remains unchecked pending
independent review. No push or pull request was created.

The `rust,ignore` placeholder in `quickstart.md` is now a runnable Rust
scenario using a complete 56-byte generic Structure. The same scenario is an
ordinary compiled rustdoc example on the public
`kmipkit_ttlv::codec::decode_with_limits` API. It imports the public
`decode`, `decode_with_limits`, `CodecLimits`, error kind, and model view
types. No decoder behavior or API changed; no public encoder or encoder
example was added.

The example verifies a Structure with three ordered children: two repeated
`0x42002c` Integer Tags carrying `0xf1234567` and `0x81000003`, followed by an
extension Tag `0x541234` with Enumeration value `0xdeadbeef`. The fixed-width
children have nonzero padding octets. Exact decode limits are 56 message
bytes, depth 1, and four Items including the root. A second call with an Item
limit of three checks rejection as `ElementLimitExceeded`.

The quickstart and rustdoc explain that padding contents are accepted when the
required extent is present, and the model contains typed values and ordered
children rather than padding or original input bytes. They do not promise
byte-identical re-emission or claim operation-schema validation.

## Red/Green evidence

T013 changes documentation and a compiled example only. A functional Red
phase does not apply because the decoder API and behavior already exist. No
failing behavior was manufactured. The requested first check,
`cargo test -p kmipkit-ttlv --doc`, passed on the first run after adding the
example; it was repeated after final assertion updates and again passed both
doctests.

## Verification evidence

All final commands completed successfully:

- `cargo test -p kmipkit-ttlv --doc` — 2 passed, 0 failed.
- `cargo fmt --all --check` — passed.
- `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings`
  — passed.
- `cargo check -p kmipkit-ttlv --all-features` — passed.
- `cargo test -p kmipkit-ttlv` — 101 passed, 0 failed, including unit,
  integration, compile-fail, and documentation tests.
- `cargo doc -p kmipkit-ttlv --all-features --no-deps` — succeeded without
  warnings.
- `git diff --check` and `git diff --cached --check` — passed.

`tasks.md` and `approval-record.md` were not changed. T013 remains unchecked.

## Review status

Pending independent review. No push or pull request was created.
