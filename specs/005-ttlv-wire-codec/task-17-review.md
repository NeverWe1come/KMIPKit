# T017 Independent Review and PR Preparation

Date: 2026-10-05  
Branch: `feature/KMIPKIT-0005-ttlv-wire-codec`  
Base: `release/1.0.0` at `d46e13dfdd83ac05e4b58d24af5b928e355e092d`

## Independent reviews

- **QA:** The independent QA follow-up found no remaining blocker. It confirmed
  that T016 permits only stable-sentinel normalization of external LLVM
  function-filename metadata. Coverage `files` records, workspace source paths,
  and the exact changed-source scan remain strict. Red `90543e5`, Green
  `8dfcb3a`, and semantics-preserving Refactor `620c8b8` are distinct commits;
  all 49 script tests pass after Refactor. The reviewer confirmed that the 10
  writer and 7 decoder verification references resolve to existing functions,
  and that catalog validation and generated report/tag/result checks pass.
- **Security:** The formal Codex Security diff scan covered
  `d46e13d..ca0ca25236e5971b2a5e00d94e7285f9e248a9b8` and completed with zero
  findings across 11 surfaces (run `c2251911-d011-407b-930d-3e8843276906`). A
  duplicate independent scan of the same range also completed with zero
  findings (run `717115da-5de3-4de7-ba67-06a80c141b3a`). The final read-only
  review checked the later documentation/catalog delta: updated references
  point to current test support files and functions; external paths are
  normalized only in LLVM function metadata, while coverage file records
  remain strictly validated. No security, documentation, or traceability
  blocker remains.
- The final scope clarification records why external LLVM function filename
  metadata may be normalized in T016 and preserves strict handling of coverage
  files and workspace sources. This is verification infrastructure only; it
  changes no KMIPKit API or protocol boundary.

## Verification evidence

| Check | Result |
|---|---|
| Normative catalog unit suite | 162 passed, 7 platform-specific skips |
| Python script contracts | 49 passed, 3 Windows symlink tests skipped |
| Normative catalog validation | Passed: 4 sources, 1,411 clauses, 4,021 records |
| Generated catalog report, TTLV tags, result values | All `--check` commands passed when run serially |
| Rust formatting, workspace Clippy, workspace tests, workspace docs | Passed on Windows with Rust 1.99.0; detailed commands and coverage are in `task-16-report.md` |
| OASIS source immutability and source-candidate audit | Passed against the exact release base; 1,411 candidates audited |
| `git diff --check` | Passed |
| Coverage | Changed writer 95.83%, decoder 96.90%, TTLV 97.79%, protocol 99.21%, transport 100%, workspace 95.32% |
| `cargo audit` | No advisories in 36 locked crates |
| `cargo deny` | Advisories, bans, and sources passed; license phase remains blocked by the repository-wide missing allowlist documented in `task-16-report.md` |

The normative test suite was rerun serially after the earlier transient
safe-I/O lock error and completed successfully. Seven skips are host-specific
tests requiring symlink privilege, POSIX symlinks, or POSIX fixtures.

## Remaining gates and release limits

- T016 is complete. Pull-request CI run
  [37326087477](https://github.com/NeverWe1come/KMIPKit/actions/runs/37326087477)
  passed Linux, Windows, and macOS stable/MSRV matrices, script contracts,
  normative inventory, per-platform coverage, and three-platform coverage
  aggregation.
- The default `cargo deny check` license phase requires the separate
  infrastructure-owned license allowlist; this feature does not invent that
  policy.
- A qualified human security review remains required before 1.0. The automated
  scan and independent source-level review recorded here do not claim to
  replace that release requirement.
- The private writer still has no production callsite, and this feature adds
  no transport send path, `Client::execute`, permit, FFI, or binding API.

## Red, Green, Refactor commits

- Writer: Red `d67b932`, Green `dfa3d9a`, Refactor `5b855ba`.
- Decoder: Red `0937633`, Green `6419a43`, Refactor `f576a82`.
- Limits: Red `b3bd550`, Green `302b8c6`, Refactor `1e90445`.
- Coverage metadata normalization: Red `90543e5`, Green `8dfcb3a`, Refactor
  `620c8b8`.

The terminal-created **draft** pull request to `release/1.0.0` is
[KMIPKit PR #30](https://github.com/NeverWe1come/KMIPKit/pull/30). It does not
claim that the separate qualified human security review has passed.
