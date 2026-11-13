# KMIPKIT-0114 Implementation Evidence

## Development commits

- Red: `eada7ab9 test(0114): add red C lifecycle facade consumer`. The compiled
  C consumer called all ten planned symbols before they existed. The focused
  command failed at C compilation with implicit-declaration errors for the
  new functions, as expected.
- Green: `d6baab53 feat(0114): add C lifecycle facade helpers`. Added typed C
  adapters, the feature-owned API capability and declarations, and generated
  public API outputs.
- Refactor: `refactor(0114): consolidate lifecycle facade validation`. The
  Request Options pointer contract is shared, and the mTLS fixture, traceability,
  and evidence now cover all five operations.

## Focused verification

| Command | Result |
|---|---|
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage public_c_header_constructs_a_batch_and_reaches_the_encoding_override_entrypoint -- --exact` | Passed; compiled C consumer called all ten symbols with invalid inputs and received the expected status without output handles. |
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage public_c_fluent_client_executes_lifecycle_helpers_over_mutual_tls -- --exact` | Passed; controlled mTLS fixture observed Create, Get, Activate, Archive, Locate, Recover, Revoke, and Destroy requests, including typed input and response checks. |
| `cargo fmt --all --check` | Passed. |
| `python3 -B tools/api_manifest/generate.py --check` | Passed; all 12 generated public API outputs are current. |

## Remaining required checks

T006 still requires the complete FFI suite, sanitizer and export audits,
traceability, coverage, workspace formatting and Clippy, compatibility checks,
and exact-head CI. Update this section with final results before requesting
review.
